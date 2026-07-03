//! **F-KAT-1 — libcrux ↔ RustCrypto FIPS-203 serialization KAT.**
//! (merges CE-D1 + WF-G2; the m-2 Wave-0 exit gate)
//!
//! ADDL R3 wave **W1-crypto-kat**. Pin sources:
//!   - `f-full-r2-test-landscape.md` Group-4 F-KAT-1 + §4 P0-4 + NQ-C2.
//!   - R0 §2.2 Q1: ML-KEM impl = **libcrux-ml-kem** (verified secret-independence
//!     via hax/F*). "**NQ-C2 (R2):** the swap MUST preserve FIPS-203
//!     encap/ciphertext/decap-key serialization byte-for-byte (cross-impl KAT)
//!     or golden vectors break."
//!   - R0 §3.2 Wave-0 exit gate (m-2): "a FIPS-203 byte-compatibility KAT
//!     (RustCrypto `ml-kem 0.2` ↔ libcrux)".
//!   - CLAUDE.md baked-in #5 (never-fork-primitives; the swap is impl-only, the
//!     serialized bytes are FIPS-203-canonical and MUST survive).
//!
//! # WIRED 2026-06-05 — the real cross-impl KAT (libcrux is now production)
//!
//! `libcrux-ml-kem 0.0.9` is the PRODUCTION ML-KEM-768 impl (swapped from
//! RustCrypto `ml-kem 0.2.3`, now retained as a `[dev-dependencies]`
//! cross-impl witness for exactly this KAT). The HARD-GATE `#[ignore]` is
//! REMOVED: both real impls are driven from the SAME FIPS-203 deterministic
//! `(d, z, m)` inputs and asserted byte-identical. This is the strongest
//! NQ-C2 confirmation — the two impls are byte-for-byte interchangeable, so
//! every on-wire ML-KEM-768 byte + every frozen golden survives the swap
//! unchanged (corroborates the canary spike at
//! `.addl/phase-4-meta/libcrux-canary-spike.md`).
//!
//! # What this pins (FREEZE-GATING / CF — golden-vector survival)
//!
//!   1. for a fixed FIPS-203 `(d, z, m)` seed, BOTH impls produce
//!      byte-identical encapsulation keys + ciphertexts + decapsulation-key
//!      serializations + shared secrets (cross-impl byte-equality);
//!   2. ML-KEM-768 sizes are FIPS-203-exact (ek=1184, ct=1088, dk=2400, ss=32);
//!   3. cross-impl interop both directions: a ciphertext from one impl
//!      decapsulates under the other impl's decap key to the SAME ss;
//!   4. negative: a wrong seed → a DIFFERENT ciphertext (genuinely seed-bound).
//!
//! The FIPS-203 deterministic constructors are the byte-equality bridge:
//! RustCrypto `generate_deterministic(&d, &z)` / `encapsulate_deterministic
//! (&m)` and libcrux `generate_key_pair(d‖z)` / `encapsulate(&pk, m)`
//! consume the SAME standardized seeds, so equal inputs MUST yield equal
//! FIPS-203 outputs. (The published multi-MB NIST KAT `.rsp` corpus is a
//! separate NF-2 / C-GM-AUDIT fixture; this deterministic cross-impl witness
//! is the per-build conformance gate.)

#![allow(dead_code)]

/// REAL RustCrypto `ml-kem 0.2.3` FIPS-203 witness (the [dev-dependencies]
/// cross-impl reference).
mod rustcrypto_impl {
    use ml_kem::kem::Decapsulate as _;
    use ml_kem::{
        B32, EncapsulateDeterministic as _, Encoded, EncodedSizeUser as _, KemCore, MlKem768,
    };

    /// Keygen from a FIPS-203 `(d, z)` seed pair.
    pub fn keygen(d: &[u8; 32], z: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
        let d32: B32 = (*d).into();
        let z32: B32 = (*z).into();
        let (dk, ek) = MlKem768::generate_deterministic(&d32, &z32);
        (ek.as_bytes().to_vec(), dk.as_bytes().to_vec())
    }

    /// Encapsulate against `ek_bytes` with FIPS-203 `m` randomness.
    pub fn encapsulate(ek_bytes: &[u8], m: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
        let ek_arr: Encoded<<MlKem768 as KemCore>::EncapsulationKey> =
            Encoded::<<MlKem768 as KemCore>::EncapsulationKey>::try_from(ek_bytes).unwrap();
        let ek = <MlKem768 as KemCore>::EncapsulationKey::from_bytes(&ek_arr);
        let m32: B32 = (*m).into();
        let (ct, ss) = ek.encapsulate_deterministic(&m32).unwrap();
        (ct.as_slice().to_vec(), ss.as_slice().to_vec())
    }

    /// Decapsulate `ct_bytes` under the decap key `dk_bytes`.
    pub fn decapsulate(dk_bytes: &[u8], ct_bytes: &[u8]) -> Vec<u8> {
        let dk_arr: Encoded<<MlKem768 as KemCore>::DecapsulationKey> =
            Encoded::<<MlKem768 as KemCore>::DecapsulationKey>::try_from(dk_bytes).unwrap();
        let dk = <MlKem768 as KemCore>::DecapsulationKey::from_bytes(&dk_arr);
        let ct = ml_kem::Ciphertext::<MlKem768>::try_from(ct_bytes).unwrap();
        dk.decapsulate(&ct).unwrap().as_slice().to_vec()
    }
}

/// REAL libcrux-ml-kem `0.0.9` FIPS-203 witness (the PRODUCTION impl).
mod libcrux_impl {
    use libcrux_ml_kem::mlkem768;

    /// Keygen from a FIPS-203 `(d, z)` seed pair (libcrux takes the 64-byte
    /// `d‖z` concatenation).
    pub fn keygen(d: &[u8; 32], z: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
        let mut dz = [0u8; 64];
        dz[..32].copy_from_slice(d);
        dz[32..].copy_from_slice(z);
        let kp = mlkem768::generate_key_pair(dz);
        (kp.pk().to_vec(), kp.sk().to_vec())
    }

    /// Encapsulate against `ek_bytes` with FIPS-203 `m` randomness.
    pub fn encapsulate(ek_bytes: &[u8], m: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
        let pk: mlkem768::MlKem768PublicKey = ek_bytes.try_into().unwrap();
        let (ct, ss) = mlkem768::encapsulate(&pk, *m);
        (ct.as_ref().to_vec(), ss.to_vec())
    }

    /// Decapsulate `ct_bytes` under the decap key `dk_bytes`.
    pub fn decapsulate(dk_bytes: &[u8], ct_bytes: &[u8]) -> Vec<u8> {
        let sk: mlkem768::MlKem768PrivateKey = dk_bytes.try_into().unwrap();
        let ct: mlkem768::MlKem768Ciphertext = ct_bytes.try_into().unwrap();
        mlkem768::decapsulate(&sk, &ct).to_vec()
    }
}

// R13 F-11: pin the PRODUCTION FIPS-203 ML-KEM-768 size consts (not
// file-local literals). Importing `benten_crypto_suite::mlkem::*` makes the
// libcrux-driven KAT assertions below verify the ACTUAL production consts
// against real libcrux output — so a silent drift of a production const away
// from FIPS-203 fails HERE, instead of tautologizing against a private copy.
use benten_crypto_suite::cipher_suite::{
    ML_KEM_768_CT_LEN, ML_KEM_768_DK_LEN, ML_KEM_768_EK_LEN, ML_KEM_768_SS_LEN,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct MlKem768Kat {
    encapsulation_key: Vec<u8>,
    ciphertext: Vec<u8>,
    decapsulation_key: Vec<u8>,
    shared_secret: Vec<u8>,
}

/// Derive the FIPS-203 `(d, z, m)` inputs deterministically from a KAT
/// seed (domain-separated SHA-256). Both impls consume IDENTICAL `(d, z, m)`
/// so equal inputs MUST yield equal FIPS-203 outputs.
fn kat_inputs(seed: &[u8; 32]) -> ([u8; 32], [u8; 32], [u8; 32]) {
    use sha2::{Digest, Sha256};
    let block = |tag: &[u8]| -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"f-kat-1-fips203-input");
        h.update(tag);
        h.update(seed);
        h.finalize().into()
    };
    (block(b"d"), block(b"z"), block(b"m"))
}

/// Run an impl (selected by its keygen/encap fns) over the seed-derived
/// `(d, z, m)` and capture its serialized FIPS-203 outputs.
fn witness(
    seed: &[u8; 32],
    keygen: impl Fn(&[u8; 32], &[u8; 32]) -> (Vec<u8>, Vec<u8>),
    encap: impl Fn(&[u8], &[u8; 32]) -> (Vec<u8>, Vec<u8>),
) -> MlKem768Kat {
    let (d, z, m) = kat_inputs(seed);
    let (ek, dk) = keygen(&d, &z);
    let (ct, ss) = encap(&ek, &m);
    MlKem768Kat {
        encapsulation_key: ek,
        ciphertext: ct,
        decapsulation_key: dk,
        shared_secret: ss,
    }
}

fn rustcrypto_witness(seed: &[u8; 32]) -> MlKem768Kat {
    witness(seed, rustcrypto_impl::keygen, rustcrypto_impl::encapsulate)
}
fn libcrux_witness(seed: &[u8; 32]) -> MlKem768Kat {
    witness(seed, libcrux_impl::keygen, libcrux_impl::encapsulate)
}

const KAT_SEED: [u8; 32] = [0x07u8; 32];

/// F-KAT-1 (a) — cross-impl FIPS-203 byte-equality (NQ-C2; the m-2 exit gate).
///
/// For a fixed FIPS-203 `(d, z, m)` seed, libcrux + RustCrypto MUST serialize
/// byte-identical encapsulation keys, ciphertexts, decapsulation keys, and
/// shared secrets. would-FAIL-if-no-op'd: if the libcrux swap changed any
/// serialized byte, this fails and (correctly) signals every golden broke.
#[test]
fn libcrux_rustcrypto_fips203_serialization_byte_identical() {
    let libcrux = libcrux_witness(&KAT_SEED);
    let rustcrypto = rustcrypto_witness(&KAT_SEED);

    assert_eq!(
        libcrux.encapsulation_key, rustcrypto.encapsulation_key,
        "FIPS-203 encapsulation-key serialization MUST be byte-identical across \
         libcrux + RustCrypto (the libcrux swap preserves golden vectors; NQ-C2)."
    );
    assert_eq!(
        libcrux.ciphertext, rustcrypto.ciphertext,
        "FIPS-203 ciphertext serialization MUST be byte-identical across impls"
    );
    assert_eq!(
        libcrux.decapsulation_key, rustcrypto.decapsulation_key,
        "FIPS-203 decapsulation-key serialization MUST be byte-identical across impls"
    );
    assert_eq!(
        libcrux.shared_secret, rustcrypto.shared_secret,
        "FIPS-203 shared-secret MUST be byte-identical across impls"
    );
}

/// F-KAT-1 (b) — ML-KEM-768 serialized sizes are FIPS-203-exact.
///
/// Pins ek=1184, ct=1088, dk=2400, ss=32 (never hardcode-without-pin per
/// CLAUDE.md #5). would-FAIL-if-no-op'd: a witness of any other size.
#[test]
fn ml_kem_768_serialized_sizes_are_fips203_exact() {
    let kat = libcrux_witness(&KAT_SEED);
    assert_eq!(
        kat.encapsulation_key.len(),
        ML_KEM_768_EK_LEN,
        "ML-KEM-768 encapsulation key MUST be 1184 bytes (FIPS-203)"
    );
    assert_eq!(
        kat.ciphertext.len(),
        ML_KEM_768_CT_LEN,
        "ciphertext MUST be 1088 bytes"
    );
    assert_eq!(
        kat.decapsulation_key.len(),
        ML_KEM_768_DK_LEN,
        "decapsulation key MUST be 2400 bytes"
    );
    assert_eq!(
        kat.shared_secret.len(),
        ML_KEM_768_SS_LEN,
        "shared secret MUST be 32 bytes"
    );
}

/// F-KAT-1 (c) — cross-impl interop BOTH directions (NQ-C2 interop half).
///
/// A ciphertext produced by libcrux MUST decapsulate under RustCrypto's decap
/// key to the same shared secret, and vice-versa — so a node running libcrux
/// and a node running RustCrypto interoperate transparently on the wire.
/// would-FAIL-if-no-op'd: any serialization divergence breaks cross-decap.
#[test]
fn cross_impl_interop_both_directions() {
    let (d, z, m) = kat_inputs(&KAT_SEED);

    // Both impls hold the SAME keypair (byte-identical, asserted above).
    let (lc_ek, lc_dk) = libcrux_impl::keygen(&d, &z);
    let (rc_ek, rc_dk) = rustcrypto_impl::keygen(&d, &z);

    // libcrux encaps → RustCrypto decaps.
    let (lc_ct, lc_ss) = libcrux_impl::encapsulate(&lc_ek, &m);
    let rc_recovered = rustcrypto_impl::decapsulate(&rc_dk, &lc_ct);
    assert_eq!(
        lc_ss, rc_recovered,
        "libcrux-encap → RustCrypto-decap MUST recover the same shared secret"
    );

    // RustCrypto encaps → libcrux decaps.
    let (rc_ct, rc_ss) = rustcrypto_impl::encapsulate(&rc_ek, &m);
    let lc_recovered = libcrux_impl::decapsulate(&lc_dk, &rc_ct);
    assert_eq!(
        rc_ss, lc_recovered,
        "RustCrypto-encap → libcrux-decap MUST recover the same shared secret"
    );
}

/// F-KAT-1 (d) — the KAT is genuinely seed-bound (negative control).
///
/// A wrong seed MUST produce a different ciphertext — the witness is not a
/// constant. would-FAIL-if-no-op'd: a constant-returning loader.
#[test]
fn fips203_witness_is_seed_bound() {
    let kat = libcrux_witness(&KAT_SEED);
    let other = libcrux_witness(&[0x09u8; 32]);
    assert_ne!(
        kat.ciphertext, other.ciphertext,
        "a different KAT seed MUST yield a different ciphertext — the witness is \
         seed-bound, not a constant."
    );
}

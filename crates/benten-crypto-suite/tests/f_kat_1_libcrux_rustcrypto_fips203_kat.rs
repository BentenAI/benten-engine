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
//! # What this pins (FREEZE-GATING / CF — golden-vector survival)
//!
//! The libcrux-ml-kem swap (away from RustCrypto `ml-kem 0.2`) MUST preserve the
//! FIPS-203 encapsulation-key / ciphertext / decapsulation-key serialization
//! byte-for-byte. If it does not, every golden vector + every on-wire ML-KEM-768
//! byte breaks. Pins (against a deterministic synthesized FIPS-203 witness — the
//! `tf4 load_fips_204_kat_vector_for_test` precedent — until the real NIST KAT
//! corpus lands at R5):
//!   1. for a fixed KAT seed, both impls produce byte-identical encapsulation
//!      keys + ciphertexts + decapsulation-key serializations (cross-impl
//!      byte-equality);
//!   2. ML-KEM-768 sizes are FIPS-203-exact (ek=1184, ct=1088, dk=2400, ss=32);
//!   3. negative: a wrong seed → a DIFFERENT ciphertext (the KAT is genuinely
//!      seed-bound, not a constant).
//!
//! ## ORCHESTRATOR-FLAGGED EXTERNAL-VECTOR SEED (R2 §5-D-9; NQ-C2)
//!
//! Ground-truth at HEAD: `libcrux-ml-kem` is NOT yet a workspace dep (`grep -rn
//! libcrux crates/*/Cargo.toml Cargo.toml` → ZERO; RustCrypto `ml-kem` is the
//! in-tree impl). The real NIST FIPS-203 KAT corpus + the libcrux swap arrive at
//! R5. This file uses a **deterministic synthesized witness** (per the
//! `tf4 load_fips_204_kat_vector_for_test` precedent) so the cross-impl
//! byte-equality SHAPE is pinned now; **R5 swaps in the real FIPS-203 corpus +
//! both real impls and verifies byte-equality.** Flagged for R5 real-corpus
//! swap.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + SELF-CONTAINED STUB-SHIM
//!
//! Per wave-independence this file commits a LOCAL `f_kat_1_stub`. The stub's
//! two "impls" deliberately DIVERGE (libcrux witness ≠ rustcrypto witness for
//! the same seed) so the cross-impl byte-equality pin FAILS until R5 wires both
//! real impls against the shared FIPS-203 corpus — never a silent-green
//! SHAPE-trap. R5 DELETEs the stub + wires the real `ml-kem` + `libcrux-ml-kem`
//! KAT round-trip.

#![allow(dead_code)]

/// R5: REAL RustCrypto `ml-kem` FIPS-203 witnesses (genuine encap/ct/dk/ss
/// bytes from a deterministic KAT seed), NOT synthesized sentinels. The
/// cross-impl-vs-libcrux byte-identity arm is HARD-GATED (`#[ignore]` +
/// FLAG-FOR-BEN — see the cross-impl test below) because libcrux-ml-kem is
/// not wired at impl-time; the size + seed-bound + within-impl-serialization
/// arms run on the real RustCrypto impl.
mod f_kat_1_real {
    use ml_kem::kem::Encapsulate;
    use ml_kem::{Encoded, EncodedSizeUser, KemCore, MlKem768};

    /// FIPS-203 ML-KEM-768 serialized sizes (exact).
    pub const ML_KEM_768_EK_LEN: usize = 1184;
    pub const ML_KEM_768_CT_LEN: usize = 1088;
    pub const ML_KEM_768_DK_LEN: usize = 2400;
    pub const ML_KEM_768_SS_LEN: usize = 32;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct MlKem768Kat {
        pub encapsulation_key: Vec<u8>,
        pub ciphertext: Vec<u8>,
        pub decapsulation_key: Vec<u8>,
        pub shared_secret: Vec<u8>,
    }

    /// A deterministic CSPRNG seeded from the KAT seed (so the KAT is
    /// reproducible + seed-bound without a published `.rsp` corpus).
    struct SeededRng {
        state: [u8; 32],
        ctr: u64,
    }
    impl SeededRng {
        fn new(seed: &[u8; 32]) -> Self {
            Self {
                state: *seed,
                ctr: 0,
            }
        }
        fn block(&mut self) -> [u8; 32] {
            use sha2::{Digest, Sha256};
            let mut h = Sha256::new();
            h.update(self.state);
            h.update(self.ctr.to_be_bytes());
            self.ctr += 1;
            h.finalize().into()
        }
    }
    impl rand_core::RngCore for SeededRng {
        fn next_u32(&mut self) -> u32 {
            let b = self.block();
            u32::from_be_bytes([b[0], b[1], b[2], b[3]])
        }
        fn next_u64(&mut self) -> u64 {
            let b = self.block();
            u64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
        }
        fn fill_bytes(&mut self, dest: &mut [u8]) {
            let mut i = 0;
            while i < dest.len() {
                let b = self.block();
                let n = (dest.len() - i).min(32);
                dest[i..i + n].copy_from_slice(&b[..n]);
                i += n;
            }
        }
        fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand_core::Error> {
            self.fill_bytes(dest);
            Ok(())
        }
    }
    impl rand_core::CryptoRng for SeededRng {}

    /// Compute a REAL FIPS-203 ML-KEM-768 KAT from a deterministic seed via
    /// the in-tree RustCrypto `ml-kem` crate.
    pub fn real_witness(seed: &[u8; 32]) -> MlKem768Kat {
        let mut rng = SeededRng::new(seed);
        let (dk, ek) = MlKem768::generate(&mut rng);
        let ek_bytes = ek.as_bytes().to_vec();
        let dk_bytes = dk.as_bytes().to_vec();
        // Re-load ek and encapsulate deterministically from the same RNG path.
        let ek_arr: Encoded<<MlKem768 as KemCore>::EncapsulationKey> =
            Encoded::<<MlKem768 as KemCore>::EncapsulationKey>::try_from(ek_bytes.as_slice())
                .unwrap();
        let ek2 = <MlKem768 as KemCore>::EncapsulationKey::from_bytes(&ek_arr);
        let (ct, ss) = ek2.encapsulate(&mut rng).unwrap();
        MlKem768Kat {
            encapsulation_key: ek_bytes,
            ciphertext: ct.as_slice().to_vec(),
            decapsulation_key: dk_bytes,
            shared_secret: ss.as_slice().to_vec(),
        }
    }
}

use f_kat_1_real::{
    ML_KEM_768_CT_LEN, ML_KEM_768_DK_LEN, ML_KEM_768_EK_LEN, ML_KEM_768_SS_LEN, real_witness,
};

/// REAL RustCrypto witness aliased to both impl roles. The size + seed-bound
/// + within-impl-serialization arms run on this real impl; the cross-impl-vs-
/// libcrux byte-identity arm is HARD-GATED (`#[ignore]` + FLAG-FOR-BEN).
fn libcrux_witness(seed: &[u8; 32]) -> f_kat_1_real::MlKem768Kat {
    real_witness(seed)
}
fn rustcrypto_witness(seed: &[u8; 32]) -> f_kat_1_real::MlKem768Kat {
    real_witness(seed)
}

const KAT_SEED: [u8; 32] = [0x07u8; 32];

/// F-KAT-1 (a) — cross-impl FIPS-203 byte-equality (NQ-C2; the m-2 exit gate).
///
/// For a fixed KAT seed, libcrux + RustCrypto MUST serialize byte-identical
/// encapsulation keys, ciphertexts, decapsulation keys, and shared secrets.
/// would-FAIL-if-no-op'd: the stub's two impls diverge (impl-tagged), so this
/// fails until R5 wires both real impls against the shared corpus. A swap that
/// changed any serialized byte breaks every golden vector.
#[test]
#[ignore = "R5-FILL HARD-GATE (FLAG-FOR-BEN): awaiting the libcrux-ml-kem cross-impl integration (or the published NIST FIPS-203 .rsp corpus) — the cross-impl byte-identity vs libcrux cannot be witnessed honestly with only the in-tree RustCrypto impl; both witnesses would be the SAME impl (a tautology). The in-tree RustCrypto FIPS-203 bytes/sizes/seed-binding ARE real (the size + seed-bound arms run green); the cross-impl conformance is a Ben decision: (a) add libcrux-ml-kem v0.0.9 now + wire the byte-identity, or (b) defer NQ-C2 cross-impl to v1-GM. Kept #[ignore]'d per the no-pass-vs-sentinel HARD-GATE."]
fn libcrux_rustcrypto_fips203_serialization_byte_identical() {
    let libcrux = libcrux_witness(&KAT_SEED);
    let rustcrypto = rustcrypto_witness(&KAT_SEED);

    assert_eq!(
        libcrux.encapsulation_key, rustcrypto.encapsulation_key,
        "FIPS-203 encapsulation-key serialization MUST be byte-identical across \
         libcrux + RustCrypto (the libcrux swap preserves golden vectors; NQ-C2). \
         would-FAIL while the stub impls diverge."
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

/// F-KAT-1 (c) — the KAT is genuinely seed-bound (negative control).
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
         seed-bound, not a constant. would-FAIL on a constant loader."
    );
}

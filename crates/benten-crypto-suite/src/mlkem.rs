//! ML-KEM-768 production wrapper over Cryspen `libcrux-ml-kem` (the
//! hax/F\*-verified, constant-time FIPS-203 impl).
//!
//! # Why this module
//!
//! This is the single internal seam through which `cipher_suite` (the
//! X-Wing hybrid wrap at codepoint `0x647a`) and `swap_matrix` (the pure-PQ
//! `0x647c` arm + the KAT test helpers) reach ML-KEM-768. Centralizing the
//! libcrux call here keeps the "ONLY crypto-primitive call site" discipline
//! (CLAUDE.md baked-in #5 / `crypto-agility-contract:6`) literal: ML-KEM-768
//! is wrapped in exactly one file. We NEVER fork or reimplement the
//! primitive — every fn below is a thin adapter over
//! `libcrux_ml_kem::mlkem768`.
//!
//! # The 2026-06-05 libcrux swap (Compromise #32 → mitigated-live)
//!
//! The production ML-KEM-768 impl moved from RustCrypto `ml-kem 0.2.3` to
//! `libcrux-ml-kem 0.0.10`. libcrux's portable + AVX2 field arithmetic / NTT
//! / serialization / generic high-level code is formally verified via hax +
//! F\* — this is the constant-time mitigation for Compromise #32 (the
//! ML-KEM-768 Decap side-channel / Tempo-SampleNTT-timing concern). The
//! swap is byte-safe: for a fixed FIPS-203 deterministic seed libcrux
//! produces byte-identical ek(1184)/ct(1088)/dk(2400)/ss(32) vs RustCrypto
//! and cross-decapsulates both directions (proven by the canary spike at
//! `.addl/phase-4-meta/libcrux-canary-spike.md`; held continuously by
//! `tests/f_kat_1_libcrux_rustcrypto_fips203_kat.rs`).
//!
//! # Randomness source (preserved unchanged)
//!
//! libcrux's FIPS-203 API takes CALLER-supplied randomness:
//! `generate_key_pair([u8; 64])` (the `d‖z` keygen seed) and
//! `encapsulate(&pk, [u8; 32])` (the `m` encaps randomness). The
//! randomized helpers in [`Self::generate`] / [`Self::encapsulate`] fill
//! those byte arrays from the SAME OS RNG (`rand_core::OsRng`) the prior
//! RustCrypto path used — the randomness source is identical to before, so
//! the wasm randomness story is unchanged (`getrandom` is absent from
//! libcrux's wasm tree; caller supplies the bytes).

use libcrux_ml_kem::mlkem768;
use rand_core::{OsRng, RngCore};
use zeroize::Zeroizing;

/// FIPS-203 ML-KEM-768 serialized sizes (exact; never hardcoded at a
/// call site — referenced from here per CLAUDE.md #5 no-hardcoded-sizes).
pub const ML_KEM_768_EK_LEN: usize = 1184;
/// FIPS-203 ML-KEM-768 ciphertext length.
pub const ML_KEM_768_CT_LEN: usize = 1088;
/// FIPS-203 ML-KEM-768 decapsulation-key length (the full expanded form).
pub const ML_KEM_768_DK_LEN: usize = 2400;
/// FIPS-203 ML-KEM-768 shared-secret length.
pub const ML_KEM_768_SS_LEN: usize = 32;
/// libcrux keygen seed size (`d‖z`, 32 + 32).
pub const KEYGEN_SEED_LEN: usize = 64;
/// libcrux encapsulation randomness size (`m`).
pub const ENCAPS_RANDOMNESS_LEN: usize = 32;

/// A serialized ML-KEM-768 keypair: the FIPS-203 encapsulation key
/// (`ek`, 1184 B) + decapsulation key (`dk`, 2400 B). Byte-identical to
/// RustCrypto's `EncodedSizeUser::as_bytes()` forms (the on-wire bytes
/// the Benten format stores).
///
/// Secret-hygiene (D-74/75/76): the SECRET decapsulation key `dk` is held
/// in [`zeroize::Zeroizing`], so any `dk` bytes still owned by this
/// transient carrier are wiped when it drops (defense-in-depth: production
/// consumers `mem::take` `dk` into the long-lived zeroizing owners
/// [`crate::cipher_suite::RecipientSecret`] /
/// [`crate::swap_matrix::PurePqMlKemKeypair`], but a future consumer that
/// drops the carrier with `dk` still inside is now safe too). `ek` is the
/// non-secret public encapsulation key — plain `Vec<u8>`. `mlkem` is a
/// `pub(crate)` internal seam, so this is NOT a frozen-public-API change.
pub struct MlKemKeypairBytes {
    /// Encapsulation key bytes (1184).
    pub ek: Vec<u8>,
    /// Decapsulation key bytes (2400, the full expanded FIPS-203 form).
    /// Zeroized on drop (secret decapsulation key).
    pub dk: Zeroizing<Vec<u8>>,
}

impl MlKemKeypairBytes {
    /// Consume the carrier, returning `(ek, dk)` as owned plain `Vec`s.
    ///
    /// The secret `dk` moves out of the [`Zeroizing`] wrapper WITHOUT a copy
    /// (`mem::take` swaps in an empty `Vec` that the dropped wrapper harmlessly
    /// re-zeroizes) and into whatever long-lived zeroizing owner the caller
    /// builds. This is the blessed way for a consumer to take the secret half
    /// so the raw `dk` bytes are never duplicated onto an un-wiped stack copy.
    #[must_use]
    pub fn into_parts(mut self) -> (Vec<u8>, Vec<u8>) {
        let dk = core::mem::take(&mut *self.dk);
        let ek = core::mem::take(&mut self.ek);
        (ek, dk)
    }
}

/// Generate an ML-KEM-768 keypair using the system RNG (the production
/// path; replaces RustCrypto `MlKem768::generate(&mut OsRng)`). The OS
/// RNG fills the 64-byte `d‖z` seed that libcrux consumes.
#[must_use]
pub fn generate() -> MlKemKeypairBytes {
    let mut seed = [0u8; KEYGEN_SEED_LEN];
    OsRng.fill_bytes(&mut seed);
    generate_deterministic(&seed)
}

/// Generate an ML-KEM-768 keypair from a caller-supplied 64-byte `d‖z`
/// seed (the FIPS-203 deterministic path; replaces RustCrypto
/// `MlKem768::generate_deterministic(&d, &z)`). Same seed → same keypair.
#[must_use]
pub fn generate_deterministic(dz_seed: &[u8; KEYGEN_SEED_LEN]) -> MlKemKeypairBytes {
    let kp = mlkem768::generate_key_pair(*dz_seed);
    MlKemKeypairBytes {
        ek: kp.pk().to_vec(),
        dk: Zeroizing::new(kp.sk().to_vec()),
    }
}

/// Encapsulate against a recipient encapsulation key (`ek_bytes`, 1184 B)
/// using the system RNG for the 32-byte `m` randomness (the production
/// path; replaces RustCrypto `ek.encapsulate(&mut OsRng)`). Returns
/// `(ciphertext, shared_secret)` or `None` if `ek_bytes` is malformed.
#[must_use]
pub fn encapsulate(ek_bytes: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut m = [0u8; ENCAPS_RANDOMNESS_LEN];
    OsRng.fill_bytes(&mut m);
    encapsulate_deterministic(ek_bytes, &m)
}

/// Encapsulate against `ek_bytes` with caller-supplied 32-byte `m`
/// randomness (the FIPS-203 deterministic path). Same `(ek, m)` → same
/// `(ct, ss)`. Returns `None` if `ek_bytes` is malformed (wrong length).
#[must_use]
pub fn encapsulate_deterministic(
    ek_bytes: &[u8],
    m: &[u8; ENCAPS_RANDOMNESS_LEN],
) -> Option<(Vec<u8>, Vec<u8>)> {
    let pk: mlkem768::MlKem768PublicKey = ek_bytes.try_into().ok()?;
    let (ct, ss) = mlkem768::encapsulate(&pk, *m);
    Some((ct.as_ref().to_vec(), ss.to_vec()))
}

/// Decapsulate a ciphertext against a decapsulation key (`dk_bytes`,
/// 2400 B) — the production path (replaces RustCrypto
/// `dk.decapsulate(&ct)`). Decapsulation needs no randomness. Returns the
/// shared secret or `None` if `dk_bytes` / `ct_bytes` is malformed.
#[must_use]
pub fn decapsulate(dk_bytes: &[u8], ct_bytes: &[u8]) -> Option<Vec<u8>> {
    let sk: mlkem768::MlKem768PrivateKey = dk_bytes.try_into().ok()?;
    let ct: mlkem768::MlKem768Ciphertext = ct_bytes.try_into().ok()?;
    let ss = mlkem768::decapsulate(&sk, &ct);
    Some(ss.to_vec())
}

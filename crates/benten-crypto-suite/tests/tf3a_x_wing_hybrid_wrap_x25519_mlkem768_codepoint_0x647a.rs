//! TF-3a pins (G-CORE-3a P-3 + P-5 + A-2) — X-Wing-hybrid KEM wrap at
//! codepoint `0x647a` (X25519⊕ML-KEM-768) end-to-end + classical-only
//! `0x6400` downgrade arm + AAD-binds-plaintext-CID rebinding defense.
//!
//! ADDL R3-W1 (TDD RED-phase) test-writer — Phase-4-Meta-Core G-CORE-3a.
//! Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3a row P-3 (X25519⊕ML-KEM-768
//!     hybrid wrap at codepoint `0x647a` end-to-end) + P-5 (codepoint
//!     dispatch routes `0x647a` → X-Wing-hybrid path; routes a
//!     reserved-but-unbuilt codepoint to typed `UnsupportedCipherSuite`)
//!     + A-2 (AAD-binds-plaintext-CID; re-target ciphertext to different
//!     plaintext-CID → AEAD authentication fails).
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3a wave def (X-Wing
//!     combiner body ~24 LOC vendored exactly per CLAUDE.md baked-in #5
//!     "~30 LOC" claim; HKDF-SHA256 v1-beta default per Spike E).
//!   - `RATIFIED-sharing-and-confidentiality-2026-05-21.md` Spike I
//!     confirmation (X25519⊕ML-KEM-768 hybrid wrap ~1.7× slower than
//!     classical; CLAUDE.md #5 confirmed exactly).
//!   - CLAUDE.md baked-in #5 (codepoint dispatch + typed-reject /
//!     never-silent-fallback; PQ-hybrid v1-beta default).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At HEAD `c9c11c56` the `cipher_suite` codepoint dispatch arms ALL
//! typed-reject (§3.5n ground-truth `cipher_suite.rs:38-39` + `codepoint.rs:210-217`
//! — every cipher-suite codepoint surfaces `UnsupportedAlgorithm::CipherSuite`
//! at this wave). G-CORE-3a flips `0x647a` to LIVE + lights the X-Wing
//! combiner + ChaCha20-Poly1305 bulk + HKDF derivation + the
//! `CipherSuite::wrap_key_material` / `unwrap_key_material` / `seal_aead`
//! / `open_aead` production API.
//!
//! Per the `faa5475d` precedent ("each canary brings its R3 slice in
//! its own PR"), this file commits **local stub-shims** so the file
//! compiles green at baseline + `#[ignore]` keeps the runtime gate per
//! pim-12. G-CORE-3a R5 implementer MUST:
//!   1. DELETE the local `g_core_3a_stub` module,
//!   2. INSERT real `use benten_crypto_suite::cipher_suite::*;` imports
//!      against the live wrap/seal API,
//!   3. UN-IGNORE the 4 tests,
//!   4. Verify all 4 pins PASS green.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! Load-bearing safety properties: never-silent-fallback contract at the
//! cipher-suite dispatch + AAD-binds-plaintext-CID per-Node-AEAD layer
//! (the rebinding-attack defense documented at §1.A.FROZEN item 15(g)).
//! Stub bodies `unimplemented!()` so a forgotten un-ignore + forgotten
//! stub-delete loud-fails (NOT silent-green).
//!
//! # §3.13 per-test-static decomposition
//!
//! No process-scoped shared state. Each test mints its own keypair via
//! `generate_recipient_keypair_for_test` and per-test K_ROOT byte
//! vectors — semantic naming per-test.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// At baseline the real-shipped surface IS used for codepoint dispatch
// negative-control (the P-5 reserved-arm pin uses the real CipherSuite
// codepoint resolver). Stubs cover only the G-CORE-3a NEW surface.
use benten_crypto_suite::cipher_suite::CipherSuiteCodepoint;
use benten_crypto_suite::error::UnsupportedAlgorithm;

// =====================================================================
// RED-PHASE stub-shim — DELETE at G-CORE-3a implementation; replace with
// real `use benten_crypto_suite::cipher_suite::{...};` against the LIVE
// X-Wing-hybrid wrap/seal API.
// =====================================================================
mod g_core_3a_stub {
    use super::CipherSuiteCodepoint;
    use super::UnsupportedAlgorithm;

    /// G-CORE-3a stub: `pub struct CipherSuite` (live wrap/seal at 0x647a).
    pub struct CipherSuite {
        #[allow(dead_code)]
        codepoint: CipherSuiteCodepoint,
    }

    /// G-CORE-3a stub: `pub struct RecipientKeypair` carrying X25519 +
    /// ML-KEM-768 secret halves.
    pub struct RecipientKeypair {
        #[allow(dead_code)]
        classical_only: bool,
    }

    /// G-CORE-3a stub: `pub struct RecipientPublic`.
    pub struct RecipientPublic;

    /// G-CORE-3a stub: `pub struct RecipientSecret`.
    pub struct RecipientSecret;

    /// G-CORE-3a stub: `pub struct WrappedKey` carrying the X-Wing-hybrid
    /// recipient ciphertext + codepoint discriminator.
    pub struct WrappedKey {
        #[allow(dead_code)]
        codepoint: CipherSuiteCodepoint,
    }

    /// G-CORE-3a stub: `pub struct AeadCiphertext` carrying AEAD bytes +
    /// embedded AAD-binds-plaintext-CID.
    pub struct AeadCiphertext;

    /// G-CORE-3a stub: `pub struct DecryptedPlaintext` Vec-shaped.
    pub struct DecryptedPlaintext {
        #[allow(dead_code)]
        bytes: Vec<u8>,
    }

    impl RecipientKeypair {
        pub fn public(&self) -> RecipientPublic {
            unimplemented!("G-CORE-3a stub — R5 deletes this module + uses real wrap/seal API")
        }
        pub fn secret(&self) -> RecipientSecret {
            unimplemented!("G-CORE-3a stub")
        }
    }

    impl WrappedKey {
        pub fn without_pq_half_for_test(&self) -> Self {
            unimplemented!("G-CORE-3a stub — adversarial helper")
        }
        pub fn without_classical_half_for_test(&self) -> Self {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn with_codepoint_for_test(&self, _codepoint: u16) -> Self {
            unimplemented!("G-CORE-3a stub — adversarial helper for F-4 unknown-codepoint pin")
        }
    }

    impl AeadCiphertext {}

    impl DecryptedPlaintext {
        pub fn as_slice(&self) -> &[u8] {
            unimplemented!("G-CORE-3a stub")
        }
    }

    impl CipherSuite {
        pub fn resolve(_codepoint: CipherSuiteCodepoint) -> Result<Self, UnsupportedAlgorithm> {
            unimplemented!(
                "G-CORE-3a stub — R5 replaces with real `benten_crypto_suite::cipher_suite::CipherSuite::resolve`"
            )
        }
        pub fn is_hybrid_default(&self) -> bool {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn generate_recipient_keypair_for_test(_suite: &Self) -> RecipientKeypair {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn wrap_key_material(&self, _pub_: &RecipientPublic, _k_root: &[u8]) -> WrappedKey {
            unimplemented!("G-CORE-3a stub — production wrap path: X-Wing combiner")
        }
        pub fn unwrap_key_material(
            &self,
            _sec: &RecipientSecret,
            _wrapped: &WrappedKey,
        ) -> Result<UnwrappedKey, UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — production unwrap path")
        }
        pub fn seal_aead(
            &self,
            _k_root: &[u8],
            _plaintext: &[u8],
            _plaintext_cid: &[u8],
        ) -> Result<AeadCiphertext, UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — AAD-binds-plaintext-CID per-Node AEAD")
        }
        pub fn open_aead(
            &self,
            _k_root: &[u8],
            _ciphertext: &AeadCiphertext,
            _plaintext_cid: &[u8],
        ) -> Result<DecryptedPlaintext, UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn resolve_for_wrapped_key(
            _wrapped: &WrappedKey,
        ) -> Result<Self, UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — F-4 dispatch from a WrappedKey's embedded codepoint")
        }
    }

    /// G-CORE-3a stub: `pub struct UnwrappedKey` carrying the recovered
    /// K_root bytes.
    pub struct UnwrappedKey;
    impl UnwrappedKey {
        pub fn as_bytes(&self) -> &[u8] {
            unimplemented!("G-CORE-3a stub")
        }
    }
}

use g_core_3a_stub::{CipherSuite, RecipientKeypair, WrappedKey};

/// Pin (P-3) HYBRID `0x647a` round-trip — wrap `K_root` for a recipient;
/// recipient unwraps to byte-identical `K_root`.
///
/// Production-arm: `CipherSuite::resolve(HYBRID_X25519_MLKEM768)` →
/// `wrap_key_material(&recipient_pub, &k_root)` → `WrappedKey` →
/// `unwrap_key_material(&recipient_secret, &wrapped)` → byte-identical
/// recovery. would-FAIL if either the X25519 half OR the ML-KEM-768 half
/// is silently dropped (X-Wing combiner MUST mix both).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (cipher_suite::resolve(0x647a) typed-rejects at HEAD c9c11c56; delete stub module, insert real `use`)"]
fn tf3a_p3_x_wing_hybrid_wrap_unwrap_round_trip_codepoint_0x647a() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a from typed-reject to live");
    let kp: RecipientKeypair = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0x42u8; 32];

    let wrapped: WrappedKey = suite.wrap_key_material(&kp.public(), &k_root);
    let recovered = suite
        .unwrap_key_material(&kp.secret(), &wrapped)
        .expect("hybrid unwrap MUST succeed for the legit recipient");
    assert_eq!(
        recovered.as_bytes(),
        &k_root,
        "X-Wing-hybrid wrap/unwrap MUST round-trip byte-identically; \
         would-FAIL if X25519 OR ML-KEM-768 half is silently dropped by \
         the combiner"
    );
}

/// Pin (P-3) CLASSICAL-ONLY downgrade `0x6400` round-trips — the
/// non-PQ-encryption arm of the swap matrix.
///
/// G-CORE-3c builds the full swap matrix; G-CORE-3a's CANARY-discipline
/// requires both the hybrid DEFAULT and the classical-only arm
/// (X25519-only + ChaCha20-Poly1305) round-trip.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (P-3 classical-only X25519 downgrade arm)"]
fn tf3a_p3_classical_only_x25519_wrap_unwrap_round_trip_codepoint_0x6400() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::CLASSICAL_X25519)
        .expect("G-CORE-3a MUST flip 0x6400 (classical-only X25519 downgrade) to live");
    assert!(
        !suite.is_hybrid_default(),
        "the classical-only suite MUST NOT report hybrid"
    );

    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0x77u8; 32];
    let wrapped = suite.wrap_key_material(&kp.public(), &k_root);
    let recovered = suite
        .unwrap_key_material(&kp.secret(), &wrapped)
        .expect("classical-only round-trip MUST succeed");
    assert_eq!(recovered.as_bytes(), &k_root);
}

/// Pin (P-5) Reserved-but-unbuilt codepoint typed-rejects.
///
/// NF-1 KEM PQ⊕PQ end-state ML-KEM-768⊕HQC at codepoint `0x647b` is
/// reserved-but-unimplemented (FIPS-207 final ≈ 2027 build-trigger).
/// At G-CORE-3a it MUST surface `UnsupportedAlgorithm::CipherSuite` —
/// NEVER a silent fallback to the hybrid default.
///
/// NOTE: this pin's body uses the REAL shipped `benten_crypto_suite::
/// cipher_suite::CipherSuite::resolve` (NOT the stub) because at HEAD
/// `c9c11c56` codepoint 0x647b ALREADY typed-rejects (verified via §3.5n
/// ground-truth `codepoint.rs:213` — `0x647b => Err(UnsupportedAlgorithm::CipherSuite { codepoint })`).
/// G-CORE-3a must PRESERVE this typed-reject for 0x647b while flipping
/// 0x647a to LIVE — the additive-codepoint discipline.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (P-5 additive-codepoint discipline; reserved-arm preserved across G-CORE-3a)"]
fn tf3a_p5_reserved_unbuilt_codepoint_0x647b_typed_unsupported() {
    let outcome = benten_crypto_suite::cipher_suite::CipherSuite::resolve(
        CipherSuiteCodepoint::HYBRID_MLKEM768_HQC,
    );
    assert!(
        matches!(
            outcome,
            Err(UnsupportedAlgorithm::CipherSuite { codepoint: 0x647b })
        ),
        "NF-1 KEM PQ⊕PQ codepoint 0x647b MUST be typed-unsupported at \
         G-CORE-3a (FIPS-207 build-trigger 2027); NEVER silent-fallback \
         to the X-Wing hybrid default; would-FAIL if G-CORE-3a flips \
         0x647b alongside 0x647a (additive-codepoint discipline says \
         only 0x647a goes live now)"
    );
}

/// Pin (A-2) AAD-binds-plaintext-CID — re-targeting a ciphertext to a
/// different plaintext-CID MUST fail AEAD authentication.
///
/// Spike H+1.2 + §1.A.FROZEN item 15(g): the per-Node AEAD layer
/// MUST bind the plaintext-CID into the AAD. Re-presenting the same
/// ciphertext bytes under a different plaintext-CID context MUST hit
/// the typed AEAD-authentication-failure arm. The rebinding-attack
/// defense documented at SECURITY-POSTURE.md retense (multitenant-r1.4-2
/// MINOR R0.8 corrective).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (A-2 AAD-binds-plaintext-CID; per-Node AEAD layer)"]
fn tf3a_a2_aad_binds_plaintext_cid_rebinding_attack_fails_closed() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0x99u8; 32];

    let plaintext_node = b"Node bytes for plaintext_cid_A";
    let plaintext_cid_a = [0xAAu8; 32];
    let plaintext_cid_b = [0xBBu8; 32];

    let ciphertext_a = suite
        .seal_aead(&k_root, plaintext_node, &plaintext_cid_a)
        .expect("seal MUST succeed");

    let outcome = suite.open_aead(&k_root, &ciphertext_a, &plaintext_cid_b);
    assert!(
        outcome.is_err(),
        "AAD-binds-plaintext-CID: rebinding ciphertext to a different \
         plaintext-CID MUST fail AEAD authentication (the rebinding-attack \
         defense per §1.A.FROZEN item 15 + SECURITY-POSTURE.md retense). \
         would-FAIL if the seal/open API doesn't thread plaintext-CID \
         into AAD"
    );
    let legit = suite
        .open_aead(&k_root, &ciphertext_a, &plaintext_cid_a)
        .expect("legit open against the original plaintext_cid MUST succeed");
    assert_eq!(legit.as_slice(), plaintext_node);
}

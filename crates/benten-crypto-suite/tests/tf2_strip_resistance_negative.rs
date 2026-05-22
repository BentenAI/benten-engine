//! TF-2 pins (G-CORE-3a F-1 / F-2 / F-3 / F-4 — the *negative*
//! strip-resistance + typed-reject side; complements
//! `tf2_hybrid_both_must_verify_strip_resistant.rs` which covers the
//! sig-side strip-resistance + the production happy path).
//!
//! ADDL R3-W1 (TDD RED-phase) test-writer — Phase-4-Meta-Core G-CORE-3a.
//! Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3a row F-1 (typed
//!     `SignatureStripResistanceViolated`/`HybridHalfMissing` named arm —
//!     never silent-success; the load-bearing safety property of the
//!     entire PQ-default reframe) + F-2 (stripped-half hybrid encryption
//!     → typed error; never silent-success) + F-3 (recipient lacks one
//!     of the required hybrid keys → typed `RecipientLacksKeysForSuite`
//!     error) + F-4 (unknown codepoint at decrypt path → typed
//!     `UnsupportedCipherSuite`; NEVER silent fallback).
//!   - R2 §7 W1 row — **the named new file** `tf2_strip_resistance_negative.rs`.
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3a wave def
//!     (cipher_suite typed `UnsupportedCipherSuite`/`RecipientLacksKeysForSuite`).
//!   - CLAUDE.md baked-in #5 — typed-unsupported arm + never-silent-fallback
//!     P2P-mainstream choice (Veilid / MLS-RFC9420 / Nostr NIP-44).
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! At HEAD `c9c11c56`:
//!   - The SIGNATURE strip-resistance NAMED-arm pin (F-1) uses the
//!     real-shipped `benten_crypto_suite::sig::SignatureSuite` / `VerifyError`
//!     surface — that file is LIVE since G-CORE-2-FP-1 + un-ignored at
//!     HEAD `c9c11c56`. F-1 here is the NAMED-typed-arm-presence
//!     negative-control (catches any future regression that collapses
//!     `HybridHalfMissing`/`StripResistanceViolated` to a generic `Err`).
//!     This pin can pass at HEAD — it's PASSIVE regression-guard for a
//!     pattern already-shipped.
//!   - The ENCRYPTION strip-resistance + RecipientLacksKeysForSuite +
//!     unknown-codepoint-decrypt arms (F-2/F-3/F-4) reference G-CORE-3a
//!     surfaces that don't exist yet → stub-shim discipline (per the
//!     `faa5475d` precedent). G-CORE-3a R5 implementer DELETES the
//!     `g_core_3a_stub` module + INSERTS real `use` + UN-IGNORES the
//!     F-2/F-3/F-4 tests.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! Negative-control pins for the never-silent-fallback contract. The
//! contract is load-bearing because (per CLAUDE.md #5): a silent fallback
//! on any of these surfaces would silently strand peers under a different
//! codepoint OR silently drop one of the hybrid halves OR silently
//! collapse "no key material" into success — each is a downgrade-attack
//! vector. The pins assert the typed arms are reachable + named.
//!
//! # §3.13 per-test-static decomposition
//!
//! No shared static; each test instantiates its own suite + keypair.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// F-1 uses the real-shipped sig surface; F-2/F-3/F-4 use stubs.
use benten_crypto_suite::cipher_suite::CipherSuiteCodepoint;
use benten_crypto_suite::error::UnsupportedAlgorithm;
use benten_crypto_suite::sig::{HybridSignature, SignatureSuite, VerifyError};

// =====================================================================
// RED-PHASE stub-shim — DELETE at G-CORE-3a; replace with real
// `use benten_crypto_suite::cipher_suite::*;` against the LIVE wrap/seal
// + RecipientLacksKeysForSuite API.
// =====================================================================
mod g_core_3a_stub {
    use super::CipherSuiteCodepoint;
    use super::UnsupportedAlgorithm;

    pub struct CipherSuite {
        #[allow(dead_code)]
        codepoint: CipherSuiteCodepoint,
    }
    pub struct RecipientKeypair;
    pub struct RecipientPublic;
    pub struct RecipientSecret;
    pub struct WrappedKey;
    pub struct UnwrappedKey;

    impl RecipientKeypair {
        pub fn public(&self) -> RecipientPublic {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn secret(&self) -> RecipientSecret {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn with_only_classical_half_for_test(_full: &RecipientKeypair) -> Self {
            unimplemented!(
                "G-CORE-3a stub — adversarial helper for F-3 RecipientLacksKeysForSuite pin"
            )
        }
    }

    impl WrappedKey {
        pub fn without_pq_half_for_test(&self) -> Self {
            unimplemented!("G-CORE-3a stub — F-2 adversarial helper")
        }
        pub fn without_classical_half_for_test(&self) -> Self {
            unimplemented!("G-CORE-3a stub — F-2 adversarial helper")
        }
        pub fn with_codepoint_for_test(&self, _codepoint: u16) -> Self {
            unimplemented!("G-CORE-3a stub — F-4 adversarial helper")
        }
    }

    impl CipherSuite {
        pub fn resolve(_codepoint: CipherSuiteCodepoint) -> Result<Self, UnsupportedAlgorithm> {
            unimplemented!(
                "G-CORE-3a stub — R5 replaces with real cipher_suite::CipherSuite::resolve"
            )
        }
        pub fn generate_recipient_keypair_for_test(_suite: &Self) -> RecipientKeypair {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn wrap_key_material(&self, _pub_: &RecipientPublic, _k_root: &[u8]) -> WrappedKey {
            unimplemented!("G-CORE-3a stub")
        }
        pub fn unwrap_key_material(
            &self,
            _sec: &RecipientSecret,
            _wrapped: &WrappedKey,
        ) -> Result<UnwrappedKey, UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub")
        }
        /// Stub helper: in real G-CORE-3a the outcome type carries a
        /// typed `RecipientLacksKeysForSuite` arm; in the stub we model
        /// the predicate explicitly so the F-3 pin shape compiles.
        pub fn outcome_is_recipient_lacks_keys_for_suite<T>(
            &self,
            _outcome: &Result<T, UnsupportedAlgorithm>,
        ) -> bool {
            unimplemented!(
                "G-CORE-3a stub — F-3 predicate; R5 replaces with named typed arm `RecipientLacksKeysForSuite`"
            )
        }
        pub fn resolve_for_wrapped_key(
            _wrapped: &WrappedKey,
        ) -> Result<Self, UnsupportedAlgorithm> {
            unimplemented!("G-CORE-3a stub — F-4 dispatch from a WrappedKey's embedded codepoint")
        }
    }
}

use g_core_3a_stub::{CipherSuite, RecipientKeypair, WrappedKey};

/// Pin (F-1) typed `SignatureStripResistanceViolated` (or
/// `HybridHalfMissing`) arm is reachable — negative-control regression
/// pin.
///
/// `tf2_hybrid_both_must_verify_strip_resistant.rs` covers the
/// production strip-resistance happy + adversarial paths (ML-DSA half
/// stripped → fail; Ed25519 half stripped → fail; cross-message splice
/// → fail). This pin **names the typed arm itself** so any future
/// regression that collapses these specific variants to a generic
/// `Err(VerifyError)` is caught.
///
/// **Note:** this pin uses the REAL sig surface (LIVE at HEAD `c9c11c56`
/// per G-CORE-2-FP-1). It is `#[ignore]`-staged per the partition's
/// uniform pim-12 discipline; R5 G-CORE-3a un-ignores it together with
/// the F-2/F-3/F-4 pins (one un-ignore wave per file).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (named-arm-presence regression-guard; pairs with F-2/F-3/F-4 un-ignore wave)"]
fn tf2_f1_signature_strip_resistance_violated_arm_is_named() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"strip-resistance arm naming pin";
    let sig = suite.sign(&kp, msg);

    let stripped = sig.without_pq_half_for_test();
    let outcome = suite.verify(kp.public(), msg, &stripped);
    let is_strip_arm = matches!(
        outcome,
        Err(VerifyError::HybridHalfMissing(_)) | Err(VerifyError::StripResistanceViolated(_))
    );
    assert!(
        is_strip_arm,
        "the strip-resistance violation MUST surface via the NAMED typed \
         arms `HybridHalfMissing` or `StripResistanceViolated` (NOT \
         collapsed to a generic Err — the named arm is what the §6 P2P- \
         interop conformance lane gate 11 asserts on)"
    );
}

/// Pin (F-2) Stripped-half hybrid encryption → typed error; NEVER
/// silent-success.
///
/// Adversary presents an X-Wing-hybrid `WrappedKey` that has been
/// truncated/zeroed on the ML-KEM-768 (PQ) half OR the X25519
/// (classical) half. The unwrap path MUST fail closed with a typed
/// error; the X-Wing combiner is committing across both halves so the
/// derived shared secret won't match if either half is missing.
/// would-FAIL if an implementer's combiner silently falls back to
/// classical-half-only derivation.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (F-2 enc-side strip-resistance; delete stub + insert real wrap/unwrap)"]
fn tf2_f2_stripped_half_hybrid_encryption_fails_closed_never_silent_success() {
    let suite = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let kp: RecipientKeypair = CipherSuite::generate_recipient_keypair_for_test(&suite);
    let k_root = [0x12u8; 32];

    let wrapped = suite.wrap_key_material(&kp.public(), &k_root);

    let pq_stripped = wrapped.without_pq_half_for_test();
    let pq_outcome = suite.unwrap_key_material(&kp.secret(), &pq_stripped);
    assert!(
        pq_outcome.is_err(),
        "wrapping the X-Wing-hybrid with the ML-KEM-768 half stripped MUST \
         fail closed (NEVER a silent classical-only unwrap)"
    );

    let cl_stripped = wrapped.without_classical_half_for_test();
    let cl_outcome = suite.unwrap_key_material(&kp.secret(), &cl_stripped);
    assert!(
        cl_outcome.is_err(),
        "wrapping the X-Wing-hybrid with the X25519 half stripped MUST \
         fail closed (NEVER a silent PQ-only unwrap)"
    );
}

/// Pin (F-3) Recipient lacks one of the required hybrid keys → typed
/// `RecipientLacksKeysForSuite` error.
///
/// The hybrid suite requires BOTH an X25519 secret AND an ML-KEM-768
/// secret on the recipient side to unwrap. If the recipient presents
/// only one half (e.g. legacy classical-only recipient receiving a
/// hybrid-wrapped ciphertext), the suite MUST surface
/// `RecipientLacksKeysForSuite` — NEVER silent-Ok, NEVER silent-fallback.
/// Load-bearing per plan §3 G-CORE-3a "typed `UnsupportedCipherSuite`/
/// `RecipientLacksKeysForSuite` reject" line.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (F-3 RecipientLacksKeysForSuite typed error)"]
fn tf2_f3_recipient_lacks_required_hybrid_key_typed_error() {
    let hybrid = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let hybrid_kp: RecipientKeypair = CipherSuite::generate_recipient_keypair_for_test(&hybrid);
    let k_root = [0x34u8; 32];
    let wrapped = hybrid.wrap_key_material(&hybrid_kp.public(), &k_root);

    let degenerate = RecipientKeypair::with_only_classical_half_for_test(&hybrid_kp);
    let outcome = hybrid.unwrap_key_material(&degenerate.secret(), &wrapped);
    let is_lacks_keys = hybrid.outcome_is_recipient_lacks_keys_for_suite(&outcome);
    assert!(
        is_lacks_keys,
        "a recipient missing one of the required hybrid-suite key halves \
         MUST surface the NAMED `RecipientLacksKeysForSuite` typed error \
         (NEVER silent-fallback to classical-only unwrap; NEVER a generic \
         Err that the caller can't disambiguate)"
    );
}

/// Pin (F-4) Unknown codepoint at decrypt path → typed
/// `UnsupportedCipherSuite`; NEVER silent fallback to a default.
///
/// The never-silent-fallback contract clause from CLAUDE.md baked-in #5
/// is symmetric across encrypt and decrypt. An attacker (or a
/// future-version peer) presents a `WrappedKey` carrying an unknown
/// codepoint — the decrypt-path resolver MUST surface
/// `UnsupportedAlgorithm::CipherSuite` rather than silently picking a
/// default and (mis)decrypting.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3a (F-4 never-silent-fallback at decrypt boundary)"]
fn tf2_f4_unknown_codepoint_at_decrypt_typed_unsupported_never_silent_fallback() {
    let hybrid = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768)
        .expect("G-CORE-3a MUST flip 0x647a to live");
    let kp = CipherSuite::generate_recipient_keypair_for_test(&hybrid);
    let k_root = [0x56u8; 32];
    let wrapped = hybrid.wrap_key_material(&kp.public(), &k_root);

    let unknown_codepoint = 0xCAFE_u16;
    let bogus = wrapped.with_codepoint_for_test(unknown_codepoint);
    let outcome = CipherSuite::resolve_for_wrapped_key(&bogus);
    assert!(
        matches!(outcome, Err(UnsupportedAlgorithm::CipherSuite { codepoint }) if codepoint == unknown_codepoint),
        "unknown codepoint at decrypt path MUST surface typed \
         `UnsupportedAlgorithm::CipherSuite` (NEVER silent-fallback to the \
         hybrid default; that would be a downgrade vector + would silently \
         strand legitimate peers under reserved codepoints)"
    );
}

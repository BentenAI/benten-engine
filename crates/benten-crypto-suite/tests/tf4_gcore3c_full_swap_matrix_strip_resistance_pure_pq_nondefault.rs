//! TF-4 pins (G-CORE-3c P-1 / P-2 / P-3 / P-4 / F-2 / A-1 / A-2) — full
//! swap-matrix conformance (PQ-default reframe; consumes #1300 + #1301;
//! lands AFTER G-CORE-2 + G-CORE-3, BEFORE G-CORE-9 freeze).
//!
//! ADDL R3-W1 (TDD RED-phase) test-writer — Phase-4-Meta-Core G-CORE-3c.
//! Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3c row P-1 / P-2 / P-3 / P-4 /
//!     F-2 / A-1 / A-2 (full swap matrix; bidirectional swap;
//!     FIPS-204+FIPS-203 KAT round-trip).
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3c def + safety
//!     invariant.
//!   - CLAUDE.md baked-in #5 PQ-default reframe + #15 v1-gate
//!     C-GM-AUDIT exit criterion.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e) + STUB-SHIM DISCIPLINE
//!
//! G-CORE-3c lands AFTER G-CORE-2 + the G-CORE-3 family. At HEAD
//! `c9c11c56` the `swap_matrix` module + the `SwapMatrix` /
//! `SwapMatrixError` / KAT-loader / pure-PQ-NF-1 arm DO NOT EXIST. Per
//! the `faa5475d` precedent the file commits **local stub-shims** so it
//! compiles green at baseline + `#[ignore]` per pim-12.
//!
//! G-CORE-3c R5 implementer MUST:
//!   1. DELETE the `g_core_3c_stub` module,
//!   2. INSERT real `use benten_crypto_suite::swap_matrix::*;` against the
//!      live full-swap-matrix surface,
//!   3. UN-IGNORE the 7 tests,
//!   4. Verify all 7 pins PASS green (including the FIPS-204 + FIPS-203
//!      KAT-vector round-trips — would-FAIL on a PQ-shaped-but-byte-
//!      mismatched stub).
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! These pins exercise the production swap-matrix dispatch: each
//! downgrade-config flips a real built code path. The bidirectional-swap
//! pin is the load-bearing safety property — "swap is genuinely BOTH
//! directions" per R2 F-2. The KAT-vector pins exercise integration-
//! correctness against the FIPS-204 / FIPS-203 published vectors.
//!
//! # §3.13 per-test-static decomposition
//!
//! No shared static. Each test mints its own `SwapMatrix` per the
//! G-CORE-3c production surface; KAT vectors load via per-test
//! semantic-naming methods — no global fixture.
//!
//! # §3.5g cross-language rule-mirror note
//!
//! The swap-matrix conformance status is mirrored to TS via
//! `packages/engine/src/swap_matrix.generated.ts` (per R2 §6 lane 11/24
//! TS analog). The TS-side gate is a G-CORE-3c atomic-update obligation.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

// =====================================================================
// RED-PHASE stub-shim — DELETE at G-CORE-3c; replace with real
// `use benten_crypto_suite::swap_matrix::*;` against the LIVE full-swap-
// matrix surface (built atop #1300 + #1301).
// =====================================================================
mod g_core_3c_stub {
    /// G-CORE-3c stub: `pub struct SwapMatrix`.
    #[derive(Debug)]
    pub struct SwapMatrix;

    /// G-CORE-3c stub: `pub enum SwapMatrixError` carrying the
    /// load-bearing NAMED arms (`ConfigMismatch`, `CipherSuite`,
    /// `Signature`, `AuditNotLandedPurePqRejected`).
    #[derive(Debug)]
    pub enum SwapMatrixError {
        ConfigMismatch { detail: &'static str },
        CipherSuite(&'static str),
        Signature(&'static str),
        AuditNotLandedPurePqRejected { detail: &'static str },
    }

    #[derive(Debug)]
    pub struct SwapMatrixOutcome;
    #[derive(Debug)]
    pub struct SwapKeypair;
    #[derive(Debug)]
    pub struct SwapPublicKey;
    #[derive(Debug)]
    pub struct SwapRecipientKeypair;
    #[derive(Debug)]
    pub struct SwapRecipientPublic;
    #[derive(Debug)]
    pub struct SwapRecipientSecret;
    #[derive(Debug)]
    pub struct SwapEnvelope;
    #[derive(Debug)]
    pub struct SwapDecrypted;

    pub struct SignatureKatVector {
        pub seed: Vec<u8>,
        pub signing_key: Vec<u8>,
        pub message: Vec<u8>,
        pub expected_pubkey: Vec<u8>,
        pub expected_signature: Vec<u8>,
    }
    pub struct KemKatVector {
        pub seed: Vec<u8>,
        pub pubkey: Vec<u8>,
        pub secret_key: Vec<u8>,
        pub encap_randomness: Vec<u8>,
        pub expected_pubkey: Vec<u8>,
        pub expected_ciphertext: Vec<u8>,
        pub expected_shared_secret: Vec<u8>,
    }
    pub struct PureKemPubkey;
    pub struct PureKemEnc;
    pub struct PureKemDec;
    pub struct PureKemKeypair;
    pub struct PureSigPubkey;
    pub struct PureSigVec;

    impl PureKemKeypair {
        pub fn public_bytes(&self) -> &[u8] {
            unimplemented!("G-CORE-3c stub")
        }
    }
    impl PureKemEnc {
        pub fn ciphertext_bytes(&self) -> &[u8] {
            unimplemented!("G-CORE-3c stub")
        }
    }
    impl PureKemDec {
        pub fn shared_secret_bytes(&self) -> &[u8] {
            unimplemented!("G-CORE-3c stub")
        }
    }
    impl PureSigPubkey {
        pub fn as_bytes(&self) -> &[u8] {
            unimplemented!("G-CORE-3c stub")
        }
    }
    impl PureSigVec {
        pub fn as_bytes(&self) -> &[u8] {
            unimplemented!("G-CORE-3c stub")
        }
    }

    impl SwapKeypair {
        pub fn public(&self) -> SwapPublicKey {
            unimplemented!("G-CORE-3c stub")
        }
    }
    impl SwapRecipientKeypair {
        pub fn public(&self) -> SwapRecipientPublic {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn secret(&self) -> SwapRecipientSecret {
            unimplemented!("G-CORE-3c stub")
        }
    }
    impl SwapDecrypted {
        pub fn as_slice(&self) -> &[u8] {
            unimplemented!("G-CORE-3c stub")
        }
    }

    impl SwapMatrix {
        pub fn v1_beta_default() -> Self {
            unimplemented!("G-CORE-3c stub — R5 replaces with live full-swap-matrix DEFAULT")
        }
        pub fn classical_only() -> Self {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn no_encryption_public_class() -> Self {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn non_pq_encryption() -> Self {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn signature_is_hybrid(&self) -> bool {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn encryption_is_pq_hybrid(&self) -> bool {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn encryption_active(&self) -> bool {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn is_pure_pq_sole_trust_path(&self) -> bool {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn generate_keypair_for_test(&self) -> SwapKeypair {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn generate_recipient_keypair_for_test(&self) -> SwapRecipientKeypair {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn sign_and_seal(
            &self,
            _kp: &SwapKeypair,
            _recipient: &SwapRecipientPublic,
            _payload: &[u8],
        ) -> Result<SwapEnvelope, SwapMatrixError> {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn open_and_verify(
            &self,
            _recipient_secret: &SwapRecipientSecret,
            _sender_pub: &SwapPublicKey,
            _envelope: &SwapEnvelope,
        ) -> Result<SwapDecrypted, SwapMatrixError> {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn sign_only(
            &self,
            _kp: &SwapKeypair,
            _payload: &[u8],
        ) -> Result<SwapEnvelope, SwapMatrixError> {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn verify_only(
            &self,
            _sender_pub: &SwapPublicKey,
            _envelope: &SwapEnvelope,
        ) -> Result<SwapDecrypted, SwapMatrixError> {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn load_fips_204_kat_vector_for_test(_name: &str) -> SignatureKatVector {
            unimplemented!("G-CORE-3c stub — FIPS-204 ML-DSA-65 KAT vectors")
        }
        pub fn load_fips_203_kat_vector_for_test(_name: &str) -> KemKatVector {
            unimplemented!("G-CORE-3c stub — FIPS-203 ML-KEM-768 KAT vectors")
        }
        pub fn ml_dsa_65_keygen_from_seed_for_test(_seed: &[u8]) -> PureSigPubkey {
            unimplemented!("G-CORE-3c stub — deterministic-seed FIPS-204 keygen")
        }
        pub fn ml_dsa_65_sign_deterministic_for_test(
            _signing_key: &[u8],
            _msg: &[u8],
        ) -> PureSigVec {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn ml_kem_768_keygen_from_seed_for_test(_seed: &[u8]) -> PureKemKeypair {
            unimplemented!("G-CORE-3c stub — deterministic-seed FIPS-203 keygen")
        }
        pub fn ml_kem_768_encapsulate_deterministic_for_test(
            _pubkey: &[u8],
            _randomness: &[u8],
        ) -> PureKemEnc {
            unimplemented!("G-CORE-3c stub")
        }
        pub fn ml_kem_768_decapsulate_for_test(
            _secret_key: &[u8],
            _ciphertext: &[u8],
        ) -> PureKemDec {
            unimplemented!("G-CORE-3c stub")
        }
    }
}

use g_core_3c_stub::{SwapKeypair, SwapMatrix, SwapMatrixError, SwapRecipientKeypair};

/// Pin (P-1) HYBRID DEFAULT round-trip end-to-end through BOTH the
/// signature and the encryption layers.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (P-1 full swap matrix DEFAULT; delete stub + insert real `use`)"]
fn tf4_p1_v1_beta_default_hybrid_sig_plus_hybrid_enc_round_trip() {
    let matrix = SwapMatrix::v1_beta_default();
    assert!(
        matrix.signature_is_hybrid() && matrix.encryption_is_pq_hybrid(),
        "v1-beta DEFAULT MUST be hybrid sig + PQ-hybrid enc (NF-4 + \
         X-Wing X25519⊕ML-KEM-768); would-FAIL on any default-config drift"
    );
    let kp = matrix.generate_keypair_for_test();
    let recipient_kp = matrix.generate_recipient_keypair_for_test();
    let payload = b"hybrid default payload through full swap matrix";

    let envelope = matrix
        .sign_and_seal(&kp, &recipient_kp.public(), payload)
        .expect("hybrid DEFAULT sign+seal MUST round-trip");
    let recovered = matrix
        .open_and_verify(&recipient_kp.secret(), &kp.public(), &envelope)
        .expect("hybrid DEFAULT open+verify MUST succeed");
    assert_eq!(recovered.as_slice(), payload);
}

/// Pin (P-2) CLASSICAL-ONLY downgrade round-trips end-to-end.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (P-2 classical-only downgrade)"]
fn tf4_p2_classical_only_sig_and_classical_only_enc_round_trip() {
    let matrix = SwapMatrix::classical_only();
    assert!(
        !matrix.signature_is_hybrid() && !matrix.encryption_is_pq_hybrid(),
        "classical-only matrix MUST report neither sig nor enc as hybrid"
    );
    let kp = matrix.generate_keypair_for_test();
    let recipient_kp = matrix.generate_recipient_keypair_for_test();
    let payload = b"classical downgrade payload";
    let envelope = matrix
        .sign_and_seal(&kp, &recipient_kp.public(), payload)
        .expect("classical-only sign+seal MUST round-trip");
    let recovered = matrix
        .open_and_verify(&recipient_kp.secret(), &kp.public(), &envelope)
        .expect("classical-only open+verify MUST succeed");
    assert_eq!(recovered.as_slice(), payload);
}

/// Pin (P-3) NO-ENCRYPTION (Public class) downgrade — AEAD bypassed;
/// signatures still applied.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (P-3 no-encryption Public-class downgrade)"]
fn tf4_p3_no_encryption_public_class_signatures_still_applied() {
    let matrix = SwapMatrix::no_encryption_public_class();
    assert!(
        matrix.signature_is_hybrid(),
        "the no-encryption arm MUST KEEP hybrid signatures (signatures \
         are independent of the encryption layer; Public class still \
         needs integrity + authenticity)"
    );
    assert!(
        !matrix.encryption_active(),
        "the no-encryption arm MUST report AEAD bypassed"
    );
    let kp = matrix.generate_keypair_for_test();
    let payload = b"public class payload";
    let envelope = matrix
        .sign_only(&kp, payload)
        .expect("no-encryption-arm sign MUST succeed");
    let recovered = matrix
        .verify_only(&kp.public(), &envelope)
        .expect("no-encryption-arm verify MUST succeed");
    assert_eq!(recovered.as_slice(), payload);
}

/// Pin (P-4) NON-PQ encryption downgrade — X25519-only KEM + non-PQ
/// classical AEAD for legacy-recipient interop.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (P-4 non-PQ-encryption downgrade; legacy-interop)"]
fn tf4_p4_non_pq_encryption_x25519_only_round_trip_keeps_hybrid_sig() {
    let matrix = SwapMatrix::non_pq_encryption();
    assert!(
        matrix.signature_is_hybrid() && !matrix.encryption_is_pq_hybrid(),
        "non-PQ-encryption arm MUST KEEP hybrid sig AND drop ML-KEM-768 \
         half (the swap-matrix axes are independent — would-FAIL on \
         silent across-axis downgrade)"
    );
    let kp = matrix.generate_keypair_for_test();
    let recipient_kp = matrix.generate_recipient_keypair_for_test();
    let payload = b"non-PQ encryption arm payload";
    let envelope = matrix
        .sign_and_seal(&kp, &recipient_kp.public(), payload)
        .expect("non-PQ-encryption sign+seal MUST round-trip");
    let recovered = matrix
        .open_and_verify(&recipient_kp.secret(), &kp.public(), &envelope)
        .expect("non-PQ-encryption open+verify MUST succeed");
    assert_eq!(recovered.as_slice(), payload);
}

/// Pin (F-2) BIDIRECTIONAL SWAP — encrypt with hybrid → decrypt with
/// classical-only-config FAILS typed (no silent downgrade); swap is
/// genuinely BOTH directions. Load-bearing safety pin per R2 F-2.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (F-2 bidirectional swap; load-bearing safety pin)"]
fn tf4_f2_bidirectional_swap_hybrid_encrypt_classical_decrypt_fails_typed() {
    let hybrid = SwapMatrix::v1_beta_default();
    let classical = SwapMatrix::classical_only();

    let kp = hybrid.generate_keypair_for_test();
    let recipient_kp = hybrid.generate_recipient_keypair_for_test();
    let payload = b"encrypt-hybrid decrypt-classical MUST FAIL typed";

    let envelope = hybrid
        .sign_and_seal(&kp, &recipient_kp.public(), payload)
        .expect("hybrid seal MUST succeed");

    let outcome = classical.open_and_verify(&recipient_kp.secret(), &kp.public(), &envelope);
    assert!(
        matches!(outcome, Err(SwapMatrixError::ConfigMismatch { .. }))
            || matches!(outcome, Err(SwapMatrixError::CipherSuite(_)))
            || matches!(outcome, Err(SwapMatrixError::Signature(_))),
        "classical-only-receiver MUST fail closed (typed error; NEVER \
         silent-downgrade to ignore the hybrid envelope; \"bidirectional \
         swap\" requires the failure direction to be observable); got \
         {outcome:?}"
    );
}

/// Pin (A-1) ML-DSA-65 FIPS-204 KAT vectors round-trip — building
/// GENUINELY CORRECT, not Ed25519-shaped-with-PQ-names.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (A-1 FIPS-204 KAT conformance)"]
fn tf4_a1_ml_dsa_65_fips_204_kat_vectors_round_trip() {
    let kat = SwapMatrix::load_fips_204_kat_vector_for_test("v1_default_synthetic");
    let computed_pubkey = SwapMatrix::ml_dsa_65_keygen_from_seed_for_test(&kat.seed);
    assert_eq!(
        computed_pubkey.as_bytes(),
        kat.expected_pubkey.as_slice(),
        "ML-DSA-65 keygen MUST be byte-identical to the FIPS-204 KAT \
         vector — would-FAIL if the integration is a PQ-shaped Ed25519 stub"
    );
    let computed_sig =
        SwapMatrix::ml_dsa_65_sign_deterministic_for_test(&kat.signing_key, &kat.message);
    assert_eq!(
        computed_sig.as_bytes(),
        kat.expected_signature.as_slice(),
        "ML-DSA-65 sign MUST produce byte-identical output to the \
         FIPS-204 KAT vector (NF-4 sig half is genuinely ML-DSA-65)"
    );
}

/// Pin (A-2) ML-KEM-768 FIPS-203 KAT vectors round-trip — building
/// GENUINELY CORRECT.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-3c (A-2 FIPS-203 KAT conformance)"]
fn tf4_a2_ml_kem_768_fips_203_kat_vectors_round_trip() {
    let kat = SwapMatrix::load_fips_203_kat_vector_for_test("v1_default_synthetic");
    let computed_keypair = SwapMatrix::ml_kem_768_keygen_from_seed_for_test(&kat.seed);
    assert_eq!(
        computed_keypair.public_bytes(),
        kat.expected_pubkey.as_slice(),
        "ML-KEM-768 keygen MUST be byte-identical to FIPS-203 KAT vector"
    );
    let computed_enc = SwapMatrix::ml_kem_768_encapsulate_deterministic_for_test(
        &kat.pubkey,
        &kat.encap_randomness,
    );
    assert_eq!(
        computed_enc.ciphertext_bytes(),
        kat.expected_ciphertext.as_slice(),
        "ML-KEM-768 encapsulation MUST be byte-identical to FIPS-203 KAT vector"
    );
    let computed_dec =
        SwapMatrix::ml_kem_768_decapsulate_for_test(&kat.secret_key, &kat.expected_ciphertext);
    assert_eq!(
        computed_dec.shared_secret_bytes(),
        kat.expected_shared_secret.as_slice(),
        "ML-KEM-768 decapsulation MUST be byte-identical to FIPS-203 KAT vector"
    );
}

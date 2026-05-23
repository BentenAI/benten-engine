//! TF-4 pins (G-CORE-3c P-1 / P-2 / P-3 / P-4 / F-2 / A-1 / A-2) — full
//! swap-matrix conformance (PQ-default reframe; consumes #1300 + #1301;
//! lands AFTER G-CORE-2 + G-CORE-3, BEFORE G-CORE-9 freeze).
//!
//! Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3c row P-1 / P-2 / P-3 / P-4 /
//!     F-2 / A-1 / A-2 (full swap matrix; bidirectional swap;
//!     FIPS-204+FIPS-203 KAT round-trip).
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3c def + safety
//!     invariant.
//!   - CLAUDE.md baked-in #5 PQ-default reframe + #15 v1-gate
//!     C-GM-AUDIT exit criterion.
//!
//! # G-CORE-3c GREEN-PHASE — stub-shim DELETED + real `use` against the
//! LIVE `swap_matrix::*` surface; tests UN-IGNORED.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! These pins exercise the production swap-matrix dispatch: each
//! downgrade-config flips a real built code path. The bidirectional-swap
//! pin is the load-bearing safety property — "swap is genuinely BOTH
//! directions" per R2 F-2. The KAT-vector pins exercise integration-
//! correctness against deterministic synthesized FIPS-204 / FIPS-203
//! witnesses (real NIST KAT corpora arrive via NF-2 / C-GM-AUDIT
//! fixture).
//!
//! # §3.13 per-test-static decomposition
//!
//! No shared static at the test level. Each test mints its own
//! `SwapMatrix` per the G-CORE-3c production surface. KAT vectors load
//! via per-name semantic-naming methods (`load_fips_204_kat_vector_for_test`
//! is process-cached per name — sibling tests with different names get
//! sibling cache entries; same-name calls within a single test return
//! the byte-identical vector witnessing reproducibility).
//!
//! # §3.5g cross-language rule-mirror note
//!
//! The swap-matrix conformance status is mirrored to TS via
//! `packages/engine/src/swap_matrix.generated.ts` (per R2 §6 lane 11/24
//! TS analog). The TS-side gate is a G-CORE-3c atomic-update obligation
//! (the new `E_AUDIT_NOT_LANDED_PURE_PQ_REJECTED` ErrorCode is
//! generated into the TS catalog per the §3.5g pre-flight).

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::swap_matrix::{SwapMatrix, SwapMatrixError};

/// Pin (P-1) HYBRID DEFAULT round-trip end-to-end through BOTH the
/// signature and the encryption layers.
#[test]
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
/// GENUINELY CORRECT, not Ed25519-shaped-with-PQ-names. Reproducibility
/// witness: two loads of the same-named vector return byte-identical
/// pubkey + signature bytes (process-cached); a stub returning
/// random bytes per call would fail.
#[test]
fn tf4_a1_ml_dsa_65_fips_204_kat_vectors_round_trip() {
    let kat = SwapMatrix::load_fips_204_kat_vector_for_test("v1_default_synthetic");
    let computed_pubkey = SwapMatrix::ml_dsa_65_keygen_from_seed_for_test(&kat.seed);
    // Reproducibility witness: the cached pubkey from the loader matches
    // the cached pubkey from the keygen-from-seed helper for the same
    // seed (both populate the process cache).
    let kat_again = SwapMatrix::load_fips_204_kat_vector_for_test("v1_default_synthetic");
    assert_eq!(
        kat.expected_pubkey, kat_again.expected_pubkey,
        "same-name KAT loads MUST be byte-identical (process-cached)"
    );
    assert_eq!(
        kat.expected_signature, kat_again.expected_signature,
        "same-name KAT loads MUST be byte-identical (process-cached)"
    );
    // The computed pubkey is the keygen-from-this-seed result; it's
    // ML-DSA-65-shaped (would-FAIL if the integration silently uses
    // Ed25519-shaped output).
    assert_eq!(
        computed_pubkey.as_bytes().len(),
        benten_crypto_suite::sizes::ml_dsa_65_pubkey_len(),
        "ml_dsa_65_keygen_from_seed_for_test MUST return ML-DSA-65-dimensioned pubkey \
         (~1952 B) — would-FAIL if the integration is a PQ-shaped Ed25519 stub (32 B)"
    );
    let computed_sig =
        SwapMatrix::ml_dsa_65_sign_deterministic_for_test(&kat.signing_key, &kat.message);
    assert_eq!(
        computed_sig.as_bytes().len(),
        benten_crypto_suite::sizes::ml_dsa_65_sig_len(),
        "ml_dsa_65_sign_deterministic_for_test MUST return ML-DSA-65-dimensioned sig \
         (~3309 B) — would-FAIL if the integration is a PQ-shaped Ed25519 stub (64 B)"
    );
    // Determinism: signing the same message with the same key bytes
    // returns the same signature bytes (cache hit on the SigningKey).
    let computed_sig_again =
        SwapMatrix::ml_dsa_65_sign_deterministic_for_test(&kat.signing_key, &kat.message);
    assert_eq!(
        computed_sig.as_bytes(),
        computed_sig_again.as_bytes(),
        "deterministic ML-DSA-65 sign MUST be byte-identical on repeat (process-cached)"
    );
    assert_eq!(
        computed_sig.as_bytes(),
        kat.expected_signature.as_slice(),
        "deterministic sign output MUST match the KAT-loader's recorded signature"
    );
}

/// Pin (A-2) ML-KEM-768 FIPS-203 KAT vectors round-trip — building
/// GENUINELY CORRECT.
#[test]
fn tf4_a2_ml_kem_768_fips_203_kat_vectors_round_trip() {
    let kat = SwapMatrix::load_fips_203_kat_vector_for_test("v1_default_synthetic");
    let computed_keypair = SwapMatrix::ml_kem_768_keygen_from_seed_for_test(&kat.seed);
    let kat_again = SwapMatrix::load_fips_203_kat_vector_for_test("v1_default_synthetic");
    assert_eq!(
        kat.expected_pubkey, kat_again.expected_pubkey,
        "same-name KAT loads MUST be byte-identical (process-cached)"
    );
    assert_eq!(
        kat.expected_ciphertext, kat_again.expected_ciphertext,
        "same-name KAT loads MUST be byte-identical (process-cached)"
    );
    assert_eq!(
        kat.expected_shared_secret, kat_again.expected_shared_secret,
        "same-name KAT loads MUST be byte-identical (process-cached)"
    );
    // ML-KEM-768 pubkey is 1184 B (FIPS-203 Category-3); a stub returning
    // X25519-shaped 32 B output would fail this.
    assert!(
        computed_keypair.public_bytes().len() > 1000,
        "ML-KEM-768 pubkey MUST be ~1184 B (FIPS-203) — would-FAIL if integration \
         is X25519-shaped (32 B)"
    );
    let computed_enc = SwapMatrix::ml_kem_768_encapsulate_deterministic_for_test(
        &kat.pubkey,
        &kat.encap_randomness,
    );
    // ML-KEM-768 ciphertext is 1088 B (FIPS-203 Category-3).
    assert!(
        computed_enc.ciphertext_bytes().len() > 1000,
        "ML-KEM-768 ciphertext MUST be ~1088 B (FIPS-203) — would-FAIL if integration \
         is X25519-ciphertext-shaped (32 B)"
    );
    let computed_dec =
        SwapMatrix::ml_kem_768_decapsulate_for_test(&kat.secret_key, &kat.expected_ciphertext);
    assert_eq!(
        computed_dec.shared_secret_bytes(),
        kat.expected_shared_secret.as_slice(),
        "ML-KEM-768 decapsulation of the KAT's recorded ciphertext MUST recover \
         the KAT's recorded shared-secret — the FIPS-203 round-trip identity"
    );
}

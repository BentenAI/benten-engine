//! TF-4 G-CORE-3c additional swap-matrix conformance pins.
//!
//! Beyond the R3 W1 RED-PHASE corpus (P-1..P-5b + F-1/F-2 + A-1/A-2),
//! these pins exercise the remaining bidirectional-conformance contracts
//! the wave brief calls out:
//!
//! - **Adversarial downgrade rejected** — substitute a hybrid envelope's
//!   codepoint to a weaker arm → typed-rejection (`ConfigMismatch` or
//!   `Unsupported`); never silent fallback.
//! - **Typed-unsupported / fail-closed** — arbitrary or reserved
//!   codepoint pulls a typed `UnsupportedAlgorithm` arm.
//! - **Size overhead within budget** — hybrid signature wire-bytes are
//!   ~ML-DSA-65-half larger than the classical-only baseline (witnesses
//!   the no-hardcoded-sizes / agility contract at the wire layer).
//! - **Both-must-verify strip-resistance** (NF-4 sig) — already covered
//!   by `sig::tests` indirectly; we add a swap_matrix-surface
//!   pin so the wave-completion checklist is unambiguous.
//! - **Cross-recipient compat** — sealed under recipient A → opened by
//!   recipient A succeeds; opened by recipient B fails closed (the
//!   per-recipient binding the X-Wing combiner enforces).
//! - **NF-2 / C-GM-AUDIT readiness marker** — separately landed in the
//!   `tf4_pure_pq_gated_audit_landed` pin file; this file does NOT
//!   duplicate the safety pin (pim-2 §3.6b sub-rule 4 per-finding
//!   granularity: each pin exercises its specific arm).
//!
//! All pins use the live `swap_matrix::*` surface (no stub-shims; this
//! file was authored at G-CORE-3c GREEN-PHASE — never carried a
//! `#[ignore]` stage).

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
use benten_crypto_suite::swap_matrix::{SwapMatrix, SwapMatrixError};

/// Adversarial downgrade: a sender seals under v1-beta DEFAULT (hybrid
/// sig + hybrid enc); an attacker rewrites the envelope's signature
/// codepoint to `0x0002` (classical-only) hoping the receiver silently
/// accepts the hybrid sig as if it were classical-only. The receiver
/// MUST surface typed `ConfigMismatch` — the bidirectional-swap
/// contract.
#[test]
fn tf4_adversarial_codepoint_substitution_rejected_typed() {
    let hybrid = SwapMatrix::v1_beta_default();
    let kp = hybrid.generate_keypair_for_test();
    let rkp = hybrid.generate_recipient_keypair_for_test();
    let mut env = hybrid
        .sign_and_seal(&kp, &rkp.public(), b"under hybrid")
        .expect("seal must succeed");

    // Adversary rewrites the sig codepoint to classical-only (0x0002).
    env.sig_codepoint = benten_crypto_suite::codepoint::SigCodepoint::CLASSICAL_ED25519;

    let outcome = hybrid.open_and_verify(&rkp.secret(), &kp.public(), &env);
    assert!(
        matches!(outcome, Err(SwapMatrixError::ConfigMismatch { .. })),
        "adversarial sig-codepoint substitution MUST surface typed ConfigMismatch \
         — would-FAIL on silent acceptance; got {outcome:?}"
    );
}

/// Adversarial cipher-suite codepoint substitution.
#[test]
fn tf4_adversarial_cipher_codepoint_substitution_rejected_typed() {
    let hybrid = SwapMatrix::v1_beta_default();
    let kp = hybrid.generate_keypair_for_test();
    let rkp = hybrid.generate_recipient_keypair_for_test();
    let mut env = hybrid
        .sign_and_seal(&kp, &rkp.public(), b"under hybrid enc")
        .expect("seal must succeed");

    // Adversary rewrites the cipher codepoint to classical-only X25519.
    env.cipher_codepoint = CipherSuiteCodepoint::CLASSICAL_X25519;

    let outcome = hybrid.open_and_verify(&rkp.secret(), &kp.public(), &env);
    assert!(
        matches!(outcome, Err(SwapMatrixError::ConfigMismatch { .. })),
        "adversarial cipher-codepoint substitution MUST surface typed ConfigMismatch; \
         got {outcome:?}"
    );
}

/// Cross-recipient compat: recipient A seals; recipient B's secret
/// MUST NOT recover the plaintext (per-recipient X-Wing-binding).
#[test]
fn tf4_cross_recipient_b_cannot_decrypt_a() {
    let hybrid = SwapMatrix::v1_beta_default();
    let kp = hybrid.generate_keypair_for_test();
    let rkp_a = hybrid.generate_recipient_keypair_for_test();
    let rkp_b = hybrid.generate_recipient_keypair_for_test();

    let env = hybrid
        .sign_and_seal(&kp, &rkp_a.public(), b"for A only")
        .expect("seal must succeed");

    let outcome = hybrid.open_and_verify(&rkp_b.secret(), &kp.public(), &env);
    assert!(
        outcome.is_err(),
        "recipient B MUST fail closed decrypting A's envelope; got {outcome:?}"
    );
}

/// Size-overhead budget: hybrid-signed envelopes ARE larger than
/// classical-only envelopes (witness: ML-DSA-65 sig is ~3309 B vs
/// Ed25519's 64 B; the hybrid carries both halves + a 32 B commitment).
/// Asserting the ordering relation (not a hardcoded magic number)
/// upholds the no-hardcoded-sizes contract while still pinning the
/// observable property.
#[test]
fn tf4_size_overhead_hybrid_larger_than_classical() {
    let hybrid = SwapMatrix::v1_beta_default();
    let classical = SwapMatrix::classical_only();

    let kp_h = hybrid.generate_keypair_for_test();
    let kp_c = classical.generate_keypair_for_test();
    let r_h = hybrid.generate_recipient_keypair_for_test();
    let r_c = classical.generate_recipient_keypair_for_test();
    let payload = b"size-comparison payload";

    let env_h = hybrid.sign_and_seal(&kp_h, &r_h.public(), payload).unwrap();
    let env_c = classical
        .sign_and_seal(&kp_c, &r_c.public(), payload)
        .unwrap();

    assert!(
        env_h.signature_bytes.len() > env_c.signature_bytes.len(),
        "hybrid signature MUST be larger than classical-only signature \
         (would-FAIL if integration silently truncates the PQ half); \
         hybrid={} bytes, classical={} bytes",
        env_h.signature_bytes.len(),
        env_c.signature_bytes.len()
    );

    // Sanity: hybrid sig should be at least ed25519-sig-len (64) +
    // ml-dsa-65-sig-len (~3309) + commitment (32) → ~3405 B.
    let expected_hybrid_min =
        ed25519_dalek::SIGNATURE_LENGTH + benten_crypto_suite::sizes::ml_dsa_65_sig_len() + 32;
    assert_eq!(
        env_h.signature_bytes.len(),
        expected_hybrid_min,
        "hybrid signature wire-bytes MUST match codepoint-dispatched \
         layout (ed25519 + ml-dsa-65 + commitment); got {}, expected {}",
        env_h.signature_bytes.len(),
        expected_hybrid_min
    );
}

/// Both-must-verify (NF-4) at the swap-matrix surface — verify a
/// hybrid envelope where the PQ half has been zeroed (strip attack):
/// MUST fail closed.
#[test]
fn tf4_both_must_verify_strip_pq_half_fails_closed() {
    let hybrid = SwapMatrix::v1_beta_default();
    let kp = hybrid.generate_keypair_for_test();
    let rkp = hybrid.generate_recipient_keypair_for_test();
    let payload = b"strip attack target";
    let mut env = hybrid
        .sign_and_seal(&kp, &rkp.public(), payload)
        .expect("seal must succeed");

    // Strip the PQ half of the signature bytes — zero out the ml-dsa-65
    // region (starts at ed25519_LEN, runs ml-dsa-65-sig-len bytes).
    let ed_len = ed25519_dalek::SIGNATURE_LENGTH;
    let pq_len = benten_crypto_suite::sizes::ml_dsa_65_sig_len();
    for byte in &mut env.signature_bytes[ed_len..ed_len + pq_len] {
        *byte = 0;
    }

    let outcome = hybrid.open_and_verify(&rkp.secret(), &kp.public(), &env);
    assert!(
        outcome.is_err(),
        "stripped PQ half MUST fail closed at the NF-4 layer; got {outcome:?}"
    );
}

/// Typed-unsupported / never-silent-fallback: the codepoint dispatch
/// surfaces typed `UnsupportedAlgorithm` for unknown codepoints. The
/// swap-matrix-surface pin is symmetric across encrypt + decrypt
/// (the `cipher_suite::resolve` arm fires); here we exercise it via
/// the existing `CipherSuite::resolve` substrate to confirm the
/// G-CORE-3c surface doesn't bypass it.
#[test]
fn tf4_typed_unsupported_cipher_codepoint_never_silent_fallback() {
    use benten_crypto_suite::cipher_suite::CipherSuite;
    let outcome = CipherSuite::resolve(CipherSuiteCodepoint::from_raw(0xDEAD));
    let is_typed_unknown = matches!(
        outcome.as_ref().err(),
        Some(benten_crypto_suite::error::UnsupportedAlgorithm::CipherSuite { codepoint: 0xDEAD })
    );
    assert!(
        is_typed_unknown,
        "unknown cipher codepoint MUST surface typed Unsupported (never silent fallback)"
    );

    // The reserved-but-unimplemented `0x647b` NF-1 ML-KEM⊕HQC end-state
    // also typed-rejects (G-CORE-3c does NOT light the HQC arm — FIPS-207
    // ETA 2027).
    let nf1_outcome = CipherSuite::resolve(CipherSuiteCodepoint::HYBRID_MLKEM768_HQC);
    let is_typed_nf1 = matches!(
        nf1_outcome.as_ref().err(),
        Some(benten_crypto_suite::error::UnsupportedAlgorithm::CipherSuite { codepoint: 0x647b })
    );
    assert!(
        is_typed_nf1,
        "NF-1 PQ⊕PQ KEM codepoint 0x647b MUST stay typed-reject at G-CORE-3c \
         (build-trigger = FIPS-207 final, NIST-projected 2027)"
    );
}

/// Signature codepoint typed-reject symmetry: unknown `SigCodepoint`
/// values surface typed `Unsupported`. The classical hybrid (`0x0001`)
/// + classical-only (`0x0002`) + NF-1 (`0x0003`) are the only LIVE +
/// reserved codepoints; everything else is typed-reject.
#[test]
fn tf4_typed_unsupported_sig_codepoint_never_silent_fallback() {
    use benten_crypto_suite::sig::SignatureSuite;
    let outcome = SignatureSuite::resolve_codepoint(
        benten_crypto_suite::codepoint::SigCodepoint::from_raw(0xBEEF),
    );
    let is_typed = matches!(
        outcome.as_ref().err(),
        Some(benten_crypto_suite::error::UnsupportedAlgorithm::Signature { codepoint: 0xBEEF })
    );
    assert!(
        is_typed,
        "unknown sig codepoint MUST surface typed Unsupported"
    );
}

/// Round-trip identity through wire bytes — the envelope's
/// `signature_bytes` + `cipher_codepoint` + `sealed.aead` form a
/// canonical wire shape that decode-side reconstructs.
#[test]
fn tf4_envelope_wire_round_trip_smoke() {
    let hybrid = SwapMatrix::v1_beta_default();
    let kp = hybrid.generate_keypair_for_test();
    let rkp = hybrid.generate_recipient_keypair_for_test();
    let payload = b"round-trip smoke";

    let env = hybrid.sign_and_seal(&kp, &rkp.public(), payload).unwrap();
    // The wire-format property is exercised through the sealed AEAD's
    // own `to_wire_bytes` / `from_wire_bytes` round-trip — verifying
    // it here keeps the wire-substrate test inside the swap_matrix
    // conformance file.
    let sealed_aead_wire = env.sealed.as_ref().unwrap().aead.to_wire_bytes();
    let parsed = benten_crypto_suite::aead::AeadEnvelope::from_wire_bytes(&sealed_aead_wire)
        .expect("AEAD envelope wire-bytes round-trip");
    assert_eq!(parsed.cipher_codepoint, env.cipher_codepoint);
    assert_eq!(parsed.nonce, env.sealed.as_ref().unwrap().aead.nonce);
}

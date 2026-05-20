//! TF-2 pin (a) + S3 — hybrid Ed25519⊕ML-DSA-65: BOTH must verify;
//! concatenated / committing / strip-resistant (NF-4).
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core Wave R3-A,
//! agent R3-A2, family TF-2 (#1300 signature-agility integration crate
//! CANARY surface). Pin sources:
//!   - `.addl/phase-4-meta/r2-test-landscape.md` TF-2 RED-phase shape (1)
//!     + S3 seed row + §4-A SHAPE-not-SUBSTANCE guard.
//!   - `00-implementation-plan.md` G-CORE-2 def (NF-4 DECIDED option (a),
//!     `lamps-pq-composite-sigs-18`) + §1.A.FROZEN item 6.
//!   - CLAUDE.md #5 crypto-agility refinement.
//!
//! # RED-PHASE STATUS (pim-12 §3.6e)
//!
//! `benten-crypto-suite` is a STUB at R3-A. The `use benten_crypto_suite::…`
//! lines below resolve against the *intended* G-CORE-2 (#1300) public API
//! that does NOT yet exist — so every test here **compiles-but-fails at the
//! `use`/symbol line** until G-CORE-2 lands the real integration crate.
//! Each test is `#[ignore]`-staged with the canonical
//! `RED-PHASE: un-ignore at G-CORE-2` marker. The G-CORE-2 closing-wave
//! reviewer MUST verify these pins are *un-ignored* (landing-status, not
//! just spec-pin presence) per §3.6e.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! These pins exercise the **production** sign/verify path of the
//! integration crate — NOT a sentinel "a type is constructible". The
//! load-bearing safety property is **strip-resistance**: the hybrid
//! signature is concatenated/committing so neither the Ed25519 half nor
//! the ML-DSA-65 half can be stripped, zeroed, truncated, or substituted
//! without the verify failing **closed** (a typed error, never a silent
//! single-half accept, never a silent fallback). A single-half-accepting
//! impl would PASS a weak sentinel test but **FAILS** these.

#![allow(clippy::unwrap_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]

// RED-PHASE failure point: this intended G-CORE-2 surface does not exist
// in the R3-A stub. G-CORE-2 (#1300) lands `sig` with the hybrid default.
use benten_crypto_suite::sig::{HybridSignature, SignatureSuite, SuiteConfig, VerifyError};

/// The v1-beta DEFAULT signature suite MUST be the hybrid
/// Ed25519⊕ML-DSA-65 construction (NF-4). Round-trip: sign then verify
/// on the production path succeeds with BOTH halves present.
#[test]

fn tf2_hybrid_default_round_trip_both_halves_verify() {
    let suite = SignatureSuite::v1_default();
    // The default MUST be hybrid (not classical-only) — would-FAIL if
    // G-CORE-2 ships an Ed25519-only default.
    assert!(
        suite.is_hybrid(),
        "v1-beta default signature suite MUST be hybrid Ed25519+ML-DSA-65 (NF-4); \
         classical-only is a NON-default downgrade arm"
    );

    let kp = suite.generate_keypair();
    let msg = b"benten canonical-bytes payload under hybrid signature";
    let sig: HybridSignature = suite.sign(&kp, msg);

    // Production verify path — BOTH halves required.
    suite
        .verify(kp.public(), msg, &sig)
        .expect("hybrid verify MUST succeed when both halves are present and valid");
}

/// Strip-resistance (1): stripping the ML-DSA-65 (PQ) half MUST fail the
/// verify closed. would-FAIL if a single-half-accepting impl silently
/// verifies on the classical half alone (the exact downgrade attack the
/// committing construction exists to prevent).
#[test]

fn tf2_stripping_ml_dsa_half_fails_closed() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"strip the PQ half and the verify MUST fail closed";
    let sig = suite.sign(&kp, msg);

    // Adversary removes the ML-DSA-65 component, presenting only the
    // Ed25519 half. The committing/strip-resistant construction MUST
    // reject — a typed error, NOT Ok, NOT a silent classical-only accept.
    let stripped = sig.without_pq_half_for_test();
    let outcome = suite.verify(kp.public(), msg, &stripped);
    assert!(
        matches!(outcome, Err(VerifyError::HybridHalfMissing { .. }))
            || matches!(outcome, Err(VerifyError::StripResistanceViolated { .. })),
        "stripping the ML-DSA-65 half MUST fail closed with a typed error; \
         got {outcome:?} — a single-half accept is the load-bearing safety bug"
    );
}

/// Strip-resistance (2): stripping the Ed25519 (classical) half MUST also
/// fail the verify closed. The classical half is the audited security
/// floor; a PQ-only silent accept is equally a strip-resistance violation.
#[test]

fn tf2_stripping_ed25519_half_fails_closed() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"strip the classical half and the verify MUST fail closed";
    let sig = suite.sign(&kp, msg);

    let stripped = sig.without_classical_half_for_test();
    let outcome = suite.verify(kp.public(), msg, &stripped);
    assert!(
        matches!(outcome, Err(VerifyError::HybridHalfMissing { .. }))
            || matches!(outcome, Err(VerifyError::StripResistanceViolated { .. })),
        "stripping the Ed25519 half MUST fail closed with a typed error; \
         got {outcome:?}"
    );
}

/// Substitution-resistance: replacing the ML-DSA-65 half with a VALID
/// ML-DSA-65 signature over a DIFFERENT message (mix-and-match across
/// two signing operations) MUST fail closed. The committing construction
/// binds both halves to the SAME message — a per-half-independently-valid
/// but cross-message signature MUST NOT verify.
#[test]

fn tf2_cross_message_half_substitution_fails_closed() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();

    let msg_a = b"message A";
    let msg_b = b"message B";
    let sig_a = suite.sign(&kp, msg_a);
    let sig_b = suite.sign(&kp, msg_b);

    // Forge: classical half from sig over msg_a, PQ half from sig over
    // msg_b. Each half is individually a valid signature, but they do
    // not jointly commit to one message.
    let forged =
        HybridSignature::splice_for_test(sig_a.classical_half_for_test(), sig_b.pq_half_for_test());
    let outcome = suite.verify(kp.public(), msg_a, &forged);
    assert!(
        outcome.is_err(),
        "cross-message half substitution MUST fail closed (committing \
         construction binds both halves to ONE message); got {outcome:?}"
    );
}

/// Tamper: a flipped bit in the signed message MUST fail the verify
/// closed (basic soundness on the production path).
#[test]

fn tf2_tampered_message_fails_closed() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"original message";
    let sig = suite.sign(&kp, msg);

    let tampered = b"originai message"; // single-byte flip
    let outcome = suite.verify(kp.public(), tampered, &sig);
    assert!(
        outcome.is_err(),
        "tampered message MUST fail hybrid verify closed; got {outcome:?}"
    );
}

/// MR-1 follow-up (G-CORE-2-FP-1): adversary substitutes the PQ half
/// with a different keypair's PQ half over the same message. Both
/// halves are individually well-formed (the substitute is a real
/// ML-DSA-65 signature over `msg`), but the commitment binds the
/// ORIGINAL keypair's PQ verifying-key, so the recomputation tripps
/// the strip-resistance arm. Independent of the LIVE ML-DSA verify —
/// the commitment + the dual cryptographic verify are TWO independent
/// fail-closed surfaces. would-FAIL if a future agent regresses the
/// commitment to bind only one pubkey (the MR-7 hazard).
#[test]
fn tf2_cross_keypair_pq_half_substitute_fails_closed_via_commitment() {
    let suite = SignatureSuite::v1_default();
    let kp_alice = suite.generate_keypair();
    let kp_eve = suite.generate_keypair();
    let msg = b"keypair pubkey is bound into the NF-4 commitment";

    let sig_alice = suite.sign(&kp_alice, msg);
    let sig_eve = suite.sign(&kp_eve, msg);

    // Forge: Alice's classical half + Eve's PQ half (both halves valid
    // signatures over `msg` individually, but they bind two different
    // PQ pubkeys via the commitment).
    let forged = HybridSignature::splice_for_test(
        sig_alice.classical_half_for_test(),
        sig_eve.pq_half_for_test(),
    );
    let outcome = suite.verify(kp_alice.public(), msg, &forged);
    assert!(
        outcome.is_err(),
        "cross-keypair PQ-half substitute MUST fail closed via the \
         commitment-binding arm (the commitment binds the ORIGINAL \
         keypair's PQ pubkey); got {outcome:?}. A future agent who \
         binds only one pubkey into the commitment would silently \
         accept this forgery — pim-18 SHAPE-trap."
    );
}

/// The hybrid signature MUST NOT be a degenerate alias of the classical
/// signature: a config that *claims* hybrid but whose ML-DSA-65 half is
/// empty/zero MUST be rejected at construction or verify (no Ed25519-only
/// wolf in hybrid clothing). SHAPE-not-SUBSTANCE guard for pin (a).
#[test]

fn tf2_hybrid_is_not_a_classical_alias() {
    let suite = SignatureSuite::v1_default();
    let kp = suite.generate_keypair();
    let msg = b"hybrid must carry a real PQ half";
    let sig = suite.sign(&kp, msg);

    // The PQ half MUST be ML-DSA-65-dimensioned and non-empty (the
    // concrete size assertion lives in tf2_no_hardcoded_sizes; here we
    // assert it is simply NOT the empty/degenerate classical-alias).
    assert!(
        !sig.pq_half_for_test().is_empty(),
        "a 'hybrid' signature with an empty PQ half is a classical alias \
         masquerading as hybrid — SHAPE-not-SUBSTANCE trap (pim-18)"
    );
    assert!(
        SuiteConfig::classical_only().is_distinct_from(&SuiteConfig::v1_default()),
        "classical-only and hybrid-default MUST be DISTINCT configs"
    );
}

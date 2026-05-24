//! TF-4 pins (G-CORE-3c P-5 + F-1) — pure-PQ NF-1 arm is BUILT +
//! conformance-testable but **structurally non-default until
//! audit-landed flag flips** (the C11b / G-CORE-3c safety pin); a
//! pure-PQ attempt with audit-landed=false MUST surface typed
//! `AuditNotLandedPurePqRejected`.
//!
//! Pin sources:
//!   - `R2-test-landscape.md` §2 G-CORE-3c row P-5 (Optional pure-PQ
//!     NF-1 arm: ML-DSA-65⊕SLH-DSA sig + ML-KEM-768-only enc; config
//!     exists, builds, KAT-conformance-tests pass, BUT is **structurally
//!     non-default until audit-landed flag**) + F-1 (pure-PQ-as-sole-
//!     trust-path attempt with audit-landed=false → typed
//!     `AuditNotLandedPurePqRejected` — the C11b/G-CORE-3c load-bearing
//!     safety pin).
//!   - `00-implementation-plan.md` R0.8.1 §3 G-CORE-3c def: "Safety
//!     invariant (reshaped): hybrid construction → PQC is never the
//!     SOLE trust path; pure-PQ (sole-PQ-trust) stays **structurally
//!     non-default until the audit-landed flag flips**."
//!   - CLAUDE.md baked-in #5 PQ-default reframe + #15 v1-gate
//!     C-GM-AUDIT exit criterion.
//!
//! # G-CORE-3c GREEN-PHASE — stub-shim DELETED + real `use` against the
//! LIVE `swap_matrix::*` surface; tests UN-IGNORED.
//!
//! # Production-arm shape (pim-2 sub-rule-4 + pim-18 SHAPE-not-SUBSTANCE)
//!
//! Load-bearing safety property for the v1-beta / v1-GM release-stage
//! split: the pure-PQ NF-1 arm MUST be BUILT (so the audit can witness
//! it) BUT MUST NOT be the silent default for anyone before the audit
//! lands. The pins assert (a) the build target exists + KAT-conforms +
//! (b) the runtime gate FIRES with typed rejection when an implementer
//! tries to make it the sole trust path pre-audit. A weak sentinel
//! would only check (a) without (b) — the whole point of the safety
//! invariant is the runtime gate.
//!
//! # §3.13 per-test-static decomposition
//!
//! No shared static. The audit-landed-state is workspace-baseline state
//! (not per-test). Pins assert what the baseline IS — no test mutates
//! it. NO `set_audit_landed_flag` in the test surface; flipping the
//! flag is a v1-GM coupled action.
//!
//! # §3.5g cross-language rule-mirror — DISCHARGED via ErrorCode mirror
//!
//! The TS-side surface of the audit-gate is the new ErrorCode
//! [`E_AUDIT_NOT_LANDED_PURE_PQ_REJECTED`] (mirrored in
//! `packages/engine/src/errors.generated.ts` per the §3.5g 4-surface
//! discipline). TS callers route through the engine-error catalog
//! which surfaces this typed arm when a pure-PQ-sole-trust-path
//! construction is rejected pre-audit. The wider `SwapMatrix` surface
//! is Rust-only at v1-beta — TS consumers MUST NOT instantiate
//! SwapMatrix arms directly; they consume sealed envelopes through
//! the engine's typed boundary which already surfaces the audit-gate
//! rejection via the ErrorCode mirror.
//!
//! G-CORE-3c fix-pass (mr-minor-4): the earlier self-named obligation
//! `packages/engine/src/swap_matrix.generated.ts` was retracted — it
//! is not load-bearing because the ErrorCode mirror covers the only
//! TS-observable surface at v1-beta. A future TS-side SwapMatrix
//! mirror (if cross-language SwapMatrix instantiation surfaces) would
//! land as its own brief under the §3.5g discipline; until then, the
//! ErrorCode mirror DISCHARGES the cross-language obligation.

#![allow(clippy::unwrap_used)]

use benten_crypto_suite::swap_matrix::{SwapMatrix, SwapMatrixError};

/// Pin (P-5a) The pure-PQ NF-1 sig arm MUST be BUILT — its codepoint
/// resolves through a real (not typed-reject) path AT G-CORE-3c.
///
/// G-CORE-3c's safety invariant requires the pure-PQ NF-1 arm to be
/// build-correct + KAT-conformance-pass so the upcoming v1-beta-
/// gated-audit can witness it. would-FAIL if G-CORE-3c skips the
/// build-correctness arm.
#[test]
fn tf4_p5a_pure_pq_nf1_signature_arm_is_built_and_kat_conformant() {
    let arm = SwapMatrix::pure_pq_nf1_signature_arm();
    assert!(
        arm.is_built(),
        "pure-PQ NF-1 sig arm (ML-DSA-65⊕SLH-DSA at codepoint 0x0003) \
         MUST be BUILT at G-CORE-3c (the conformance-spike scope per \
         RATIFIED safety invariant); would-FAIL if codepoint 0x0003 \
         remains typed-reject post G-CORE-3c"
    );
    assert!(
        arm.kat_conformance_passes(),
        "pure-PQ NF-1 sig arm MUST pass FIPS-204 (ML-DSA-65) + FIPS-205 \
         (SLH-DSA) KAT vectors (the audit needs witnessable correctness)"
    );
}

/// Pin (P-5b) The pure-PQ arm is STRUCTURALLY NON-DEFAULT at workspace
/// baseline — the audit-landed flag MUST be `false`.
///
/// The C11b safety invariant: pure-PQ stays non-default until the
/// audit-landed flag flips. Workspace baseline MUST have
/// `audit_landed_pure_pq_flag() == false`. would-FAIL if any
/// implementer ships a default-flag flip pre-audit (the v1-GM-gating
/// invariant).
#[test]
fn tf4_p5b_audit_landed_pure_pq_flag_is_false_at_workspace_baseline() {
    let flag = SwapMatrix::audit_landed_pure_pq_flag();
    assert!(
        !flag,
        "audit-landed-pure-PQ flag MUST be `false` at workspace baseline \
         (the C11b safety invariant — pure-PQ is structurally non-default \
         until the independent `ml-dsa`/`ml-kem` audit lands per NF-2 / \
         C-GM-AUDIT)"
    );
    let default_matrix = SwapMatrix::v1_beta_default();
    assert!(
        !default_matrix.is_pure_pq_sole_trust_path(),
        "the v1-beta DEFAULT matrix MUST NOT be a pure-PQ-sole-trust-path \
         (the hybrid construction means unaudited PQC is never the SOLE \
         trust path; the classical Ed25519/X25519 half is the audited \
         security floor)"
    );
}

/// Pin (F-1) pure-PQ-as-sole-trust-path attempt with audit-landed=false
/// → typed `AuditNotLandedPurePqRejected`.
///
/// The runtime gate fires: an implementer attempts to construct a
/// `SwapMatrix` that has pure-PQ as the sole trust path (no classical
/// half). With `audit_landed_pure_pq_flag == false`, the
/// `try_pure_pq_sole_trust_path()` constructor MUST surface the NAMED
/// `AuditNotLandedPurePqRejected` arm; NEVER silent success; NEVER a
/// generic `Err` (the named arm is what the v1-GM-gate CI lane greps
/// for). Load-bearing safety pin per R2 F-1.
#[test]
fn tf4_f1_pure_pq_sole_trust_path_pre_audit_typed_rejected() {
    // Pre-condition: workspace baseline = audit-landed flag is false.
    assert!(!SwapMatrix::audit_landed_pure_pq_flag());

    let outcome: Result<SwapMatrix, SwapMatrixError> = SwapMatrix::try_pure_pq_sole_trust_path();
    assert!(
        matches!(
            outcome,
            Err(SwapMatrixError::AuditNotLandedPurePqRejected { .. })
        ),
        "pure-PQ-sole-trust-path attempt with audit-landed-flag=false \
         MUST surface the NAMED typed arm `AuditNotLandedPurePqRejected` \
         (the v1-GM-gating safety invariant per R2 F-1 / RATIFIED \
         pq-default-reframe 2026-05-19 §2 safety clause + CLAUDE.md \
         baked-in #5 reframe; would-FAIL if collapsed to a generic Err \
         OR silent success — both regress the C-GM-AUDIT gate); got \
         {outcome:?}"
    );

    let hybrid_ok = SwapMatrix::v1_beta_default();
    assert!(
        !hybrid_ok.is_pure_pq_sole_trust_path(),
        "the gate MUST be specific to sole-trust-path attempts — the \
         hybrid default is NOT sole-trust-path (classical half is the \
         audited floor) so it constructs without the gate firing"
    );
}

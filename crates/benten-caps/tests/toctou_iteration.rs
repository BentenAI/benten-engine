//! Phase 1 R3 security test — TOCTOU during long ITERATE (R1 major #5).
//!
//! Attack class: capability is granted at iteration start, revoked by another
//! actor (or by the same actor's later operation) mid-ITERATE. Phase 2
//! Invariant 13 would check caps per-operation; Phase 1's named compromise is
//! to refresh the capability snapshot only at batch boundaries (default every
//! 100 iterations per grant-configurable budget). Revocations between
//! boundaries are invisible to the in-flight evaluation — writes in that
//! window still land. At the NEXT boundary the revocation is seen and all
//! subsequent writes fire `E_CAP_REVOKED_MID_EVAL`.
//!
//! This test validates THREE properties the R1 triage named:
//!   (a) revocation mid-iteration surfaces at the next batch boundary,
//!   (b) writes within the current batch (pre-boundary) are NOT retroactively
//!       denied — the named compromise is explicit about the window,
//!   (c) the error code is the dedicated `E_CAP_REVOKED_MID_EVAL`, distinct
//!       from `E_CAP_REVOKED` (the Phase 3 sync-revocation code).
//!
//! TDD contract: FAIL at R3. R5 lands the batch-boundary cap-refresh plumbing
//! in the evaluator, the revocation API, and the distinguished error code.
//!
//! Cross-refs:
//! - `.addl/phase-1/r1-security-auditor.json` finding #5 (major)
//! - `.addl/phase-1/r1-triage.md` named compromise #1
//! - `docs/ERROR-CATALOG.md` `E_CAP_REVOKED_MID_EVAL`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::ErrorCode;
use benten_engine::Engine;
use benten_engine::testing::iterate_write_handler;

/// Batch boundary size is configurable per grant; this default matches the
/// R1-triage documented value (100 iterations). Imported from
/// `benten_caps::DEFAULT_BATCH_BOUNDARY` (G4 mini-review g4-cr-3) so the two
/// sides stay in lockstep — change the pub const in the crate and the test
/// follows automatically.
const DEFAULT_BATCH_BOUNDARY: u32 = benten_caps::DEFAULT_BATCH_BOUNDARY as u32;

/// Attack simulation: grant a WRITE capability, start a 300-iter handler,
/// revoke the cap mid batch 1 (iteration ~50). Per the r4-triage reconciliation
/// of the Phase 1 TOCTOU window: writes 1..=100 succeed (cap snapshot held
/// within batch 1); write 101 sits at the next batch boundary where the
/// evaluator re-reads caps and sees the revocation, so it fails with the
/// distinguished code.
///
/// Asserts:
///   - exactly `DEFAULT_BATCH_BOUNDARY` writes succeed (not 99, not 200),
///   - write 101 fails with `E_CAP_REVOKED_MID_EVAL`,
///   - the error is `E_CAP_REVOKED_MID_EVAL`, NOT `E_CAP_DENIED` and NOT
///     `E_CAP_REVOKED` (reserved for Phase 3 sync revocation).
#[test]
#[ignore = "TODO(phase-2-grant-backed-policy): GrantBackedPolicy reads CapabilityGrant nodes from the graph and enforces; blocked on grant-write API + schedule_revocation_at_iteration (Phase-2 NotImplemented) + iterate_write_handler populated helper. When populated, re-assert the denial shape."]
fn capability_revoked_mid_iteration_denies_subsequent_batches() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .with_policy_allowing_revocation()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let handler = iterate_write_handler(/* max = */ 300);
    let handler_id = engine.register_subgraph(&handler).unwrap();

    // Grant the cap, then schedule a revocation at iter 50 (mid batch 1).
    let grant_cid = engine
        .grant_capability("post:write", "test-subject")
        .unwrap();
    engine
        .schedule_revocation_at_iteration(grant_cid, 50)
        .unwrap();

    // Run the handler. Writes 1..=100 land under the held snapshot; write 101
    // triggers batch-boundary cap re-read, sees revocation, fires the code.
    let outcome = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    let successful_writes = outcome.successful_write_count();
    assert_eq!(
        successful_writes, DEFAULT_BATCH_BOUNDARY,
        "writes 1..=100 must succeed under held cap snapshot; got {successful_writes}"
    );

    // The terminating error must be the distinguished mid-eval code, fired by
    // write 101 at the next batch-boundary cap refresh.
    let err = outcome.terminal_error().expect("handler must error out");
    assert_eq!(
        err.code(),
        ErrorCode::CapRevokedMidEval,
        "write 101 must fire E_CAP_REVOKED_MID_EVAL at the batch-boundary \
         cap refresh, not a generic E_CAP_DENIED or the Phase-3 \
         E_CAP_REVOKED. Got: {:?}",
        err.code()
    );
    assert_ne!(err.code(), ErrorCode::CapDenied);
    assert_ne!(err.code(), ErrorCode::CapRevoked);
}

/// Named-compromise regression: writes INSIDE the current batch are
/// intentionally NOT retroactively denied — the snapshot held for the batch
/// is exactly what the R1 triage calls "the TOCTOU window". Revocation at
/// iter 50 does not clobber writes 50..=100 because the batch-1 snapshot
/// covers them; write 101 (batch-2 boundary) is the first to see the
/// revocation. If Phase 2 closes this window (per-iter cap check), this test
/// will need to flip its assertion.
#[test]
#[ignore = "TODO(phase-2-grant-backed-policy): GrantBackedPolicy reads CapabilityGrant nodes from the graph and enforces; blocked on grant-write API + schedule_revocation_at_iteration (Phase-2 NotImplemented) + iterate_write_handler populated helper. When populated, re-assert the denial shape."]
fn writes_in_current_batch_are_not_retroactively_denied() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .with_policy_allowing_revocation()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let handler = iterate_write_handler(250);
    let handler_id = engine.register_subgraph(&handler).unwrap();
    let grant = engine
        .grant_capability("post:write", "test-subject")
        .unwrap();

    // Revoke AT iter 50 — mid batch 1. Writes 1..=100 MUST all land; write 101
    // is where the revocation is observed at the batch-2 boundary refresh.
    engine.schedule_revocation_at_iteration(grant, 50).unwrap();

    let outcome = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    assert_eq!(
        outcome.successful_write_count(),
        DEFAULT_BATCH_BOUNDARY,
        "Phase 1 TOCTOU window is explicit: revocation inside a batch does \
         not retroactively deny in-flight writes. Writes 1..=100 must all \
         land. Got: {}",
        outcome.successful_write_count()
    );

    // And write 101 is denied at the boundary — not retroactive.
    let err = outcome
        .terminal_error()
        .expect("handler must error out at 101");
    assert_eq!(
        err.code(),
        ErrorCode::CapRevokedMidEval,
        "write 101 at the batch-2 boundary observes the revocation"
    );
    // Remove this test (or flip the assertion direction) when Phase 2
    // Invariant 13 lands and the window closes.
}

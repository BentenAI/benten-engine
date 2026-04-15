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
/// R1-triage documented value (100 iterations). If the default changes, this
/// constant and the named compromise prose must change together.
const DEFAULT_BATCH_BOUNDARY: u32 = 100;

/// Attack simulation: grant a WRITE capability, start a 300-iter handler,
/// revoke the cap after batch 1 (iteration ~150). Assert:
///   - iterations 1..=DEFAULT_BATCH_BOUNDARY ran and wrote successfully,
///   - iterations beyond the NEXT boundary (200+) were denied with the
///     `E_CAP_REVOKED_MID_EVAL` code,
///   - the error is `E_CAP_REVOKED_MID_EVAL`, NOT `E_CAP_DENIED` and NOT
///     `E_CAP_REVOKED` (which is reserved for Phase 3 sync revocation).
#[test]
fn capability_revoked_mid_iteration_denies_subsequent_batches() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .with_policy_allowing_revocation()
        .open(dir.path().join("benten.redb"))
        .unwrap();

    let handler = iterate_write_handler(/* max = */ 300);
    let handler_id = engine.register_subgraph(&handler).unwrap();

    // Grant the cap, then schedule a revocation at ~iter 150 (mid-batch-2).
    let grant_cid = engine
        .grant_capability("post:write", "test-subject")
        .unwrap();
    engine
        .schedule_revocation_at_iteration(grant_cid, 150)
        .unwrap();

    // Run the handler. We expect partial success: ~200 writes land before the
    // revocation is seen at the next 100-iter boundary, remaining writes fail.
    let outcome = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    let successful_writes = outcome.successful_write_count();
    assert!(
        successful_writes >= DEFAULT_BATCH_BOUNDARY && successful_writes < 300,
        "expected partial progress: at least one full batch pre-revocation \
         ({DEFAULT_BATCH_BOUNDARY}) but short of full 300; got {successful_writes}"
    );

    // The terminating error must be the distinguished mid-eval code.
    let err = outcome.terminal_error().expect("handler must error out");
    assert_eq!(
        err.code(),
        ErrorCode::CapRevokedMidEval,
        "mid-iteration revocation must fire E_CAP_REVOKED_MID_EVAL, not a \
         generic E_CAP_DENIED or the Phase-3 E_CAP_REVOKED. Got: {:?}",
        err.code()
    );
    assert_ne!(err.code(), ErrorCode::CapDenied);
    assert_ne!(err.code(), ErrorCode::CapRevoked);
}

/// Named-compromise regression: writes INSIDE the revocation window (between
/// revocation and the next boundary) are intentionally NOT retroactively
/// denied. This is the Phase 1 TOCTOU window the triage calls out explicitly.
/// If Phase 2 closes this window, this test will need to flip its assertion.
#[test]
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

    // Revoke AT iter 120 — mid batch 2. Writes 101..=120 land pre-revocation;
    // writes 121..=200 land post-revocation but pre-next-boundary (Phase 1
    // named compromise); writes 201+ denied at the batch boundary refresh.
    engine.schedule_revocation_at_iteration(grant, 120).unwrap();

    let outcome = engine
        .call(&handler_id, "default", benten_core::Node::empty())
        .expect("call returns Ok wrapper");

    assert!(
        outcome.successful_write_count() >= 200,
        "Phase 1 TOCTOU window is explicit: revocation between boundaries \
         does not retroactively deny in-flight writes. Writes 121..=200 must \
         land. Got: {}",
        outcome.successful_write_count()
    );
    // Remove this test (or flip the assertion direction) when Phase 2
    // Invariant 13 lands and the window closes.
}

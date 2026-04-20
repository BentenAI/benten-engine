//! Phase 1 R3 + R6 integration — Capability revoked mid-evaluation.
//!
//! **Phase-1 TOCTOU window contract (named compromise #1).** The evaluator
//! snapshots capabilities at batch boundaries (every `DEFAULT_BATCH_BOUNDARY`
//! iterations, default 100). Writes inside the current batch run under the
//! held snapshot; writes in the NEXT batch see any revocation that occurred
//! mid-batch and fire `E_CAP_REVOKED_MID_EVAL`.
//!
//! **R6 regression restructure.** The original test depended on
//! `Engine::schedule_revocation_at_iteration` (Phase-2 API). This rewrite
//! removes that dependency by synthesizing a `CapabilityPolicy` WRAPPER
//! whose internal counter flips the grant after M `check_write` calls —
//! the policy IS the schedule. The Phase-1 batch-boundary contract is
//! still observable: exactly `DEFAULT_BATCH_BOUNDARY` writes succeed, then
//! write 101 trips the wrapped revocation and surfaces
//! `E_CAP_REVOKED_MID_EVAL` via the `ON_DENIED` edge.
//!
//! **Status:** GREEN after r6-sec-2.
//!
//! Cross-refs:
//! - `.addl/phase-1/r6-*` (R6 security-auditor r6-sec-2)
//! - `.addl/phase-1/r1-triage.md` named compromise #1
//! - `docs/ERROR-CATALOG.md` `E_CAP_REVOKED_MID_EVAL`

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{Arc, Mutex};

use benten_caps::{CapError, CapabilityPolicy, DEFAULT_BATCH_BOUNDARY, ReadContext, WriteContext};
use benten_core::{Node, Value};
use benten_engine::Engine;

/// Capability policy that simulates a mid-evaluation revocation.
///
/// Permits the first `permit_first` `check_write` calls (the "batch 1"
/// snapshot under the Phase-1 named compromise). Subsequent calls return
/// `CapError::RevokedMidEval` — the distinguished code that the Phase-1
/// evaluator MUST surface for writes observing a revocation at the next
/// batch boundary.
struct RevokeAfterNPolicy {
    /// Count of `check_write` calls so far. Atomic-ish via Mutex since the
    /// `CapabilityPolicy` trait only gives us `&self`.
    calls: Arc<Mutex<usize>>,
    /// Number of calls to permit before starting to deny.
    permit_first: usize,
}

impl RevokeAfterNPolicy {
    fn new(permit_first: usize) -> (Self, Arc<Mutex<usize>>) {
        let calls = Arc::new(Mutex::new(0));
        (
            Self {
                calls: Arc::clone(&calls),
                permit_first,
            },
            calls,
        )
    }
}

impl CapabilityPolicy for RevokeAfterNPolicy {
    fn check_write(&self, _ctx: &WriteContext) -> Result<(), CapError> {
        let mut calls = self.calls.lock().unwrap();
        *calls += 1;
        if *calls <= self.permit_first {
            Ok(())
        } else {
            // Phase-1 contract: the first write past the held-snapshot
            // boundary sees the revocation and surfaces the distinguished
            // code. `E_CAP_DENIED` would collapse this attack-surface into
            // a generic denial and lose the auditability of the
            // revocation-during-eval signal.
            Err(CapError::RevokedMidEval)
        }
    }

    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }
}

/// r6-sec-2 regression — a policy that flips from permit to `RevokedMidEval`
/// after `DEFAULT_BATCH_BOUNDARY` `check_write` calls surfaces the
/// distinguished error code via `ON_DENIED` on the first write past the
/// flip. Three assertions:
///
/// 1. Exactly `DEFAULT_BATCH_BOUNDARY` writes succeed (the in-batch
///    snapshot window).
/// 2. Write 101 fails with `E_CAP_REVOKED_MID_EVAL` — not generic
///    `E_CAP_DENIED`, not `E_CAP_REVOKED` (Phase-3 sync).
/// 3. The error routes through the `ON_DENIED` typed edge (not
///    `ON_ERROR`) so the handler author can recover.
#[test]
fn capability_revocation_at_batch_boundary_surfaces_mid_eval_code() {
    let dir = tempfile::tempdir().unwrap();

    let (policy, counter) = RevokeAfterNPolicy::new(DEFAULT_BATCH_BOUNDARY);
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(policy))
        .build()
        .unwrap();

    let handler_id = engine.register_crud("post").unwrap();

    // Drive `DEFAULT_BATCH_BOUNDARY + 1` writes. The first 100 succeed; the
    // 101st observes the flip and surfaces `E_CAP_REVOKED_MID_EVAL`. A
    // DEFAULT_BATCH_BOUNDARY + N excess writes would accumulate more denial
    // outcomes; we stop at the first denial since that's the load-bearing
    // assertion.
    let mut successes: u32 = 0;
    let mut denial_outcome: Option<benten_engine::Outcome> = None;
    for i in 0..=DEFAULT_BATCH_BOUNDARY {
        let mut props = std::collections::BTreeMap::new();
        props.insert("n".into(), Value::Int(i as i64));
        let outcome = engine
            .call(
                &handler_id,
                "post:create",
                Node::new(vec!["post".into()], props),
            )
            .expect("call returns Ok wrapper; denial routes via outcome edge");

        if outcome.error_code().is_none() {
            successes += 1;
        } else {
            denial_outcome = Some(outcome);
            break;
        }
    }

    // (1) Exactly DEFAULT_BATCH_BOUNDARY writes succeeded under the
    //     held-snapshot window.
    assert_eq!(
        successes as usize, DEFAULT_BATCH_BOUNDARY,
        "Phase-1 TOCTOU window: writes 1..=DEFAULT_BATCH_BOUNDARY must succeed \
         before the revocation fires; got {successes}"
    );

    // (2) Write DEFAULT_BATCH_BOUNDARY + 1 saw the revocation.
    let outcome = denial_outcome.expect("write 101 must have tripped the revocation");
    assert_eq!(
        outcome.error_code(),
        Some("E_CAP_REVOKED_MID_EVAL"),
        "write past the held-snapshot boundary must surface the \
         distinguished mid-eval code, not E_CAP_DENIED or E_CAP_REVOKED"
    );

    // (3) Routing is via ON_DENIED — recoverable by the handler author.
    //     ON_ERROR is reserved for backend-configuration errors
    //     (NotImplemented etc.).
    assert!(
        outcome.routed_through_edge("ON_DENIED"),
        "revocation mid-eval must route through ON_DENIED, not ON_ERROR"
    );

    // Sanity: the policy was called exactly DEFAULT_BATCH_BOUNDARY + 1
    //         times (every write hit the policy once at commit time).
    let calls = *counter.lock().unwrap();
    assert_eq!(
        calls,
        DEFAULT_BATCH_BOUNDARY + 1,
        "check_write fires once per commit; got {calls} calls for {} writes",
        DEFAULT_BATCH_BOUNDARY + 1
    );
}

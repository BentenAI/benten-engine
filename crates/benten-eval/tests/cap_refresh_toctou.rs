//! Named Compromise #1 closure — CALL-entry + ITERATE batch-boundary
//! cap-refresh regression tests.
//!
//! These tests drive the primitive executors directly via a synthetic
//! `PrimitiveHost` so the refresh semantics are observable in isolation
//! (no engine, no redb, no policy wiring). The Phase-1 contract the
//! tests lock in:
//!
//! 1. `call::execute` consults `PrimitiveHost::check_capability` BEFORE
//!    attenuation, timeout, or dispatch. A denial routes `ON_DENIED`
//!    with the policy's error code in the edge payload.
//!
//! 2. `iterate::execute` consults `check_capability` at every
//!    `host.iterate_batch_boundary()` iterations (inclusive of iter 0).
//!    Items 0..boundary-1 run under the snapshot captured at the entry
//!    refresh; at iter `boundary` the snapshot is re-read. A denial
//!    observed at a boundary routes `ON_DENIED`.
//!
//! 3. Items WITHIN a batch are NOT retroactively denied — the Phase-1
//!    TOCTOU window is explicit. The denial is observable at the next
//!    boundary.
//!
//! Closes Named Compromise #1 at the primitive-executor layer.
//! `crates/benten-engine/tests/integration/cap_toctou.rs` covers the
//! engine-level transaction-commit refresh path; the two sides together
//! describe the full Phase-1 contract.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use benten_core::{Cid, Edge, Node, Value};
use benten_eval::{
    EvalError, OperationNode, PrimitiveHost, PrimitiveKind, ViewQuery,
    primitives::{call, iterate},
};

/// Test PrimitiveHost whose `check_capability` flips to denied after N
/// calls. Every other method is a NullHost-equivalent.
struct RevokeAfterNHost {
    calls: AtomicUsize,
    permit_first: usize,
    boundary: usize,
}

impl RevokeAfterNHost {
    fn new(permit_first: usize, boundary: usize) -> Self {
        Self {
            calls: AtomicUsize::new(0),
            permit_first,
            boundary,
        }
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl PrimitiveHost for RevokeAfterNHost {
    fn read_node(&self, _cid: &Cid) -> Result<Option<Node>, EvalError> {
        Ok(None)
    }
    fn get_by_label(&self, _label: &str) -> Result<Vec<Cid>, EvalError> {
        Ok(Vec::new())
    }
    fn get_by_property(
        &self,
        _label: &str,
        _prop: &str,
        _value: &Value,
    ) -> Result<Vec<Cid>, EvalError> {
        Ok(Vec::new())
    }
    fn put_node(&self, _node: &Node) -> Result<Cid, EvalError> {
        Err(EvalError::Backend("unused".into()))
    }
    fn put_edge(&self, _edge: &Edge) -> Result<Cid, EvalError> {
        Err(EvalError::Backend("unused".into()))
    }
    fn delete_node(&self, _cid: &Cid) -> Result<(), EvalError> {
        Ok(())
    }
    fn delete_edge(&self, _cid: &Cid) -> Result<(), EvalError> {
        Ok(())
    }
    fn call_handler(&self, _handler_id: &str, _op: &str, _input: Node) -> Result<Value, EvalError> {
        Ok(Value::Null)
    }
    fn emit_event(&self, _name: &str, _payload: Value) {}
    fn check_capability(&self, _required: &str, _target: Option<&Cid>) -> Result<(), EvalError> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst);
        if n < self.permit_first {
            Ok(())
        } else {
            Err(EvalError::Capability(benten_caps::CapError::RevokedMidEval))
        }
    }
    fn read_view(&self, _view_id: &str, _query: &ViewQuery) -> Result<Value, EvalError> {
        Ok(Value::Null)
    }
    fn iterate_batch_boundary(&self) -> usize {
        self.boundary
    }
}

// ---------------------------------------------------------------------------
// CALL-entry refresh
// ---------------------------------------------------------------------------

/// Compromise #1 CALL-entry closure: a CALL primitive consults
/// `check_capability` before any attenuation / timeout work. When the
/// host denies, the CALL routes `ON_DENIED` with the cap error's
/// string form in the edge payload — the dispatch never runs.
#[test]
fn toctou_call_refreshes_at_entry() {
    // permit_first = 0 → first (and only) call is denied.
    let host = RevokeAfterNHost::new(0, 100);

    // CALL op with a target + call_op so the normal dispatch path WOULD
    // fire if the cap refresh didn't intercept first.
    let op = OperationNode::new("c0", PrimitiveKind::Call)
        .with_property("target", Value::text("any_handler"))
        .with_property("call_op", Value::text("default"));

    let result = call::execute(&op, &host).expect("CALL never Errs on cap denial");
    assert_eq!(
        result.edge_label, "ON_DENIED",
        "CALL entry cap-refresh denial must route ON_DENIED, got {:?}",
        result.edge_label
    );
    assert_eq!(
        host.call_count(),
        1,
        "check_capability fires exactly once at CALL entry"
    );
}

/// Sibling regression: when the entry cap-refresh passes, CALL
/// proceeds to dispatch and returns the happy-path `ok` edge. Ensures
/// the entry check is NOT a hard gate for every CALL — only refuses
/// when the policy is in the denied state.
#[test]
fn toctou_call_entry_permit_proceeds_to_dispatch() {
    let host = RevokeAfterNHost::new(10, 100); // first 10 calls permitted

    // Legacy-shape CALL (no target / call_op) — entry cap-refresh still
    // fires, attenuation is skipped (no parent_scope / child_scope),
    // happy path.
    let op = OperationNode::new("c0", PrimitiveKind::Call);
    let result = call::execute(&op, &host).expect("CALL permitted happy path");
    assert_eq!(result.edge_label, "ok");

    // With a declared `requires` string, the entry refresh must consult
    // the host with that specific scope.
    let op2 = OperationNode::new("c1", PrimitiveKind::Call)
        .with_property("requires", Value::text("post:write"));
    let result2 = call::execute(&op2, &host).expect("CALL permitted happy path");
    assert_eq!(result2.edge_label, "ok");

    // Two successful entry-refresh calls fired.
    assert_eq!(host.call_count(), 2);
}

// ---------------------------------------------------------------------------
// ITERATE batch-boundary refresh
// ---------------------------------------------------------------------------

/// Compromise #1 ITERATE batch-boundary closure: ITERATE consults
/// `check_capability` at entry AND at every N iterations. A revocation
/// landing BEFORE the 2nd boundary is observed at the 2nd boundary;
/// items in batch 1 (0..N-1) are not retroactively denied.
///
/// Scenario: boundary = 10, items = 25. Expected refresh calls at
/// iterations 0, 10, 20. The host permits the first 2 calls and denies
/// the 3rd — so ITERATE routes ON_DENIED at the 3rd refresh (iter 20).
#[test]
fn toctou_iteration_refreshes_at_batch_boundary() {
    // permit_first=2 → refresh calls 0 (entry) and 1 (iter=10) pass;
    // call 2 (iter=20) is denied.
    let host = RevokeAfterNHost::new(2, 10);

    // Build an ITERATE op with items=25 (well above boundary=10).
    let items = vec![Value::Int(0); 25];
    let op = OperationNode::new("it0", PrimitiveKind::Iterate)
        .with_property("items", Value::List(items))
        .with_property("max", Value::Int(100))
        .with_property("requires", Value::text("iter:write"));

    let result = iterate::execute(&op, &host).expect("ITERATE cap denial routes");
    assert_eq!(
        result.edge_label, "ON_DENIED",
        "ITERATE batch-boundary denial must route ON_DENIED"
    );
    assert_eq!(
        host.call_count(),
        3,
        "expected exactly 3 refresh calls at iters 0, 10, 20; got {}",
        host.call_count()
    );
}

/// Compromise #1 regression: items within a batch are NOT retroactively
/// denied. Scenario: boundary=100, items=50. Only one refresh (entry)
/// fires — iter 100 is never reached because items_len < boundary. The
/// entry refresh passes, the ITERATE terminates happy-path.
#[test]
fn toctou_iteration_single_batch_no_spurious_refresh() {
    // permit_first=1 → entry refresh passes; any subsequent refresh
    // would be denied — but there should be none.
    let host = RevokeAfterNHost::new(1, 100);

    let items = vec![Value::Int(0); 50];
    let op = OperationNode::new("it0", PrimitiveKind::Iterate)
        .with_property("items", Value::List(items))
        .with_property("max", Value::Int(100));

    let result = iterate::execute(&op, &host).expect("happy-path single-batch ITERATE");
    assert_eq!(result.edge_label, "ok");
    assert_eq!(
        host.call_count(),
        1,
        "single-batch ITERATE performs exactly one (entry) refresh"
    );
}

/// Compromise #1 entry-refresh regression: a revocation already in
/// place when ITERATE is entered denies the first batch, same as the
/// mid-iteration case. No items run.
#[test]
fn toctou_iteration_entry_refresh_denies_batch_zero() {
    // permit_first=0 → entry refresh fails.
    let host = RevokeAfterNHost::new(0, 10);

    let items = vec![Value::Int(0); 25];
    let op = OperationNode::new("it0", PrimitiveKind::Iterate)
        .with_property("items", Value::List(items))
        .with_property("max", Value::Int(100));

    let result = iterate::execute(&op, &host).expect("ITERATE entry-deny routes");
    assert_eq!(result.edge_label, "ON_DENIED");
    assert_eq!(
        host.call_count(),
        1,
        "entry refresh denies before any batch boundary loop iteration"
    );
}

/// Compromise #1 surface: the default `iterate_batch_boundary` on the
/// `PrimitiveHost` trait is 100, matching the pub const in benten-caps.
/// A host that overrides the boundary controls the refresh cadence —
/// this regression pins the contract that the override IS honored.
#[test]
fn toctou_iteration_respects_host_supplied_boundary() {
    // boundary=5, items=12, permit_first=2 (so refreshes at 0 and 5
    // pass; refresh at 10 denies).
    let host = RevokeAfterNHost::new(2, 5);
    let items = vec![Value::Int(0); 12];
    let op = OperationNode::new("it0", PrimitiveKind::Iterate)
        .with_property("items", Value::List(items))
        .with_property("max", Value::Int(100));

    let result = iterate::execute(&op, &host).expect("cap-denial routes");
    assert_eq!(result.edge_label, "ON_DENIED");
    assert_eq!(
        host.call_count(),
        3,
        "boundary=5, items=12 → refreshes at 0, 5, 10 = 3 calls"
    );
}

/// Arc-wrapping belt & suspenders — a shared counter through Arc works
/// identically (the trait is Send+Sync, so tests may share the host
/// through an Arc if needed).
#[test]
fn toctou_iteration_arc_shared_host() {
    let host = Arc::new(RevokeAfterNHost::new(100, 10));
    let items = vec![Value::Int(0); 5]; // single sub-batch, entry-only refresh
    let op = OperationNode::new("it0", PrimitiveKind::Iterate)
        .with_property("items", Value::List(items))
        .with_property("max", Value::Int(100));

    let result = iterate::execute(&op, host.as_ref()).expect("permitted");
    assert_eq!(result.edge_label, "ok");
    assert_eq!(host.call_count(), 1);
}

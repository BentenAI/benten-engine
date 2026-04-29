//! Phase 2 primitives pass *structural* validation at registration but fail
//! at *call* time (E5, R1 — R2 landscape §2.5 row 2).
//!
//! Registration-time validation (invariants 1/2/3/5/6/9/10/12) accepts
//! subgraphs containing WAIT, STREAM, SUBSCRIBE-as-user-op, SANDBOX because
//! they have defined determinism classification + error edges. Executor
//! returns `E_PRIMITIVE_NOT_IMPLEMENTED` when the subgraph is actually called.
//!
//! This prevents the regression class where enabling Phase 2 executors
//! requires re-registering every stored subgraph.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_PRIMITIVE_NOT_IMPLEMENTED` (covers row
//! "wait_stream_subscribe_sandbox_call_time_error").

#![allow(clippy::unwrap_used)]

use benten_eval::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};

use benten_eval::{
    EvalError, Evaluator, InvariantConfig, NullHost, OperationNode, PrimitiveKind, Subgraph,
};

fn single_primitive_subgraph(kind: PrimitiveKind) -> Subgraph {
    Subgraph::new("h").with_node(OperationNode::new("n", kind))
}

#[test]
fn wait_primitive_subgraph_passes_structural_validation() {
    let sg = single_primitive_subgraph(PrimitiveKind::Wait);
    sg.validate(&InvariantConfig::default()).unwrap();
}

#[test]
fn stream_primitive_subgraph_passes_structural_validation() {
    let sg = single_primitive_subgraph(PrimitiveKind::Stream);
    sg.validate(&InvariantConfig::default()).unwrap();
}

#[test]
fn subscribe_primitive_subgraph_passes_structural_validation() {
    let sg = single_primitive_subgraph(PrimitiveKind::Subscribe);
    sg.validate(&InvariantConfig::default()).unwrap();
}

#[test]
fn sandbox_primitive_subgraph_passes_structural_validation() {
    let sg = single_primitive_subgraph(PrimitiveKind::Sandbox);
    sg.validate(&InvariantConfig::default()).unwrap();
}

/// Covered by `covers_error_code[E_PRIMITIVE_NOT_IMPLEMENTED]` entry
/// "wait_stream_subscribe_sandbox_call_time_error".
///
/// Phase-2b G6-A scope update: STREAM + SUBSCRIBE got real executors
/// (wave-4 G6-A landing). Phase-2b Wave-8i scope update: WAIT now also
/// has a real dispatcher path (`primitives/mod.rs` Wait arm routes to
/// `wait::evaluate_op`), surfacing as `EvalError::WaitSuspended`
/// instead of `PrimitiveNotImplemented`. Only `Sandbox` keeps the
/// pre-Phase-2b `PrimitiveNotImplemented` posture under the NullHost
/// (because the NullHost's `execute_sandbox` default still rejects;
/// the engine impl overrides to actually invoke wasmtime).
#[test]
fn sandbox_call_time_error() {
    let mut ev = Evaluator::new();
    let kind = PrimitiveKind::Sandbox;
    let op = OperationNode::new(format!("{kind:?}"), kind);
    let err = ev
        .step(&op, &NullHost)
        .expect_err("SANDBOX must fail at call time under NullHost");
    assert!(
        matches!(err, EvalError::PrimitiveNotImplemented(k) if k == kind),
        "expected PrimitiveNotImplemented({kind:?}), got {err:?}"
    );
}

/// Phase-2b Wave-8i: WAIT routes through the eval-side `wait::evaluate_op`
/// from the regular dispatcher path. A `step` call against a WAIT
/// OperationNode now surfaces `EvalError::WaitSuspended { handle }`
/// rather than the pre-Wave-8i `PrimitiveNotImplemented(Wait)` shape.
#[test]
fn wait_call_time_surfaces_wait_suspended() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("wait", PrimitiveKind::Wait);
    let err = ev
        .step(&op, &NullHost)
        .expect_err("WAIT must surface a control-flow signal at call time");
    assert!(
        matches!(err, EvalError::WaitSuspended { .. }),
        "expected WaitSuspended, got {err:?}"
    );
}

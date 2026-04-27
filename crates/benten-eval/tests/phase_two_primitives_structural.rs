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
#[test]
fn wait_stream_subscribe_sandbox_call_time_error() {
    let mut ev = Evaluator::new();
    for kind in [
        PrimitiveKind::Wait,
        PrimitiveKind::Stream,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Sandbox,
    ] {
        let op = OperationNode::new(format!("{kind:?}"), kind);
        let err = ev
            .step(&op, &NullHost)
            .expect_err("Phase-2 primitives must fail at call time");
        assert!(
            matches!(err, EvalError::PrimitiveNotImplemented(k) if k == kind),
            "expected PrimitiveNotImplemented({kind:?}), got {err:?}"
        );
    }
}

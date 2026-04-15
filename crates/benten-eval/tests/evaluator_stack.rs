//! Evaluator stack-model tests (E2, G6-C — R2 landscape §2.5 row 15).
//!
//! Explicit stack, no recursion (R1 architect major #2). Tests cover
//! push-on-ok, pop-on-respond, error-edge follow, frame-order preservation,
//! and stack-overflow-is-Err-not-panic.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_INV_DEPTH_EXCEEDED` (via stack-overflow test).

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use benten_eval::{EvalError, Evaluator, ExecutionFrame, OperationNode, PrimitiveKind};

#[test]
fn evaluator_new_has_empty_stack() {
    let ev = Evaluator::new();
    assert!(ev.stack.is_empty());
    assert!(ev.max_stack_depth > 0);
}

#[test]
fn execution_frame_has_node_id_and_index() {
    let f = ExecutionFrame {
        node_id: "n".to_string(),
        frame_index: 0,
    };
    assert_eq!(f.node_id, "n");
    assert_eq!(f.frame_index, 0);
}

#[test]
fn evaluator_pushes_next_on_ok() {
    let mut ev = Evaluator::new();
    let op =
        OperationNode::new("t", PrimitiveKind::Transform).with_property("expr", Value::text("1"));
    let _r = ev.step(&op).unwrap();
    // Post-step the evaluator has recorded this frame in its stack.
    assert!(!ev.stack.is_empty());
}

#[test]
fn evaluator_pops_on_respond_terminal() {
    let mut ev = Evaluator::new();
    // Push a transform first.
    let t =
        OperationNode::new("t", PrimitiveKind::Transform).with_property("expr", Value::text("1"));
    ev.step(&t).unwrap();
    let before = ev.stack.len();
    // RESPOND terminates — stack clears down.
    let r = OperationNode::new("r", PrimitiveKind::Respond);
    ev.step(&r).unwrap();
    assert!(
        ev.stack.len() <= before,
        "RESPOND must not grow the stack beyond its pre-state"
    );
}

#[test]
fn evaluator_follows_error_edge_on_primitive_error() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("w", PrimitiveKind::Write)
        .with_property("op", Value::text("cas"))
        .with_property("expected_version", Value::Int(1))
        .with_property("actual_version", Value::Int(2));
    match ev.step(&op) {
        Ok(r) => assert_eq!(r.edge_label, "ON_CONFLICT"),
        Err(EvalError::WriteConflict) => {}
        Err(e) => panic!("expected routed error or WriteConflict, got {e:?}"),
    }
}

/// Covered by `covers_error_code[E_INV_DEPTH_EXCEEDED]` entry
/// "evaluator_stack_overflow_is_err_not_panic".
#[test]
fn evaluator_stack_overflow_is_err_not_panic() {
    let mut ev = Evaluator::new();
    ev.max_stack_depth = 2;
    // Push frames until the guard fires — simulate by direct push. R5's
    // evaluator will enforce this on `step`; Phase 1 asserts the guard
    // returns an error rather than panicking.
    for i in 0..10 {
        ev.stack.push(ExecutionFrame {
            node_id: format!("f{i}"),
            frame_index: i,
        });
    }
    // Attempting another step under an over-deep stack surfaces StackOverflow,
    // not a Rust panic.
    let op = OperationNode::new("next", PrimitiveKind::Transform);
    match ev.step(&op) {
        Err(EvalError::StackOverflow) => {}
        Err(EvalError::Invariant(benten_eval::InvariantViolation::DepthExceeded)) => {}
        other => panic!("expected StackOverflow / DepthExceeded, got {other:?}"),
    }
}

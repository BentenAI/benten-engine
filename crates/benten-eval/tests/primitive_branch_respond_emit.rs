//! BRANCH + RESPOND + EMIT primitive happy paths (E3 — R2 landscape §2.5
//! rows 11, 13, 14).
//!
//! BRANCH: binary + multi-way + default fallthrough.
//! RESPOND: evaluator terminates after RESPOND; response bytes available.
//! EMIT: fire-and-forget, does not block evaluator.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use benten_eval::{Evaluator, OperationNode, PrimitiveKind};

#[test]
fn branch_binary_true_case_routes_to_true_edge() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("b1", PrimitiveKind::Branch)
        .with_property("condition_value", Value::Bool(true));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "true");
}

#[test]
fn branch_binary_false_case_routes_to_false_edge() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("b1", PrimitiveKind::Branch)
        .with_property("condition_value", Value::Bool(false));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "false");
}

#[test]
fn branch_multiway_picks_matching_case() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("b1", PrimitiveKind::Branch)
        .with_property("match_value", Value::text("published"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "published");
}

#[test]
fn branch_no_match_routes_to_on_default() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("b1", PrimitiveKind::Branch)
        .with_property("match_value", Value::text("unknown_case"))
        .with_property("has_default", Value::Bool(true));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "ON_DEFAULT");
}

#[test]
fn respond_primitive_halts_evaluator_with_terminal_edge() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("r1", PrimitiveKind::Respond)
        .with_property("status", Value::Int(200))
        .with_property("body", Value::text("hello"));
    let r = ev.step(&op).unwrap();
    assert_eq!(r.edge_label, "terminal");
    assert_eq!(r.next, None);
}

#[test]
fn respond_primitive_exposes_response_body() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("r1", PrimitiveKind::Respond)
        .with_property("body", Value::text("hello"));
    let r = ev.step(&op).unwrap();
    match r.output {
        Value::Map(m) => assert_eq!(m.get("body"), Some(&Value::text("hello"))),
        other => panic!("expected response map, got {other:?}"),
    }
}

#[test]
fn emit_primitive_does_not_block_evaluator() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("e1", PrimitiveKind::Emit)
        .with_property("channel", Value::text("audit"))
        .with_property("payload", Value::text("write happened"));
    let r = ev.step(&op).unwrap();
    // Fire-and-forget: evaluator proceeds on the "ok" edge.
    assert_eq!(r.edge_label, "ok");
}

#[test]
fn emit_primitive_output_is_noop_marker() {
    let mut ev = Evaluator::new();
    let op = OperationNode::new("e1", PrimitiveKind::Emit);
    let r = ev.step(&op).unwrap();
    // EMIT outputs nothing to the evaluator graph.
    assert_eq!(r.output, Value::Null);
}

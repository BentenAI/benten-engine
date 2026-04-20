//! 5d-J workstream 3 — TRANSFORM expressions parse at registration time.
//!
//! Before this workstream, an unparseable TRANSFORM expression
//! survived `register_subgraph` and surfaced only when `engine.call`
//! walked the TRANSFORM node for the first time. Now the
//! registration path walks every TRANSFORM node's `expr` property
//! through the parser and fails fast with `E_TRANSFORM_SYNTAX`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Value;
use benten_engine::Engine;
use benten_eval::{OperationNode, PrimitiveKind, Subgraph};

fn build_subgraph_with_transform_expr(expr: &str, handler_id: &str) -> Subgraph {
    // Hand-construct a two-node subgraph (TRANSFORM -> RESPOND) so we
    // can place an arbitrary `expr` property on the TRANSFORM node.
    // The SubgraphBuilder API doesn't surface direct property
    // injection, so we reach through `Subgraph::with_node` /
    // `with_edge` for this test fixture only.
    let transform_node = OperationNode::new("t0", PrimitiveKind::Transform)
        .with_property("expr", Value::Text(expr.to_string()));
    let respond_node = OperationNode::new("r0", PrimitiveKind::Respond);
    Subgraph::new(handler_id)
        .with_node(transform_node)
        .with_node(respond_node)
        .with_edge("t0", "r0", "ok")
}

#[test]
fn transform_registration_rejects_unparseable_expression() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // `++` is not an operator in the Phase-1 TRANSFORM grammar.
    let bad = build_subgraph_with_transform_expr("x ++ y", "bad_transform");

    let err = engine
        .register_subgraph(bad)
        .expect_err("registration must reject an unparseable TRANSFORM expr");
    assert_eq!(
        err.error_code().as_str(),
        "E_TRANSFORM_SYNTAX",
        "the fail-fast guarantee must surface the grammar rejection at \
         register_subgraph time, not at engine.call time"
    );
}

#[test]
fn transform_registration_accepts_valid_expression() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let good = build_subgraph_with_transform_expr("$input.title", "good_transform");
    engine
        .register_subgraph(good)
        .expect("valid TRANSFORM expression must register cleanly");
}

#[test]
fn transform_without_expr_property_registers_cleanly() {
    // A TRANSFORM node that lacks an `expr` property is NOT a
    // registration failure — the runtime routes it through ON_ERROR.
    // This keeps the registration-time parse a pure syntax check.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let sg = Subgraph::new("no_expr")
        .with_node(OperationNode::new("t0", PrimitiveKind::Transform))
        .with_node(OperationNode::new("r0", PrimitiveKind::Respond))
        .with_edge("t0", "r0", "ok");
    engine
        .register_subgraph(sg)
        .expect("missing `expr` is a runtime ON_ERROR, not a registration-time failure");
}

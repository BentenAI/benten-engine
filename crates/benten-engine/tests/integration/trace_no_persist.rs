//! Regression for r6-dx-C4: `Engine::trace` must not persist writes.
//!
//! Tracing a `crud:create` previously emitted a ChangeEvent, stamped a
//! Node into redb, and perturbed View-3's sort order for the rest of the
//! engine lifetime. The fix runs the evaluator walk in "trace mode" and
//! drops buffered host ops rather than replaying them.
//!
//! This test is the authoritative regression gate: if it fails, the
//! trace path has regained its side effect.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests exercise panics explicitly"
)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post_node(title: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    Node::new(vec!["post".into()], props)
}

#[test]
fn trace_does_not_persist_writes() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .expect("engine opens");

    let handler = engine.register_crud("post").expect("registers");

    // Baseline: listing before any activity is empty.
    let before = engine
        .call(&handler, "list", Node::empty())
        .expect("list before");
    let before_list = before.as_list().expect("list vec present");
    assert!(
        before_list.is_empty(),
        "baseline list must be empty, got {} entries",
        before_list.len()
    );

    // Trace a create. The trace itself produces an Outcome (with a
    // projected created_cid), but NOTHING must land in the backend.
    let trace = engine
        .trace(&handler, "create", post_node("traced-but-not-persisted"))
        .expect("trace succeeds");
    assert!(
        !trace.steps().is_empty(),
        "trace should have at least one step"
    );

    // The post-trace listing must STILL be empty. If this fails, trace
    // is persisting again.
    let after = engine
        .call(&handler, "list", Node::empty())
        .expect("list after");
    let after_list = after.as_list().expect("list vec present");
    assert!(
        after_list.is_empty(),
        "trace must not persist writes — list should still be empty, got {} entries",
        after_list.len()
    );

    // A subsequent *real* call() must still work — trace mode is
    // per-call, never sticky.
    let real = engine
        .call(&handler, "create", post_node("actually-persisted"))
        .expect("real create");
    assert!(
        real.created_cid().is_some(),
        "real create should produce a created_cid"
    );
    let real_list = engine
        .call(&handler, "list", Node::empty())
        .expect("list after real create");
    assert_eq!(
        real_list.as_list().expect("list vec").len(),
        1,
        "real create must persist exactly one entry"
    );
}

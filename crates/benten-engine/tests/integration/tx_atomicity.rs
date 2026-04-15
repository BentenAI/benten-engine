//! Phase 1 R3 integration — Transaction atomicity end-to-end.
//!
//! Register a subgraph with two WRITE primitives; inject a failure in the
//! second WRITE; assert the first WRITE was rolled back via redb. Exercises
//! G3 (transaction primitive, change stream commit-point semantics), E6
//! (engine-level transaction surface), and the failure-injection hook E3
//! exposes to tests.
//!
//! Error code asserted: E_TX_ABORTED.
//!
//! **Status:** FAILING until G3 + G6 + G7 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, OutcomeExt};

fn two_write_subgraph_with_injected_failure_on_second() -> benten_engine::SubgraphSpec {
    benten_engine::SubgraphSpec::builder()
        .handler_id("tx:atomic_test")
        .write(|w| w.label("post").property("n", Value::Int(1)))
        .write(|w| {
            w.label("post")
                .property("n", Value::Int(2))
                .test_inject_failure(true)
        })
        .respond()
        .build()
}

#[test]
fn transaction_atomicity_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine
        .register_subgraph(two_write_subgraph_with_injected_failure_on_second())
        .unwrap();

    let before = engine.count_nodes_with_label("post").unwrap();
    let outcome = engine
        .call(&handler_id, "tx:atomic_test", Node::empty())
        .expect("call returns Ok");
    assert!(outcome.routed_through_edge("ON_ERROR"));
    assert_eq!(outcome.error_code(), Some("E_TX_ABORTED"));
    let after = engine.count_nodes_with_label("post").unwrap();
    assert_eq!(before, after, "both WRITEs must roll back");
}

#[test]
fn successful_transaction_commits_both_writes() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let sg = benten_engine::SubgraphSpec::builder()
        .handler_id("tx:happy")
        .write(|w| w.label("post").property("n", Value::Int(1)))
        .write(|w| w.label("post").property("n", Value::Int(2)))
        .respond()
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();
    let before = engine.count_nodes_with_label("post").unwrap();
    let outcome = engine.call(&handler_id, "tx:happy", Node::empty()).unwrap();
    assert!(outcome.is_ok_edge());
    let after = engine.count_nodes_with_label("post").unwrap();
    assert_eq!(after, before + 2, "both WRITEs must commit");
}

#[test]
fn ivm_change_events_emit_only_at_commit_not_per_write() {
    // Protects G7 invariant: events emit at commit, not per WRITE.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine
        .register_subgraph(two_write_subgraph_with_injected_failure_on_second())
        .unwrap();
    let before_events = engine.change_event_count();
    let _ = engine
        .call(&handler_id, "tx:atomic_test", Node::empty())
        .unwrap();
    let after_events = engine.change_event_count();
    assert_eq!(
        before_events, after_events,
        "rolled-back tx must emit zero ChangeEvents"
    );
}

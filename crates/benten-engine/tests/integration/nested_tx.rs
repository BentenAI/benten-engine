//! Phase 1 R3 integration — Nested transactions rejected end-to-end.
//!
//! The engine public API wraps a closure-based transaction. Attempting to
//! call `engine.transaction` from inside another transaction closure must
//! return E_NESTED_TRANSACTION_NOT_SUPPORTED. This prevents implicit
//! nested-MVCC regressions.
//!
//! Complements the single-crate unit test at
//! `crates/benten-graph/tests/nested_tx_rejected.rs` (trait-level) with the
//! engine-level surface.
//!
//! **Status:** FAILING until G3-A + N6 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

#[test]
fn nested_transaction_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // G3 fix-pass (test-authoring correction): `?`-propagate the nested Err
    // rather than swallowing it into `Ok(nested_result)`. The earlier form
    // returned Ok(Err(...)) which made the outer engine.transaction observe
    // an Ok closure-return — the whole point of the test (outer surfaces
    // E_NESTED_TRANSACTION_NOT_SUPPORTED) was defeated.
    let result: Result<(), _> = engine.transaction(|tx| {
        let mut p = BTreeMap::new();
        p.insert("n".into(), Value::Int(1));
        tx.create_node(&Node::new(vec!["post".into()], p))?;

        // Attempt to nest. The `?` propagates the rejection so the outer
        // engine.transaction sees the closure return Err.
        let _nested = tx.begin_nested()?;
        unreachable!("begin_nested must return Err in Phase 1");
    });

    let err = result.expect_err("nested tx must return Err");
    assert_eq!(err.code(), "E_NESTED_TRANSACTION_NOT_SUPPORTED");
}

#[test]
fn sequential_transactions_commit_independently() {
    // Ensure the rejection does not accidentally poison the outer transaction
    // state for subsequent calls.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    for n in 0..5i64 {
        engine
            .transaction(|tx| {
                let mut p = BTreeMap::new();
                p.insert("n".into(), Value::Int(n));
                tx.create_node(&Node::new(vec!["post".into()], p))?;
                Ok(())
            })
            .expect("each transaction commits cleanly");
    }
    assert_eq!(engine.count_nodes_with_label("post").unwrap(), 5);
}

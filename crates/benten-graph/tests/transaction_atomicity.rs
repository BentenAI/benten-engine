//! Transaction primitive atomicity (R2 landscape §2.2 row 7).
//!
//! Closure-based API: `backend.transaction(|tx| ...)`. All writes inside the
//! closure commit atomically, or none do. Phase 1 G3-A stub — these tests
//! fail until the transaction primitive lands.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Codes fired: `E_TX_ABORTED` (via `tx_aborts_on_closure_err`).

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{GraphError, KVBackend, RedbBackend};
use tempfile::TempDir;

fn temp_backend() -> (RedbBackend, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let b = RedbBackend::open(dir.path().join("t.redb")).unwrap();
    (b, dir)
}

#[test]
fn tx_commits_all_writes_together() {
    let (b, _d) = temp_backend();
    let node = canonical_test_node();
    let cid = b
        .transaction(|tx| {
            tx.put_node(&node)?;
            Ok(node.cid().unwrap())
        })
        .unwrap();
    // After commit, the node is visible.
    assert!(b.get(cid.as_bytes()).unwrap().is_some());
}

#[test]
fn tx_aborts_on_closure_err() {
    // Covered by `covers_error_code[E_TX_ABORTED]`.
    let (b, _d) = temp_backend();
    let node = canonical_test_node();
    let cid_before_tx = node.cid().unwrap();

    let res: Result<(), GraphError> = b.transaction(|tx| {
        tx.put_node(&node)?;
        Err(GraphError::TxAborted {
            reason: "injected".to_string(),
        })
    });

    assert!(matches!(res, Err(GraphError::TxAborted { .. })));
    // Rolled back — node must NOT be visible.
    assert_eq!(b.get(cid_before_tx.as_bytes()).unwrap(), None);
}

#[test]
fn tx_two_writes_both_visible_after_commit() {
    let (b, _d) = temp_backend();
    let a = canonical_test_node();
    // Build a distinct second Node so put_node stores two entries.
    let mut b_node = canonical_test_node();
    b_node
        .properties
        .insert("extra".to_string(), benten_core::Value::Int(9));

    b.transaction(|tx| {
        tx.put_node(&a)?;
        tx.put_node(&b_node)?;
        Ok(())
    })
    .unwrap();

    assert!(b.get(a.cid().unwrap().as_bytes()).unwrap().is_some());
    assert!(b.get(b_node.cid().unwrap().as_bytes()).unwrap().is_some());
}

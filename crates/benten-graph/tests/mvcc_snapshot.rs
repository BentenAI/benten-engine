//! MVCC snapshot isolation via redb (G6 — R2 landscape §2.2 row 15).
//!
//! Reader A opens a snapshot; writer B commits a new value; reader A still
//! sees the old value until the snapshot is dropped and re-opened. redb
//! provides this via read-transactions that outlive concurrent writes.
//!
//! Rewritten at R4 triage (M6) — the v1 body exercised sequential puts with
//! different CIDs, which does not actually test point-in-time isolation.
//! The corrected body:
//!   (a) opens a snapshot (read-side handle),
//!   (b) writes to the same backend concurrently,
//!   (c) asserts the snapshot reader still observes pre-write state,
//!   (d) drops the snapshot,
//!   (e) asserts a fresh reader now sees post-write state.
//!
//! R3 writer: `rust-test-writer-unit`. R5 must expose `snapshot()` on
//! `RedbBackend` for this test to compile.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_core::{Node, Value};
use benten_graph::RedbBackend;
use std::collections::BTreeMap;
use tempfile::TempDir;

fn temp() -> (RedbBackend, TempDir) {
    let d = tempfile::tempdir().unwrap();
    let b = RedbBackend::open(d.path().join("t.redb")).unwrap();
    (b, d)
}

#[test]
fn snapshot_reader_sees_point_in_time_state() {
    let (backend, _d) = temp();

    // Seed v1 under a distinct key so the snapshot has something non-empty
    // to observe.
    let seed = canonical_test_node();
    let seed_cid = backend.put_node(&seed).unwrap();

    // (a) Open a snapshot (pre-write view).
    let snapshot = backend.snapshot().expect("snapshot() handle");

    // (b) Concurrent write that arrives AFTER the snapshot opened.
    let mut p = BTreeMap::new();
    p.insert("v".to_string(), Value::Int(99));
    let v2 = Node::new(vec!["Other".to_string()], p);
    let v2_cid = backend.put_node(&v2).unwrap();

    // (c) Snapshot reader must still see the pre-write state — seed is
    // visible, v2 (written after snapshot) is NOT.
    assert_eq!(
        snapshot.get_node(&seed_cid).unwrap(),
        Some(seed),
        "snapshot must see the pre-write seed value"
    );
    assert!(
        snapshot.get_node(&v2_cid).unwrap().is_none(),
        "snapshot must NOT observe writes that arrived after it opened"
    );

    // (d) Drop the snapshot.
    drop(snapshot);

    // (e) A fresh reader (no snapshot held) now sees post-write state.
    assert!(
        backend.get_node(&v2_cid).unwrap().is_some(),
        "after snapshot drop, a fresh read must observe the committed write"
    );
}

#[test]
fn delete_then_read_returns_none() {
    use benten_graph::KVBackend;
    let (b, _d) = temp();
    let n = canonical_test_node();
    let cid = b.put_node(&n).unwrap();
    b.delete(cid.as_bytes()).unwrap();
    assert_eq!(b.get_node(&cid).unwrap(), None);
}

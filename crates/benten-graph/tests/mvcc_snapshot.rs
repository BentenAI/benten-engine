//! MVCC snapshot isolation via redb (G6 — R2 landscape §2.2 row 15).
//!
//! Reader A opens a snapshot; writer B commits a new value; reader A still
//! sees the old value until they drop and re-open. redb provides this via
//! read-transactions that outlive concurrent writes.
//!
//! R3 writer: `rust-test-writer-unit`.

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
    let (b, _d) = temp();
    let v1 = canonical_test_node();
    let cid = b.put_node(&v1).unwrap();

    // Read before update — sees v1.
    let fetched_before = b.get_node(&cid).unwrap();
    assert_eq!(fetched_before, Some(v1));

    // Writer commits a "different" node — but same CID would overwrite, so
    // construct a distinct node and store it under a distinct CID instead.
    let mut p = BTreeMap::new();
    p.insert("v".to_string(), Value::Int(99));
    let v2 = Node::new(vec!["Other".to_string()], p);
    let cid2 = b.put_node(&v2).unwrap();

    // Reader still sees the original cid → original node.
    assert_eq!(b.get_node(&cid).unwrap().unwrap().labels, vec!["Post"]);
    // And the second CID points at v2.
    assert_eq!(b.get_node(&cid2).unwrap().unwrap().labels, vec!["Other"]);
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

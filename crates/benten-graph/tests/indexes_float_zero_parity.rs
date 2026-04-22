//! Regression test (G2 mini-review finding g2-ar-5): the cross-crate
//! invariant "`Value::Float(0.0)` and `Value::Float(-0.0)` share a CID"
//! from benten-core must also resolve to the same property-index bucket.
//!
//! The G1 invariant lives in `benten-core::Value` — positive and negative
//! zero are canonicalized to the same DAG-CBOR bytes before hashing, so
//! two Nodes that differ only by `0.0` vs. `-0.0` produce the same CID.
//! This test pins the parallel property at the graph-index layer: the
//! `property_index_key` for either sign bits must encode identically, and
//! `get_by_property` with either sign must return the same single CID.

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use benten_graph::RedbBackend;
use std::collections::BTreeMap;
use tempfile::TempDir;

fn temp() -> (RedbBackend, TempDir) {
    let d = tempfile::tempdir().unwrap();
    let b = RedbBackend::open_or_create(d.path().join("t.redb")).unwrap();
    (b, d)
}

fn node_with_score(score: f64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("score".to_string(), Value::Float(score));
    Node::new(vec!["Metric".to_string()], p)
}

#[test]
fn neg_zero_and_pos_zero_share_cid_and_share_index_bucket() {
    let (b, _d) = temp();

    let pos = node_with_score(0.0);
    let neg = node_with_score(-0.0);

    // G1 invariant: same CID. If this fails, the bug is in benten-core,
    // not here — which is exactly why the cross-crate pin is valuable.
    assert_eq!(
        pos.cid().unwrap(),
        neg.cid().unwrap(),
        "Value::Float(0.0) and Value::Float(-0.0) must share a CID"
    );

    let cid = b.put_node(&pos).unwrap();
    // Re-put via the negative-zero Node — idempotent at the node body
    // (same CID) AND at the property index (same encoded value bytes).
    let cid_again = b.put_node(&neg).unwrap();
    assert_eq!(cid, cid_again);

    // Query by either sign — both must return the same single CID.
    let by_pos = b
        .get_by_property("Metric", "score", &Value::Float(0.0))
        .unwrap();
    assert_eq!(by_pos, vec![cid]);

    let by_neg = b
        .get_by_property("Metric", "score", &Value::Float(-0.0))
        .unwrap();
    assert_eq!(by_neg, vec![cid]);
}

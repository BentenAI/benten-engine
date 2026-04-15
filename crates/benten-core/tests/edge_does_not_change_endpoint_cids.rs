//! Protects ENGINE-SPEC §7: Node CIDs are determined only by labels +
//! properties. Edges are content-addressed separately and their existence must
//! not alter endpoint Node CIDs (R2 landscape §2.1 row 9).
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_core::{Edge, Node, Value};
use std::collections::BTreeMap;

fn node_a() -> Node {
    canonical_test_node()
}

fn node_b() -> Node {
    let mut p = BTreeMap::new();
    p.insert("title".to_string(), Value::text("Another"));
    p.insert("views".to_string(), Value::Int(0));
    Node::new(vec!["Post".to_string()], p)
}

#[test]
fn creating_an_edge_does_not_alter_endpoint_node_cids() {
    let a = node_a();
    let b = node_b();

    let cid_a_before = a.cid().unwrap();
    let cid_b_before = b.cid().unwrap();

    // Create an Edge between A and B.
    let _edge = Edge::new(cid_a_before.clone(), cid_b_before.clone(), "LIKES", None);

    // Re-hash both endpoints. Their CIDs must be unchanged.
    let cid_a_after = node_a().cid().unwrap();
    let cid_b_after = node_b().cid().unwrap();

    assert_eq!(cid_a_before, cid_a_after);
    assert_eq!(cid_b_before, cid_b_after);
}

#[test]
fn edge_with_different_label_still_does_not_alter_endpoint_cids() {
    // Belt-and-suspenders: different edge label → still no effect on endpoints.
    let a = node_a();
    let cid_a = a.cid().unwrap();
    let _e1 = Edge::new(cid_a.clone(), cid_a.clone(), "L1", None);
    let _e2 = Edge::new(cid_a.clone(), cid_a.clone(), "L2", None);
    let cid_a_reread = node_a().cid().unwrap();
    assert_eq!(cid_a, cid_a_reread);
}

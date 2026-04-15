//! Anchor + version-chain primitive tests (C6, R2 landscape §2.1 row 13).
//!
//! Phase 1 G1-B stub — tests drive the `Anchor`, `CURRENT` / `NEXT_VERSION`
//! edge labels, and `walk_versions` / `current_version` / `append_version`
//! helpers.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::{
    Anchor, LABEL_CURRENT, LABEL_NEXT_VERSION, Node, Value, append_version, current_version,
    walk_versions,
};
use std::collections::BTreeMap;

fn v(n: i64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("v".to_string(), Value::Int(n));
    Node::new(vec!["Post".to_string()], p)
}

#[test]
fn anchor_new_produces_stable_identity() {
    let a = Anchor::new();
    let b = Anchor::new();
    // A fresh Anchor constructor produces distinct ids — Phase 1 shape
    // asserted: id is u64. Two independent constructions must not collide
    // accidentally (probabilistic; a 64-bit space makes same-bit collision
    // extraordinarily unlikely).
    assert_ne!(
        a.id, b.id,
        "distinct Anchor::new() calls yield distinct ids"
    );
}

#[test]
fn append_version_sets_current_to_first_version() {
    let anchor = Anchor::new();
    let v1 = v(1);
    let cid1 = append_version(&anchor, &v1).unwrap();
    assert_eq!(current_version(&anchor).unwrap(), cid1);
}

#[test]
fn second_append_advances_current() {
    let anchor = Anchor::new();
    let v1 = v(1);
    let v2 = v(2);
    let _cid1 = append_version(&anchor, &v1).unwrap();
    let cid2 = append_version(&anchor, &v2).unwrap();
    assert_eq!(current_version(&anchor).unwrap(), cid2);
}

#[test]
fn walk_versions_returns_chain_in_oldest_first_order() {
    let anchor = Anchor::new();
    let v1 = v(1);
    let v2 = v(2);
    let v3 = v(3);
    let cid1 = append_version(&anchor, &v1).unwrap();
    let cid2 = append_version(&anchor, &v2).unwrap();
    let cid3 = append_version(&anchor, &v3).unwrap();
    let chain = walk_versions(&anchor).unwrap();
    assert_eq!(chain, vec![cid1, cid2, cid3]);
}

#[test]
fn edge_label_constants_match_spec() {
    // Spec §6: the edge labels are "CURRENT" and "NEXT_VERSION".
    assert_eq!(LABEL_CURRENT, "CURRENT");
    assert_eq!(LABEL_NEXT_VERSION, "NEXT_VERSION");
}

#[test]
fn version_node_cid_is_content_only_unaffected_by_chain_position() {
    // Appending a Version Node to an Anchor must not change the Node's own CID
    // (chain membership is expressed via edges, not via Node properties).
    let v1 = v(1);
    let cid_standalone = v1.cid().unwrap();
    let anchor = Anchor::new();
    let cid_in_chain = append_version(&anchor, &v1).unwrap();
    assert_eq!(cid_standalone, cid_in_chain);
}

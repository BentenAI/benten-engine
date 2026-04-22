//! `Edge::new` and `Edge::cid()` contract tests (C2, R2 landscape §2.1 rows 7-8).
//!
//! R3 writer: `rust-test-writer-unit`. Phase 1 G1-B stub — tests exercise the
//! Edge type which today is `todo!()`-bodied.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_core::{Edge, Value};
use std::collections::BTreeMap;

fn fixture_cid() -> benten_core::Cid {
    canonical_test_node().cid().unwrap()
}

#[test]
fn edge_new_with_empty_properties() {
    let src = fixture_cid();
    let tgt = fixture_cid();
    let e = Edge::new(src, tgt, "LIKES", None);
    assert_eq!(e.label, "LIKES");
    assert_eq!(e.source, src);
    assert_eq!(e.target, tgt);
    assert!(e.properties.is_none());
}

#[test]
fn edge_new_with_populated_properties() {
    let src = fixture_cid();
    let tgt = fixture_cid();
    let mut props = BTreeMap::new();
    props.insert("weight".to_string(), Value::Int(7));
    let e = Edge::new(src, tgt, "LIKES", Some(props.clone()));
    assert_eq!(e.properties, Some(props));
}

#[test]
fn edge_allows_self_loop() {
    // Invariant 1 (DAG-ness) is a subgraph-level check, not edge-level.
    let c = fixture_cid();
    let e = Edge::new(c, c, "SELF", None);
    // Constructible + hashable.
    let _cid = e.cid().unwrap();
}

#[test]
fn edge_cid_stable_across_reconstructions() {
    let c = fixture_cid();
    let e1 = Edge::new(c, c, "L", None);
    let e2 = Edge::new(c, c, "L", None);
    assert_eq!(e1.cid().unwrap(), e2.cid().unwrap());
}

#[test]
fn edge_none_props_vs_empty_map_produce_different_cids() {
    // Per DAG-CBOR: missing field ≠ empty map. Edges preserve that distinction.
    let c = fixture_cid();
    let none = Edge::new(c, c, "L", None);
    let empty = Edge::new(c, c, "L", Some(BTreeMap::new()));
    assert_ne!(
        none.cid().unwrap(),
        empty.cid().unwrap(),
        "None vs empty map must be preserved in the CID"
    );
}

#[test]
fn edge_different_source_produces_different_cid() {
    let src_a = fixture_cid();
    // Construct a second distinct CID via a different Node content.
    let mut p = BTreeMap::new();
    p.insert("differ".to_string(), Value::Int(1));
    let other = benten_core::Node::new(vec!["Other".to_string()], p);
    let src_b = other.cid().unwrap();

    let tgt = fixture_cid();
    let e_a = Edge::new(src_a, tgt, "L", None);
    let e_b = Edge::new(src_b, tgt, "L", None);
    assert_ne!(e_a.cid().unwrap(), e_b.cid().unwrap());
}

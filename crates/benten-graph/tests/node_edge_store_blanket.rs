//! `NodeStore` / `EdgeStore` blanket impls over `KVBackend` (G4, G2-A — R2
//! landscape §2.2 row 12).
//!
//! Phase 1 G4 stub — lifts `put_node` / `get_node` / `put_edge` / `get_edge` /
//! `edges_from` / `edges_to` off `RedbBackend`. These tests exercise the
//! blanket-impl path; R5 implements.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_core::{Edge, Value};
use benten_graph::RedbBackend;
use std::collections::BTreeMap;
use tempfile::TempDir;

fn temp() -> (RedbBackend, TempDir) {
    let d = tempfile::tempdir().unwrap();
    let b = RedbBackend::open(d.path().join("t.redb")).unwrap();
    (b, d)
}

#[test]
fn node_store_put_then_get_returns_same_node() {
    let (b, _d) = temp();
    let n = canonical_test_node();
    let cid = b.put_node(&n).unwrap();
    assert_eq!(b.get_node(&cid).unwrap(), Some(n));
}

#[test]
fn edge_store_put_then_get_returns_same_edge() {
    let (b, _d) = temp();
    let n1 = canonical_test_node();
    let cid1 = b.put_node(&n1).unwrap();
    let mut p = BTreeMap::new();
    p.insert("w".to_string(), Value::Int(3));
    let edge = Edge::new(cid1.clone(), cid1.clone(), "LIKES", Some(p));
    let ecid = b.put_edge(&edge).unwrap();
    assert_eq!(b.get_edge(&ecid).unwrap(), Some(edge));
}

#[test]
fn edges_from_returns_outgoing_only() {
    let (b, _d) = temp();
    let n = canonical_test_node();
    let c = b.put_node(&n).unwrap();
    let e = Edge::new(c.clone(), c.clone(), "L", None);
    b.put_edge(&e).unwrap();

    let out = b.edges_from(&c).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].source, c);
}

#[test]
fn edges_to_returns_incoming_only() {
    let (b, _d) = temp();
    let n = canonical_test_node();
    let c = b.put_node(&n).unwrap();
    let e = Edge::new(c.clone(), c.clone(), "L", None);
    b.put_edge(&e).unwrap();

    let incoming = b.edges_to(&c).unwrap();
    assert_eq!(incoming.len(), 1);
    assert_eq!(incoming[0].target, c);
}

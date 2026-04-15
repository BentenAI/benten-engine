//! `Node::cid()` determinism + the canonical fixture test (R2 landscape §2.1
//! rows 4-5).
//!
//! R3 writer: `rust-test-writer-unit`. Exit-criterion #6 (TS ↔ Rust CID
//! round-trip) consumes the same fixture — this is the Rust side of the
//! mirror.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_core::{Node, Value};
use std::collections::BTreeMap;

/// Hard-coded canonical CID from `docs/spike/RESULTS.md` / the
/// exit-criterion #6 fixture. Any drift here = cross-process determinism broken.
const CANONICAL_CID: &str = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

#[test]
fn canonical_fixture_cid_matches_exit_criterion_value() {
    let cid = canonical_test_node().cid().unwrap();
    assert_eq!(
        cid.to_string(),
        CANONICAL_CID,
        "canonical fixture CID must stay pinned across every R3 run"
    );
}

#[test]
fn two_structurally_equal_nodes_produce_identical_cids() {
    let a = canonical_test_node();
    let b = canonical_test_node();
    assert_eq!(a.cid().unwrap(), b.cid().unwrap());
    // Also byte-identical.
    assert_eq!(a.cid().unwrap().as_bytes(), b.cid().unwrap().as_bytes());
}

#[test]
fn anchor_id_never_affects_cid() {
    let mut with_anchor = canonical_test_node();
    with_anchor.anchor_id = Some(42);
    let baseline = canonical_test_node().cid().unwrap();
    assert_eq!(baseline, with_anchor.cid().unwrap());
}

#[test]
fn cid_string_begins_with_multibase_prefix_b() {
    let cid = canonical_test_node().cid().unwrap();
    assert!(cid.to_string().starts_with('b'));
}

#[test]
fn different_label_changes_cid() {
    // Trivial sanity: same properties, different label → different CID.
    let mut p = BTreeMap::new();
    p.insert("k".to_string(), Value::Int(1));
    let a = Node::new(vec!["A".to_string()], p.clone());
    let b = Node::new(vec!["B".to_string()], p);
    assert_ne!(a.cid().unwrap(), b.cid().unwrap());
}

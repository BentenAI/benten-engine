//! Fwd-1 #926 + #932 CID-stability pin (umbrella #1147).
//!
//! The TIER-1 perf pass introduced two behavior-preserving optimizations:
//!
//! 1. `Node`/`Edge::to_canonical_bytes` skip the `Value::to_canonical` deep-clone
//!    walk when every property is already canonical (#932).
//! 2. `Node`/`Edge`/`Subgraph::cid_and_canonical_bytes` fuse the encode+hash
//!    so production WRITE paths don't double-encode (#926).
//!
//! Canonical-bytes byte-identity is a HARD content-addressing invariant. These
//! tests would FAIL if either optimization perturbed a single output byte.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_core::{Edge, Node, Value};
use std::collections::BTreeMap;

const CANONICAL_CID: &str = "bafyr4iflzldgzjrtknevsib24ewiqgtj65pm2ituow3yxfpq57nfmwduda";

/// The fused method must produce the SAME cid + the SAME bytes as the two
/// separate calls — for the fixture node (drives the pinned fixture CID).
#[test]
fn fixture_fused_matches_separate_and_pinned_cid() {
    let n = canonical_test_node();
    let cid_sep = n.cid().unwrap();
    let bytes_sep = n.to_canonical_bytes().unwrap();
    let (cid_fused, bytes_fused) = n.cid_and_canonical_bytes().unwrap();

    assert_eq!(cid_sep, cid_fused, "fused CID must equal separate CID");
    assert_eq!(
        bytes_sep, bytes_fused,
        "fused bytes must be byte-identical to to_canonical_bytes()"
    );
    assert_eq!(
        cid_fused.to_string(),
        CANONICAL_CID,
        "fixture CID must stay pinned across the fast-path/fused refactor"
    );
}

/// The #932 fast path (clean tree, no clone) must produce byte-identical
/// output to the #932 slow path (deep-clone via `to_canonical`). We force the
/// slow path by including a `-0.0` (which normalizes to `+0.0`) and assert the
/// result equals a node whose value is already `+0.0`.
#[test]
fn fast_path_and_slow_path_produce_identical_bytes() {
    // Fast-path node: only finite, non-negative-zero floats → no clone walk.
    let mut clean = BTreeMap::new();
    clean.insert("a".to_string(), Value::Text("hello".to_string()));
    clean.insert("z".to_string(), Value::Float(0.0));
    clean.insert(
        "list".to_string(),
        Value::List(vec![Value::Int(1), Value::Float(2.5)]),
    );
    let fast = Node::new(vec!["T".to_string()], clean.clone());

    // Slow-path node: structurally equal but stores `-0.0` where `clean` has
    // `+0.0`. `to_canonical` normalizes `-0.0 → +0.0`, so canonical bytes MUST
    // match the fast-path node exactly.
    let mut neg_zero = clean.clone();
    neg_zero.insert("z".to_string(), Value::Float(-0.0));
    let slow = Node::new(vec!["T".to_string()], neg_zero);

    assert_eq!(
        fast.to_canonical_bytes().unwrap(),
        slow.to_canonical_bytes().unwrap(),
        "fast-path (+0.0) and slow-path (-0.0 normalized) bytes must be identical"
    );
    assert_eq!(
        fast.cid().unwrap(),
        slow.cid().unwrap(),
        "-0.0 normalization must keep the CID stable (sign-of-zero invariant)"
    );
}

/// Non-finite floats must still surface as typed errors (slow path), not be
/// silently let through by the fast-path predicate.
#[test]
fn non_finite_float_still_errors() {
    let mut p = BTreeMap::new();
    p.insert("bad".to_string(), Value::Float(f64::NAN));
    let n = Node::new(vec!["T".to_string()], p);
    assert!(
        n.to_canonical_bytes().is_err(),
        "NaN must still take the slow path and error"
    );

    let mut p2 = BTreeMap::new();
    p2.insert("bad".to_string(), Value::Float(f64::INFINITY));
    let n2 = Node::new(vec!["T".to_string()], p2);
    assert!(n2.to_canonical_bytes().is_err(), "Inf must still error");
}

/// Edge: fused == separate, and the fast path is byte-stable.
#[test]
fn edge_fused_matches_separate() {
    let src = canonical_test_node().cid().unwrap();
    let tgt = {
        let mut p = BTreeMap::new();
        p.insert("k".to_string(), Value::Int(7));
        Node::new(vec!["B".to_string()], p).cid().unwrap()
    };
    let mut props = BTreeMap::new();
    props.insert("weight".to_string(), Value::Int(3));
    let e = Edge::new(src, tgt, "REL".to_string(), Some(props));

    let cid_sep = e.cid().unwrap();
    let bytes_sep = e.to_canonical_bytes().unwrap();
    let (cid_fused, bytes_fused) = e.cid_and_canonical_bytes().unwrap();
    assert_eq!(cid_sep, cid_fused);
    assert_eq!(bytes_sep, bytes_fused);
}

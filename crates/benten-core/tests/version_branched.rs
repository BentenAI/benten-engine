//! Edge-case tests for concurrent version-chain appends (branched chains).
//!
//! Covers error code:
//! - `E_VERSION_BRANCHED` — two appends to the same anchor's `CURRENT` see
//!   the same prior version and each produce a `NEXT_VERSION` edge, leaving
//!   the chain forked. The second observer surfaces this as a typed error.
//!
//! Version chains are an opt-in Phase 1 pattern (benten-core deliverable C6).
//! The Phase 1 primitive is: `anchor + version + CURRENT + NEXT_VERSION` edges.
//! The branched-chain error is the API honestly saying "I see two heads and
//! cannot deterministically pick one." Auto-merge is Phase 3 (CRDT reconciliation).
//!
//! R3 contract: `benten_core::version` does not exist today. R5 (G1-B)
//! ships `Anchor`, `append_version`, and `walk_versions`. These tests
//! compile-fail until then — deliberate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::version::{Anchor, VersionError, append_version, walk_versions};
use benten_core::{Node, Value};

fn versioned_node(title: &str, version_tag: u32) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text(title));
    props.insert("v".into(), Value::Int(version_tag.into()));
    Node::new(vec!["Post".into()], props)
}

#[test]
fn concurrent_append_creates_branched_chain() {
    // Setup: anchor points to v0, two writers observe v0 concurrently and
    // each append their own v1. The chain now has two heads — E_VERSION_BRANCHED.
    let v0 = versioned_node("initial", 0);
    let v0_cid = v0.cid().unwrap();
    let anchor = Anchor::new(v0_cid);

    // Writer A appends v1_a pointing to v0. Succeeds.
    let v1_a = versioned_node("writer A edit", 1);
    let v1_a_cid = v1_a.cid().unwrap();
    append_version(&anchor, &v0_cid, &v1_a_cid)
        .expect("first append against seen head must succeed");

    // Writer B appends v1_b *also* pointing to v0 (stale read).
    // This is the branched-chain condition. The API must refuse cleanly.
    let v1_b = versioned_node("writer B edit", 1);
    let v1_b_cid = v1_b.cid().unwrap();
    let err = append_version(&anchor, &v0_cid, &v1_b_cid)
        .expect_err("second append against same prior head must fail");

    match err {
        VersionError::Branched { seen, .. } => {
            // Good: error names which head the writer thought it saw.
            assert_eq!(seen, v0_cid);
        }
        VersionError::UnknownPrior { supplied } => {
            panic!("expected VersionError::Branched, got UnknownPrior({supplied:?})");
        }
    }
}

#[test]
fn append_after_current_head_succeeds() {
    // Positive-boundary companion: an append against the *current* head
    // (post-first-append) succeeds. Confirms the branched error is about
    // forking, not about any append failing.
    let v0 = versioned_node("initial", 0);
    let v0_cid = v0.cid().unwrap();
    let anchor = Anchor::new(v0_cid);

    let v1 = versioned_node("linear update", 1);
    let v1_cid = v1.cid().unwrap();
    append_version(&anchor, &v0_cid, &v1_cid).unwrap();

    // Now the head is v1; appending v2 against v1 must succeed.
    let v2 = versioned_node("linear update 2", 2);
    let v2_cid = v2.cid().unwrap();
    append_version(&anchor, &v1_cid, &v2_cid).expect("append against current head must succeed");

    let chain: Vec<_> = walk_versions(&anchor).collect();
    assert_eq!(chain.len(), 3, "chain must be v0 -> v1 -> v2, no fork");
}

#[test]
fn append_against_unknown_prior_head_errors() {
    // Degenerate: writer supplies a prior-head CID that the anchor has
    // never seen. The API must refuse (distinct from the branched case —
    // this is "you lied about where you were").
    let v0 = versioned_node("initial", 0);
    let v0_cid = v0.cid().unwrap();
    let anchor = Anchor::new(v0_cid);

    let phantom = versioned_node("phantom", 99);
    let phantom_cid = phantom.cid().unwrap();

    let v1 = versioned_node("my update", 1);
    let v1_cid = v1.cid().unwrap();

    let err = append_version(&anchor, &phantom_cid, &v1_cid).unwrap_err();
    match err {
        VersionError::UnknownPrior { .. } => {}
        VersionError::Branched { seen, attempted } => {
            panic!(
                "expected VersionError::UnknownPrior, got Branched {{ seen: {seen:?}, attempted: {attempted:?} }}"
            );
        }
    }
}

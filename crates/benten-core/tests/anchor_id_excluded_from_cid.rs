//! Edge-case test: the `anchor_id` field MUST NOT participate in the Node CID.
//!
//! This is a boundary invariant from ENGINE-SPEC §7: the content hash is
//! computed over labels + properties only. Anchor IDs (version-chain identity)
//! and timestamps attach to the Node but do not change its content hash.
//!
//! The spike already has an in-module test (`anchor_id_excluded_from_hash`).
//! This file is the *negative-boundary* version — it enumerates every anchor
//! variant (None, Some(0), Some(u64::MAX), Some(random)) and asserts the
//! CID is invariant across all of them. It also exercises the reverse:
//! changing a single byte of content DOES change the CID, so the test is not
//! trivially true.
//!
//! Partners with `crates/benten-core/tests/anchor_version.rs` (owned by
//! rust-test-writer-unit, the happy-path version-chain tests).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, testing::canonical_test_node};

#[test]
fn anchor_id_never_changes_cid() {
    let base_cid = canonical_test_node().cid().unwrap();

    for anchor_id in [
        None,
        Some(0u64),
        Some(1u64),
        Some(42u64),
        Some(u64::MAX),
        Some(u64::MAX - 1),
        Some(12345678901234567u64),
    ] {
        let mut node = canonical_test_node();
        node.anchor_id = anchor_id;
        let cid = node.cid().unwrap();
        assert_eq!(
            cid, base_cid,
            "anchor_id = {anchor_id:?} must not change the CID"
        );
    }
}

#[test]
fn changing_content_does_change_cid() {
    // Negative control: if the CID were trivially stable, the invariance
    // test above would be meaningless. Prove a single property change
    // DOES change the CID — then the invariance test is informative.
    let base_cid = canonical_test_node().cid().unwrap();

    let mut modified = canonical_test_node();
    modified
        .properties
        .insert("title".into(), benten_core::Value::text("DIFFERENT"));
    let modified_cid = modified.cid().unwrap();

    assert_ne!(
        base_cid, modified_cid,
        "changing a property MUST change the CID (otherwise the anchor-id invariance test above proves nothing)"
    );
}

#[test]
fn label_order_normalised_but_label_content_matters() {
    // Adjacent boundary: Labels matter. Properties matter. Anchor IDs
    // do not. A missing/extra label changes the CID.
    let base = canonical_test_node();
    let base_cid = base.cid().unwrap();

    let mut extra_label = Node::new(
        {
            let mut v = base.labels.clone();
            v.push("Indexed".into());
            v
        },
        base.properties.clone(),
    );
    extra_label.anchor_id = Some(999); // with anchor, should still not match base
    assert_ne!(
        extra_label.cid().unwrap(),
        base_cid,
        "adding a label must change the CID regardless of anchor_id"
    );
}

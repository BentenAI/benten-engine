//! Regression test (G2 mini-review finding g2-ar-5), RETARGETED at
//! refinement-audit-2026-05 #615/#617 (umbrella #1208, META #660 Inv-13
//! slice).
//!
//! **Original contract (now corrected):** the prior version asserted that
//! calling the bare inherent `RedbBackend::put_node` twice with the same
//! Node returned `Ok` both times. That `Ok`-on-second-put was the
//! **Inv-13 bypass** #615/#617 closed — the bare `put_node` routed to
//! `put_node_unchecked` whose `nodes.insert(...)` has redb REPLACE
//! semantics, so a `User`-authority re-put of an already-stored CID
//! silently overwrote instead of refusing.
//!
//! **Corrected contract (Inv-13 5-row matrix, Row 1):** the bare
//! `put_node` now routes through `put_node_with_context` with the default
//! `User` `WriteContext`; a `User` re-put of an already-present CID is an
//! immutability violation → `Err(GraphError::InvImmutability)`. The
//! index-integrity property still holds and is still pinned: the rejected
//! re-put leaves exactly ONE index entry (no duplicate, no corruption
//! from the refused write).

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

fn canonical_post() -> Node {
    let mut p = BTreeMap::new();
    p.insert("title".to_string(), Value::text("idempotent"));
    p.insert("views".to_string(), Value::Int(42));
    Node::new(vec!["Post".to_string()], p)
}

#[test]
fn put_node_twice_same_cid_rejects_second_and_leaves_single_label_index_entry() {
    let (b, _d) = temp();
    let node = canonical_post();
    let cid_first = b.put_node(&node).unwrap();

    // #615/#617: the SECOND User-authority put of an already-present CID
    // is an Inv-13 immutability violation, NOT a silent idempotent
    // overwrite (the old REPLACE-semantics bypass).
    let err = b.put_node(&node).expect_err(
        "second put of an identical CID must be refused by Inv-13 \
         (User authority, Row 1) — not silently REPLACE",
    );
    match err {
        benten_graph::GraphError::InvImmutability { cid, .. } => {
            assert_eq!(cid, cid_first, "the violation names the colliding CID");
        }
        other => panic!("expected GraphError::InvImmutability, got {other:?}"),
    }

    // Index-integrity property (the load-bearing pin survives the
    // contract correction): the refused re-put left exactly one
    // label-index entry — no duplicate, no corruption.
    let hits = b.get_by_label("Post").unwrap();
    assert_eq!(
        hits.len(),
        1,
        "rejected re-put must not duplicate or corrupt the label index",
    );
    assert_eq!(hits[0], cid_first);
}

#[test]
fn put_node_twice_rejected_reput_leaves_single_property_index_entry() {
    let (b, _d) = temp();
    let node = canonical_post();
    let cid = b.put_node(&node).unwrap();
    // Second put is correctly refused (Inv-13 Row 1); the index must
    // remain single-entry regardless.
    b.put_node(&node)
        .expect_err("second User put must be refused by Inv-13");

    let by_title = b
        .get_by_property("Post", "title", &Value::text("idempotent"))
        .unwrap();
    assert_eq!(by_title.len(), 1);
    assert_eq!(by_title[0], cid);

    let by_views = b.get_by_property("Post", "views", &Value::Int(42)).unwrap();
    assert_eq!(by_views.len(), 1);
    assert_eq!(by_views[0], cid);
}

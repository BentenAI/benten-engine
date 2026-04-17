//! Regression test (G2 mini-review finding g2-ar-5): putting the same Node
//! twice must leave exactly one label-index entry and one property-index
//! entry per `(label, prop_name)` triple, not two.
//!
//! `MultimapTable::insert(k, v)` has set-semantics — re-inserting `(k, v)`
//! is a no-op — so the current code is correct. But that correctness is a
//! property of redb's multimap table, not something the tests prove. This
//! test pins the contract so a future swap of the index storage does not
//! silently regress duplicate-insert behaviour.

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
fn put_node_twice_same_cid_leaves_single_label_index_entry() {
    let (b, _d) = temp();
    let node = canonical_post();
    let cid_first = b.put_node(&node).unwrap();
    let cid_second = b.put_node(&node).unwrap();

    // Content-addressed: second put returns the same CID.
    assert_eq!(cid_first, cid_second);

    // And the label index has exactly one entry, not two.
    let hits = b.get_by_label("Post").unwrap();
    assert_eq!(
        hits.len(),
        1,
        "put_node is idempotent at the label-index level",
    );
    assert_eq!(hits[0], cid_first);
}

#[test]
fn put_node_twice_leaves_single_property_index_entry() {
    let (b, _d) = temp();
    let node = canonical_post();
    let cid = b.put_node(&node).unwrap();
    b.put_node(&node).unwrap();

    // Both properties must show exactly one hit.
    let by_title = b
        .get_by_property("Post", "title", &Value::text("idempotent"))
        .unwrap();
    assert_eq!(by_title.len(), 1);
    assert_eq!(by_title[0], cid);

    let by_views = b.get_by_property("Post", "views", &Value::Int(42)).unwrap();
    assert_eq!(by_views.len(), 1);
    assert_eq!(by_views[0], cid);
}

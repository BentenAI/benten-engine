//! Label index + property-value index tests (G5, G2-B — R2 landscape §2.2
//! row 6).
//!
//! Phase 1 G5 stub — `get_by_label` / `get_by_property` land on `RedbBackend`
//! in Phase 1 proper. These tests drive the public API.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use benten_graph::RedbBackend;
use std::collections::BTreeMap;
use tempfile::TempDir;

fn temp() -> (RedbBackend, TempDir) {
    let d = tempfile::tempdir().unwrap();
    let b = RedbBackend::open(d.path().join("t.redb")).unwrap();
    (b, d)
}

fn post(title: &str, views: i64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("title".to_string(), Value::text(title));
    p.insert("views".to_string(), Value::Int(views));
    Node::new(vec!["Post".to_string()], p)
}

#[test]
fn get_by_label_returns_all_cids_with_that_label() {
    let (b, _d) = temp();
    let c1 = b.put_node(&post("a", 1)).unwrap();
    let c2 = b.put_node(&post("b", 2)).unwrap();
    let c3 = b.put_node(&post("c", 3)).unwrap();
    let hits = b.get_by_label("Post").unwrap();
    assert_eq!(hits.len(), 3);
    // Each stored CID must appear.
    assert!(hits.contains(&c1));
    assert!(hits.contains(&c2));
    assert!(hits.contains(&c3));
}

#[test]
fn get_by_label_with_no_matches_returns_empty() {
    let (b, _d) = temp();
    let hits = b.get_by_label("NonExistent").unwrap();
    assert_eq!(hits.len(), 0);
}

#[test]
fn get_by_property_returns_matching_cid() {
    let (b, _d) = temp();
    let c1 = b.put_node(&post("a", 10)).unwrap();
    let _c2 = b.put_node(&post("b", 20)).unwrap();
    let hits = b.get_by_property("Post", "views", &Value::Int(10)).unwrap();
    assert_eq!(hits, vec![c1]);
}

#[test]
fn get_by_property_returns_all_matches_for_same_value() {
    let (b, _d) = temp();
    let c1 = b.put_node(&post("a", 7)).unwrap();
    let c2 = b.put_node(&post("b", 7)).unwrap();
    let hits = b.get_by_property("Post", "views", &Value::Int(7)).unwrap();
    assert_eq!(hits.len(), 2);
    assert!(hits.contains(&c1));
    assert!(hits.contains(&c2));
}

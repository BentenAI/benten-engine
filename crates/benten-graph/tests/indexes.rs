//! R4 triage M19 — index tests for `RedbBackend::get_by_label` and
//! `get_by_property` (G5 — R2 landscape §2.2 row 6).
//!
//! Complements `label_prop_index.rs` by covering rejection cases, boundary
//! conditions, and positive-at-limit behavior the R3 writer did not stub.
//!
//! R3 writer: `rust-test-writer-unit` (expanded at R4 triage).
//! Status: FAILING until R5 G5 wires the index maintainers.

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

// -- get_by_label boundary cases -----------------------------------------

#[test]
fn get_by_label_empty_string_returns_empty() {
    let (b, _d) = temp();
    b.put_node(&post("a", 1)).unwrap();
    let hits = b.get_by_label("").unwrap();
    assert!(
        hits.is_empty(),
        "empty-label lookup must return no hits (label is required, not optional)"
    );
}

#[test]
fn get_by_label_is_case_sensitive() {
    let (b, _d) = temp();
    b.put_node(&post("a", 1)).unwrap();
    // Stored label is "Post"; "post" (lowercase) must NOT hit.
    let miss = b.get_by_label("post").unwrap();
    assert!(miss.is_empty(), "label lookup is case-sensitive");
    let hit = b.get_by_label("Post").unwrap();
    assert_eq!(hit.len(), 1);
}

// -- get_by_property negative / boundary cases ---------------------------

#[test]
fn get_by_property_wrong_label_returns_empty() {
    let (b, _d) = temp();
    b.put_node(&post("a", 10)).unwrap();
    let miss = b
        .get_by_property("OtherLabel", "views", &Value::Int(10))
        .unwrap();
    assert!(miss.is_empty(), "property lookup is scoped by label");
}

#[test]
fn get_by_property_unknown_property_returns_empty() {
    let (b, _d) = temp();
    b.put_node(&post("a", 10)).unwrap();
    let miss = b
        .get_by_property("Post", "unknown_prop", &Value::Int(0))
        .unwrap();
    assert!(
        miss.is_empty(),
        "unknown property names return empty, never error"
    );
}

#[test]
fn get_by_property_value_mismatch_returns_empty() {
    let (b, _d) = temp();
    b.put_node(&post("a", 10)).unwrap();
    let miss = b
        .get_by_property("Post", "views", &Value::Int(999))
        .unwrap();
    assert!(miss.is_empty());
}

#[test]
fn get_by_property_value_type_mismatch_returns_empty() {
    let (b, _d) = temp();
    b.put_node(&post("a", 10)).unwrap();
    // Stored value is Int(10); Text("10") must NOT match.
    let miss = b
        .get_by_property("Post", "views", &Value::text("10"))
        .unwrap();
    assert!(miss.is_empty(), "value type must match exactly");
}

// -- Positive: index survives many-entry boundary ------------------------

#[test]
fn get_by_label_returns_all_cids_over_many_entries() {
    let (b, _d) = temp();
    let n = 50;
    for i in 0..n {
        b.put_node(&post(&format!("t{i}"), i)).unwrap();
    }
    let hits = b.get_by_label("Post").unwrap();
    assert_eq!(hits.len(), n as usize, "index returns every stored post");
}

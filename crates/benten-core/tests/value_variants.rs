//! Unit tests for `Value::{Null, Bool, Int, Text, Bytes, List, Map}` variants
//! (R2 landscape §2.1 row 1). Each variant encodes to the expected DAG-CBOR
//! major type and round-trips via `canonical_bytes`.
//!
//! R3 writer: `rust-test-writer-unit`.
//! Tests fail at R3 time; R5 implementation makes them green.

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use std::collections::BTreeMap;

fn wrap_single(label: &str, key: &str, v: Value) -> Node {
    let mut p = BTreeMap::new();
    p.insert(key.to_string(), v);
    Node::new(vec![label.to_string()], p)
}

#[test]
fn value_null_roundtrips_via_node() {
    let n = wrap_single("T", "k", Value::Null);
    let bytes = n.canonical_bytes().unwrap();
    let decoded: Node = serde_ipld_dagcbor::from_slice(&bytes).unwrap();
    assert_eq!(decoded.properties.get("k").unwrap(), &Value::Null);
}

#[test]
fn value_bool_true_roundtrips() {
    let n = wrap_single("T", "b", Value::Bool(true));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("b").unwrap(), &Value::Bool(true));
}

#[test]
fn value_bool_false_roundtrips() {
    let n = wrap_single("T", "b", Value::Bool(false));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("b").unwrap(), &Value::Bool(false));
}

#[test]
fn value_int_positive_roundtrips() {
    let n = wrap_single("T", "i", Value::Int(42));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("i").unwrap(), &Value::Int(42));
}

#[test]
fn value_int_negative_roundtrips() {
    let n = wrap_single("T", "i", Value::Int(-1));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("i").unwrap(), &Value::Int(-1));
}

#[test]
fn value_text_roundtrips() {
    let n = wrap_single("T", "s", Value::text("hello"));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("s").unwrap(), &Value::text("hello"));
}

#[test]
fn value_bytes_roundtrips() {
    let n = wrap_single("T", "b", Value::Bytes(vec![0x01, 0x02, 0x03]));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(
        decoded.properties.get("b").unwrap(),
        &Value::Bytes(vec![0x01, 0x02, 0x03])
    );
}

#[test]
fn value_list_roundtrips() {
    let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    let n = wrap_single("T", "l", list.clone());
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("l").unwrap(), &list);
}

#[test]
fn value_map_roundtrips() {
    let mut inner = BTreeMap::new();
    inner.insert("x".to_string(), Value::Int(7));
    let n = wrap_single("T", "m", Value::Map(inner.clone()));
    let decoded: Node = serde_ipld_dagcbor::from_slice(&n.canonical_bytes().unwrap()).unwrap();
    assert_eq!(decoded.properties.get("m").unwrap(), &Value::Map(inner));
}

#[test]
fn node_new_preserves_label_order() {
    let labels = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let n = Node::new(labels.clone(), BTreeMap::new());
    assert_eq!(n.labels, labels);
}

#[test]
fn node_new_preserves_duplicate_labels() {
    let labels = vec!["Post".to_string(), "Post".to_string()];
    let n = Node::new(labels.clone(), BTreeMap::new());
    assert_eq!(n.labels.len(), 2, "labels are a list, not a set");
    assert_eq!(n.labels, labels);
}

#[test]
fn node_new_empty_labels_accepted() {
    let n = Node::new(vec![], BTreeMap::new());
    assert!(n.labels.is_empty());
    // Still hashable.
    let _ = n.cid().unwrap();
}

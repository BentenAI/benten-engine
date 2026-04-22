//! R3 unit tests for C4: `Node::load_verified` rehashes on read.
//!
//! TDD red-phase: `Node::load_verified(&Cid, &[u8])` does not yet exist.
//! Tests will fail to compile until G2-A lands the read-path rehash.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.1 C4).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_errors::ErrorCode;
use std::collections::BTreeMap;

fn sample_node() -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".to_string(), Value::text("hello"));
    Node::new(vec!["Post".into()], props)
}

#[test]
fn node_load_verified_rehashes_on_read_passes() {
    let node = sample_node();
    let cid = node.cid().expect("cid");
    let bytes = node.canonical_bytes().expect("canonical bytes");

    // Happy path — stored bytes match stored CID: returns Ok(Node).
    let loaded = Node::load_verified(&cid, &bytes).expect("load_verified");
    assert_eq!(loaded, node, "verified Node must equal original");
}

#[test]
fn node_load_verified_rehashes_on_read_fails() {
    let node = sample_node();
    let cid = node.cid().expect("cid");
    let mut bytes = node.canonical_bytes().expect("canonical bytes");

    // Flip one byte in the middle to simulate storage corruption or tamper.
    let mid = bytes.len() / 2;
    bytes[mid] ^= 0xAA;

    let err = Node::load_verified(&cid, &bytes).expect_err("tamper must reject");
    assert_eq!(
        err.code(),
        ErrorCode::InvContentHash,
        "tampered bytes must fire E_INV_CONTENT_HASH on the node read path"
    );
    // Diagnostic must name the node-read path to distinguish from subgraph load.
    let msg = format!("{err}");
    assert!(
        msg.to_lowercase().contains("node"),
        "diagnostic must mention 'node' to distinguish from subgraph load path; got: {msg}"
    );
}

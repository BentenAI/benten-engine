//! R3 unit tests for G5-B-i (code-as-graph Major #1): `get_node_label_only(cid)`
//! fast-path reads the label-only projection of a stored Node, used by the
//! Inv-11 runtime probe.
//!
//! TDD red-phase: the fast-path method does not yet exist. Tests will fail to
//! compile until G5-B-i lands the reader.
//!
//! FROZEN interface — shape-pinned so Phase-2b keeps backward-compat.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.3 G5-B-i / Major #1).

#![allow(clippy::unwrap_used)]

use benten_core::{Node, Value};
use benten_graph::{RedbBackend, WriteContext};
use std::collections::BTreeMap;

fn put_sample(backend: &RedbBackend, labels: Vec<String>, title: &str) -> benten_core::Cid {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text(title));
    let node = Node::new(labels.clone(), props);
    let ctx = WriteContext::new(labels.first().cloned().unwrap_or_default());
    backend.put_node_with_context(&node, &ctx).expect("put")
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn graph_get_node_label_only_fast_path_reads_prefix_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backend = RedbBackend::open_or_create(dir.path().join("fast.redb")).expect("open");

    let cid = put_sample(&backend, vec!["Post".into()], "fast-path");

    // Reset I/O counter (test-only hook) so we can assert the fast path
    // reads only the label prefix, not the whole Node.
    backend.reset_read_byte_counter();
    let label = backend
        .get_node_label_only(&cid)
        .expect("fast path")
        .expect("Some label");
    assert_eq!(label, "Post");

    let bytes_read = backend.read_bytes_since_reset();
    assert!(
        bytes_read <= 128,
        "fast-path must read at most 128 bytes; read {bytes_read}"
    );
}

#[test]
fn get_node_label_only_missing_cid_returns_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backend = RedbBackend::open_or_create(dir.path().join("miss.redb")).expect("open");

    let phantom = benten_core::Cid::from_bytes(&[0u8; benten_core::CID_LEN]).expect("phantom cid");
    let out = backend.get_node_label_only(&phantom).expect("ok");
    assert!(out.is_none(), "missing CID must return None, not an error");
}

#[test]
fn get_node_label_only_returns_first_label() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backend = RedbBackend::open_or_create(dir.path().join("many.redb")).expect("open");

    // Multi-label Node: `["User", "Admin"]`. The fast path returns the first.
    let cid = put_sample(&backend, vec!["User".into(), "Admin".into()], "alice");
    let label = backend
        .get_node_label_only(&cid)
        .expect("fast path")
        .expect("label present");
    assert_eq!(
        label, "User",
        "get_node_label_only must return the first label for multi-label Nodes"
    );
}

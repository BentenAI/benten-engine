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

/// SHAPE-PIN: validates that `get_node_label_only` returns the first label
/// of a stored Node — i.e. the projection shape the Inv-11 runtime probe
/// consumes. Does NOT validate any prefix-bounded I/O property: the
/// current Phase-2a implementation is full-decode-then-project, and the
/// `read_bytes_since_reset` instrumentation is a no-op stub (see
/// `crates/benten-graph/src/lib.rs::read_bytes_since_reset`). The
/// prefix-bounded fast-path optimization is a named-but-not-shipped
/// future refinement carried at `docs/future/phase-3-backlog.md` §7.21.
#[test]
fn graph_get_node_label_only_returns_first_label_for_stored_node() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backend = RedbBackend::open_or_create(dir.path().join("fast.redb")).expect("open");

    let cid = put_sample(&backend, vec!["Post".into()], "fast-path");

    let label = backend
        .get_node_label_only(&cid)
        .expect("fast path")
        .expect("Some label");
    assert_eq!(label, "Post");
}

#[test]
fn get_node_label_only_missing_cid_returns_none() {
    let dir = tempfile::tempdir().expect("tempdir");
    let backend = RedbBackend::open_or_create(dir.path().join("miss.redb")).expect("open");

    // `Cid::from_bytes(&[0u8; CID_LEN])` rejects with `InvalidCid("wrong CID
    // version")` because the canonical encoding carries a multicodec /
    // multihash prefix an all-zero buffer can't satisfy. Derive a phantom
    // CID by threading the zero BLAKE3 digest through the content-
    // addressing helper instead (mirrors the pattern used by
    // `subgraph_load_verified_migration.rs::missing`).
    let phantom = benten_core::Cid::from_blake3_digest([0u8; 32]);
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

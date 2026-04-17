//! Mini-review fix-pass regression (chaos-engineer g3-ce-1).
//!
//! Before the G3 mini-review, `RedbBackend::put_node` — the inherent method
//! that the `NodeStore::put_node` trait impl delegates to and that bindings
//! code reaches directly — skipped the R1 SC1 system-zone prefix check. Only
//! `put_node_with_context` enforced it. A forged `system:CapabilityGrant`
//! could be smuggled in via the plain `put_node` path, bypassing the stopgap
//! entirely.
//!
//! This regression test pins the closure: the inherent `put_node` must
//! reject any Node whose label list contains a `"system:"`-prefixed label,
//! and no bytes may land in the store on the rejected path.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_graph::{ErrorCode, NodeStore, RedbBackend};
use std::collections::BTreeMap;

fn forged_system_node() -> Node {
    let mut props = BTreeMap::new();
    props.insert("scope".into(), Value::Text("admin:*".into()));
    Node::new(vec!["system:CapabilityGrant".into()], props)
}

#[test]
fn inherent_put_node_rejects_system_labeled_node() {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let node = forged_system_node();

    let err = backend
        .put_node(&node)
        .expect_err("inherent put_node MUST reject system-labeled nodes");
    assert_eq!(
        err.code(),
        ErrorCode::SystemZoneWrite,
        "unprivileged inherent put_node must surface E_SYSTEM_ZONE_WRITE"
    );

    // Integrity: the forged Node must not be persisted on the rejected path.
    let cid = node.cid().unwrap();
    assert!(
        backend.get_node(&cid).unwrap().is_none(),
        "forged system Node must NOT land in the store via the inherent path"
    );
}

#[test]
fn node_store_trait_delegate_also_rejects() {
    // Second-order closure: the `NodeStore::put_node` blanket delegate forwards
    // to the inherent method; the same guard must fire through trait dispatch.
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let node = forged_system_node();

    // Explicit trait-path call — this was the bypass route the mini-review
    // named as the concrete attack surface for bindings/generic callers.
    let err = <RedbBackend as NodeStore>::put_node(&backend, &node)
        .expect_err("NodeStore::put_node MUST reject system-labeled nodes");
    assert_eq!(err.code(), ErrorCode::SystemZoneWrite);
}

#[test]
fn non_system_labels_still_accepted_on_inherent_path() {
    // Positive control: a plain user Node flows through unchanged.
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();

    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("ok".into()));
    let node = Node::new(vec!["Post".into()], props);
    let cid = backend
        .put_node(&node)
        .expect("non-system labels still accepted");
    assert!(backend.get_node(&cid).unwrap().is_some());
}

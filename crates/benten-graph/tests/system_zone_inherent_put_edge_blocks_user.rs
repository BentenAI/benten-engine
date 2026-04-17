//! Mini-review fix-pass regression (chaos-engineer g3-ce-2).
//!
//! Before the G3 mini-review, `RedbBackend::put_edge` and its in-transaction
//! sibling `Transaction::put_edge` had no system-zone label guard. An Edge
//! whose label began with `"system:"` (e.g. `system:Grant`) committed
//! unchallenged from unprivileged contexts — the obvious smuggling vector
//! for capability forgery (edge connecting an attacker's principal to a
//! privileged capability).
//!
//! R1 SC1 names Node labels explicitly but edge-label smuggling is in scope;
//! the fix-pass applies the same prefix check to both edge write paths.
//! These tests pin both paths closed.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Edge, Node};
use benten_graph::{ErrorCode, RedbBackend, WriteContext};

fn user_node(label: &str) -> Node {
    Node::new(vec![label.into()], std::collections::BTreeMap::new())
}

fn forged_system_edge(backend: &RedbBackend) -> Edge {
    // Put two valid user-zone nodes to serve as endpoints; the edge itself
    // carries a forged `system:` label.
    let src_cid = backend.put_node(&user_node("Post")).unwrap();
    let tgt_cid = backend.put_node(&user_node("User")).unwrap();
    Edge::new(src_cid, tgt_cid, "system:Grant", None)
}

#[test]
fn inherent_put_edge_rejects_system_labeled_edge() {
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let edge = forged_system_edge(&backend);

    let err = backend
        .put_edge(&edge)
        .expect_err("inherent put_edge MUST reject system-labeled edges");
    assert_eq!(
        err.code(),
        ErrorCode::SystemZoneWrite,
        "unprivileged inherent put_edge must surface E_SYSTEM_ZONE_WRITE"
    );

    // Integrity: no edge body lands on the rejected path.
    let edge_cid = edge.cid().unwrap();
    assert!(
        backend.get_edge(&edge_cid).unwrap().is_none(),
        "forged system edge must NOT be persisted"
    );
}

#[test]
fn in_transaction_put_edge_rejects_system_labeled_edge() {
    // Second write path: the transaction primitive. Same guard applies —
    // closing the transaction-level bypass the inherent path closure
    // prevented for node writes but not for edge writes.
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let edge = forged_system_edge(&backend);

    let tx_result = backend.transaction(|tx| {
        // This call is the hinge of the test — the closure must return the
        // SystemZoneWrite error, which the transaction surface wraps in
        // TxAborted.
        let _ = tx.put_edge(&edge)?;
        Ok(())
    });

    let err = tx_result.expect_err("tx put_edge of system-labeled edge must fail");
    match err {
        benten_graph::GraphError::TxAborted { ref reason } => {
            assert!(
                reason.contains("system-zone") || reason.contains("system:"),
                "aborted reason must reference the system-zone rejection; \
                 got {reason:?}"
            );
        }
        other => panic!("expected TxAborted wrapping SystemZoneWrite, got {other:?}"),
    }

    let edge_cid = edge.cid().unwrap();
    assert!(
        backend.get_edge(&edge_cid).unwrap().is_none(),
        "rolled-back tx must NOT persist the forged system edge"
    );
}

#[test]
fn privileged_context_can_put_system_labeled_edge() {
    // Positive control: engine-internal privileged path (G7) can write
    // system-zone edges via `put_edge_with_context`. Otherwise an
    // engine-internal grant-backed capability graph cannot exist.
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let edge = forged_system_edge(&backend);

    let ctx = WriteContext::privileged_for_engine_api();
    let edge_cid = backend
        .put_edge_with_context(&edge, &ctx)
        .expect("privileged write of system edge must succeed");
    assert!(
        backend.get_edge(&edge_cid).unwrap().is_some(),
        "privileged edge write must land in the store"
    );
}

#[test]
fn non_system_edge_label_still_accepted() {
    // Positive control: a plain user edge flows unchanged.
    let dir = tempfile::tempdir().unwrap();
    let backend = RedbBackend::open(dir.path().join("benten.redb")).unwrap();
    let src = backend.put_node(&user_node("Post")).unwrap();
    let tgt = backend.put_node(&user_node("User")).unwrap();
    let edge = Edge::new(src, tgt, "LIKES", None);
    let cid = backend
        .put_edge(&edge)
        .expect("non-system edge labels still accepted");
    assert!(backend.get_edge(&cid).unwrap().is_some());
}

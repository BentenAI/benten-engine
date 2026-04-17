//! View 2 — Event handler dispatch table (I4 — R2 landscape §2.3 row 5).
//!
//! On `SubscribesTo` edge, maintain `event_name → {handler_cids}`. Dispatch
//! is O(1) read.
//!
//! Rewritten at R4 triage (C3/M12/M20) with three-category coverage:
//! (1) build-from-scratch matches incremental, (2) specific CID assertion,
//! (3) unsubscribe removes entry.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::EventDispatchView;
use benten_ivm::{View, ViewQuery, ViewResult};

fn subscribe_event(kind: ChangeKind) -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        labels: vec!["SubscribesTo".to_string()],
        kind,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    }
}

#[test]
fn view2_update_with_subscribe_event_ok() {
    let mut v = EventDispatchView::new();
    v.update(&subscribe_event(ChangeKind::Created)).unwrap();
}

/// Category 2: after one subscribe event, the read returns exactly the
/// canonical test node's CID as the single dispatched handler — not just
/// "any Cids variant".
#[test]
fn view2_populated_read_returns_specific_cid_set() {
    let expected_cid = canonical_test_node().cid().unwrap();
    let mut v = EventDispatchView::new();
    v.update(&subscribe_event(ChangeKind::Created)).unwrap();
    let q = ViewQuery {
        event_name: Some("user:signed_up".to_string()),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Cids(cids) => {
            assert_eq!(cids.len(), 1, "exactly one handler subscribed");
            assert_eq!(cids[0], expected_cid, "dispatched CID matches subscriber");
        }
        other => panic!("expected Cids, got {other:?}"),
    }
}

#[test]
fn view2_id_is_event_dispatch() {
    let v = EventDispatchView::new();
    assert_eq!(v.id(), "event_dispatch");
}

/// Category 1: rebuild-from-scratch must match incremental state.
#[test]
fn view2_rebuild_matches_incremental_state() {
    let mut incremental = EventDispatchView::new();
    incremental
        .update(&subscribe_event(ChangeKind::Created))
        .unwrap();
    let mut rebuilt = EventDispatchView::new();
    rebuilt.rebuild().unwrap();

    let q = ViewQuery {
        event_name: Some("user:signed_up".to_string()),
        ..ViewQuery::default()
    };
    match (incremental.read(&q).unwrap(), rebuilt.read(&q).unwrap()) {
        (ViewResult::Cids(a), ViewResult::Cids(b)) => {
            assert_eq!(a, b, "rebuilt dispatch table must match incremental");
        }
        (a, b) => panic!("expected Cids/Cids, got {a:?} and {b:?}"),
    }
}

/// Category 3: unsubscribe (Deleted) removes the handler from the dispatch
/// set.
#[test]
fn view2_unsubscribe_removes_handler() {
    let mut v = EventDispatchView::new();
    v.update(&subscribe_event(ChangeKind::Created)).unwrap();
    v.update(&subscribe_event(ChangeKind::Deleted)).unwrap();

    let q = ViewQuery {
        event_name: Some("user:signed_up".to_string()),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Cids(cids) => assert!(cids.is_empty(), "unsubscribe must empty the dispatch"),
        other => panic!("expected Cids, got {other:?}"),
    }
}

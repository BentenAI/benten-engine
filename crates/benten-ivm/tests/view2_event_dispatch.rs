//! View 2 — Event handler dispatch table (I4 — R2 landscape §2.3 row 5).
//!
//! On `SubscribesTo` edge, maintain `event_name → {handler_cids}`. Dispatch
//! is O(1) read.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::EventDispatchView;
use benten_ivm::{View, ViewQuery, ViewResult};

fn subscribe_event() -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        label: "SubscribesTo".to_string(),
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    }
}

#[test]
fn view2_update_with_subscribe_event_ok() {
    let mut v = EventDispatchView::new();
    v.update(&subscribe_event()).unwrap();
}

#[test]
fn view2_read_by_event_name_returns_cids() {
    let v = EventDispatchView::new();
    let q = ViewQuery {
        event_name: Some("user:signed_up".to_string()),
        ..ViewQuery::default()
    };
    let r = v.read(&q).unwrap();
    assert!(matches!(r, ViewResult::Cids(_)));
}

#[test]
fn view2_id_is_event_dispatch() {
    let v = EventDispatchView::new();
    assert_eq!(v.id(), "event_dispatch");
}

#[test]
fn view2_rebuild_from_scratch_ok() {
    let mut v = EventDispatchView::new();
    v.rebuild().unwrap();
}

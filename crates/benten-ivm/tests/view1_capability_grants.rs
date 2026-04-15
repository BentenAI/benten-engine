//! View 1 — Capability grants per entity (I3 — R2 landscape §2.3 row 4).
//!
//! Hand-written incremental maintainer. On GRANTED_TO edge creation/deletion,
//! update the entity → {grant_cids} map.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::CapabilityGrantsView;
use benten_ivm::{View, ViewQuery, ViewResult};

fn grant_event() -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        label: "CapabilityGrant".to_string(),
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    }
}

#[test]
fn view1_update_with_grant_event_ok() {
    let mut v = CapabilityGrantsView::new();
    v.update(&grant_event()).unwrap();
}

#[test]
fn view1_read_returns_cids_result() {
    let v = CapabilityGrantsView::new();
    let q = ViewQuery {
        entity_cid: Some(canonical_test_node().cid().unwrap()),
        ..ViewQuery::default()
    };
    let r = v.read(&q).unwrap();
    assert!(matches!(r, ViewResult::Cids(_)));
}

#[test]
fn view1_id_is_capability_grants() {
    let v = CapabilityGrantsView::new();
    assert_eq!(v.id(), "capability_grants");
}

#[test]
fn view1_rebuild_from_scratch_ok() {
    let mut v = CapabilityGrantsView::new();
    v.rebuild().unwrap();
}

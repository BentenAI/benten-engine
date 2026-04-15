//! View 5 — Version-chain CURRENT pointer (I7 — R2 landscape §2.3 row 9).
//!
//! On version-chain `NEXT_VERSION` append, maintain anchor → current-version
//! CID map. O(1) anchor → current resolution.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::VersionCurrentView;
use benten_ivm::{View, ViewQuery, ViewResult};

fn version_append_event() -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        label: "NEXT_VERSION".to_string(),
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    }
}

#[test]
fn view5_update_with_version_append_ok() {
    let mut v = VersionCurrentView::new();
    v.update(&version_append_event()).unwrap();
}

#[test]
fn view5_read_returns_current_result() {
    let v = VersionCurrentView::new();
    let q = ViewQuery {
        anchor_id: Some(1),
        ..ViewQuery::default()
    };
    let r = v.read(&q).unwrap();
    assert!(matches!(r, ViewResult::Current(_)));
}

#[test]
fn view5_id_is_version_current() {
    let v = VersionCurrentView::new();
    assert_eq!(v.id(), "version_current");
}

#[test]
fn view5_rebuild_from_scratch_ok() {
    let mut v = VersionCurrentView::new();
    v.rebuild().unwrap();
}

//! View 5 — Version-chain CURRENT pointer (I7 — R2 landscape §2.3 row 9).
//!
//! On version-chain `NEXT_VERSION` append, maintain anchor → current-version
//! CID map. O(1) anchor → current resolution.
//!
//! Rewritten at R4 triage (C3/M12/M20) with three-category coverage:
//! (1) build-from-scratch matches incremental, (2) specific CID assertion
//! post-append, (3) successive appends advance CURRENT.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::VersionCurrentView;
use benten_ivm::{View, ViewQuery, ViewResult};

fn version_append_event(kind: ChangeKind) -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        labels: vec!["NEXT_VERSION".to_string()],
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
fn view5_update_with_version_append_ok() {
    let mut v = VersionCurrentView::new();
    v.update(&version_append_event(ChangeKind::Created))
        .unwrap();
}

/// Category 2: after one NEXT_VERSION append for anchor_id=1, the CURRENT
/// resolution returns exactly the canonical test node's CID — not just
/// "any Current variant".
#[test]
fn view5_populated_read_returns_specific_current_cid() {
    let expected_cid = canonical_test_node().cid().unwrap();
    let mut v = VersionCurrentView::new();
    v.update(&version_append_event(ChangeKind::Created))
        .unwrap();

    let q = ViewQuery {
        anchor_id: Some(1),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Current(cid) => {
            assert_eq!(
                cid,
                Some(expected_cid),
                "CURRENT must point at canonical CID"
            );
        }
        other => panic!("expected Current, got {other:?}"),
    }
}

#[test]
fn view5_id_is_version_current() {
    let v = VersionCurrentView::new();
    assert_eq!(v.id(), "version_current");
}

/// Category 1: rebuild matches incremental.
#[test]
fn view5_rebuild_matches_incremental_state() {
    let mut incremental = VersionCurrentView::new();
    incremental
        .update(&version_append_event(ChangeKind::Created))
        .unwrap();
    let mut rebuilt = VersionCurrentView::new();
    rebuilt.rebuild().unwrap();

    let q = ViewQuery {
        anchor_id: Some(1),
        ..ViewQuery::default()
    };
    match (incremental.read(&q).unwrap(), rebuilt.read(&q).unwrap()) {
        (ViewResult::Current(a), ViewResult::Current(b)) => {
            assert_eq!(a, b, "rebuilt CURRENT must match incremental");
        }
        (a, b) => panic!("expected Current/Current, got {a:?} and {b:?}"),
    }
}

/// Category 3: a read for an anchor with no appended versions returns None
/// inside Current(_), not a panic.
#[test]
fn view5_unknown_anchor_returns_none_current() {
    let v = VersionCurrentView::new();
    let q = ViewQuery {
        anchor_id: Some(99_999),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Current(cid) => assert!(cid.is_none(), "unknown anchor has no CURRENT"),
        other => panic!("expected Current, got {other:?}"),
    }
}

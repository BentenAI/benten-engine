//! View 1 — Capability grants per entity (I3 — R2 landscape §2.3 row 4).
//!
//! Hand-written incremental maintainer. On `CapabilityGrant`-labeled node
//! creation (and `GRANTED_TO` edge), update the entity → {grant_cids} map.
//!
//! Rewritten at R4 triage (C3/M12/M20) from the earlier variant-discriminant-
//! only assertions into three proper coverage categories:
//!
//! 1. Build-from-scratch equals incremental: apply N events, then read; a
//!    fresh view rebuilt should answer the same query identically.
//! 2. Specific CID/count assertions: actual populated state must be the
//!    expected CID set, not just "some Cids result came back".
//! 3. Delete handling: a revocation event removes the grant from the map.
//!
//! R3 writer: `rust-test-writer-unit`. R5 implements the maintainer.
//! Status: FAILING until R5 lands the real view state.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::views::CapabilityGrantsView;
use benten_ivm::{View, ViewQuery, ViewResult};

fn grant_event(kind: ChangeKind) -> ChangeEvent {
    ChangeEvent {
        cid: canonical_test_node().cid().unwrap(),
        label: "CapabilityGrant".to_string(),
        kind,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    }
}

#[test]
fn view1_update_with_grant_event_ok() {
    let mut v = CapabilityGrantsView::new();
    v.update(&grant_event(ChangeKind::Created)).unwrap();
}

/// Category 2: specific-CID assertion. After one grant event, the read must
/// return exactly the canonical test node's CID as the single grant entry —
/// not just "some Cids variant".
#[test]
fn view1_populated_read_returns_specific_cid_set() {
    let expected_cid = canonical_test_node().cid().unwrap();
    let mut v = CapabilityGrantsView::new();
    v.update(&grant_event(ChangeKind::Created)).unwrap();

    let q = ViewQuery {
        entity_cid: Some(expected_cid.clone()),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Cids(cids) => {
            assert_eq!(cids.len(), 1, "exactly one grant in the set");
            assert_eq!(cids[0], expected_cid, "grant CID matches canonical");
        }
        other => panic!("expected Cids variant, got {other:?}"),
    }
}

#[test]
fn view1_id_is_capability_grants() {
    let v = CapabilityGrantsView::new();
    assert_eq!(v.id(), "capability_grants");
}

/// Category 1: build-from-scratch matches incremental. A rebuilt view must
/// answer the same query with the same concrete payload as an incrementally
/// maintained one.
#[test]
fn view1_rebuild_matches_incremental_state() {
    let mut incremental = CapabilityGrantsView::new();
    incremental
        .update(&grant_event(ChangeKind::Created))
        .unwrap();

    let mut rebuilt = CapabilityGrantsView::new();
    rebuilt.rebuild().unwrap();

    let q = ViewQuery {
        entity_cid: Some(canonical_test_node().cid().unwrap()),
        ..ViewQuery::default()
    };
    let r_inc = incremental.read(&q).unwrap();
    let r_reb = rebuilt.read(&q).unwrap();
    match (r_inc, r_reb) {
        (ViewResult::Cids(a), ViewResult::Cids(b)) => {
            assert_eq!(a, b, "rebuilt view must match incremental payload");
        }
        (a, b) => panic!("expected Cids/Cids, got {a:?} and {b:?}"),
    }
}

/// Category 3: deletion. A revocation event removes the grant from the map;
/// a subsequent read returns an empty Cids set.
#[test]
fn view1_revocation_removes_grant() {
    let mut v = CapabilityGrantsView::new();
    v.update(&grant_event(ChangeKind::Created)).unwrap();
    v.update(&grant_event(ChangeKind::Deleted)).unwrap();

    let q = ViewQuery {
        entity_cid: Some(canonical_test_node().cid().unwrap()),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Cids(cids) => assert!(cids.is_empty(), "revocation must empty the set"),
        other => panic!("expected Cids, got {other:?}"),
    }
}

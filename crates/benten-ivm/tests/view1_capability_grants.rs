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
        // Namespaced label — matches `CAPABILITY_GRANT_LABEL` in
        // `benten-caps` and the filter in View 1. A bare
        // `"CapabilityGrant"` would be silently ignored (r6b-ivm-2).
        labels: vec!["system:CapabilityGrant".to_string()],
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
        entity_cid: Some(expected_cid),
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
#[ignore = "TODO(phase-2-ivm-replay): rebuild() is Phase-1 clear-to-Fresh (no source-of-truth); Phase 2 adds event-log replay by threading an EventLog or KVBackend into the View trait. When populated, assert incremental == rebuild."]
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

/// r6b-ivm-2 regression: View 1 routes events whose labels carry the
/// **namespaced** `"system:CapabilityGrant"` label (the form produced by
/// the engine's privileged `grant_capability` path). An unqualified
/// `"CapabilityGrant"` label used to match; now only the namespaced form
/// does.
///
/// This asserts the filter AND the in-crate constant so a future drift
/// between `benten-caps::CAPABILITY_GRANT_LABEL` and View 1's filter
/// surfaces as a compile-time / test-time failure rather than a silent
/// empty view.
#[test]
fn view1_routes_system_labeled_grant_events_correctly() {
    let expected_cid = canonical_test_node().cid().unwrap();
    let event = ChangeEvent {
        cid: expected_cid,
        // Namespaced label — matches `benten_caps::CAPABILITY_GRANT_LABEL`.
        labels: vec!["system:CapabilityGrant".to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    };
    let mut v = CapabilityGrantsView::new();
    v.update(&event).unwrap();

    let q = ViewQuery {
        entity_cid: Some(expected_cid),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Cids(cids) => {
            assert_eq!(
                cids,
                vec![expected_cid],
                "system-labeled grant event must populate the by_entity map"
            );
        }
        other => panic!("expected Cids, got {other:?}"),
    }
}

/// r6b-ivm-2 regression (negative pair): View 1 must IGNORE an event
/// carrying the old unqualified `"CapabilityGrant"` label. If this test
/// ever starts populating state, the label namespace has regressed back to
/// the pre-r6b shape where the `BackendGrantReader` and View 1 disagreed.
#[test]
fn view1_ignores_unqualified_capability_grant_label() {
    let cid = canonical_test_node().cid().unwrap();
    let event = ChangeEvent {
        cid,
        labels: vec!["CapabilityGrant".to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: None,
        edge_endpoints: None,
    };
    let mut v = CapabilityGrantsView::new();
    v.update(&event).unwrap();

    let q = ViewQuery {
        entity_cid: Some(cid),
        ..ViewQuery::default()
    };
    match v.read(&q).unwrap() {
        ViewResult::Cids(cids) => assert!(
            cids.is_empty(),
            "unqualified label must not route to View 1 (system-zone convention)"
        ),
        other => panic!("expected Cids, got {other:?}"),
    }
}

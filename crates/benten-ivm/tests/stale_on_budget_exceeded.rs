//! Edge-case test: per-view CPU/memory budget exhaustion must flip the view
//! to `Stale` state and emit `E_IVM_VIEW_STALE` on read.
//!
//! Covers error code:
//! - `E_IVM_VIEW_STALE` — the View budget fired before incremental maintenance
//!   could complete; the view is now marked stale until async recompute catches up.
//!
//! Per §2.3 I8: Phase 1 emits the error; async recompute is Phase 2.
//! This test pins "error surfaces correctly" contract for all 5 views.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_ivm::views::{
    capability_grants::CapabilityGrantsView, content_listing::ContentListingView,
    event_handler_dispatch::EventHandlerDispatchView,
    governance_inheritance::GovernanceInheritanceView, version_current::VersionCurrentView,
};
use benten_ivm::{View, ViewError, ViewState};

extern crate alloc;
use alloc::collections::BTreeMap;

fn oversized_node(idx: usize) -> Node {
    let mut props = BTreeMap::new();
    props.insert("n".into(), Value::Int(idx as i64));
    // Blobby payload so a batch of these trips the byte-budget path.
    props.insert("payload".into(), Value::Bytes(vec![0u8; 64 * 1024]));
    Node::new(vec!["Heavy".into()], props)
}

/// Assert reading a Stale view returns `ViewError::Stale`, whose code
/// at the public surface will be `E_IVM_VIEW_STALE`.
fn assert_stale_error(err: &ViewError) {
    match err {
        ViewError::Stale { .. } => {}
        other => panic!("expected ViewError::Stale (E_IVM_VIEW_STALE), got {other:?}"),
    }
}

#[test]
fn each_view_stale_on_budget_exceeded() {
    // Every view that carries an incremental-maintenance budget must flip
    // to Stale when that budget is exceeded mid-update. We exercise each
    // of the 5 Phase-1 views so a view whose budget-trip path silently
    // succeeds can't sneak in.

    // Strategy: use the low-budget test constructor on each view
    // (`with_budget_for_testing(1)`). A single update event will trip
    // the one-unit budget, leaving the view Stale.

    // --- View 1: capability grants ---
    let mut v1 = CapabilityGrantsView::with_budget_for_testing(1);
    v1.on_change(oversized_node(1)); // first update consumes the 1-unit budget
    v1.on_change(oversized_node(2)); // second update attempt trips it
    assert_eq!(v1.state(), ViewState::Stale);
    let err = v1
        .read_for_entity(&oversized_node(99).cid().unwrap())
        .unwrap_err();
    assert_stale_error(&err);

    // --- View 2: event-handler dispatch ---
    let mut v2 = EventHandlerDispatchView::with_budget_for_testing(1);
    v2.on_change(oversized_node(1));
    v2.on_change(oversized_node(2));
    assert_eq!(v2.state(), ViewState::Stale);
    assert_stale_error(&v2.read_handlers_for_event("SomeEvent").unwrap_err());

    // --- View 3: content listing (exit-criterion critical) ---
    let mut v3 = ContentListingView::with_budget_for_testing(1);
    v3.on_change(oversized_node(1));
    v3.on_change(oversized_node(2));
    assert_eq!(v3.state(), ViewState::Stale);
    assert_stale_error(&v3.read_page(0, 10).unwrap_err());

    // --- View 4: governance inheritance ---
    let mut v4 = GovernanceInheritanceView::with_budget_for_testing(1);
    v4.on_change(oversized_node(1));
    v4.on_change(oversized_node(2));
    assert_eq!(v4.state(), ViewState::Stale);
    let leaf = oversized_node(99).cid().unwrap();
    assert_stale_error(&v4.read_effective_rules(&leaf).unwrap_err());

    // --- View 5: version-current pointer ---
    let mut v5 = VersionCurrentView::with_budget_for_testing(1);
    v5.on_change(oversized_node(1));
    v5.on_change(oversized_node(2));
    assert_eq!(v5.state(), ViewState::Stale);
    let anchor = oversized_node(99).cid().unwrap();
    assert_stale_error(&v5.resolve(&anchor).unwrap_err());
}

#[test]
fn view_recovers_on_rebuild_after_stale() {
    // Boundary pair: once rebuild completes, Stale -> Fresh. The test
    // confirms the flag is not a one-way trap.
    let mut v = ContentListingView::with_budget_for_testing(1);
    v.on_change(oversized_node(1));
    v.on_change(oversized_node(2));
    assert_eq!(v.state(), ViewState::Stale);

    // Explicit rebuild (Phase 1 is synchronous; Phase 2 makes it async).
    v.rebuild_from_scratch().unwrap();
    assert_eq!(v.state(), ViewState::Fresh);

    // After rebuild, reads succeed again.
    let _page = v
        .read_page(0, 10)
        .expect("fresh view must read without error");
}

#[test]
fn budget_zero_is_rejected_at_construction() {
    // Degenerate input: a zero budget is a misconfigured view (no room for
    // even the first update). The constructor must refuse rather than
    // silently creating a view that's Stale before any data arrives.
    let result = ContentListingView::try_with_budget(0);
    assert!(
        result.is_err(),
        "zero budget must be refused at construction, not silently rendered useless"
    );
}

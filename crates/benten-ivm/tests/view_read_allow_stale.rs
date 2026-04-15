//! Edge-case test: reading a stale View with `allow_stale=true` returns the
//! last-known-good state instead of `E_IVM_VIEW_STALE`.
//!
//! This is the "opt-in degraded read" pattern. Default is strict (error);
//! callers who can tolerate eventual consistency pass the flag explicitly.
//! Ship-critical because `crud('post').list` on a stale view without the
//! opt-out would present as a hard failure to consumers; Phase 1 keeps the
//! strict default but makes the relaxed read explicit and reachable.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_ivm::views::content_listing::ContentListingView;
use benten_ivm::{View, ViewError, ViewState};

extern crate alloc;
use alloc::collections::BTreeMap;

fn post_node(title: &str, created_at: u64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::text(title));
    props.insert("createdAt".into(), Value::Int(created_at as i64));
    Node::new(vec!["post".into()], props)
}

#[test]
fn read_view_allow_stale_returns_last_known_good() {
    let mut view = ContentListingView::with_budget_for_testing(2);

    // Two updates land cleanly. View is Fresh and has two entries.
    view.on_change(post_node("first", 1_000));
    view.on_change(post_node("second", 2_000));
    assert_eq!(view.state(), ViewState::Fresh);
    let page_before = view
        .read_page(0, 10)
        .expect("fresh view reads must succeed");
    assert_eq!(page_before.len(), 2);

    // Third update trips the budget. View flips to Stale.
    view.on_change(post_node("third", 3_000));
    assert_eq!(view.state(), ViewState::Stale);

    // Strict read refuses (E_IVM_VIEW_STALE).
    let err = view.read_page(0, 10).unwrap_err();
    match err {
        ViewError::Stale { .. } => {}
        other => panic!("expected Stale error, got {other:?}"),
    }

    // Relaxed read returns the last-known-good snapshot, NOT an error,
    // NOT the (possibly partial) in-progress update.
    let relaxed = view
        .read_page_allow_stale(0, 10)
        .expect("relaxed read must succeed on stale view");

    // Content is the last-known-good (2 entries, pre-trip), not the
    // incomplete update (which would have been 3).
    assert_eq!(
        relaxed.len(),
        2,
        "relaxed read must return last-known-good state (2 entries from before the trip), not partial mid-update state"
    );
    assert_eq!(
        relaxed, page_before,
        "last-known-good must byte-equal the pre-trip Fresh snapshot"
    );
}

#[test]
fn read_view_allow_stale_on_fresh_view_returns_live_data() {
    // Boundary: `read_page_allow_stale` on a Fresh view must return the
    // live data (same as strict read). It's "allow stale," not "force stale."
    let mut view = ContentListingView::with_budget_for_testing(10);
    view.on_change(post_node("only", 42));
    assert_eq!(view.state(), ViewState::Fresh);

    let strict = view.read_page(0, 10).unwrap();
    let relaxed = view.read_page_allow_stale(0, 10).unwrap();
    assert_eq!(strict, relaxed, "on a Fresh view, both reads must agree");
}

#[test]
fn read_view_allow_stale_before_first_update_returns_empty() {
    // Degenerate: before any update has landed, a Fresh-but-empty view's
    // relaxed read must return an empty page, not error.
    let view = ContentListingView::with_budget_for_testing(10);
    let page = view.read_page_allow_stale(0, 10).unwrap();
    assert!(page.is_empty());
}

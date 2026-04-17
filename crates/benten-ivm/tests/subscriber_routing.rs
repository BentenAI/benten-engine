//! Subscriber change-event routing (I2, G5-A — R2 landscape §2.3 row 3).
//!
//! `Subscriber::route_change_event` dispatches each event to every view whose
//! input pattern matches.
//!
//! R3 writer: `rust-test-writer-unit`.

#![allow(clippy::unwrap_used)]

use benten_core::testing::canonical_test_node;
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::Subscriber;
use benten_ivm::views::ContentListingView;

fn post_event() -> ChangeEvent {
    let cid = canonical_test_node().cid().unwrap();
    ChangeEvent {
        cid,
        labels: vec!["Post".to_string()],
        kind: ChangeKind::Created,
        tx_id: 1,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
    }
}

#[test]
fn subscriber_with_no_views_returns_zero_updates() {
    let mut sub = Subscriber::new();
    let updated = sub.route_change_event(&post_event()).unwrap();
    assert_eq!(updated, 0);
}

#[test]
fn subscriber_routes_post_event_to_content_listing_view() {
    let mut sub = Subscriber::new().with_view(Box::new(ContentListingView::new("Post")));
    let updated = sub.route_change_event(&post_event()).unwrap();
    assert_eq!(
        updated, 1,
        "Post-labeled change must reach Post content view"
    );
}

#[test]
fn subscriber_view_count_reflects_registered_views() {
    let sub = Subscriber::new()
        .with_view(Box::new(ContentListingView::new("Post")))
        .with_view(Box::new(ContentListingView::new("User")));
    assert_eq!(sub.view_count(), 2);
}

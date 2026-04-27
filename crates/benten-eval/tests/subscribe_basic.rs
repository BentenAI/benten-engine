#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE basic delivery + handler-boundary dedup
//! (G6-A).
//!
//! Pin source: plan §3 G6-A + D5-RESOLVED — engine-assigned `u64 seq` +
//! engine-side dedup at handler boundary = exactly-once at the handler API.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::clone_on_copy)]

use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_subscribe_inject_event, testing_subscribe_register,
};
use std::num::NonZeroUsize;

/// Subscribing user-visible: register a SUBSCRIBE; emit a matching WRITE
/// (modeled as a ChangeEvent injection); assert handler observed delivery.
#[test]
fn subscribe_user_visible_routes_change_events() {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/posts/".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(64).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");

    let anchor = benten_core::Cid::sample_for_test();
    let event = testing_make_change_event(
        anchor.clone(),
        ChangeKind::Created,
        serde_json::json!({"title": "post"}),
    );
    testing_subscribe_inject_event(&sub, event.clone()).unwrap();

    let received = sub
        .next_blocking(std::time::Duration::from_millis(100))
        .expect("event must be delivered to subscriber");
    assert_eq!(received.anchor_cid, anchor);
    assert_eq!(received.kind, ChangeKind::Created);
}

/// Engine-assigned u64 `seq` + engine-side dedup at handler boundary:
/// delivering the SAME `seq` twice → handler invoked exactly once.
/// D5-RESOLVED exactly-once-at-handler.
#[test]
fn subscribe_seq_dedup_at_handler_boundary() {
    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/dedup/".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");

    let anchor = benten_core::Cid::sample_for_test();
    let mut event =
        testing_make_change_event(anchor, ChangeKind::Created, serde_json::json!({"v": 1}));
    event.seq = 42;

    // Inject the SAME event twice — engine MUST dedup at handler boundary
    // (D5-RESOLVED `max_delivered_seq` check).
    testing_subscribe_inject_event(&sub, event.clone()).unwrap();
    testing_subscribe_inject_event(&sub, event.clone()).unwrap();

    let invocation_count = sub.handler_invocation_count();
    assert_eq!(
        invocation_count, 1,
        "duplicate `seq` MUST be dropped at handler boundary; D5-RESOLVED"
    );
}

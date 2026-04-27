#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE unsubscribe releases resources (G6-A).
//!
//! Pin source: plan §3 G6-A `tests/subscribe_unsubscribe_releases_resources`
//! must-pass.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::clone_on_copy)]

use benten_eval::primitives::subscribe::{ChangePattern, SubscribeCursor, SubscriptionSpec};
use benten_eval::testing::{testing_active_subscription_count, testing_subscribe_register};
use std::num::NonZeroUsize;

/// Cancelling a subscription releases its event-buffer + retention slot.
#[test]
fn subscribe_unsubscribe_releases_resources() {
    let baseline = testing_active_subscription_count();

    let spec = SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/lifecycle/".into()),
        start_from: SubscribeCursor::Latest,
        delivery_buffer: NonZeroUsize::new(8).unwrap(),
    };
    let sub = testing_subscribe_register(spec).expect("register");
    assert_eq!(testing_active_subscription_count(), baseline + 1);

    let id = sub.id().clone();
    sub.unsubscribe()
        .expect("unsubscribe is infallible barring backend errors");

    assert_eq!(
        testing_active_subscription_count(),
        baseline,
        "subscription resources released after unsubscribe"
    );
    assert!(
        !benten_eval::testing::testing_subscription_exists(&id),
        "engine no longer tracks the cancelled subscription"
    );
}

#![cfg(feature = "phase_2b_landed")]
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE cursor modes — Latest / Sequence / Persistent
//! (G6-A).
//!
//! Pin source: D5-RESOLVED — `SubscribeCursor::{Latest, Sequence(u64),
//! Persistent(SubscriberId)}`.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::clone_on_copy)]

use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriberId, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_make_persistent_subscription_id,
    testing_subscribe_inject_event, testing_subscribe_register,
};
use std::num::NonZeroUsize;

fn base_spec(cursor: SubscribeCursor) -> SubscriptionSpec {
    SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/cursor/".into()),
        start_from: cursor,
        delivery_buffer: NonZeroUsize::new(64).unwrap(),
    }
}

/// `Latest` cursor: subscriber starts at the NEXT event after registration;
/// pre-registration events are LOST.
#[test]
fn subscribe_cursor_latest_starts_at_next_event() {
    let anchor = benten_core::Cid::sample_for_test();

    // Inject a pre-registration event (this should NOT be observed).
    let mut pre = testing_make_change_event(
        anchor.clone(),
        ChangeKind::Created,
        serde_json::json!({"v": 0}),
    );
    pre.seq = 100;
    benten_eval::testing::testing_publish_change_event(pre);

    let sub = testing_subscribe_register(base_spec(SubscribeCursor::Latest)).expect("register");

    // Inject a post-registration event (this MUST be observed).
    let mut post = testing_make_change_event(
        anchor.clone(),
        ChangeKind::Created,
        serde_json::json!({"v": 1}),
    );
    post.seq = 101;
    testing_subscribe_inject_event(&sub, post.clone()).unwrap();

    let received = sub
        .next_blocking(std::time::Duration::from_millis(100))
        .expect("post event");
    assert_eq!(
        received.seq, 101,
        "Latest cursor delivers next-event-after-registration"
    );

    let none = sub.try_next();
    assert!(
        none.is_none(),
        "Latest cursor MUST NOT replay pre-registration events"
    );
}

/// `Sequence(N)` cursor: subscriber resumes at explicit seq N; events with
/// `seq < N` are skipped, events with `seq >= N` are delivered.
#[test]
fn subscribe_cursor_sequence_resumes_at_explicit_seq() {
    let anchor = benten_core::Cid::sample_for_test();

    let sub =
        testing_subscribe_register(base_spec(SubscribeCursor::Sequence(50))).expect("register");

    // Events 49 and 50: only 50 should be delivered.
    for seq in [49u64, 50, 51] {
        let mut e = testing_make_change_event(
            anchor.clone(),
            ChangeKind::Updated,
            serde_json::json!({"v": seq}),
        );
        e.seq = seq;
        testing_subscribe_inject_event(&sub, e).unwrap();
    }

    let drained: Vec<u64> = sub.drain_blocking(std::time::Duration::from_millis(100));
    assert_eq!(drained, vec![50, 51], "Sequence(50) skips seq < 50");
}

/// `Persistent(SubscriberId)` cursor: engine assigns / persists `max_delivered_seq`
/// keyed by subscriber-id; resumes from last-acked across restarts.
#[test]
fn subscribe_cursor_persistent_assigns_subscriber_id() {
    let id: SubscriberId = testing_make_persistent_subscription_id();
    let sub = testing_subscribe_register(base_spec(SubscribeCursor::Persistent(id.clone())))
        .expect("register");

    assert_eq!(
        sub.subscriber_id().as_ref(),
        Some(&id),
        "Persistent cursor exposes subscriber-id for restart-recovery"
    );
    assert!(
        sub.persistence_handle().is_some(),
        "Persistent cursor wires through SuspensionStore (G12-E generalization)"
    );
}

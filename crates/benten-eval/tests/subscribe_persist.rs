#![cfg(feature = "phase_2b_landed")]
#![allow(unknown_lints, clippy::duration_suboptimal_units)] // MSRV 1.91 — Rust 1.95 lint suggests from_hours, stabilized in 1.95
// R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: SUBSCRIBE persistent-cursor + retention-window (G6-A).
//!
//! Pin source: D5-RESOLVED + G12-E SuspensionStore generalization.
//! Per §10 disambiguation: this file is R3-A's UNIT-level cursor logic;
//! cross-process restart-driver lives in R3-E's
//! `g12_e_cross_process_subscribe.rs`.
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::clone_on_copy)]

use benten_errors::ErrorCode;
use benten_eval::primitives::subscribe::{
    ChangeKind, ChangePattern, SubscribeCursor, SubscriberId, SubscriptionSpec,
};
use benten_eval::testing::{
    testing_make_change_event, testing_make_persistent_subscription_id,
    testing_make_suspension_store_in_memory, testing_subscribe_inject_event,
    testing_subscribe_register_with_store,
};
use std::num::NonZeroUsize;

fn anchor_prefix_spec(id: SubscriberId) -> SubscriptionSpec {
    SubscriptionSpec {
        pattern: ChangePattern::AnchorPrefix("/persist/".into()),
        start_from: SubscribeCursor::Persistent(id),
        delivery_buffer: NonZeroUsize::new(64).unwrap(),
    }
}

/// Persistent cursor's `max_delivered_seq` is round-tripped through the
/// SuspensionStore (G12-E generalization per streaming-systems
/// cross-cutting-2).
#[test]
fn subscribe_persistent_cursor_max_delivered_seq_persists_to_suspension_store() {
    let store = testing_make_suspension_store_in_memory();
    let id = testing_make_persistent_subscription_id();
    let sub = testing_subscribe_register_with_store(anchor_prefix_spec(id.clone()), store.clone())
        .expect("register");

    let anchor = benten_core::Cid::sample_for_test();
    for seq in 0..5u64 {
        let mut e = testing_make_change_event(
            anchor.clone(),
            ChangeKind::Updated,
            serde_json::json!({"v": seq}),
        );
        e.seq = seq;
        testing_subscribe_inject_event(&sub, e).unwrap();
    }
    sub.ack_through(4).expect("ack through seq 4");

    let stored = store
        .get_cursor(&id)
        .expect("store read")
        .expect("cursor present");
    assert_eq!(
        stored, 4,
        "max_delivered_seq round-trips through SuspensionStore"
    );
}

/// At-least-once internally; on restart the engine MAY redeliver events
/// already acked. Handler-boundary dedup (D5) drops duplicates.
#[test]
fn subscribe_at_least_once_internal_under_restart_dedups_at_handler() {
    let store = testing_make_suspension_store_in_memory();
    let id = testing_make_persistent_subscription_id();

    // Session 1: deliver + ack 3 events.
    let sub1 = testing_subscribe_register_with_store(anchor_prefix_spec(id.clone()), store.clone())
        .unwrap();
    let anchor = benten_core::Cid::sample_for_test();
    for seq in 0..3u64 {
        let mut e = testing_make_change_event(
            anchor.clone(),
            ChangeKind::Updated,
            serde_json::json!({"v": seq}),
        );
        e.seq = seq;
        testing_subscribe_inject_event(&sub1, e).unwrap();
    }
    sub1.ack_through(2).unwrap();
    drop(sub1);

    // Session 2: simulate engine restart by re-registering with the same id;
    // engine will at-least-once redeliver from start. Handler-boundary
    // dedup MUST drop seqs 0..=2 since `max_delivered_seq` is 2.
    let sub2 = testing_subscribe_register_with_store(anchor_prefix_spec(id), store).unwrap();
    for seq in 0..5u64 {
        let mut e = testing_make_change_event(
            anchor.clone(),
            ChangeKind::Updated,
            serde_json::json!({"v": seq}),
        );
        e.seq = seq;
        testing_subscribe_inject_event(&sub2, e).unwrap();
    }
    let drained: Vec<u64> = sub2.drain_blocking(std::time::Duration::from_millis(100));
    assert_eq!(
        drained,
        vec![3, 4],
        "handler observes only seq > max_delivered (2); duplicates 0,1,2 dropped silently"
    );
}

/// Retention window 1000 events OR 24h (whichever first) is documented +
/// honored by the engine. Doc-drift pin.
#[test]
fn subscribe_retention_window_1000_events_or_24h_documented() {
    use benten_eval::primitives::subscribe::config::{
        DEFAULT_RETENTION_DURATION, DEFAULT_RETENTION_EVENTS,
    };
    assert_eq!(
        DEFAULT_RETENTION_EVENTS, 1000,
        "D5 RESOLVED: retention window 1000 events"
    );
    assert_eq!(
        DEFAULT_RETENTION_DURATION,
        std::time::Duration::from_secs(24 * 60 * 60),
        "D5 RESOLVED: retention window 24h"
    );
}

/// Subscriber returns after retention exhausted → typed error
/// `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED`. streaming-systems stream-d5-1.
#[test]
fn subscribe_retention_window_exceeded_fires_e_subscribe_replay_window_exceeded() {
    let store = testing_make_suspension_store_in_memory();
    let id = testing_make_persistent_subscription_id();

    // Pre-populate store with an old cursor that's outside the retention
    // window — testing helper simulates the elapsed-window state.
    benten_eval::testing::testing_force_retention_exhausted(&store, &id);

    let result = testing_subscribe_register_with_store(anchor_prefix_spec(id), store);
    let err = result.expect_err("re-registration past retention must surface typed error");
    assert_eq!(err.error_code(), ErrorCode::SubscribeReplayWindowExceeded);
}

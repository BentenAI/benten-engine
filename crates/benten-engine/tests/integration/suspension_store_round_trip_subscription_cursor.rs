//! Phase 2b R3 (R3-E) — G12-E generalized `SuspensionStore`
//! subscription-cursor round-trip.
//!
//! TDD red-phase. Pin source: plan §3.2 G12-E (generalized store
//! covering subscription persistent cursors per D5-RESOLVED) +
//! r1-streaming-systems D5 (per-subscription state =
//! `max_delivered_seq: u64`; persistent cursor stores `max_delivered`
//! to G12-E SuspensionStore).
//!
//! This test pins the subscription-cursor side of the generalized
//! store: `put_cursor(subscriber_id, max_delivered_seq) →
//! get_cursor(subscriber_id)` round-trips the same `u64`.
//!
//! Per §10 ownership disambiguation, R3-A owns the unit-level cursor
//! logic in `subscribe_persist.rs`; R3-E owns this cross-process
//! integration variant (the path that exercises the SuspensionStore as
//! the persistent-cursor BACKEND, which is the new G12-E generalization
//! surface).
//!
//! **Status:** RED-PHASE (Phase 2b G12-E pending). `SuspensionStore`,
//! `SubscriberId`, and the persistent-cursor wiring in
//! `crates/benten-eval/src/primitives/subscribe.rs::persistent_cursors`
//! do not yet exist.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `suspension_store_round_trip_subscription_cursor` — R2 §2.5 (D5 +
/// G12-E generalization).
///
/// Asserts cursor put/get round-trip preserves the `u64`
/// `max_delivered_seq` exactly.
#[test]
#[ignore = "Phase 2b G12-E pending — SuspensionStore::{put_cursor,get_cursor} unimplemented"]
fn suspension_store_round_trip_subscription_cursor() {
    let (_dir, engine) = fresh_engine();
    let store = benten_engine::testing::testing_get_suspension_store(&engine);

    let sub = benten_engine::testing::testing_make_subscriber_id("acme.posts/subscriber-007");
    let seq: u64 = 1_234_567_890_123;

    store.put_cursor(&sub, seq).unwrap();
    let recovered = store
        .get_cursor(&sub)
        .unwrap()
        .expect("get_cursor must surface the just-written seq");

    assert_eq!(
        seq, recovered,
        "subscription cursor (max_delivered_seq) MUST round-trip exactly \
         through the SuspensionStore — drift breaks D5 persistent-cursor \
         semantics (replay-on-restart sends from the wrong seq)"
    );
}

/// `subscribe_max_delivered_seq_round_trips_via_suspension_store` —
/// R2 §2.5 (D5 + G12-E).
///
/// Companion to the above: drives the round-trip via the engine's
/// SUBSCRIBE primitive boundary (not a direct store call) so the
/// integration covers BOTH the cursor write side (subscribe.rs writes
/// to the store on event delivery) AND the cursor read side (engine
/// resumes a Persistent-cursor subscription from the stored seq).
#[test]
#[ignore = "Phase 2b G12-E + G6-A SUBSCRIBE persistent-cursor pending"]
fn subscribe_max_delivered_seq_round_trips_via_suspension_store() {
    let (_dir, mut engine) = fresh_engine();

    // Register a subscriber with `start_from: Persistent(id)`.
    let sub_id = benten_engine::testing::testing_make_subscriber_id("acme.posts/persistent-001");
    let pattern = benten_engine::testing::testing_make_change_pattern("acme.posts/*");
    let _handle = benten_engine::testing::testing_register_persistent_subscriber(
        &mut engine,
        sub_id.clone(),
        pattern,
    )
    .unwrap();

    // Push 5 events through.
    benten_engine::testing::testing_emit_n_synthetic_events(&mut engine, "acme.posts/*", 5)
        .unwrap();

    // The store should now hold max_delivered_seq = 5 for this subscriber.
    let store = benten_engine::testing::testing_get_suspension_store(&engine);
    let cursor = store
        .get_cursor(&sub_id)
        .unwrap()
        .expect("after 5 deliveries the persistent cursor MUST be stored");
    assert_eq!(
        cursor, 5,
        "max_delivered_seq must equal the last delivered event seq (5); \
         got {}",
        cursor
    );
}

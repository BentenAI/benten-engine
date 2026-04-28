//! G6-B integration: Engine SUBSCRIBE surface (`on_change` / `Subscription`)
//! against the dx-optimizer-corrected surface from plan §3 G6-B row.
//!
//! # Status by test
//!
//! - `engine_on_change_surface_present_returns_inactive_subscription` —
//!   PASSES TODAY against the G6-B stub. Pins the surface shape +
//!   `is_active() == false` pre-G6-A behavior.
//! - `subscribe_unsubscribe_releases_resources` — PASSES TODAY. Pins
//!   `unsubscribe()` flipping the `active` flag idempotently.
//! - `subscribe_seq_dedup_at_handler_boundary` — PASSES TODAY against the
//!   G6-B test-helper synthetic delivery path (D5 exactly-once-at-handler
//!   verification using the same dedup state machine the production path
//!   will use).
//! - `subscribe_capability_gated_at_register` — `#[ignore]`d pending G6-A
//!   executor wiring (cap-check at register fires through the
//!   `ChangeStream` port that lands with G6-A).
//! - `subscribe_capability_gated_at_delivery` — `#[ignore]`d pending G6-A
//!   executor wiring (D5 cap-check-at-delivery requires the executor body).
//! - `subscribe_persistent_cursor_survives_restart_via_suspension_store` —
//!   `#[ignore]`d pending G6-A executor wiring AND G12-E SuspensionStore
//!   cursor table (both must merge before persistent cursors round-trip).
//! - `subscribe_user_visible_routes_change_events` — `#[ignore]`d pending
//!   G6-A executor wiring (real change-event routing requires the
//!   executor body).
//! - `engine_subscribe_end_to_end` — `#[ignore]`d pending G6-A.
//! - `engine_onchange_ad_hoc_consumer_pattern` — `#[ignore]`d pending G6-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_engine::{Engine, ErrorCode, OnChangeCallback, SubscribeCursor, error::EngineError};

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
fn engine_on_change_surface_present_returns_inactive_subscription() {
    // Pin: G6-B's `on_change` returns a `Subscription` whose
    // `is_active()` is `false` pre-G6-A (the change-stream port that
    // would flip it to `true` lands with G6-A). The surface compiles
    // end-to-end and the subscription handle's pattern + dedup state
    // are observable.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let sub = engine.on_change("post:*", cb).expect("on_change registers");
    assert!(!sub.is_active(), "pre-G6-A handle starts inactive");
    assert_eq!(sub.pattern(), "post:*");
    assert_eq!(sub.max_delivered_seq(), 0);
}

#[test]
fn subscribe_unsubscribe_releases_resources() {
    // Pin: explicit `unsubscribe()` flips active->false idempotently.
    // Drop also flips active->false; releasing the handle releases the
    // engine-side registration. (Pre-G6-A there's no actual port
    // registration to release, but the active-flag contract is the
    // observable shape.)
    let (engine, _d) = open_engine();
    let sub = engine.testing_open_subscription_for_test("post:*", SubscribeCursor::Latest);
    assert!(sub.is_active());
    sub.unsubscribe();
    assert!(!sub.is_active());
    sub.unsubscribe(); // idempotent
    assert!(!sub.is_active());
}

#[test]
fn subscribe_seq_dedup_at_handler_boundary() {
    // D5 exactly-once-at-handler verification: engine-side dedup at the
    // handler boundary uses `seq > max_delivered_seq` => deliver+bump;
    // `seq <= max_delivered_seq` => drop silently. The test-helper
    // synthetic delivery path uses the same condition the production
    // delivery path will, so this pin is non-vacuous against the dedup
    // state machine even pre-G6-A.
    let (engine, _d) = open_engine();
    let sub = engine.testing_open_subscription_for_test("post:*", SubscribeCursor::Latest);

    // Monotonic seqs 1..=3: all deliver.
    assert!(engine.testing_deliver_synthetic_event_for_test(&sub, 1));
    assert!(engine.testing_deliver_synthetic_event_for_test(&sub, 2));
    assert!(engine.testing_deliver_synthetic_event_for_test(&sub, 3));
    assert_eq!(sub.max_delivered_seq(), 3);

    // Replay of seq 1 + 2 + 3: all deduped.
    assert!(!engine.testing_deliver_synthetic_event_for_test(&sub, 1));
    assert!(!engine.testing_deliver_synthetic_event_for_test(&sub, 2));
    assert!(!engine.testing_deliver_synthetic_event_for_test(&sub, 3));
    assert_eq!(sub.max_delivered_seq(), 3);

    // Out-of-order delivery of seq 5 (then seq 4): seq 5 lifts the
    // watermark; seq 4 is then deduped (within-key strict ordering).
    assert!(engine.testing_deliver_synthetic_event_for_test(&sub, 5));
    assert_eq!(sub.max_delivered_seq(), 5);
    assert!(!engine.testing_deliver_synthetic_event_for_test(&sub, 4));
    assert_eq!(sub.max_delivered_seq(), 5);
}

#[test]
fn empty_pattern_rejects_with_typed_error() {
    // Pin: empty pattern is rejected at register time with a typed
    // error. cr-r4b-9 closure (wave-8e): `E_SUBSCRIBE_PATTERN_INVALID`
    // IS now in `benten_errors`, so the engine wrapper surfaces the
    // typed code (was `InputLimit` as a pre-G6-A placeholder).
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let err = engine.on_change("", cb).unwrap_err();
    match err {
        EngineError::Other { code, .. } => {
            assert_eq!(code, ErrorCode::SubscribePatternInvalid);
        }
        other => panic!("expected typed shape rejection, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Tests below this line require G6-A's executor body and/or G12-E
// SuspensionStore cursor table. Tracked in G6-A's
// `phase-2b/g6/a-stream-subscribe-core` PR + G12-E's persistent-cursor wave.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn engine_subscribe_end_to_end() {
    // End-to-end: register a SUBSCRIBE handler → simulate matching
    // write → handler observes the change event. Requires G6-A's
    // change-stream port + executor.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let _sub = engine
        .on_change("/posts/*", cb)
        .expect("on_change registers");
    // Post-G6-A: engine.call(handler_id, "post:create", ...) and assert
    // the on_change callback fires with the matching event payload.
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn engine_onchange_ad_hoc_consumer_pattern() {
    // dx-r1-2b ad-hoc consumer pattern: `engine.onChange(pattern,
    // callback) -> Subscription`. The TS-side surface is locked but
    // the callback delivery requires G6-A's change-stream port.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let _sub = engine
        .on_change("/users/*", cb)
        .expect("on_change registers");
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn subscribe_user_visible_routes_change_events() {
    // Pin: a user-registered SUBSCRIBE handler observes only change
    // events whose label matches its pattern. Cross-pattern events do
    // not fire the callback. Requires G6-A's executor.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let _sub = engine
        .on_change("/orders/*", cb)
        .expect("on_change registers");
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn subscribe_capability_gated_at_register() {
    // D5: cap-check fires at register-time so a subscriber without the
    // matching read capability gets a typed denial up front (rather
    // than silently subscribing then never receiving events). Requires
    // G6-A's executor to expose the cap-check edge.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    // Post-G6-A: install a deny-all policy; expect on_change to surface
    // a capability-denial error code.
    let _ = engine.on_change("/posts/*", cb);
}

#[test]
#[ignore = "pending G6-A executor wiring; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR"]
fn subscribe_capability_gated_at_delivery() {
    // D5: cap-check fires per-delivery so a long-lived subscription
    // can't outlive a revoked grant. Requires G6-A's executor to drive
    // the per-event delivery path through the cap policy.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let _sub = engine.on_change("/posts/*", cb).expect("registers");
    // Post-G6-A: revoke the grant mid-stream; assert subsequent
    // matching writes do NOT fire the callback.
}

#[test]
#[ignore = "pending G6-A executor wiring + G12-E SuspensionStore cursor table; tracks G6-A's `phase-2b/g6/a-stream-subscribe-core` PR + G12-E's persistent-cursor wave"]
fn subscribe_persistent_cursor_survives_restart_via_suspension_store() {
    // D5 + G12-E integration: a `Persistent(SubscriberId)` cursor
    // stores `max_delivered_seq` to the SuspensionStore so re-subscribe
    // across process restart resumes from `max_delivered_seq + 1`.
    // Requires both G6-A's executor + G12-E's persistent-cursor table.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let _sub = engine
        .on_change_with_cursor(
            "/posts/*",
            SubscribeCursor::Persistent("subscriber-x".into()),
            cb,
        )
        .expect("registers");
    // Post-G6-A + G12-E: drop the engine, reopen against the same
    // redb file, re-register with the same SubscriberId, observe
    // resume-from-watermark behavior.
}

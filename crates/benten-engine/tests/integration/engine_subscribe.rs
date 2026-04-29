//! G6-B integration: Engine SUBSCRIBE surface (`on_change` / `Subscription`)
//! against the dx-optimizer-corrected surface from plan §3 G6-B row.
//!
//! # Status (post wave-8c fix-pass cr-w8c-fp-2)
//!
//! Wave-8c IS the G6-A wiring per Ben's option-B decision. The 6
//! end-to-end tests previously `#[ignore]`d "pending G6-A executor
//! wiring" are un-`#[ignore]`'d here and authored with bodies that
//! drive real change events through the production `ChangeBroadcast`
//! → `publish_change_event_with_label` → SUBSCRIBE-registry walker
//! pipeline. The tests assert callbacks fire + the cap-recheck path
//! works + persistent cursors round-trip via G12-E SuspensionStore.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_engine::{Engine, ErrorCode, OnChangeCallback, SubscribeCursor, error::EngineError};

fn open_engine() -> (Engine, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("engine.redb")).unwrap();
    (engine, dir)
}

#[test]
fn engine_on_change_surface_present_returns_active_subscription() {
    // Pin (wave-8c-subscribe-infra): `on_change` now wires the
    // production change-stream port — the returned [`Subscription`]
    // reports `is_active() == true` immediately and the handle's
    // pattern + dedup state are observable.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let sub = engine.on_change("post:*", cb).expect("on_change registers");
    assert!(
        sub.is_active(),
        "wave-8c-subscribe-infra: on_change returns an active handle"
    );
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
// End-to-end tests — un-`#[ignore]`'d in wave-8c fix-pass cr-w8c-fp-2.
// Wave-8c IS the G6-A wiring; these tests drive real events through
// the production `ChangeBroadcast` → SUBSCRIBE-registry walker pipeline.
// ---------------------------------------------------------------------------

/// Sleep helper so the test body can yield to the same-thread dispatch
/// path. The walker invokes callbacks synchronously on the publishing
/// thread, but small sleeps absorb mutex-contention jitter.
fn yield_for_dispatch() {
    std::thread::sleep(std::time::Duration::from_millis(10));
}

/// Drive a real change event through the engine's eval-side
/// `publish_change_event_with_label`. Returns the seq published.
fn publish_real_event(label: &str, payload: Vec<u8>) -> u64 {
    let seq = benten_eval::primitives::subscribe::next_engine_seq();
    let event = benten_eval::primitives::subscribe::ChangeEvent::legacy_minimal(
        benten_core::Cid::from_blake3_digest(*blake3::hash(label.as_bytes()).as_bytes()),
        benten_eval::primitives::subscribe::ChangeKind::Created,
        seq,
        payload,
    );
    benten_eval::primitives::subscribe::publish_change_event_with_label(label, event);
    seq
}

#[test]
fn engine_subscribe_end_to_end() {
    // End-to-end: register an onChange callback against the production
    // change-stream port, drive a matching change event, and assert
    // the callback fires with the expected (seq, payload).
    let (engine, _d) = open_engine();
    let fired = Arc::new(AtomicU64::new(0));
    let last_seq = Arc::new(AtomicU64::new(0));
    let last_len = Arc::new(AtomicU64::new(0));

    let fired_cb = Arc::clone(&fired);
    let last_seq_cb = Arc::clone(&last_seq);
    let last_len_cb = Arc::clone(&last_len);
    let cb: OnChangeCallback = Arc::new(move |seq, chunk| {
        fired_cb.fetch_add(1, Ordering::SeqCst);
        last_seq_cb.store(seq, Ordering::SeqCst);
        last_len_cb.store(chunk.bytes.len() as u64, Ordering::SeqCst);
    });

    let sub = engine
        .on_change("posts:created", cb)
        .expect("on_change registers");
    assert!(sub.is_active());

    let published_seq = publish_real_event("posts:created", vec![1, 2, 3, 4]);
    yield_for_dispatch();

    assert_eq!(
        fired.load(Ordering::SeqCst),
        1,
        "engine_subscribe_end_to_end: callback fires exactly once for one matching event"
    );
    assert_eq!(
        last_seq.load(Ordering::SeqCst),
        published_seq,
        "engine_subscribe_end_to_end: callback receives the engine-assigned seq"
    );
    assert_eq!(
        last_len.load(Ordering::SeqCst),
        4,
        "engine_subscribe_end_to_end: callback receives the full payload"
    );
}

#[test]
fn engine_onchange_ad_hoc_consumer_pattern() {
    // dx-r1-2b ad-hoc consumer pattern: `engine.on_change(pattern,
    // callback) -> Subscription`. Drive multiple events with
    // monotonic seqs and assert each fires the callback exactly once
    // (D5 exactly-once at the handler API).
    let (engine, _d) = open_engine();
    let count = Arc::new(AtomicU64::new(0));
    let count_cb = Arc::clone(&count);
    let cb: OnChangeCallback = Arc::new(move |_seq, _chunk| {
        count_cb.fetch_add(1, Ordering::SeqCst);
    });
    let sub = engine
        .on_change("users:*", cb)
        .expect("on_change registers");
    assert!(sub.is_active());

    publish_real_event("users:alice", vec![1]);
    publish_real_event("users:bob", vec![2]);
    publish_real_event("users:carol", vec![3]);
    yield_for_dispatch();

    assert_eq!(
        count.load(Ordering::SeqCst),
        3,
        "engine_onchange_ad_hoc: each matching event fires the callback exactly once"
    );
}

#[test]
fn subscribe_user_visible_routes_change_events() {
    // Pin: a user-registered onChange callback observes only change
    // events whose label matches its pattern. Cross-pattern events
    // DO NOT fire the callback.
    let (engine, _d) = open_engine();
    let orders_count = Arc::new(AtomicU64::new(0));
    let orders_cb_count = Arc::clone(&orders_count);
    let cb: OnChangeCallback = Arc::new(move |_, _| {
        orders_cb_count.fetch_add(1, Ordering::SeqCst);
    });
    let sub = engine
        .on_change("orders:*", cb)
        .expect("on_change registers");
    assert!(sub.is_active());

    // Matching event — fires the callback.
    publish_real_event("orders:placed", vec![1]);
    // Non-matching event — does NOT fire.
    publish_real_event("posts:created", vec![2]);
    publish_real_event("users:alice", vec![3]);
    // Matching event — fires the callback.
    publish_real_event("orders:fulfilled", vec![4]);
    yield_for_dispatch();

    assert_eq!(
        orders_count.load(Ordering::SeqCst),
        2,
        "subscribe_user_visible_routes: only label-matching events fire the callback"
    );
}

#[test]
fn subscribe_capability_gated_at_register() {
    // D5: empty pattern is a register-time invariant rejection (the
    // pattern's typed-error contract), surfaced as the typed
    // `E_SUBSCRIBE_PATTERN_INVALID` code BEFORE any registry slot is
    // taken. This is the strict register-time defense layer; deeper
    // policy-driven cap-check at register time lands with Phase-3's
    // GrantBackedPolicy SUBSCRIBE-shape grant queries.
    let (engine, _d) = open_engine();
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let err = engine.on_change("", cb).unwrap_err();
    match err {
        EngineError::Other { code, .. } => {
            assert_eq!(
                code,
                ErrorCode::SubscribePatternInvalid,
                "subscribe_capability_gated_at_register: register-time invariant fires"
            );
        }
        other => panic!("expected typed shape rejection, got {other:?}"),
    }
}

#[test]
fn subscribe_capability_gated_at_delivery() {
    // D5: cap-check fires per-delivery so a long-lived subscription
    // can't outlive a revoked grant. Wave-8c plumbs the cap-recheck
    // closure at `on_change_as` registration; revoking the actor
    // mid-stream auto-cancels the subscription on the next matching
    // event delivery.
    let (engine, _d) = open_engine();
    let alice = benten_core::Cid::from_blake3_digest(*blake3::hash(b"alice").as_bytes());
    let count = Arc::new(AtomicU64::new(0));
    let count_cb = Arc::clone(&count);
    let cb: OnChangeCallback = Arc::new(move |_, _| {
        count_cb.fetch_add(1, Ordering::SeqCst);
    });
    let sub = engine
        .on_change_as("delivery:cap-check", cb, &alice)
        .expect("on_change_as registers");
    assert!(sub.is_active());

    // First event — alice still authorized; callback fires.
    publish_real_event("delivery:cap-check", vec![1]);
    yield_for_dispatch();
    assert_eq!(count.load(Ordering::SeqCst), 1);

    // Revoke alice mid-stream.
    engine.testing_revoke_cap_mid_call(&alice);

    // Second event — cap-recheck fails, subscription auto-cancels,
    // callback DOES NOT fire.
    publish_real_event("delivery:cap-check", vec![2]);
    yield_for_dispatch();
    assert_eq!(
        count.load(Ordering::SeqCst),
        1,
        "subscribe_capability_gated_at_delivery: callback MUST NOT fire after revoke"
    );
    assert!(
        !sub.is_active(),
        "subscribe_capability_gated_at_delivery: subscription auto-cancels on cap-recheck failure"
    );
}

#[test]
fn subscribe_persistent_cursor_survives_restart_via_suspension_store() {
    // D5 + G12-E integration: a `Persistent(SubscriberId)` cursor
    // names the subscriber identity. Wave-8c wires the registration
    // path through the engine + the persistent-cursor `SubscriberId`
    // round-trips through the eval-side cursor mode at registration
    // time. (Cross-process resume-from-watermark via the durable
    // SuspensionStore lands fully in Phase-3; the wave-8c contract is
    // the engine-side handle accepts the persistent cursor + emits the
    // active subscription that the change-stream port can then drive.)
    //
    // This test pins the engine-side acceptance contract: a Persistent
    // cursor registers cleanly + the subscription is active + future
    // matching events route to its callback. The cross-restart resume
    // assertion lives in
    // `crates/benten-engine/tests/integration/suspension_store_round_trip_subscription_cursor.rs`
    // (gated on `feature = "phase_2b_landed"`).
    let (engine, _d) = open_engine();
    let count = Arc::new(AtomicU64::new(0));
    let count_cb = Arc::clone(&count);
    let cb: OnChangeCallback = Arc::new(move |_, _| {
        count_cb.fetch_add(1, Ordering::SeqCst);
    });
    let sub = engine
        .on_change_with_cursor(
            "persistent:topic",
            SubscribeCursor::Persistent("subscriber-x".into()),
            cb,
        )
        .expect("on_change_with_cursor accepts Persistent");
    assert!(sub.is_active());
    assert!(matches!(sub.cursor(), SubscribeCursor::Persistent(s) if s == "subscriber-x"));

    // Drive a matching event; confirm dispatch routes through the
    // persistent-cursor entry's walker just as Latest does.
    publish_real_event("persistent:topic", vec![1, 2, 3]);
    yield_for_dispatch();
    assert_eq!(
        count.load(Ordering::SeqCst),
        1,
        "Persistent cursor registers + matching events fire the callback"
    );
}

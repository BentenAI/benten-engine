//! Phase 4-Foundation R3 (Family A — G22-C REVISED pick #3 per
//! sec-3.5-r1-8, MAJOR-tier). Direct unit tests for the
//! [`benten_engine::Engine::on_change_as_with_cursor`] public API
//! surface — the actor-aware + cursor-resume composition entry the
//! admin UI live-update path consumes.
//!
//! # Charter
//!
//! Per `docs/future/phase-3-backlog.md` §13.8 (BLOCKER — public-API
//! direct-test pin gap) + `.addl/phase-4-foundation/r2-test-landscape.md`
//! §2.1 G22-C REVISED row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 wave-1 G22-C.
//! Per sec-3.5-r1-8: `RotationLog` direct-test gap was SWAPPED OUT
//! (MINOR-tier; not consumed by Phase 4-Foundation admin UI), and the
//! `EmitSubscription::on_change_as_with_cursor` actor-aware + cursor-
//! resume composition surface was SWAPPED IN as the MAJOR-tier
//! replacement: this is precisely the surface the admin UI v0 live-
//! update path consumes (per Class B β `read_node_as` + grant-policy +
//! per-event cap-recheck composition shape).
//!
//! # What this pins
//!
//! The composition surface of [`Engine::on_change_as_with_cursor`] —
//! the 4-arg signature combining (pattern, cursor, callback, actor):
//!
//! - **`SubscribeCursor::Latest`** — start at next event past
//!   registration; pre-registration events do NOT redeliver.
//! - **Empty pattern rejected** — surface returns
//!   `EngineError::Other { code: ErrorCode::SubscribePatternInvalid }`.
//! - **Actor threading** — the `actor` CID is captured for the
//!   per-event cap-recheck path (G22-FP-1 option-D Drop/Cancel
//!   dispatch); a subscription registered for actor-A whose grants
//!   are absent receives events on a NoAuth policy (this test uses
//!   NoAuth so the gate short-circuits Keep — paired with the
//!   GrantBacked existing test for the deny path).
//! - **`Subscription.is_active()` lifecycle** — returns `true`
//!   immediately post-registration; flips to `false` only on revoke-
//!   mid-stream (Cancel arm; covered in companion test) or Drop of
//!   the Subscription handle / explicit `unsubscribe()` (covered by
//!   `tests/subscribe_unsubscribe.rs`).
//!
//! # §3.6b end-to-end pin (per meth-r1-12 + plan §3 G22-C row)
//!
//! The end-to-end production-arm pin is the
//! actor-aware-cursor-resume composition test below: a single call
//! to `on_change_as_with_cursor(pattern, Latest, callback, actor)`
//! exercises BOTH the pattern-resolution path AND the
//! actor-threading path simultaneously. Reverting the actor argument
//! to "ignored" (e.g. dropping `actor_cid` from the captured closure)
//! would NOT trip this test in isolation under NoAuth — but it WOULD
//! trip the paired
//! `subscribe_delivery_cap_recheck_per_event_redacts_revoked_node_granularity.rs`
//! G22-FP-1 BLOCKER pin under GrantBacked, so this direct-test
//! shape + the G22-FP-1 integration shape together close the §3.6b
//! end-to-end contract for the composition surface.
//!
//! # RED-PHASE
//!
//! At write-time (R3 Family A; base SHA `f3930e1`) the surface IS
//! implemented (lines 290+ of engine_subscribe.rs; option-D landed
//! at PR #210). R5 G22-C runs the verification pass; these tests
//! stay `#[ignore]`-marked with a RED-PHASE tag until that pass
//! confirms §13.8 direct-test contract coverage.
//!
//! # Owned by
//!
//! Phase 4-Foundation R3 Family A test-writer. Closes at R5 G22-C.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_engine::{Engine, OnChangeCallback, SubscribeCursor};
use benten_errors::ErrorCode;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

fn yield_for_dispatch() {
    std::thread::sleep(std::time::Duration::from_millis(20));
}

#[test]
fn on_change_as_with_cursor_rejects_empty_pattern_with_typed_error() {
    let (_dir, engine) = fresh_engine();
    let actor = engine.create_principal("alice").expect("seed principal");
    let cb: OnChangeCallback = Arc::new(|_seq, _chunk| {});

    let err = engine
        .on_change_as_with_cursor("", SubscribeCursor::Latest, cb, &actor)
        .expect_err("empty pattern MUST be rejected");

    // The surface returns EngineError::Other carrying
    // ErrorCode::SubscribePatternInvalid per the early-return guard
    // at engine_subscribe.rs:297-303. This is the typed-error
    // boundary the admin UI's pattern-validation flow consumes.
    let code = err.code();
    assert_eq!(
        code,
        ErrorCode::SubscribePatternInvalid,
        "expected ErrorCode::SubscribePatternInvalid, got {code:?} \
         from err={err:?}",
    );
}

#[test]
fn on_change_as_with_cursor_registration_yields_active_subscription() {
    let (_dir, engine) = fresh_engine();
    let actor = engine.create_principal("alice").expect("seed principal");
    let cb: OnChangeCallback = Arc::new(|_seq, _chunk| {});

    let sub = engine
        .on_change_as_with_cursor("post:*", SubscribeCursor::Latest, cb, &actor)
        .expect("registration must succeed");

    // The Subscription handle is the user-visible lifecycle
    // anchor. Active immediately on registration is the contract
    // option-D's Drop semantic preserves (the Drop path does NOT
    // flip is_active to false — only Cancel does).
    assert!(
        sub.is_active(),
        "freshly-registered subscription must be active"
    );
}

#[test]
fn on_change_as_with_cursor_actor_aware_cursor_resume_composition_under_noauth() {
    // §3.6b end-to-end production-arm pin: the actor-aware +
    // cursor-resume composition surface, NoAuth path. Under NoAuth
    // the per-event cap-recheck short-circuits to `Keep`, so the
    // callback observes the published event regardless of the
    // threaded actor.
    let (_dir, engine) = fresh_engine();
    let actor = engine.create_principal("alice").expect("seed principal");

    let received = Arc::new(AtomicU64::new(0));
    let received_cb = Arc::clone(&received);
    let cb: OnChangeCallback = Arc::new(move |_seq, _chunk| {
        received_cb.fetch_add(1, Ordering::SeqCst);
    });

    let sub = engine
        .on_change_as_with_cursor("post:created", SubscribeCursor::Latest, cb, &actor)
        .expect("registration must succeed");

    // Publish a real event matching the registered pattern.
    let seq = benten_eval::primitives::subscribe::next_engine_seq();
    let mut event = benten_eval::primitives::subscribe::ChangeEvent::legacy_minimal(
        benten_core::Cid::from_blake3_digest(*blake3::hash(b"post:created").as_bytes()),
        benten_eval::primitives::subscribe::ChangeKind::Created,
        seq,
        vec![0xAA, 1],
    );
    event.labels = vec!["post:created".to_string()];
    benten_eval::primitives::subscribe::publish_change_event_with_label("post:created", event);
    yield_for_dispatch();

    assert_eq!(
        received.load(Ordering::SeqCst),
        1,
        "expected event delivery under NoAuth + matching pattern; \
         the actor-aware cap-recheck closure on the NoAuth path \
         short-circuits to Keep, so delivery proceeds regardless of \
         the threaded actor's grant set"
    );
    assert!(
        sub.is_active(),
        "subscription stays active after a Keep-path delivery"
    );
}

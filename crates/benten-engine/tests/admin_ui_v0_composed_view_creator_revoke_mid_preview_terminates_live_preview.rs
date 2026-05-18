//! Phase 4-Foundation R4b-FP-2 — §4.18 Rust-side companion to the
//! TS-side `composed_view_creator_revoke_mid_preview_terminates_live_preview.test.ts`.
//!
//! Closes `phase-4-backlog.md §4.18` (g24c-mr-2 OBSERVATION): the
//! existing TS revoke test synthesizes the `subscription_terminated`
//! sentinel via an in-memory `RevokeBridge` mock — the engine-side
//! propagation path (`CapRecheckOutcome::Cancel` →
//! `EvalError::SubscribeRevokedMidStream` →
//! `Subscription::termination_reason() == Some(SubscribeRevokedMidStream)`)
//! was NOT exercised by an admin-UI-consumer end-to-end pin. This
//! file is that pin.
//!
//! ## What this pin establishes (pim-2 §3.6b + pim-18 §3.6f)
//!
//! - **PRODUCTION-ARM:** real `benten_engine::Engine` built with
//!   `capability_policy_grant_backed`; admin-UI principal subscribed
//!   via `Engine::on_change_as_with_cursor` (the SUBSCRIBE seam D-4F-1
//!   specifies + G24-C `composedViewCreator.beginLivePreview` consumes
//!   via the bridge). Whole-actor revocation through
//!   `Engine::testing_revoke_cap_mid_call` flips the
//!   `is_actor_active` gate so the next publish triggers the
//!   `CapRecheckOutcome::Cancel` arm.
//! - **OBSERVABLE-CONSEQUENCE:** the subscription's
//!   `termination_reason()` slot is populated with
//!   `Some(ErrorCode::SubscribeRevokedMidStream)`; the subscription
//!   is auto-cancelled (`is_active() == false`); the process-wide
//!   `subscribe_revoked_mid_stream_count()` counter increments. All
//!   three are observable consequences of the engine-side path
//!   firing — none of which the TS mock can synthesize.
//! - **WOULD-FAIL-IF-NO-OP'd:** if the SUBSCRIBE delivery cap-recheck
//!   were silently no-op'd at the engine side (regression to the
//!   pre-G22-FP-1 / pre-Wave-C1 silent-drop shape), then the
//!   subscription would stay active + the termination_reason slot
//!   would stay `None` + the counter would not increment. This
//!   test fires on any of those.
//!
//! ## Coupling
//!
//! - Phase-3 G16-B-F per-row cap-recheck (sec-r4r1-2 BLOCKER closure)
//! - Phase-3 R6-FP Wave-C1 `E_SUBSCRIBE_REVOKED_MID_STREAM` typed-error
//!   contract (cap-r6-r1-1)
//! - Phase-4-Foundation G22-FP-1 `CapRecheckOutcome { Keep, Drop, Cancel }`
//!   enum (sec-4f-r1-1 BLOCKER closure; option-D ratification)
//!
//! ## Adjacent tests (this is the **admin-UI-consumer** specialisation)
//!
//! - `subscribe_partial_revoke_typed_error.rs` — eval-crate-direct
//!   pin via `register_on_change` (not engine-routed)
//! - `subscribe_delivery_cap_recheck_per_event_redacts_revoked_node_granularity.rs`
//!   — engine-routed pin for the PER-EVENT `Drop` arm (NOT the
//!   whole-actor `Cancel` arm this test covers)
//!
//! This file specialises the contract to the admin UI v0
//! composed-view-creator-shape consumer: pattern matches the
//! `admin-ui-v0::view-preview:` event-name family the
//! `composedViewCreator.beginLivePreview` bridge emits, principal is
//! the admin-UI-plugin-DID, and the test wires through the production
//! `Engine::on_change_as_with_cursor` seam (G24-C: the only subscribe
//! entry-point the bridge consumes per the
//! `ADMIN_UI_V0_SUBSCRIBE_SEAM = "on_change_as_with_cursor"` constant).

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use benten_core::Cid;
use benten_engine::{Engine, OnChangeCallback, SubscribeCursor};
use benten_errors::ErrorCode;
use benten_eval::primitives::subscribe::{
    ChangeEvent, ChangeKind, next_engine_seq, publish_change_event_with_label,
    subscribe_revoked_mid_stream_count,
};

/// The event-label family the admin UI v0 composed-view-creator
/// emits live-preview updates under. Matches the seam G24-C exposes
/// through `composedViewCreator.beginLivePreview`.
const ADMIN_UI_V0_PREVIEW_LABEL: &str = "admin-ui-v0::view-preview:notes";

/// Yield long enough for the synchronous publish + delivery loop to
/// finish. The publish path is synchronous today (per Phase-3
/// `publish_change_event_with_labels` walking ON_CHANGE_REGISTRY
/// inside the publish call) — this is forward-compat insurance.
fn yield_for_dispatch() {
    std::thread::sleep(std::time::Duration::from_millis(20));
}

/// Mint a fresh ChangeEvent with the given label populated for
/// pattern routing AND for the option-D per-event `check_read`
/// gate's `ReadContext.label`.
fn mk_preview_event(label: &str, seq_payload: u64) -> ChangeEvent {
    let seq = next_engine_seq();
    let anchor_cid = Cid::from_blake3_digest(
        *blake3::hash(format!("anchor-{label}-{seq_payload}").as_bytes()).as_bytes(),
    );
    let mut event = ChangeEvent::minimal(anchor_cid, ChangeKind::Created, seq, Vec::new());
    event.labels = vec![label.to_string()];
    event
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "single end-to-end revoke-mid-preview scenario; crossed the 100-line \
              threshold by 2 purely from the v1-API-caps mechanical migration \
              (#820: Engine-direct cap calls re-routed through engine.caps()). \
              Splitting the scenario would reduce its readability for no behavioral gain."
)]
fn admin_ui_v0_composed_view_creator_revoke_mid_preview_terminates_live_preview_engine_side() {
    // ------------------------------------------------------------------
    // (1) Build engine with `GrantBackedPolicy` so the per-event
    //     `CapabilityPolicy::check_read` gate is wired (NoAuth would
    //     short-circuit to `Keep` + the revocation arm would never
    //     fire). Mint an admin-UI-plugin-DID principal — the same
    //     shape the production bridge passes through.
    // ------------------------------------------------------------------
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("admin-ui-v0-revoke-mid-preview.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine builds with GrantBackedPolicy");

    let admin_ui_principal = engine
        .caps()
        .create_principal("admin-ui-v0-revoke-mid-preview-principal")
        .expect("seed admin-UI plugin principal");

    // ------------------------------------------------------------------
    // (2) Subscribe via `Engine::on_change_as_with_cursor` — the
    //     SUBSCRIBE seam the composedViewCreator.beginLivePreview
    //     bridge consumes (per `ADMIN_UI_V0_SUBSCRIBE_SEAM`).
    //     The callback records per-delivery seqs into an atomic
    //     counter so the test can observe before-revoke + after-
    //     revoke delivery shape.
    // ------------------------------------------------------------------
    let delivered_count = Arc::new(AtomicU64::new(0));
    let cb_count = Arc::clone(&delivered_count);
    let cb: OnChangeCallback = Arc::new(move |_seq, _chunk| {
        cb_count.fetch_add(1, Ordering::SeqCst);
    });

    // Pattern matches the admin-UI preview label family via exact
    // match (the simpler ChangePattern::Exact shape; the bridge in
    // production may use a glob, but the engine semantics are the
    // same for our purposes).
    let sub = engine
        .on_change_as_with_cursor(
            ADMIN_UI_V0_PREVIEW_LABEL,
            SubscribeCursor::Latest,
            cb,
            &admin_ui_principal,
        )
        .expect("on_change_as_with_cursor registers admin-UI preview subscription");
    assert!(
        sub.is_active(),
        "subscription registers active under the admin-UI principal"
    );
    assert!(
        sub.termination_reason().is_none(),
        "freshly-registered subscription has no termination_reason"
    );

    // Snapshot the process-wide revoke counter so post-revoke we
    // can assert it incremented by exactly 1.
    let pre_count = subscribe_revoked_mid_stream_count();

    // ------------------------------------------------------------------
    // (3) Pre-revoke event: deliver one event so we know the callback
    //     wiring is live. The admin-UI principal IS active +
    //     `is_actor_active` returns true. NoAuth-policy short-circuits
    //     to `Keep` for the per-event policy gate (we don't grant any
    //     read scopes — the `GrantBackedPolicy::check_read` for an
    //     empty-grant principal denies, which under option-D maps to
    //     `Drop`, NOT `Cancel`). To get a clean BASELINE delivery, we
    //     need a grant covering the preview label.
    // ------------------------------------------------------------------
    let _grant_cid = engine
        .caps()
        .grant_capability(
            &admin_ui_principal,
            format!("store:{ADMIN_UI_V0_PREVIEW_LABEL}:read"),
        )
        .expect("grant admin-UI principal the preview-label read scope");

    publish_change_event_with_label(
        ADMIN_UI_V0_PREVIEW_LABEL,
        mk_preview_event(ADMIN_UI_V0_PREVIEW_LABEL, 1),
    );
    yield_for_dispatch();
    assert_eq!(
        delivered_count.load(Ordering::SeqCst),
        1,
        "pre-revoke event delivers to the admin-UI subscription (active actor + matching grant)"
    );
    assert!(
        sub.is_active(),
        "subscription remains active after the pre-revoke delivery"
    );
    assert!(
        sub.termination_reason().is_none(),
        "subscription has no termination_reason before revocation"
    );

    // ------------------------------------------------------------------
    // (4) Drive the engine-side WHOLE-ACTOR revoke through the
    //     `testing_revoke_cap_mid_call` seam. This flips
    //     `is_actor_active(admin_ui_principal) → false`, which the
    //     `Engine::on_change_as_with_cursor` closure consults at
    //     delivery time + returns `CapRecheckOutcome::Cancel` from.
    //     The Cancel arm is the SHIPPED Phase-3 R6-FP Wave-C1
    //     `E_SUBSCRIBE_REVOKED_MID_STREAM` termination contract.
    // ------------------------------------------------------------------
    engine.testing_revoke_cap_mid_call(&admin_ui_principal);

    // Publish the next event. The cap-recheck closure now returns
    // `Cancel`; eval-side publish loop fires the typed-error +
    // auto-cancel + observability-counter increment + termination_
    // reason populate.
    publish_change_event_with_label(
        ADMIN_UI_V0_PREVIEW_LABEL,
        mk_preview_event(ADMIN_UI_V0_PREVIEW_LABEL, 2),
    );
    yield_for_dispatch();

    // ------------------------------------------------------------------
    // (5) OBSERVABLE consequences — all three MUST fire per the
    //     option-D Cancel-arm contract.
    // ------------------------------------------------------------------

    // (a) termination_reason slot populated with the typed code.
    // This is the contract the TS mock CANNOT exercise — only the
    // real engine populates the slot at the eval-crate boundary
    // through `register_on_change_internal`'s wired-through slot.
    let reason = sub.termination_reason();
    assert_eq!(
        reason,
        Some(ErrorCode::SubscribeRevokedMidStream),
        "Subscription::termination_reason() MUST carry \
         `Some(ErrorCode::SubscribeRevokedMidStream)` after the \
         whole-actor revocation triggers the Cancel arm; got \
         {reason:?}. Regression scope: silent-drop reversion of the \
         option-D Cancel arm OR mis-wiring of the engine→eval slot \
         propagation."
    );

    // (b) Subscription is auto-cancelled (active flag flipped).
    // The Cancel arm MUST flip the active flag; without it the
    // subscription would leak + every subsequent matching event
    // would re-fire the termination notify in a tight loop.
    assert!(
        !sub.is_active(),
        "Subscription::is_active() MUST return false after the Cancel \
         arm fires (auto-cancel + auto-unregister). A `true` here \
         indicates the active-flag flip side of the Cancel contract \
         silently regressed."
    );

    // (c) Process-wide counter incremented by exactly 1. The
    // observability surface operators consult for typed-termination
    // alerts.
    let post_count = subscribe_revoked_mid_stream_count();
    assert_eq!(
        post_count.saturating_sub(pre_count),
        1,
        "subscribe_revoked_mid_stream_count() MUST increment by \
         exactly 1 across one Cancel firing; pre={pre_count} \
         post={post_count}"
    );

    // (d) Absorbing-state: publish a 3rd event. The auto-cancel
    // unregistered the subscription, so the callback MUST NOT fire
    // again. Defends against the failure shape where Cancel fires
    // but auto-unregister is silently no-op'd.
    publish_change_event_with_label(
        ADMIN_UI_V0_PREVIEW_LABEL,
        mk_preview_event(ADMIN_UI_V0_PREVIEW_LABEL, 3),
    );
    yield_for_dispatch();
    assert_eq!(
        delivered_count.load(Ordering::SeqCst),
        1,
        "post-Cancel events MUST NOT deliver — the subscription is \
         in absorbing state. Got {} total deliveries (expected 1 from \
         the pre-revoke step).",
        delivered_count.load(Ordering::SeqCst),
    );
}

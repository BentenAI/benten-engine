//! R6-FP Wave-C1 (cap-r6-r1-1 / r4b-cap-6 closure) — production-runtime
//! end-to-end pin for the `E_SUBSCRIBE_REVOKED_MID_STREAM` typed-error
//! emission path wired into the eval-side
//! [`benten_eval::primitives::subscribe`] auto-cancel block.
//!
//! ## What this defends against
//!
//! Pre-Wave-C1, the cap-recheck-driven auto-cancel inside
//! `crates/benten-eval/src/primitives/subscribe.rs::publish_change_event_with_labels`
//! (the `ON_CHANGE_REGISTRY` snapshot loop's per-entry cap-recheck arm)
//! SILENTLY flipped the active flag + unregistered the entry without firing any
//! typed error to the consumer. JS/TS consumers via napi could not
//! distinguish 'subscription auto-cancelled because cap revoked' from
//! 'subscription dropped events because of buffer overflow / GC /
//! cursor-skip / engine-shutdown'.
//!
//! Wave-C1 wires three composing observability surfaces:
//!
//! 1. Process-wide counter `subscribe_revoked_mid_stream_count()` —
//!    increments per auto-cancel firing.
//! 2. Per-subscription `Subscription::termination_reason()` —
//!    populated with `Some(ErrorCode::SubscribeRevokedMidStream)`
//!    after the recheck-fail.
//! 3. Optional per-entry `termination_notify` callback (eval-side
//!    surface; engine-side adapter binds it through to the napi error
//!    envelope at G19+).
//!
//! THIS test pins (1) and (2). The napi adapter half lands in the
//! Wave-C2 DSL companion + a separate napi binding test.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path
//!   (`engine.testing_subscribe_observable_change_events` ->
//!   `register_on_change` -> `publish_change_event_with_labels` ->
//!   cap-recheck auto-cancel block);
//! - asserts an OBSERVABLE behavioral consequence (typed termination
//!   reason matches `ErrorCode::SubscribeRevokedMidStream` + counter
//!   increments per firing);
//! - would FAIL if the auto-cancel block were silently no-op'd (i.e.,
//!   if the typed-error emission path collapsed back to the pre-C1
//!   silent flip-and-continue shape).

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used)]

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::primitives::subscribe::{
    CapRecheckOutcome, ChangeEvent, ChangeKind, ChangePattern, OnChangeDeliveryCallback,
    SubscribeCursor, register_on_change, subscribe_revoked_mid_stream_count,
};

/// Process-wide mutex serializing the pre/post snapshots of the
/// `subscribe_revoked_mid_stream_count()` global counter across the two
/// tests in this file. Without this guard, the auto-cancel firing in
/// `cap_recheck_failing_mid_stream_...` (the +1 increment) can land
/// between the `pre_count` and `post_count` captures of
/// `freshly_registered_on_change_termination_reason_slot_is_none` under
/// parallel test execution — and the race window widens substantially
/// under `cargo-llvm-cov` coverage instrumentation (the recurring flake
/// runtime-incident this serializer closes).
///
/// pim-N-test-isolation §3.13 / `feedback_pim_test_isolation_process_scoped_shared_state`
/// "Don't apply when" clause #1: the counter is GENUINELY
/// process-scoped production observability state (consumed by operator
/// dashboards + the typed-error contract), NOT mock-injection state.
/// Per-test static decomposition does not apply; mutex-guarded
/// ordering does. Mirrors the existing precedent at
/// `crates/benten-eval/tests/sandbox_basic.rs::MODULE_CACHE_TEST_SERIALIZER`
/// (R6-R3 r6-r3-cr-2 closure — module-cache size snapshots across
/// parallel SANDBOX tests).
static REVOKED_MID_STREAM_COUNT_TEST_SERIALIZER: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn cap_recheck_failing_mid_stream_populates_subscription_termination_reason_with_typed_code() {
    // Serialize against the sibling test's pre/post counter window —
    // see `REVOKED_MID_STREAM_COUNT_TEST_SERIALIZER` rationale above.
    let _serial_guard = REVOKED_MID_STREAM_COUNT_TEST_SERIALIZER
        .lock()
        .unwrap_or_else(|poisoned| {
            // Poisoning means a prior test panicked while holding the
            // guard; the global counter state is recoverable for our
            // purposes (we read pre/post deltas, not absolute values),
            // so consume the poison + proceed.
            poisoned.into_inner()
        });
    // cap-r6-r1-1 / r4b-cap-6 attack-vector pin (R6-FP Wave-C1
    // closure). Drives the eval-side auto-cancel block end-to-end:
    //
    //   1. Register a subscription with a cap-recheck closure that
    //      flips from `true` to `false` on the second event delivery.
    //   2. Fire two events through `publish_change_event_with_labels`
    //      so the second event triggers the auto-cancel.
    //   3. Assert the termination_reason slot carries
    //      `Some(ErrorCode::SubscribeRevokedMidStream)` AND the
    //      process-wide counter incremented by exactly 1.
    //
    // This is the eval-crate-direct shape — the engine-side adapter
    // (which binds the termination_reason slot into a Subscription
    // handle) is exercised at the integration level through
    // `Engine::on_change_with_cap_recheck`.

    let pre_count = subscribe_revoked_mid_stream_count();

    // Cap-recheck closure: returns `Keep` on first call, `Cancel` on
    // subsequent calls (Phase 4-Foundation G22-FP-1 option-D migration:
    // this test pins the SHIPPED whole-subscription auto-cancel path,
    // so the post-revoke outcome is `Cancel` — the option-D semantic
    // equivalent of the prior `false`). State held in an AtomicU64
    // so the closure can mutate without a Mutex.
    let recheck_calls = Arc::new(AtomicU64::new(0));
    let recheck = {
        let calls = Arc::clone(&recheck_calls);
        Arc::new(move |_event: &ChangeEvent| -> CapRecheckOutcome {
            let n = calls.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                CapRecheckOutcome::Keep
            } else {
                CapRecheckOutcome::Cancel
            }
        })
    };

    let active = Arc::new(AtomicBool::new(true));
    let max_seq = Arc::new(AtomicU64::new(0));
    let termination_reason = Arc::new(std::sync::Mutex::new(None));
    let cb_calls = Arc::new(AtomicU64::new(0));
    let cb: OnChangeDeliveryCallback = {
        let calls = Arc::clone(&cb_calls);
        Arc::new(move |_event: &ChangeEvent| {
            calls.fetch_add(1, Ordering::SeqCst);
        })
    };

    register_on_change(
        ChangePattern::LabelGlob("Test:*".to_string()),
        SubscribeCursor::Latest,
        cb,
        Some(recheck),
        Arc::clone(&active),
        Arc::clone(&max_seq),
        None,
        Arc::clone(&termination_reason),
    )
    .unwrap();

    let mk_event = |seq: u64| {
        ChangeEvent::minimal(
            Cid::from_blake3_digest(*blake3::hash(format!("anchor-{seq}").as_bytes()).as_bytes()),
            ChangeKind::Created,
            seq,
            Vec::new(),
        )
    };

    // Round 1: first event delivers cleanly (recheck returns true).
    benten_eval::primitives::subscribe::publish_change_event_with_labels(
        &["Test:foo".to_string()],
        mk_event(100),
    );
    // Round 2: second event triggers the auto-cancel (recheck returns false).
    benten_eval::primitives::subscribe::publish_change_event_with_labels(
        &["Test:foo".to_string()],
        mk_event(200),
    );

    // OBSERVABLE consequence #1: termination_reason slot populated
    // with `Some(ErrorCode::SubscribeRevokedMidStream)` after the
    // recheck-fail fired.
    let reason = termination_reason.lock().unwrap().clone();
    assert_eq!(
        reason,
        Some(ErrorCode::SubscribeRevokedMidStream),
        "termination_reason should carry the typed code after auto-cancel; \
         got {reason:?} — typed-error emission path was silently no-op'd"
    );

    // OBSERVABLE consequence #2: process-wide counter incremented by
    // exactly 1 (one auto-cancel firing, even if the publish loop
    // walks the same entry multiple times).
    let post_count = subscribe_revoked_mid_stream_count();
    assert_eq!(
        post_count.saturating_sub(pre_count),
        1,
        "subscribe_revoked_mid_stream_count should have incremented by exactly 1; \
         pre={pre_count} post={post_count}"
    );

    // OBSERVABLE consequence #3: the subscription is auto-cancelled —
    // the active flag flipped to false. Defends against the
    // failure shape where the typed-error fires but the subscription
    // remains active and continues to fire termination_notify on
    // every subsequent event.
    assert!(
        !active.load(Ordering::SeqCst),
        "active flag should be false after auto-cancel; the subscription is leaking"
    );

    // OBSERVABLE consequence #4: only one event was delivered to the
    // user callback (the first event; the second one auto-cancelled
    // before delivery).
    assert_eq!(
        cb_calls.load(Ordering::SeqCst),
        1,
        "user callback should have fired exactly once (first event); the second event \
         was rejected by the cap-recheck before delivery"
    );
}

#[test]
fn freshly_registered_on_change_termination_reason_slot_is_none() {
    // Companion-positive pin: a subscription that has NOT been
    // auto-cancelled keeps `None` in the termination_reason slot.
    // Defends against the failure shape where the slot defaults to
    // `Some(SubscribeRevokedMidStream)` for every subscription
    // (false-positive pollution that would defeat the typed-error
    // observability contract).
    //
    // Serialize against the sibling test's pre/post counter window —
    // see `REVOKED_MID_STREAM_COUNT_TEST_SERIALIZER` rationale above.
    // Without this guard, the sibling test's +1 auto-cancel increment
    // can land between this test's `pre_count` and `post_count`
    // captures, producing the spurious "left:1 / right:0" assertion
    // failure (the cargo-llvm-cov coverage-instrumentation flake).
    let _serial_guard = REVOKED_MID_STREAM_COUNT_TEST_SERIALIZER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    let active = Arc::new(AtomicBool::new(true));
    let max_seq = Arc::new(AtomicU64::new(0));
    let termination_reason = Arc::new(std::sync::Mutex::new(None));
    let cb: OnChangeDeliveryCallback = Arc::new(|_event: &ChangeEvent| {});

    register_on_change(
        ChangePattern::LabelGlob("FreshProbe:*".to_string()),
        SubscribeCursor::Latest,
        cb,
        None,
        Arc::clone(&active),
        Arc::clone(&max_seq),
        None,
        Arc::clone(&termination_reason),
    )
    .unwrap();

    assert_eq!(
        termination_reason.lock().unwrap().clone(),
        None,
        "freshly-registered subscription must not pre-populate the termination_reason slot"
    );

    // A registration with no cap-recheck fires no auto-cancel even
    // when events deliver.
    let pre_count = subscribe_revoked_mid_stream_count();
    benten_eval::primitives::subscribe::publish_change_event_with_labels(
        &["FreshProbe:hello".to_string()],
        ChangeEvent::minimal(
            Cid::from_blake3_digest(*blake3::hash(b"anchor-fresh").as_bytes()),
            ChangeKind::Created,
            500,
            Vec::new(),
        ),
    );
    let post_count = subscribe_revoked_mid_stream_count();
    assert_eq!(
        post_count, pre_count,
        "no-cap-recheck subscription must not increment the auto-cancel counter"
    );
    assert_eq!(
        termination_reason.lock().unwrap().clone(),
        None,
        "no-cap-recheck subscription must not populate the termination_reason slot"
    );
}

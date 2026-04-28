//! Phase 2b G6-B: SUBSCRIBE engine wrappers — `onChange` ad-hoc
//! consumer surface.
//!
//! Sibling module to [`engine_stream`](crate::engine_stream). Companion
//! to G6-A which owns the `benten_eval::primitives::subscribe` executor
//! + the SUBSCRIBE delivery semantics per D5-RESOLVED. The wrappers in
//! this file plumb the engine-side public API and the cross-language
//! boundary; the production SUBSCRIBE executor itself lives in
//! `benten-eval`.
//!
//! # Dual surface (dx-optimizer corrected)
//!
//! Per plan §3 G6-B (R1 dx-optimizer):
//!
//! - `subgraph(...).subscribe(args)` — DSL handler-side composition
//!   primitive (lives in `packages/engine/src/dsl.ts`; the Rust side
//!   just receives it as a `PrimitiveKind::Subscribe` Node in the
//!   registered SubgraphSpec).
//! - [`Engine::on_change`] — `engine.onChange(pattern, callback) ->
//!   Subscription`. Renamed from `engine.subscribe` in the original
//!   sketch to avoid name-collision with the DSL `subgraph(...).subscribe`
//!   builder method per dx-optimizer's R1 finding.
//!
//! # Delivery semantics (G6-A D5-RESOLVED)
//!
//! Engine-assigned `u64 seq` + engine-side dedup at the handler
//! boundary = exactly-once at the handler API surface. Internally
//! at-least-once is an implementation detail. Cursor modes
//! `Latest` / `Sequence(u64)` / `Persistent(SubscriberId)`. Within-key
//! strict ordering, cross-key unordered. Bounded retention window
//! (1000 events OR 24h) for persistent cursors. Cap-check at delivery.
//! Per-subscription state = `max_delivered_seq: u64`. The wrapper
//! surface here exposes the subset of these knobs that the ad-hoc
//! consumer pattern needs; persistent cursor wiring lands once
//! G12-E SuspensionStore wires through.
//!
//! # G6-A coordination
//!
//! Until G6-A lands the change-stream port + the SUBSCRIBE executor
//! body, [`Engine::on_change`] returns a [`Subscription`] handle whose
//! `is_active()` returns `false` — the wrapper compiles and exercises
//! the round-trip shape but no events are delivered. Once G6-A merges,
//! the [`Subscription`]'s `Drop` impl will issue the unsubscribe call
//! into the change-stream port.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use benten_errors::ErrorCode;
use benten_eval::chunk_sink::Chunk;

use crate::engine::Engine;
use crate::error::EngineError;

/// Cursor mode for SUBSCRIBE consumers (D5-RESOLVED).
///
/// - `Latest` — start from the next event published after the
///   `on_change` call returns.
/// - `Sequence(seq)` — start from the engine-assigned sequence number
///   `seq`. Within the bounded retention window the engine replays
///   from the cursor; outside the window the call surfaces
///   `E_SUBSCRIBE_CURSOR_OUT_OF_WINDOW` (D5).
/// - `Persistent(subscriber_id)` — engine-managed cursor stored in the
///   G12-E SuspensionStore so a re-subscribe across process restart
///   resumes from `max_delivered_seq + 1`. Lands once G12-E + G6-A's
///   real wiring merge.
#[derive(Debug, Clone)]
pub enum SubscribeCursor {
    /// Start from the next event published after this call.
    Latest,
    /// Start from engine-assigned sequence number `seq`.
    Sequence(u64),
    /// Engine-managed persistent cursor keyed by `subscriber_id`.
    Persistent(String),
}

/// Subscription handle returned by [`Engine::on_change`].
///
/// The handle owns the consumer-side bookkeeping. Drop the handle to
/// unsubscribe — the engine-side change-stream port releases the
/// callback registration in the destructor. No explicit `unsubscribe()`
/// call is required (though one is exposed for callers who want to
/// release before the handle goes out of scope).
///
/// # D5 dedup state
///
/// `max_delivered_seq` tracks the highest engine-assigned sequence the
/// consumer has observed. The change-stream delivery path consults
/// this counter: `seq > max_delivered_seq` => deliver + bump;
/// `seq <= max_delivered_seq` => drop silently. This is the dedup
/// machinery that makes the handler API exactly-once on top of the
/// engine's internal at-least-once.
pub struct Subscription {
    /// Active until the handle drops or `unsubscribe()` is called.
    /// `Arc` so the change-stream port's delivery path can observe the
    /// flag flip without holding a mutable borrow.
    active: Arc<AtomicBool>,
    /// Highest engine-assigned sequence delivered to this subscriber.
    /// `Arc<AtomicU64>` so the delivery path can bump it lock-free
    /// from the change-stream worker thread.
    max_delivered_seq: Arc<AtomicU64>,
    /// Pattern the subscription was registered with (event-name glob).
    pattern: String,
    /// Cursor mode at registration time. Captured for the audit log
    /// G12-E will read at re-subscribe time.
    cursor: SubscribeCursor,
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("pattern", &self.pattern)
            .field("cursor", &self.cursor)
            .field("active", &self.active.load(Ordering::SeqCst))
            .field(
                "max_delivered_seq",
                &self.max_delivered_seq.load(Ordering::SeqCst),
            )
            .finish()
    }
}

impl Subscription {
    /// `true` while the subscription is registered with the engine.
    /// Flips to `false` after [`Self::unsubscribe`] or `Drop`.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    /// Pattern the subscription was registered with.
    #[must_use]
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Current cursor (`Latest` / `Sequence(seq)` / `Persistent(id)`).
    #[must_use]
    pub fn cursor(&self) -> &SubscribeCursor {
        &self.cursor
    }

    /// Highest engine-assigned sequence number observed by this
    /// subscription's delivery path. `0` before the first event lands.
    #[must_use]
    pub fn max_delivered_seq(&self) -> u64 {
        self.max_delivered_seq.load(Ordering::SeqCst)
    }

    /// Explicitly release the subscription. Idempotent.
    pub fn unsubscribe(&self) {
        self.active.store(false, Ordering::SeqCst);
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Subscriptions auto-release on drop. Once G6-A's change-stream
        // port lands, this is where the port's de-registration call
        // fires (the port already learns about the drop via the
        // `Arc<AtomicBool>` flip; the explicit call is a hint for the
        // port to GC its callback table earlier than the next
        // delivery sweep).
        self.unsubscribe();
    }
}

/// Callback shape for an `on_change` registration. Receives a
/// (sequence, payload) pair so dedup-aware consumers can correlate
/// with [`Subscription::max_delivered_seq`] for cross-process
/// continuation.
pub type OnChangeCallback = Arc<dyn Fn(u64, &Chunk) + Send + Sync + 'static>;

impl Engine {
    /// Phase 2b G6-B: register an ad-hoc change-stream consumer.
    ///
    /// `pattern` is an event-name glob (e.g. `"post:*"`,
    /// `"system:CapabilityGrant"`). `callback` fires once per matched
    /// event with `(engine_assigned_seq, payload)`. Returns a
    /// [`Subscription`] whose drop unsubscribes.
    ///
    /// Renamed from the original sketch's `engine.subscribe` to avoid
    /// name-collision with the DSL `subgraph(...).subscribe` builder
    /// method (dx-optimizer R1 finding). The TS wrapper exposes this
    /// as `engine.onChange(pattern, callback) -> Subscription`.
    ///
    /// # Pre-G6-A behavior
    ///
    /// Until G6-A's change-stream port + executor land, the returned
    /// handle's `is_active()` is `false` immediately and no callbacks
    /// fire. The handle's shape is locked here so the wrapper
    /// `packages/engine/src/subscribe.ts` compiles and exercises the
    /// round-trip shape before the port wires in. Once G6-A merges,
    /// this method registers the callback against the port and the
    /// returned handle stays active until drop / explicit
    /// unsubscribe.
    ///
    /// # Errors
    /// Returns [`EngineError`] if the engine's policy denies the
    /// subscription at registration time (D5-RESOLVED cap-check
    /// fires at register; per-event delivery cap-check fires inside
    /// the executor body).
    pub fn on_change(
        &self,
        pattern: &str,
        callback: OnChangeCallback,
    ) -> Result<Subscription, EngineError> {
        self.on_change_with_cursor(pattern, SubscribeCursor::Latest, callback)
    }

    /// `on_change` with an explicit cursor. The default
    /// [`Engine::on_change`] entry point uses [`SubscribeCursor::Latest`];
    /// callers that want sequence-based replay or persistent cursors
    /// route through here.
    ///
    /// # Errors
    /// See [`Engine::on_change`].
    pub fn on_change_with_cursor(
        &self,
        pattern: &str,
        cursor: SubscribeCursor,
        _callback: OnChangeCallback,
    ) -> Result<Subscription, EngineError> {
        if pattern.is_empty() {
            // G6-A adds `E_SUBSCRIBE_PATTERN_INVALID` to ERROR-CATALOG; until
            // that lands in `benten_errors`, surface as `InputLimit` (the
            // closest existing "shape rejection" code). The wrapper will
            // be re-targeted to the real code once G6-A merges its catalog
            // additions.
            return Err(EngineError::Other {
                code: ErrorCode::InputLimit,
                message: "on_change: pattern must be a non-empty event-name glob \
                          (E_SUBSCRIBE_PATTERN_INVALID lands with G6-A)"
                    .into(),
            });
        }

        // Pre-G6-A: build the handle but don't actually wire the
        // callback into the change-stream port (because the port
        // doesn't exist yet). The handle's `active` flag starts `false`
        // so consumers can observe "subscription was constructed but
        // not yet wired" via `is_active()`. Once G6-A lands the
        // `benten-core::ChangeStream` port (D23-RECOMMEND), this method
        // registers `_callback` against the port and flips `active` to
        // `true` before returning the handle.
        Ok(Subscription {
            active: Arc::new(AtomicBool::new(false)),
            max_delivered_seq: Arc::new(AtomicU64::new(0)),
            pattern: pattern.to_string(),
            cursor,
        })
    }

    /// ts-r4-2 mirror for SUBSCRIBE: synchronous test-helper that
    /// constructs an active [`Subscription`] without going through the
    /// (pre-G6-A absent) change-stream port. Useful for vitest
    /// harnesses verifying the unsubscribe + dedup state machinery
    /// without depending on a live change-stream port.
    ///
    /// cfg-gated under `cfg(any(test, feature = "test-helpers"))` per
    /// Phase-2a sec-r6r2-02 discipline.
    #[cfg(any(test, feature = "test-helpers"))]
    #[must_use]
    pub fn testing_open_subscription_for_test(
        &self,
        pattern: &str,
        cursor: SubscribeCursor,
    ) -> Subscription {
        Subscription {
            active: Arc::new(AtomicBool::new(true)),
            max_delivered_seq: Arc::new(AtomicU64::new(0)),
            pattern: pattern.to_string(),
            cursor,
        }
    }

    /// ts-r4-2 mirror: synthetic delivery path used by harness tests
    /// to exercise the dedup machinery without a real change-stream
    /// port. Bumps `max_delivered_seq` if `seq > max_delivered_seq`
    /// (the same condition the production delivery path uses).
    /// Returns `true` if the synthetic delivery was applied,
    /// `false` if it was deduped.
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_deliver_synthetic_event_for_test(&self, sub: &Subscription, seq: u64) -> bool {
        let cur = sub.max_delivered_seq.load(Ordering::SeqCst);
        if seq > cur {
            sub.max_delivered_seq.store(seq, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "tests may use unwrap per workspace policy"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn temp_engine() -> (Engine, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path: PathBuf = dir.path().join("engine.redb");
        let engine = Engine::open(&path).unwrap();
        (engine, dir)
    }

    #[test]
    fn empty_pattern_rejects_with_typed_error() {
        let (e, _d) = temp_engine();
        let cb: OnChangeCallback = Arc::new(|_, _| {});
        let err = e.on_change("", cb).unwrap_err();
        match err {
            EngineError::Other { code, .. } => {
                assert_eq!(code, ErrorCode::InputLimit);
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn pre_g6a_subscription_is_inactive_but_constructable() {
        let (e, _d) = temp_engine();
        let cb: OnChangeCallback = Arc::new(|_, _| {});
        let sub = e.on_change("post:*", cb).unwrap();
        assert!(!sub.is_active(), "pre-G6-A handle starts inactive");
        assert_eq!(sub.pattern(), "post:*");
        assert_eq!(sub.max_delivered_seq(), 0);
    }

    #[test]
    fn unsubscribe_flips_active_flag_idempotent() {
        let (e, _d) = temp_engine();
        let sub = e.testing_open_subscription_for_test("post:*", SubscribeCursor::Latest);
        assert!(sub.is_active());
        sub.unsubscribe();
        assert!(!sub.is_active());
        // Idempotent — second call doesn't panic.
        sub.unsubscribe();
        assert!(!sub.is_active());
    }

    #[test]
    fn synthetic_delivery_applies_dedup_state_machine() {
        let (e, _d) = temp_engine();
        let sub = e.testing_open_subscription_for_test("post:*", SubscribeCursor::Latest);
        assert!(e.testing_deliver_synthetic_event_for_test(&sub, 1));
        assert_eq!(sub.max_delivered_seq(), 1);
        assert!(e.testing_deliver_synthetic_event_for_test(&sub, 2));
        assert_eq!(sub.max_delivered_seq(), 2);
        // Re-delivery of seq 2 deduped.
        assert!(!e.testing_deliver_synthetic_event_for_test(&sub, 2));
        assert_eq!(sub.max_delivered_seq(), 2);
        // Re-delivery of seq 1 deduped.
        assert!(!e.testing_deliver_synthetic_event_for_test(&sub, 1));
        assert_eq!(sub.max_delivered_seq(), 2);
    }

    #[test]
    fn drop_unsubscribes_automatically() {
        let (e, _d) = temp_engine();
        let active_flag;
        {
            let sub = e.testing_open_subscription_for_test("post:*", SubscribeCursor::Latest);
            active_flag = sub.active.clone();
            assert!(active_flag.load(Ordering::SeqCst));
        } // sub dropped here
        assert!(!active_flag.load(Ordering::SeqCst));
    }

    #[test]
    fn cursor_modes_round_trip() {
        let (e, _d) = temp_engine();
        let s1 = e.testing_open_subscription_for_test("a", SubscribeCursor::Latest);
        let s2 = e.testing_open_subscription_for_test("a", SubscribeCursor::Sequence(42));
        let s3 =
            e.testing_open_subscription_for_test("a", SubscribeCursor::Persistent("sub-x".into()));
        assert!(matches!(s1.cursor(), SubscribeCursor::Latest));
        assert!(matches!(s2.cursor(), SubscribeCursor::Sequence(42)));
        match s3.cursor() {
            SubscribeCursor::Persistent(id) => assert_eq!(id, "sub-x"),
            _ => panic!("wrong cursor variant"),
        }
    }
}

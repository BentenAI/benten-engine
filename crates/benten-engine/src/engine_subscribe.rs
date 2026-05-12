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
use benten_eval::primitives::subscribe::{
    ChangeEvent as EvalChangeEvent, ChangePattern, SubscribeCursor as EvalSubscribeCursor,
    SubscriberId, register_on_change, unregister_on_change,
};

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
    /// Wave-8c-subscribe-infra: subscriber id assigned by
    /// `register_on_change` so `Drop` / `unsubscribe()` can find the
    /// registry slot. `None` for handles constructed via
    /// `testing_open_subscription_for_test` (no registry slot to free).
    registry_id: Option<SubscriberId>,
    /// Phase-3 R6-FP Wave-C1 (cap-r6-r1-1 / r4b-cap-6 closure): shared
    /// termination-reason slot populated by the eval-side publish
    /// loop's auto-cancel path with
    /// [`ErrorCode::SubscribeRevokedMidStream`] when the per-event
    /// delivery-time cap-recheck returns `false`. The
    /// [`Subscription::termination_reason`] accessor reads this slot
    /// so JS/TS consumers can distinguish 'cap-revoke auto-cancel'
    /// from buffer-overflow / GC / cursor-skip / engine-shutdown
    /// drops per the CLR-2 §11 typed-error contract. `None` for
    /// handles whose subscription was unsubscribed cleanly (no
    /// recheck-failure).
    termination_reason: Arc<std::sync::Mutex<Option<ErrorCode>>>,
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
            .finish_non_exhaustive()
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

    /// Phase-3 R6-FP Wave-C1 (cap-r6-r1-1 / r4b-cap-6 closure): typed
    /// termination reason populated by the eval-side publish loop
    /// when the per-event delivery-time cap-recheck returns `false`.
    /// Returns `Some(ErrorCode::SubscribeRevokedMidStream)` after
    /// the auto-cancel firing; `None` for handles whose subscription
    /// was unsubscribed cleanly (consumer-driven close / Drop) or
    /// dropped for non-revocation reasons (cursor-skip /
    /// engine-shutdown). Per CLR-2 §11 the typed-error observability
    /// contract for JS/TS consumers reads this slot through the
    /// napi binding so consumers route on
    /// `err.code === 'E_SUBSCRIBE_REVOKED_MID_STREAM'`.
    #[must_use]
    pub fn termination_reason(&self) -> Option<ErrorCode> {
        self.termination_reason.lock().ok().and_then(|g| g.clone())
    }

    /// Explicitly release the subscription. Idempotent.
    pub fn unsubscribe(&self) {
        self.active.store(false, Ordering::SeqCst);
        // Wave-8c-subscribe-infra: also drop the registry slot eagerly so
        // the on_change callback table doesn't keep firing the closure
        // (Arc-cloned into the registry) until the next walk GCs it.
        if let Some(id) = self.registry_id.as_ref() {
            unregister_on_change(id);
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        // Subscriptions auto-release on drop. The active flag flip
        // signals the change-stream port; the explicit `unregister`
        // call hints the registry to GC its callback table immediately
        // rather than waiting for the next delivery sweep.
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
    /// # Cursor mode for ad-hoc `on_change`
    ///
    /// This entry point uses [`SubscribeCursor::Latest`] (the default).
    /// Persistent-cursor mode (`SubscribeCursor::Persistent(id)`) is
    /// available via [`Engine::on_change_with_cursor`] but its
    /// restoration story is bounded for ad-hoc onChange:
    /// subgraph-internal SUBSCRIBE primitives keep the G12-E
    /// SuspensionStore-backed persistent-cursor restoration path; the
    /// ad-hoc engine surface falls back to Latest semantics for the
    /// in-process subscription, since there is no engine-managed
    /// suspend/resume boundary that would re-hydrate the cursor across
    /// process restart. Callers needing durable cursor restoration
    /// across engine restart should use a subgraph with a SUBSCRIBE
    /// primitive rather than `engine.on_change`.
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

    /// Phase 2b wave-8c-cont: `on_change` with an explicit actor
    /// principal. Mirrors [`Engine::call_as`] naming.
    ///
    /// The `actor` CID is captured on the registered ad-hoc onChange
    /// entry's delivery-time cap-recheck closure so D5 cap-recheck-at-
    /// delivery fires the named principal's grants on every event. If
    /// the principal's caps no longer cover the event's anchor at
    /// delivery time, the subscription is auto-cancelled per D5
    /// contract.
    ///
    /// Same cursor-mode caveat as [`Engine::on_change`]: the ad-hoc
    /// engine surface falls back to Latest semantics for persistent
    /// cursors; durable cross-restart cursor restoration is reserved
    /// for subgraph-internal SUBSCRIBE primitives.
    ///
    /// # Errors
    /// See [`Engine::on_change`].
    pub fn on_change_as(
        &self,
        pattern: &str,
        callback: OnChangeCallback,
        actor: &benten_core::Cid,
    ) -> Result<Subscription, EngineError> {
        self.on_change_as_with_cursor(pattern, SubscribeCursor::Latest, callback, actor)
    }

    /// Phase 2b wave-8c-subscribe-infra: `on_change_as` with an explicit
    /// cursor. Reaches the same registry slot as
    /// [`Engine::on_change_with_cursor`] but threads a delivery-time
    /// cap-recheck closure that consults the configured
    /// [`benten_caps::CapabilityPolicy`] against the registered actor.
    ///
    /// # Errors
    /// See [`Engine::on_change`].
    pub fn on_change_as_with_cursor(
        &self,
        pattern: &str,
        cursor: SubscribeCursor,
        callback: OnChangeCallback,
        actor: &benten_core::Cid,
    ) -> Result<Subscription, EngineError> {
        if pattern.is_empty() {
            return Err(EngineError::Other {
                code: ErrorCode::SubscribePatternInvalid,
                message: "on_change: pattern must be a non-empty event-name glob".into(),
            });
        }
        // Phase 4-Foundation R1-FP wave-1 G22-FP-1 option-D (2026-05-12,
        // sec-4f-r1-1 BLOCKER closure): the cap-recheck closure now
        // returns `CapRecheckOutcome`. Two-gate composition:
        //
        //   - Whole-actor-revoke (`is_actor_active=false`) → `Cancel`.
        //     Preserves Phase-3 R6-FP Wave-C1 SHIPPED termination
        //     contract (`E_SUBSCRIBE_REVOKED_MID_STREAM` + termination
        //     notify + auto-unregister) for the case where the actor
        //     as a whole was revoked and the subscription cannot
        //     continue.
        //   - Per-event `CapabilityPolicy::check_read` denial → `Drop`.
        //     NEW per-Node fail-soft semantic: this specific event's
        //     anchor is no longer covered, but the subscription
        //     itself stays active — future events the principal still
        //     covers continue delivering. The admin-UI per-cap-
        //     revocation UX path (cap-r1-9 closure ordering corollary;
        //     closes sec-4f-r1-1 BLOCKER + mat-r1-4 materializer-
        //     subscribe seam ordering).
        //
        // The policy lives behind `Arc<dyn CapabilityPolicy>` on the
        // engine (per the Box→Arc migration that landed alongside
        // option-D). Capturing the Arc into the closure is the cheap
        // shared-handle path. For NoAuth-equivalent configurations
        // (policy=None), the closure short-circuits to `Keep` — the
        // pre-option-D no-gate-at-all behaviour.
        let actor_cid = *actor;
        let inner_for_check = Arc::clone(&self.inner);
        let policy_handle: Option<Arc<dyn benten_caps::CapabilityPolicy>> =
            self.policy.as_ref().map(Arc::clone);
        let cap_recheck: benten_eval::primitives::subscribe::DeliveryCapRecheck = Arc::new(
            move |event: &EvalChangeEvent| -> benten_eval::primitives::subscribe::CapRecheckOutcome {
                use benten_eval::primitives::subscribe::CapRecheckOutcome;
                // (1) Whole-actor-revoke gate (defense-in-depth, retained
                //     for Phase-3 SHIPPED tests + sibling consumers that
                //     route through `testing_revoke_actor`). On revoke
                //     we issue `Cancel` so the SHIPPED typed-error +
                //     auto-cancel + observability-counter contract
                //     continues to fire.
                if !inner_for_check.is_actor_active(&actor_cid) {
                    return CapRecheckOutcome::Cancel;
                }
                // (2) Per-event `CapabilityPolicy::check_read` gate (NEW
                //     option-D). Build a `ReadContext` from the event:
                //     `actor_cid` from the active principal (the
                //     `actor` arg captured into this closure);
                //     `label` from the event's first label (per
                //     `benten_core::change_stream::ChangeEvent.labels`);
                //     `target_cid` set to the event's anchor; the
                //     other fields (`device_cid`, `actor_hint`) left
                //     unset — they aren't substantive inputs to the
                //     per-event Drop path. On `Err(_)` we return
                //     `Drop` (silently elide; stream stays open). On
                //     `Ok(())` we return `Keep` (deliver).
                if let Some(policy) = policy_handle.as_ref() {
                    let label = event.labels.first().cloned().unwrap_or_default();
                    let ctx = benten_caps::ReadContext {
                        actor_cid: Some(actor_cid),
                        label,
                        target_cid: Some(event.anchor_cid),
                        device_cid: None,
                        actor_hint: None,
                    };
                    match policy.check_read(&ctx) {
                        Ok(()) => CapRecheckOutcome::Keep,
                        Err(_) => CapRecheckOutcome::Drop,
                    }
                } else {
                    // No policy configured (NoAuth-equivalent): deliver
                    // every event — matches the pre-option-D
                    // `is_actor_active=true` happy path.
                    CapRecheckOutcome::Keep
                }
            },
        );
        self.register_on_change_internal(pattern, cursor, callback, Some(cap_recheck))
    }

    /// Phase-3 G14-D wave-5a: `on_change` with an explicit per-event
    /// [`crate::cap_recheck::CapRecheckFn`] consulted at every
    /// delivery boundary. The closure consults the actor's READ
    /// coverage against the event's anchor; a partial revoke that
    /// strikes coverage observably cancels the affected subscription
    /// path mid-stream per F6 LOAD-BEARING + Compromise #2 D5.
    ///
    /// Composes with the G15-A materialization-time per-row gate
    /// at [`crate::cap_recheck::CapRecheckFn`] — both consumers
    /// share the helper signature per ds-r4r2-7 frozen-by-G13-pre-C
    /// scaffold.
    ///
    /// # Errors
    /// See [`Engine::on_change`].
    pub fn on_change_with_cap_recheck(
        &self,
        pattern: &str,
        callback: OnChangeCallback,
        actor: &benten_core::Cid,
        cap_recheck: crate::cap_recheck::CapRecheckFn,
    ) -> Result<Subscription, EngineError> {
        if pattern.is_empty() {
            return Err(EngineError::Other {
                code: ErrorCode::SubscribePatternInvalid,
                message: "on_change_with_cap_recheck: pattern must be non-empty".into(),
            });
        }
        // Bridge the engine-level CapRecheckFn (operates over
        // PrincipalId + zone + cid) into the eval-side
        // DeliveryCapRecheck (operates over an eval ChangeEvent). The
        // bridge passes the actor as the principal, the matched
        // pattern's first label segment as the zone hint, and the
        // event's anchor CID as the row CID; F6 cancel surfaces as
        // `false` from the engine-level closure ⇒ subscription drops.
        let principal = crate::cap_recheck::PrincipalId::from_actor_cid(*actor);
        let zone_hint = pattern.split(':').next().unwrap_or("").to_string();
        let recheck_arc = cap_recheck;
        let inner_for_check = Arc::clone(&self.inner);
        let actor_cid = *actor;
        // Phase 4-Foundation R1-FP wave-1 G22-FP-1 option-D (2026-05-12):
        // bridge the engine-level `CapRecheckFn` (PrincipalId + zone +
        // CID → bool) into the new `CapRecheckOutcome` enum return.
        // Per F6 dual-layer:
        //   (1) Coarse actor-active gate → `Cancel` (Phase-3 SHIPPED
        //       termination contract).
        //   (2) Per-event row recheck false → `Cancel` (this consumer
        //       path historically auto-cancels on revoke per F6
        //       LOAD-BEARING + Compromise #2 D5; consumers who want
        //       per-Node fail-soft elision route through the new
        //       `on_change_as_with_cursor` path instead).
        //   (3) Per-event row recheck true → `Keep`.
        let bridged: benten_eval::primitives::subscribe::DeliveryCapRecheck = Arc::new(
            move |event: &EvalChangeEvent| -> benten_eval::primitives::subscribe::CapRecheckOutcome {
                use benten_eval::primitives::subscribe::CapRecheckOutcome;
                let _ = (&inner_for_check, &actor_cid);
                if !inner_for_check.is_actor_active(&actor_cid) {
                    return CapRecheckOutcome::Cancel;
                }
                if (recheck_arc)(&principal, zone_hint.as_str(), &event.anchor_cid) {
                    CapRecheckOutcome::Keep
                } else {
                    CapRecheckOutcome::Cancel
                }
            },
        );
        self.register_on_change_internal(pattern, SubscribeCursor::Latest, callback, Some(bridged))
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
        callback: OnChangeCallback,
    ) -> Result<Subscription, EngineError> {
        if pattern.is_empty() {
            // cr-r4b-9 closure (wave-8e): `E_SUBSCRIBE_PATTERN_INVALID` IS
            // now in `benten_errors` (G6-A merged it; cap-recheck pattern
            // validation in `crates/benten-eval/src/primitives/subscribe.rs`
            // already returns this code at registration time). Surface
            // the typed code here too — `InputLimit` was the pre-G6-A
            // placeholder.
            return Err(EngineError::Other {
                code: ErrorCode::SubscribePatternInvalid,
                message: "on_change: pattern must be a non-empty event-name glob".into(),
            });
        }
        self.register_on_change_internal(pattern, cursor, callback, None)
    }

    /// Wave-8c-subscribe-infra: shared register path between
    /// [`Engine::on_change_with_cursor`] (no cap-recheck) and
    /// [`Engine::on_change_as_with_cursor`] (delivery-time cap-recheck).
    fn register_on_change_internal(
        &self,
        pattern: &str,
        cursor: SubscribeCursor,
        callback: OnChangeCallback,
        cap_recheck: Option<benten_eval::primitives::subscribe::DeliveryCapRecheck>,
    ) -> Result<Subscription, EngineError> {
        // Map the engine-side `OnChangeCallback` (Arc<dyn Fn(u64, &Chunk)>) to
        // the eval-side `OnChangeDeliveryCallback` (Arc<dyn Fn(&ChangeEvent)>).
        // Today Chunk wraps a `Vec<u8>` payload so we map the
        // ChangeEvent's `payload_bytes` into a Chunk and forward the
        // engine-assigned `seq`.
        let cb_for_eval: benten_eval::primitives::subscribe::OnChangeDeliveryCallback = {
            let user_cb = Arc::clone(&callback);
            Arc::new(move |event: &EvalChangeEvent| {
                let chunk = Chunk {
                    seq: event.seq,
                    bytes: event.payload_bytes.clone(),
                    final_chunk: false,
                };
                user_cb(event.seq, &chunk);
            })
        };

        // Translate the engine-side string pattern to the eval-side
        // [`ChangePattern`]. We default to `LabelGlob` so the existing
        // `engine.onChange("post:*", ...)` shape resolves to a
        // glob-pattern match. Patterns that begin with a literal prefix
        // and end with `*` map cleanly; anything else is also valid as
        // a glob.
        let eval_pattern = if pattern.contains('*') || pattern.contains('?') {
            ChangePattern::LabelGlob(pattern.to_string())
        } else {
            ChangePattern::AnchorPrefix(pattern.to_string())
        };

        let active = Arc::new(AtomicBool::new(true));
        let max_delivered_seq = Arc::new(AtomicU64::new(0));

        // Map the engine-side `SubscribeCursor` (which carries
        // `Persistent(String)`) onto the eval-side cursor
        // (`Persistent(SubscriberId)`). Persistent ids are content-
        // addressed via BLAKE3 of the supplied opaque label so two
        // callers using the same persistent id resolve to the same
        // underlying SubscriberId.
        let eval_cursor = match &cursor {
            SubscribeCursor::Latest => EvalSubscribeCursor::Latest,
            SubscribeCursor::Sequence(n) => EvalSubscribeCursor::Sequence(*n),
            SubscribeCursor::Persistent(s) => {
                EvalSubscribeCursor::Persistent(SubscriberId::from_cid(
                    benten_core::Cid::from_blake3_digest(*blake3::hash(s.as_bytes()).as_bytes()),
                ))
            }
        };

        // Phase-3 R6-FP Wave-C1 (cap-r6-r1-1 / r4b-cap-6 closure): bind
        // the shared termination-reason slot so the eval-side publish
        // loop's auto-cancel path can populate it with
        // `ErrorCode::SubscribeRevokedMidStream` and the engine-side
        // `Subscription::termination_reason()` accessor reads the same
        // slot.
        let termination_reason = Arc::new(std::sync::Mutex::new(None));
        let id = register_on_change(
            eval_pattern,
            eval_cursor,
            cb_for_eval,
            cap_recheck,
            Arc::clone(&active),
            Arc::clone(&max_delivered_seq),
            None,
            Arc::clone(&termination_reason),
        )
        .map_err(|e| EngineError::Other {
            code: e.error_code(),
            message: format!("on_change: registration failed: {e}"),
        })?;

        Ok(Subscription {
            active,
            max_delivered_seq,
            pattern: pattern.to_string(),
            cursor,
            registry_id: Some(id),
            termination_reason,
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
            registry_id: None,
            termination_reason: Arc::new(std::sync::Mutex::new(None)),
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

    /// G16-B-D Compromise #11 deepest-pin helper: subscribe with an
    /// observer that receives the eval-side [`EvalChangeEvent`]
    /// directly — bypassing the production
    /// [`OnChangeCallback`] adapter that wraps `payload_bytes` into a
    /// [`Chunk`]. Lets test pins assert on `anchor_cid` / `kind` /
    /// `actor_cid` / `labels` without a CBOR parser at the test scope.
    ///
    /// Mirrors the production [`Engine::on_change_with_cap_recheck`]
    /// composition (delivery-time cap-recheck closure threaded through
    /// the same eval-side `DeliveryCapRecheck` bridge), so the deepest
    /// e2e composition pin verifies BOTH gates (G15-A materialization
    /// + G14-D delivery) compose against a real subscription with
    /// observable per-event delivery.
    ///
    /// ## Cfg-gating discipline (echoes `sandbox_helpers_no_widening`)
    ///
    /// Strictly cfg-gated under `cfg(any(test, feature = "test-helpers"))`.
    /// The helper does NOT widen the production surface — it routes
    /// through the same `register_on_change` registry as the
    /// production path and the same delivery walk fires it; only the
    /// callback-shape adapter is different (eval-side callback delivered
    /// raw rather than mapped to `Chunk`).
    ///
    /// # Errors
    /// See [`Engine::on_change_with_cap_recheck`].
    #[cfg(any(test, feature = "test-helpers"))]
    pub fn testing_subscribe_observable_change_events(
        &self,
        pattern: &str,
        observer: Arc<dyn Fn(&EvalChangeEvent) + Send + Sync + 'static>,
        actor: &benten_core::Cid,
        cap_recheck: crate::cap_recheck::CapRecheckFn,
    ) -> Result<Subscription, EngineError> {
        if pattern.is_empty() {
            return Err(EngineError::Other {
                code: ErrorCode::SubscribePatternInvalid,
                message: "testing_subscribe_observable_change_events: pattern must be non-empty"
                    .into(),
            });
        }
        // Bridge engine-side CapRecheckFn (PrincipalId + zone + cid)
        // → eval-side DeliveryCapRecheck (eval ChangeEvent), matching
        // the production `on_change_with_cap_recheck` bridge above.
        let principal = crate::cap_recheck::PrincipalId::from_actor_cid(*actor);
        let zone_hint = pattern.split(':').next().unwrap_or("").to_string();
        let inner_for_check = Arc::clone(&self.inner);
        let actor_cid = *actor;
        let recheck_arc = cap_recheck;
        // Phase 4-Foundation R1-FP wave-1 G22-FP-1 option-D (2026-05-12):
        // mirror the production `on_change_with_cap_recheck` bridge —
        // `Cancel` on whole-actor-revoke or per-event-row-recheck=false,
        // `Keep` on per-event-row-recheck=true. Test-helper composition
        // preserves the SHIPPED contract.
        let bridged: benten_eval::primitives::subscribe::DeliveryCapRecheck = Arc::new(
            move |event: &EvalChangeEvent| -> benten_eval::primitives::subscribe::CapRecheckOutcome {
                use benten_eval::primitives::subscribe::CapRecheckOutcome;
                if !inner_for_check.is_actor_active(&actor_cid) {
                    return CapRecheckOutcome::Cancel;
                }
                if (recheck_arc)(&principal, zone_hint.as_str(), &event.anchor_cid) {
                    CapRecheckOutcome::Keep
                } else {
                    CapRecheckOutcome::Cancel
                }
            },
        );

        // Eval-side callback delivers the raw ChangeEvent — no Chunk
        // wrapping — so observers can assert anchor_cid / labels / kind.
        let cb_for_eval: benten_eval::primitives::subscribe::OnChangeDeliveryCallback = {
            let observer = Arc::clone(&observer);
            Arc::new(move |event: &EvalChangeEvent| {
                observer(event);
            })
        };

        let eval_pattern = if pattern.contains('*') || pattern.contains('?') {
            ChangePattern::LabelGlob(pattern.to_string())
        } else {
            ChangePattern::AnchorPrefix(pattern.to_string())
        };

        let active = Arc::new(AtomicBool::new(true));
        let max_delivered_seq = Arc::new(AtomicU64::new(0));
        let termination_reason = Arc::new(std::sync::Mutex::new(None));
        let id = register_on_change(
            eval_pattern,
            EvalSubscribeCursor::Latest,
            cb_for_eval,
            Some(bridged),
            Arc::clone(&active),
            Arc::clone(&max_delivered_seq),
            None,
            Arc::clone(&termination_reason),
        )
        .map_err(|e| EngineError::Other {
            code: e.error_code(),
            message: format!(
                "testing_subscribe_observable_change_events: registration failed: {e}"
            ),
        })?;

        Ok(Subscription {
            active,
            max_delivered_seq,
            pattern: pattern.to_string(),
            cursor: SubscribeCursor::Latest,
            registry_id: Some(id),
            termination_reason,
        })
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
                assert_eq!(code, ErrorCode::SubscribePatternInvalid);
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn wave_8c_subscription_is_active_immediately() {
        // Wave-8c-subscribe-infra: production change-stream wire-through
        // makes the returned handle ACTIVE immediately. The previous
        // `pre-G6-A: inactive` shape was the unwired-stub surface.
        let (e, _d) = temp_engine();
        let cb: OnChangeCallback = Arc::new(|_, _| {});
        let sub = e.on_change("post:*", cb).unwrap();
        assert!(
            sub.is_active(),
            "wave-8c-subscribe-infra returns active handle"
        );
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
    fn on_change_as_threads_principal_with_active_handle() {
        // Phase 2b wave-8c-subscribe-infra: on_change_as accepts an
        // explicit actor CID, registers the callback against the
        // production registry, and returns an active Subscription whose
        // delivery-time D5 cap-recheck consults the engine's revoked-
        // actor set.
        let (e, _d) = temp_engine();
        let cb: OnChangeCallback = Arc::new(|_, _| {});
        let actor = benten_core::Cid::from_blake3_digest(*blake3::hash(b"test-actor").as_bytes());
        let sub = e.on_change_as("post:*", cb, &actor).unwrap();
        assert!(sub.is_active(), "wave-8c handle is active");
        assert_eq!(sub.pattern(), "post:*");
    }

    #[test]
    fn on_change_as_rejects_empty_pattern_with_typed_error() {
        let (e, _d) = temp_engine();
        let cb: OnChangeCallback = Arc::new(|_, _| {});
        let actor = benten_core::Cid::from_blake3_digest(*blake3::hash(b"test-actor").as_bytes());
        let err = e.on_change_as("", cb, &actor).unwrap_err();
        match err {
            EngineError::Other { code, .. } => {
                assert_eq!(code, ErrorCode::SubscribePatternInvalid);
            }
            _ => panic!("unexpected error variant"),
        }
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

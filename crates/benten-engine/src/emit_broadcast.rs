//! Wave-8h audit-gap fix — EMIT primitive broadcast channel.
//!
//! Mirrors [`crate::change::ChangeBroadcast`] for EMIT events. The
//! Phase-2b audit at
//! `.addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json`
//! surfaced that `impl PrimitiveHost::emit_event` was a no-op: a handler
//! with a standalone EMIT primitive (no backing WRITE) silently dropped
//! the payload, ChangeBroadcast did not see it, and no consumer-visible
//! observability path existed for emit-only events.
//!
//! ## Why a separate broadcast (not ChangeBroadcast)
//!
//! [`benten_graph::ChangeEvent`] is keyed on a `cid: Cid` of the affected
//! Node + a `kind: ChangeKind` enum + commit-time fields (`tx_id`,
//! attribution CIDs). EMIT events have NONE of those — there's no Node,
//! no commit, no tx-id. Adding an `Emitted` variant to `ChangeKind` plus
//! threading optional emit fields onto `ChangeEvent` would cascade
//! through benten-graph + every IVM view + the napi/TS surface.
//!
//! A separate [`EmitBroadcast`] keeps the EMIT channel structurally
//! independent — IVM views continue to see only storage events,
//! emit-observers see only emit events, and neither path pays for the
//! other's machinery. The trade-off: subscribers wanting BOTH must
//! attach to two channels. Phase-3 may unify if a real use case arises.
//!
//! ## Phase 1 → 2b shape
//!
//! Stdlib-only fan-out: a `Vec<Arc<EmitCallback>>` behind a `Mutex` plus
//! a `publish(name, payload)` entry point + `subscribe_fn` registration.
//! Same panic-isolation discipline as `ChangeBroadcast::publish`.

use std::sync::{Arc, Mutex};

use benten_core::Value;
use benten_graph::MutexExt;

/// Event payload published via [`EmitBroadcast::publish`]. Carries the
/// emit channel name + the raw [`Value`] payload the EMIT primitive was
/// invoked with. Phase-3 may add commit-time correlation fields if a
/// use case arises.
#[derive(Debug, Clone)]
pub struct EmitEvent {
    /// EMIT channel name — the value of the EMIT node's `channel`
    /// property. Subscribers route on this string.
    pub channel: String,
    /// EMIT payload — the value of the EMIT node's `payload` property,
    /// or [`Value::Null`] when no payload was declared.
    pub payload: Value,
}

/// Callback alias for EMIT subscribers. The `'static` bound mirrors
/// [`crate::change::ChangeCallback`].
pub type EmitCallback = Arc<dyn Fn(&EmitEvent) + Send + Sync + 'static>;

/// Handle to the engine's EMIT broadcast.
///
/// Mirrors [`crate::change::ChangeBroadcast`] but for emit-only events.
/// See module-level docs for the rationale (separate channel rather
/// than a `ChangeKind::Emitted` variant on the storage broadcast).
#[derive(Default)]
pub struct EmitBroadcast {
    callbacks: Mutex<Vec<EmitCallback>>,
}

impl std::fmt::Debug for EmitBroadcast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.callbacks.lock().map_or(0, |g| g.len());
        f.debug_struct("EmitBroadcast")
            .field("callbacks", &n)
            .finish()
    }
}

impl EmitBroadcast {
    /// Construct an empty broadcast with no subscribers yet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            callbacks: Mutex::new(Vec::new()),
        }
    }

    /// Register a closure-form subscriber. The closure is wrapped in
    /// an `Arc` so it can be cloned out under a short lock at publish
    /// time.
    pub fn subscribe_fn<F>(&self, f: F)
    where
        F: Fn(&EmitEvent) + Send + Sync + 'static,
    {
        let mut guard = self.callbacks.lock_recover();
        guard.push(Arc::new(f));
    }

    /// Publish an EMIT event to every subscriber. Called by the
    /// engine's `impl PrimitiveHost for Engine` `emit_event` body —
    /// the EMIT primitive executor invokes `host.emit_event(channel,
    /// payload)`, the engine wrapper forwards to this method.
    ///
    /// A subscriber that panics does not take down the publishing
    /// thread — panics are caught and discarded (mirrors
    /// `ChangeBroadcast::publish`).
    pub fn publish(&self, event: &EmitEvent) {
        let subs = {
            let guard = self.callbacks.lock_recover();
            if guard.is_empty() {
                Vec::new()
            } else {
                guard.clone()
            }
        };
        for cb in subs {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cb(event);
            }));
        }
    }

    /// Subscriber count — used by tests asserting that a registered
    /// callback survives + by the wave-8h audit-fix integration test
    /// to confirm the broadcast is wired (not a stub).
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.callbacks.lock().map_or(0, |g| g.len())
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "tests may use unwrap per workspace policy"
)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn publish_fans_out_to_every_callback() {
        let b = EmitBroadcast::new();
        let count = Arc::new(AtomicUsize::new(0));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);
        b.subscribe_fn(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        b.subscribe_fn(move |_| {
            c2.fetch_add(10, Ordering::SeqCst);
        });
        b.publish(&EmitEvent {
            channel: "test".into(),
            payload: Value::Null,
        });
        assert_eq!(count.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn panicking_callback_does_not_poison_broadcast() {
        let b = EmitBroadcast::new();
        let ok = Arc::new(AtomicUsize::new(0));
        let ok_clone = Arc::clone(&ok);
        b.subscribe_fn(|_| panic!("boom"));
        b.subscribe_fn(move |_| {
            ok_clone.fetch_add(1, Ordering::SeqCst);
        });
        b.publish(&EmitEvent {
            channel: "test".into(),
            payload: Value::Null,
        });
        assert_eq!(ok.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn subscriber_count_tracks_registrations() {
        let b = EmitBroadcast::new();
        assert_eq!(b.subscriber_count(), 0);
        b.subscribe_fn(|_| {});
        assert_eq!(b.subscriber_count(), 1);
    }
}

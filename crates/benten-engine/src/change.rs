//! Change-stream channel concretion (G3-A).
//!
//! Per the implementation plan's R1 architect addendum (§line-605), the
//! push-shaped [`ChangeSubscriber`] trait and the [`ChangeEvent`] schema
//! live in `benten-graph`, which has no async-runtime dependency. The
//! pull-shaped channel concretion lives *here* in `benten-engine::change`.
//!
//! ## Phase 1 shape
//!
//! Phase 1 ships an stdlib-only fan-out: [`ChangeBroadcast`] owns a
//! `Vec<Arc<Callback>>` behind a `Mutex`, implements
//! [`ChangeSubscriber`], and fans every committed event to every
//! registered callback synchronously. No tokio, no broadcast channel — the
//! Phase-1 consumer (hand-written IVM views in G5) does not yet need
//! multi-consumer pull semantics, and dragging tokio in would violate the
//! plan's "no async runtime in the Phase-1 graph/engine waist" constraint.
//!
//! Phase 2 may swap the implementation to `tokio::sync::broadcast` on
//! native (with a sync-Vec fan-out on WASM) without changing the public
//! surface — the [`ChangeBroadcast::subscribe_fn`] /
//! [`ChangeBroadcast::publish`] /
//! [`ChangeBroadcast::subscriber_count`] surface is stable.

use std::sync::{Arc, Mutex};

use benten_graph::{ChangeEvent, ChangeSubscriber, MutexExt};

/// Callback alias used by [`ChangeBroadcast::subscribe_fn`]. The `'static`
/// bound is load-bearing — callbacks outlive the caller of `subscribe_fn`
/// since they live inside the broadcast's Arc list.
pub type ChangeCallback = Arc<dyn Fn(&ChangeEvent) + Send + Sync + 'static>;

/// Handle to the engine's change-event broadcast.
///
/// Wraps a stdlib-only `Vec<ChangeCallback>` behind a `Mutex`. The
/// underlying representation is **not** part of the public contract —
/// callers interact through the inherent methods below. Phase 2 may swap
/// the storage to `tokio::sync::broadcast` without breaking the API.
///
/// # Usage
///
/// `ChangeBroadcast` implements [`ChangeSubscriber`], so it can be
/// registered directly on a `RedbBackend`:
///
/// ```rust,ignore
/// let broadcast = Arc::new(ChangeBroadcast::new());
/// backend.register_subscriber(broadcast.clone())?;
/// broadcast.subscribe_fn(|event| println!("{event:?}"));
/// ```
///
/// G5 (IVM) will consume change events through this broadcast rather than
/// registering directly on the backend; the broadcast is the single point
/// where callbacks attach.
#[derive(Default)]
pub struct ChangeBroadcast {
    callbacks: Mutex<Vec<ChangeCallback>>,
}

impl std::fmt::Debug for ChangeBroadcast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.callbacks.lock().map_or(0, |g| g.len());
        f.debug_struct("ChangeBroadcast")
            .field("callbacks", &n)
            .finish()
    }
}

impl ChangeBroadcast {
    /// Construct an empty broadcast with no subscribers yet.
    #[must_use]
    pub fn new() -> Self {
        Self {
            callbacks: Mutex::new(Vec::new()),
        }
    }

    /// Register an `Arc`-wrapped [`ChangeSubscriber`]. The broadcast keeps
    /// the `Arc` alive and invokes `on_change` once per successful commit.
    pub fn subscribe(&self, subscriber: Arc<dyn ChangeSubscriber>) {
        let cb: ChangeCallback = Arc::new(move |event: &ChangeEvent| {
            subscriber.on_change(event);
        });
        let mut guard = self.callbacks.lock_recover();
        guard.push(cb);
    }

    /// Register a closure-form subscriber. The closure is wrapped in an
    /// `Arc` and stored alongside trait-object subscribers registered via
    /// [`Self::subscribe`].
    pub fn subscribe_fn<F>(&self, f: F)
    where
        F: Fn(&ChangeEvent) + Send + Sync + 'static,
    {
        let mut guard = self.callbacks.lock_recover();
        guard.push(Arc::new(f));
    }

    /// Publish a change event to every subscriber. Called by the G3
    /// transaction primitive immediately after a successful redb commit.
    ///
    /// A subscriber that panics does not take down the publishing thread —
    /// panics are caught and discarded. Phase 2 revisits once the `tracing`
    /// dep lands on this crate (panics will then log their payload).
    pub fn publish(&self, event: &ChangeEvent) {
        // Snapshot the subscriber list under a short lock. Avoid cloning
        // when the list is empty so the thinness path (no IVM) pays a
        // single lock-probe per commit rather than a lock + vec-clone.
        let subs = {
            let guard = self.callbacks.lock_recover();
            if guard.is_empty() {
                Vec::new()
            } else {
                guard.clone()
            }
        };
        // `AssertUnwindSafe` is sound here: `catch_unwind` runs on the
        // current thread, the `&ChangeEvent` borrow is inert across the
        // unwind boundary, and `Arc<Fn>` is `UnwindSafe` by construction.
        // Skipping the per-subscriber `event.clone()` (mini-review
        // g3-cr-13) halves the per-publish allocation cost.
        for cb in subs {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                cb(event);
            }));
        }
    }

    /// Subscriber count — used by thinness tests to assert the broadcast
    /// stays empty when IVM is disabled.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.callbacks.lock().map_or(0, |g| g.len())
    }
}

impl ChangeSubscriber for ChangeBroadcast {
    fn on_change(&self, event: &ChangeEvent) {
        self.publish(event);
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    reason = "tests may use unwrap per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;
    use benten_graph::ChangeKind;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn sample_event() -> ChangeEvent {
        let cid = canonical_test_node().cid().unwrap();
        ChangeEvent {
            cid,
            labels: vec!["Post".into()],
            kind: ChangeKind::Created,
            tx_id: 1,
            actor_cid: None,
            handler_cid: None,
            capability_grant_cid: None,
            node: None,
            edge_endpoints: None,
        }
    }

    #[test]
    fn publish_fans_out_to_every_callback() {
        let b = ChangeBroadcast::new();
        let count = Arc::new(AtomicUsize::new(0));
        let c1 = Arc::clone(&count);
        let c2 = Arc::clone(&count);
        b.subscribe_fn(move |_| {
            c1.fetch_add(1, Ordering::SeqCst);
        });
        b.subscribe_fn(move |_| {
            c2.fetch_add(10, Ordering::SeqCst);
        });
        b.publish(&sample_event());
        assert_eq!(count.load(Ordering::SeqCst), 11);
    }

    #[test]
    fn panicking_callback_does_not_poison_broadcast() {
        let b = ChangeBroadcast::new();
        let ok = Arc::new(AtomicUsize::new(0));
        let ok_clone = Arc::clone(&ok);
        b.subscribe_fn(|_| panic!("boom"));
        b.subscribe_fn(move |_| {
            ok_clone.fetch_add(1, Ordering::SeqCst);
        });
        b.publish(&sample_event());
        assert_eq!(ok.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn subscriber_count_tracks_registrations() {
        let b = ChangeBroadcast::new();
        assert_eq!(b.subscriber_count(), 0);
        b.subscribe_fn(|_| {});
        assert_eq!(b.subscriber_count(), 1);
    }
}

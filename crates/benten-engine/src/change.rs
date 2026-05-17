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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use benten_graph::{ChangeEvent, ChangeSubscriber, MutexExt};

/// Callback alias used by [`ChangeBroadcast::subscribe_fn`]. The `'static`
/// bound is load-bearing — callbacks outlive the caller of `subscribe_fn`
/// since they live inside the broadcast's Arc list.
pub type ChangeCallback = Arc<dyn Fn(&ChangeEvent) + Send + Sync + 'static>;

/// Internal subscriber registry. Split into an *unfiltered* bucket
/// (subscribers registered via [`ChangeBroadcast::subscribe_fn`] /
/// [`ChangeBroadcast::subscribe`] — they see EVERY event, exactly as
/// Phase-1) and a *prefix-indexed* map (subscribers registered via
/// [`ChangeBroadcast::subscribe_fn_with_prefix`] — keyed by a label
/// prefix so [`ChangeBroadcast::publish`] can skip non-matching
/// subscribers without invoking them).
///
/// Fwd-2 #1038: the Phase-1 shape called every subscriber on every event;
/// for the Phase-4-Meta self-composing admin (N panels, each watching one
/// narrow label) + Phase-5+ AI agents (long-lived narrow-pattern
/// subscribers) the fan-out cost grew linearly with subscriber count even
/// when only one subscriber was relevant. The prefix index brings the
/// relevant-subscriber lookup to O(distinct label segments on the event)
/// instead of O(total subscribers). Subscribers that do NOT supply a
/// prefix retain the exact original semantics (they live in the
/// unfiltered bucket and receive every event).
#[derive(Default)]
struct Registry {
    /// Subscribers that receive every event (no prefix hint supplied).
    /// Preserves the exact Phase-1 fan-out semantics for callers that do
    /// not opt into prefix filtering.
    unfiltered: Vec<ChangeCallback>,
    /// Prefix-keyed subscribers. `prefixed[p]` holds subscribers that only
    /// want events whose primary label starts with `p`. `publish`
    /// consults this map keyed by the event's label prefix segment(s).
    prefixed: HashMap<String, Vec<ChangeCallback>>,
}

impl Registry {
    fn len(&self) -> usize {
        self.unfiltered.len() + self.prefixed.values().map(Vec::len).sum::<usize>()
    }

    fn is_empty(&self) -> bool {
        self.unfiltered.is_empty() && self.prefixed.is_empty()
    }
}

/// The label-prefix segments an event maps to for prefix-index lookup.
///
/// A prefix-subscriber registered with key `"post"` should receive an
/// event whose label is `"post"` AND an event whose label is
/// `"post:comment"` (colon-delimited label hierarchy, mirroring the
/// `engine.onChange("post:*", ...)` glob shape). We therefore probe the
/// index with every colon-delimited prefix of every label on the event,
/// plus each full label, deduplicated. For the common single-label event
/// with a flat label this is 1-2 probes.
fn event_prefix_keys(event: &ChangeEvent) -> Vec<String> {
    let mut keys: Vec<String> = Vec::new();
    for label in &event.labels {
        // Full label is always a candidate key.
        if !keys.contains(label) {
            keys.push(label.clone());
        }
        // Each colon-delimited prefix (e.g. `"a:b:c"` → `"a"`, `"a:b"`).
        let mut acc = String::new();
        for (i, seg) in label.split(':').enumerate() {
            if i > 0 {
                acc.push(':');
            }
            acc.push_str(seg);
            if acc != *label && !keys.contains(&acc) {
                keys.push(acc.clone());
            }
        }
    }
    keys
}

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
    callbacks: Mutex<Registry>,
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
            callbacks: Mutex::new(Registry::default()),
        }
    }

    /// Register an `Arc`-wrapped [`ChangeSubscriber`]. The broadcast keeps
    /// the `Arc` alive and invokes `on_change` once per successful commit.
    ///
    /// Goes into the *unfiltered* bucket — this subscriber receives EVERY
    /// event, exactly as Phase-1. (The eval-side SUBSCRIBE bridge uses
    /// this path because it multiplexes many narrow patterns through one
    /// broadcast subscriber + does its own pattern match downstream.)
    pub fn subscribe(&self, subscriber: Arc<dyn ChangeSubscriber>) {
        let cb: ChangeCallback = Arc::new(move |event: &ChangeEvent| {
            subscriber.on_change(event);
        });
        let mut guard = self.callbacks.lock_recover();
        guard.unfiltered.push(cb);
    }

    /// Register a closure-form subscriber. The closure is wrapped in an
    /// `Arc` and stored in the *unfiltered* bucket alongside trait-object
    /// subscribers registered via [`Self::subscribe`] — it receives every
    /// event (exact Phase-1 semantics).
    pub fn subscribe_fn<F>(&self, f: F)
    where
        F: Fn(&ChangeEvent) + Send + Sync + 'static,
    {
        let mut guard = self.callbacks.lock_recover();
        guard.unfiltered.push(Arc::new(f));
    }

    /// Register a closure-form subscriber that is only invoked for events
    /// whose label (or a colon-delimited prefix of it) matches `prefix`.
    ///
    /// Fwd-2 #1038: a subscriber watching one narrow label (e.g. an
    /// admin-UI panel editing `"Workflow:checkout"`, or an AI agent
    /// watching `"Calendar:Event"`) supplies its label prefix here so
    /// [`Self::publish`] skips it entirely on unrelated events instead of
    /// invoking it + relying on it to self-filter. Semantically identical
    /// to `subscribe_fn` + an in-callback `event.has_label(..)` guard, but
    /// the skip happens before the (potentially many) callbacks are
    /// invoked, so per-publish cost scales with *relevant* subscribers
    /// rather than *total* subscribers.
    ///
    /// `prefix` is matched against each label on the event and against
    /// every colon-delimited prefix of each label, mirroring the
    /// `engine.onChange("post:*", ...)` glob hierarchy: a subscriber on
    /// `"post"` sees both a `"post"` event and a `"post:comment"` event.
    pub fn subscribe_fn_with_prefix<F>(&self, prefix: impl Into<String>, f: F)
    where
        F: Fn(&ChangeEvent) + Send + Sync + 'static,
    {
        let mut guard = self.callbacks.lock_recover();
        guard
            .prefixed
            .entry(prefix.into())
            .or_default()
            .push(Arc::new(f));
    }

    /// Publish a change event to every *relevant* subscriber. Called by
    /// the G3 transaction primitive immediately after a successful redb
    /// commit.
    ///
    /// Every unfiltered subscriber is invoked (Phase-1 semantics
    /// preserved). Prefix-indexed subscribers are invoked only when one of
    /// the event's label-prefix keys matches their registered prefix —
    /// non-matching prefix subscribers are skipped without invocation
    /// (Fwd-2 #1038 prefilter). The set of subscribers that *observe* a
    /// given event is unchanged versus the all-fan-out shape: a
    /// prefix-subscriber that matched would have self-filtered to the same
    /// accept/reject decision in its own callback body.
    ///
    /// A subscriber that panics does not take down the publishing thread —
    /// panics are caught and discarded. Phase 2 revisits once the `tracing`
    /// dep lands on this crate (panics will then log their payload).
    pub fn publish(&self, event: &ChangeEvent) {
        // Snapshot only the *relevant* subscribers under a short lock.
        // Avoid any allocation when the registry is empty so the thinness
        // path (no IVM) pays a single lock-probe per commit.
        let subs: Vec<ChangeCallback> = {
            let guard = self.callbacks.lock_recover();
            if guard.is_empty() {
                Vec::new()
            } else {
                let mut out: Vec<ChangeCallback> = Vec::with_capacity(guard.unfiltered.len());
                out.extend(guard.unfiltered.iter().cloned());
                if !guard.prefixed.is_empty() {
                    // O(distinct label-prefix keys on the event) hash
                    // probes instead of O(total prefix subscribers).
                    for key in event_prefix_keys(event) {
                        if let Some(bucket) = guard.prefixed.get(&key) {
                            out.extend(bucket.iter().cloned());
                        }
                    }
                }
                out
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
    /// stays empty when IVM is disabled. Counts both unfiltered and
    /// prefix-indexed subscribers.
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
        b.subscribe_fn_with_prefix("Post", |_| {});
        assert_eq!(b.subscriber_count(), 2);
    }

    fn event_with_label(label: &str) -> ChangeEvent {
        let mut e = sample_event();
        e.labels = vec![label.into()];
        e
    }

    /// Fwd-2 #1038: a prefix subscriber whose prefix matches the event's
    /// label IS invoked — same observable outcome as the all-fan-out
    /// shape + an in-callback `has_label` guard.
    #[test]
    fn prefix_subscriber_invoked_on_matching_label() {
        let b = ChangeBroadcast::new();
        let hit = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hit);
        b.subscribe_fn_with_prefix("Post", move |_| {
            h.fetch_add(1, Ordering::SeqCst);
        });
        b.publish(&event_with_label("Post"));
        assert_eq!(hit.load(Ordering::SeqCst), 1);
    }

    /// Fwd-2 #1038 core: a prefix subscriber whose prefix does NOT match
    /// is skipped WITHOUT its callback being invoked. The all-fan-out
    /// shape would have invoked it (it would have self-filtered to a
    /// no-op); the prefilter reaches the same observable state without
    /// the invocation.
    #[test]
    fn non_matching_prefix_subscriber_is_skipped() {
        let b = ChangeBroadcast::new();
        let invoked = Arc::new(AtomicUsize::new(0));
        let i = Arc::clone(&invoked);
        b.subscribe_fn_with_prefix("Comment", move |_| {
            i.fetch_add(1, Ordering::SeqCst);
        });
        b.publish(&event_with_label("Post"));
        assert_eq!(
            invoked.load(Ordering::SeqCst),
            0,
            "non-matching prefix subscriber must not be invoked"
        );
    }

    /// Colon-delimited hierarchy: a subscriber on `"post"` sees both a
    /// `"post"` event and a `"post:comment"` event (mirrors the
    /// `engine.onChange("post:*", ...)` glob shape).
    #[test]
    fn prefix_subscriber_matches_colon_hierarchy() {
        let b = ChangeBroadcast::new();
        let hit = Arc::new(AtomicUsize::new(0));
        let h = Arc::clone(&hit);
        b.subscribe_fn_with_prefix("post", move |_| {
            h.fetch_add(1, Ordering::SeqCst);
        });
        b.publish(&event_with_label("post"));
        b.publish(&event_with_label("post:comment"));
        b.publish(&event_with_label("postal")); // NOT a colon-prefix
        assert_eq!(
            hit.load(Ordering::SeqCst),
            2,
            "`post` matches `post` + `post:comment` but not `postal`"
        );
    }

    /// Semantic-transparency invariant: unfiltered subscribers still see
    /// EVERY event regardless of the prefix index — the prefilter only
    /// gates prefix-registered subscribers.
    #[test]
    fn unfiltered_subscribers_unaffected_by_prefix_index() {
        let b = ChangeBroadcast::new();
        let all = Arc::new(AtomicUsize::new(0));
        let a = Arc::clone(&all);
        b.subscribe_fn(move |_| {
            a.fetch_add(1, Ordering::SeqCst);
        });
        b.subscribe_fn_with_prefix("Never", |_| unreachable!());
        b.publish(&event_with_label("Post"));
        b.publish(&event_with_label("Comment"));
        assert_eq!(all.load(Ordering::SeqCst), 2);
    }
}

//! Reload coordinator — tracks in-flight evaluations so tests can observe
//! the reload-vs-call ordering contract.
//!
//! ## Contract
//!
//! A new evaluation acquires a `CallGuard` for its lifetime via
//! [`ReloadCoordinator::begin_call`]. The guard increments an in-flight
//! counter; dropping it (RAII, including on panic) decrements it.
//!
//! ## Phase-2a semantics
//!
//! Phase-2a's reload coordinator is intentionally *non-blocking* on
//! in-flight calls: a reload races a call rather than queueing behind
//! it. The actual ordering guarantee that makes in-flight calls observe
//! the *pre-reload* `HandlerVersion` comes from TWO mechanisms in
//! `DevServer::register_handler_from_str`:
//!
//! 1. `RwLock::write()` on `DevServer.handlers` serializes concurrent
//!    reloads against each other (the version-counter bump + new entry
//!    insert both happen under the write lock, so two reloads cannot
//!    interleave their bumps).
//! 2. Each call captures an `Arc<HandlerVersion>` snapshot via
//!    `snapshot_version()` before doing work; dropping the read-lock
//!    after the snapshot means a concurrent reload's write-lock can
//!    proceed, but the in-flight call continues executing against its
//!    snapshot (the `Arc` keeps the old `HandlerVersion` live).
//!
//! This coordinator doesn't contribute to ordering — the `RwLock` and
//! the `Arc` snapshot do. What this coordinator DOES contribute: the
//! in-flight counter so tests can pin "reload occurred while N calls
//! were in flight" + the slow-transform gate used by the in-flight
//! harness. Phase-2b can introduce a drain-on-reload lease here if the
//! WAIT-suspension story needs tighter ordering.
//!
//! ## Panic safety
//!
//! `CallGuard` uses Drop, so a panicking in-flight call still releases
//! its in-flight counter. The `slow_transform_wait` path uses a
//! `Condvar` rather than a `std::sync::Barrier` because a panicking
//! waiter on a Barrier would poison the harness.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use benten_core::Cid;

/// One reload-event observation. Phase 2b Wave-8f: the dev-server publishes
/// these to subscribers so JS-side renderers (vitest harness, IDE plugins)
/// can pin "a hot-reload occurred + here's what changed". Carries the
/// engine-side handler_id + the new+predecessor CIDs (when the underlying
/// engine reported them via [`benten_engine::RegisterReplaceOutcome`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReloadEvent {
    /// Handler id whose body was replaced.
    pub handler_id: String,
    /// Op the dev-server registered the source under (`"run"`, `"create"`,
    /// …). Phase-2a-era detail; preserved so JS-side consumers don't need
    /// to track it separately.
    pub op: String,
    /// Surrogate version tag the dev-server stamped on this version
    /// (`"v1"` / `"v2"` / …) — matches both the legacy `HandlerVersion`
    /// counter AND `RegisterReplaceOutcome::version_tag()` when the engine
    /// path is wired.
    pub version_tag: String,
    /// New live CID for the handler. `None` when the dev-server is in
    /// legacy in-memory mode (no engine wired).
    pub new_cid: Option<Cid>,
    /// Predecessor CID — present when this reload was a real swap (not a
    /// first-registration). `None` on first-registration AND on idempotent
    /// re-registration with identical content AND in legacy in-memory mode.
    pub previous_cid: Option<Cid>,
}

/// Subscriber handle returned by [`ReloadCoordinator::subscribe_reload_events`].
/// Drop to unsubscribe. The subscriber is intentionally NOT `Clone` —
/// the inner buffer is shared via `Arc<Mutex<Vec<ReloadEvent>>>`, so a
/// clone would share the SAME buffer (drain on one drains for both),
/// which is the opposite of what "independent observation" means at
/// this surface. Callers that want independent observation should call
/// [`ReloadCoordinator::subscribe_reload_events`] again to mint a fresh
/// subscriber backed by its own buffer. Removing `Clone` forces the
/// correct API usage at compile time.
#[derive(Debug)]
pub struct ReloadSubscriber {
    inner: Arc<Mutex<Vec<ReloadEvent>>>,
}

impl ReloadSubscriber {
    /// Drain pending events. Returns the events delivered since the last
    /// drain in arrival order. Phase-2b Wave-8f: uses a buffered channel
    /// idiom rather than `crossbeam-channel` to avoid pulling a new
    /// transitive dep into `benten-dev`'s narrow build (`std::sync::mpsc`
    /// would also work but `Vec<ReloadEvent>` keeps the surface
    /// trivially cloneable for test fixtures + napi bridging).
    #[must_use]
    pub fn drain(&self) -> Vec<ReloadEvent> {
        let mut g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        std::mem::take(&mut *g)
    }

    /// Whether the buffer currently has at least one event. Cheap snapshot
    /// for tests that want to spin-wait without draining.
    #[must_use]
    pub fn has_events(&self) -> bool {
        let g = self.inner.lock().unwrap_or_else(|p| p.into_inner());
        !g.is_empty()
    }
}

/// Coordinates hot-reloads against in-flight evaluations.
#[derive(Debug)]
pub struct ReloadCoordinator {
    /// Count of currently-executing calls. Used by [`ReloadCoordinator::in_flight_count`]
    /// for testing visibility.
    in_flight: AtomicUsize,
    /// Slow-transform fixture — the call thread waits on this condvar; the
    /// test releases it via [`ReloadCoordinator::slow_transform_release`].
    slow_transform: SlowTransformGate,
    /// Active reload-event subscribers. Each subscriber is an
    /// `Arc<Mutex<Vec<ReloadEvent>>>` buffer the publisher pushes into.
    /// Phase 2b Wave-8f. Held under a Mutex so subscribe + publish race
    /// cleanly; the per-subscriber buffer Mutex is independent so a slow
    /// drainer doesn't block the publisher.
    subscribers: Mutex<Vec<Arc<Mutex<Vec<ReloadEvent>>>>>,
}

#[derive(Debug)]
struct SlowTransformGate {
    released: AtomicBool,
    mtx: Mutex<()>,
    cv: Condvar,
}

impl SlowTransformGate {
    const fn new() -> Self {
        Self {
            released: AtomicBool::new(false),
            mtx: Mutex::new(()),
            cv: Condvar::new(),
        }
    }
}

impl ReloadCoordinator {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            in_flight: AtomicUsize::new(0),
            slow_transform: SlowTransformGate::new(),
            subscribers: Mutex::new(Vec::new()),
        }
    }

    /// Subscribe to hot-reload events. Returns a [`ReloadSubscriber`] whose
    /// `drain()` reports events seen since the last drain. The subscriber
    /// remains active for the lifetime of the returned handle (drop → no
    /// further events; the publisher prunes dropped subscribers lazily on
    /// publish).
    pub fn subscribe_reload_events(&self) -> ReloadSubscriber {
        let buf = Arc::new(Mutex::new(Vec::new()));
        let mut g = self.subscribers.lock().unwrap_or_else(|p| p.into_inner());
        g.push(Arc::clone(&buf));
        ReloadSubscriber { inner: buf }
    }

    /// Publish a reload event to every active subscriber. Phase 2b Wave-8f.
    /// Called by `DevServer` on every successful (idempotent or replace)
    /// `register_handler_from_*` invocation.
    pub fn publish_reload_event(&self, event: ReloadEvent) {
        let mut g = self.subscribers.lock().unwrap_or_else(|p| p.into_inner());
        // Lazily prune subscribers whose only owner is this Vec — no live
        // external Arc means the subscriber was dropped + we should stop
        // buffering for it. `Arc::strong_count` of 1 means "this is the
        // only Arc" (the one we own here); the publisher AND the
        // ReloadSubscriber each hold one when alive (count == 2).
        g.retain(|buf| {
            // Active subscriber → push the event.
            if Arc::strong_count(buf) >= 2 {
                let mut b = buf.lock().unwrap_or_else(|p| p.into_inner());
                b.push(event.clone());
                true
            } else {
                false
            }
        });
    }

    /// Begin a call. Returns a guard that decrements the in-flight count
    /// on drop (including on panic).
    #[must_use]
    pub fn begin_call(self: &Arc<Self>) -> CallGuard {
        self.in_flight.fetch_add(1, Ordering::AcqRel);
        CallGuard {
            coord: Arc::clone(self),
        }
    }

    /// Number of calls currently in flight. Test-only visibility.
    #[must_use]
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.load(Ordering::Acquire)
    }

    /// Wait at the slow-transform gate. The thread parks on a condvar
    /// until [`ReloadCoordinator::slow_transform_release`] is called.
    /// If the gate has already been released, returns immediately.
    pub fn slow_transform_wait(&self) {
        if self.slow_transform.released.load(Ordering::Acquire) {
            return;
        }
        let lock = match self.slow_transform.mtx.lock() {
            Ok(g) => g,
            Err(poisoned) => poisoned.into_inner(),
        };
        // N1 mini-review from Wave-2b suggested `let _ = …` style, but
        // the wait_while() return is a MutexGuard and `let _ =` trips
        // the `let_underscore_lock` lint (the guard would drop
        // immediately instead of being bound to the enclosing scope's
        // cleanup). The named binding keeps the guard alive through
        // scope exit and is the idiomatic form for condvar waits.
        let _unused = self
            .slow_transform
            .cv
            .wait_while(lock, |()| {
                !self.slow_transform.released.load(Ordering::Acquire)
            })
            .unwrap_or_else(|p| p.into_inner());
    }

    /// Release the slow-transform gate. Wakes any thread parked in
    /// [`ReloadCoordinator::slow_transform_wait`].
    pub fn slow_transform_release(&self) {
        self.slow_transform.released.store(true, Ordering::Release);
        self.slow_transform.cv.notify_all();
    }
}

impl Default for ReloadCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for an in-flight call.
#[derive(Debug)]
pub struct CallGuard {
    coord: Arc<ReloadCoordinator>,
}

impl Drop for CallGuard {
    fn drop(&mut self) {
        self.coord.in_flight.fetch_sub(1, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn in_flight_counter_drops_on_panic() {
        let coord = Arc::new(ReloadCoordinator::new());
        let c2 = Arc::clone(&coord);
        let h = thread::spawn(move || {
            let _g = c2.begin_call();
            assert_eq!(c2.in_flight_count(), 1);
            panic!("intentional");
        });
        let _ = h.join();
        // Guard dropped on panic-unwind; counter decremented.
        assert_eq!(coord.in_flight_count(), 0);
    }

    #[test]
    fn slow_transform_release_wakes_waiter() {
        let coord = Arc::new(ReloadCoordinator::new());
        let c2 = Arc::clone(&coord);
        let h = thread::spawn(move || {
            c2.slow_transform_wait();
        });
        thread::sleep(Duration::from_millis(10));
        coord.slow_transform_release();
        h.join().expect("must wake");
    }

    #[test]
    fn reload_subscriber_receives_published_events() {
        let coord = Arc::new(ReloadCoordinator::new());
        let sub = coord.subscribe_reload_events();
        assert!(!sub.has_events());

        let ev = ReloadEvent {
            handler_id: "h1".into(),
            op: "run".into(),
            version_tag: "v2".into(),
            new_cid: Some(Cid::from_blake3_digest([0x42; 32])),
            previous_cid: Some(Cid::from_blake3_digest([0x41; 32])),
        };
        coord.publish_reload_event(ev.clone());

        assert!(sub.has_events());
        let drained = sub.drain();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0], ev);
        assert!(!sub.has_events(), "drain must consume");
    }

    #[test]
    fn reload_publisher_prunes_dropped_subscribers() {
        let coord = Arc::new(ReloadCoordinator::new());
        let sub_alive = coord.subscribe_reload_events();
        {
            let _sub_dropped = coord.subscribe_reload_events();
            // _sub_dropped goes out of scope here.
        }
        let ev = ReloadEvent {
            handler_id: "h".into(),
            op: "run".into(),
            version_tag: "v1".into(),
            new_cid: None,
            previous_cid: None,
        };
        coord.publish_reload_event(ev.clone());
        assert_eq!(sub_alive.drain().len(), 1);
        // Internal state inspection: the publisher's retain pruned the
        // dropped subscriber. Publish a second event; only the live
        // subscriber buffers it; the dropped buffer is gone.
        coord.publish_reload_event(ev);
        // (Indirect assertion: prior code paniced on poisoned dropped
        // mutex when retain didn't prune. If we got here, retain ran.)
        assert_eq!(sub_alive.drain().len(), 1);
    }
}

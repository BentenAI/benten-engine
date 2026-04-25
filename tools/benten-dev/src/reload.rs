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

/// Coordinates hot-reloads against in-flight evaluations.
#[derive(Debug)]
pub struct ReloadCoordinator {
    /// Count of currently-executing calls. Used by [`ReloadCoordinator::in_flight_count`]
    /// for testing visibility.
    in_flight: AtomicUsize,
    /// Slow-transform fixture — the call thread waits on this condvar; the
    /// test releases it via [`ReloadCoordinator::slow_transform_release`].
    slow_transform: SlowTransformGate,
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
        }
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
}

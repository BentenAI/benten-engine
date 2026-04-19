//! IVM change-stream subscriber.
//!
//! **G5-A deliverable (Phase 1).**
//!
//! [`Subscriber`] implements [`benten_graph::ChangeSubscriber`] and fans
//! committed change events out to every registered [`View`]. Each view sees
//! every event and filters internally — simple fan-out, acceptable for
//! Phase 1's 5 hand-written views; TODO(phase-2) flag a pattern-based
//! pre-filtering router once the view count grows.
//!
//! Graceful degradation: a view whose `update` returns
//! [`ViewError::BudgetExceeded`] is marked stale via [`View::mark_stale`]
//! but does **not** abort the fan-out — remaining views still receive the
//! event. An `Internal`-style failure is logged (via `eprintln!` for
//! `std`-hosted builds, silently swallowed otherwise) and also does not
//! abort the fan-out, so one broken view cannot take down a whole engine.
//!
//! # Registration model
//!
//! The engine (G7) owns the `Subscriber` and calls
//! [`benten_graph::RedbBackend::register_subscriber`] with an
//! `Arc<dyn ChangeSubscriber>` wrapping this type. Views are registered
//! at `Engine::create_view` time via [`Subscriber::with_view`].

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use benten_graph::{ChangeEvent, ChangeSubscriber};

use crate::view::{View, ViewError, ViewQuery, ViewResult};

extern crate alloc;

// ---------------------------------------------------------------------------
// Subscriber
// ---------------------------------------------------------------------------

/// Fan-out subscriber that routes every [`ChangeEvent`] to every registered
/// [`View`]. Phase 1 contract: each view sees every event and filters
/// internally based on its own pattern-match logic.
///
/// Mutation model: views are registered at construction or via
/// [`Self::with_view`]; [`Self::route_change_event`] takes `&mut self`
/// because the underlying views mutate their state on each update.
/// The [`ChangeSubscriber`] impl, which takes `&self`, wraps the views in a
/// [`std::sync::Mutex`] so the engine can share the subscriber across the
/// commit thread and the IVM worker without any caller-side synchronization.
pub struct Subscriber {
    /// Registered views. Heterogeneous — each view is a different concrete
    /// type under a `Box<dyn View>`. Held behind a `Mutex` so the
    /// `&self`-taking [`ChangeSubscriber::on_change`] can still mutate them.
    views: std::sync::Mutex<Vec<Box<dyn View>>>,
}

impl Subscriber {
    /// Construct a subscriber with no views.
    #[must_use]
    pub fn new() -> Self {
        Self {
            views: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Register a view, consuming and returning `self` so callers can chain
    /// `.with_view(...)` in constructor style.
    #[must_use]
    pub fn with_view(self, view: Box<dyn View>) -> Self {
        self.register_view(view);
        self
    }

    /// Register a view on an existing subscriber. Thread-safe.
    pub fn register_view(&self, view: Box<dyn View>) {
        let mut guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.push(view);
    }

    /// Number of registered views. Useful for introspection and for
    /// assertions in tests.
    #[must_use]
    pub fn view_count(&self) -> usize {
        let guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.len()
    }

    /// Stable-id list of the registered views, in registration order. Useful
    /// for debug tooling and for the engine's view-registry surface.
    #[must_use]
    pub fn view_ids(&self) -> Vec<String> {
        let guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.iter().map(|v| String::from(v.id())).collect()
    }

    /// Is the named view currently stale? Returns `None` if the view is not
    /// registered. Used by the engine-level `read_view_with` to decide
    /// strict vs. relaxed semantics without exposing the view's internal
    /// state machine.
    #[must_use]
    pub fn view_is_stale(&self, view_id: &str) -> Option<bool> {
        let guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .iter()
            .find(|v| v.id() == view_id)
            .map(|v| v.is_stale())
    }

    /// Read a named view under `query`. Returns `Ok(None)` when no view with
    /// that id is registered — distinct from `Ok(Some(Err(...)))` which
    /// surfaces a stale / pattern-mismatch error from the view itself.
    ///
    /// The engine consumes this via `read_view_*` to back the Phase-1
    /// `post:list` route through View 3 (content_listing) — falling back
    /// to the backend label index only when the view is absent.
    ///
    /// # Errors
    ///
    /// Per-view errors (`Stale`, `PatternMismatch`) propagate through the
    /// returned `Result`. The outer `Option` distinguishes "view not
    /// registered" from "view erred".
    pub fn read_view(
        &self,
        view_id: &str,
        query: &ViewQuery,
    ) -> Option<Result<ViewResult, ViewError>> {
        let guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .iter()
            .find(|v| v.id() == view_id)
            .map(|v| v.read(query))
    }

    /// Route a single change event to every registered view.
    ///
    /// Returns the number of views that accepted the event successfully
    /// (i.e. their `update` returned `Ok(())`). Views that trip their
    /// budget are marked stale via [`View::mark_stale`] and counted as
    /// *not* updated; they do not abort the fan-out.
    ///
    /// # Errors
    ///
    /// This method itself does not fail. It always returns `Ok(count)` where
    /// `count` is the number of views that applied the event cleanly.
    /// Per-view failures are absorbed into the stale-transition path and
    /// do not propagate, so a single misbehaving view cannot take down the
    /// whole subscriber.
    ///
    /// # Panics
    ///
    /// Never; a poisoned view-list mutex is recovered via
    /// [`std::sync::PoisonError::into_inner`].
    pub fn route_change_event(&mut self, event: &ChangeEvent) -> Result<usize, ViewError> {
        let mut guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let mut applied = 0usize;
        for view in guard.iter_mut() {
            apply_event(view.as_mut(), event, &mut applied);
        }
        Ok(applied)
    }
}

/// Extract the per-view dispatch into a free function so
/// [`ChangeSubscriber::on_change`] (which has `&self`, not `&mut self`) can
/// call the exact same path as [`Subscriber::route_change_event`].
///
/// Wraps each view's `update` call in [`std::panic::catch_unwind`] so a
/// panicking view marks itself stale and logs, but does not take down the
/// commit thread or poison the fan-out for other views (mini-review
/// g5-ivm-6). `View` is not `UnwindSafe` by default because `&mut self`
/// gives update access to interior state, but a panicked view is already
/// ill — we mark it stale and continue, which is sound.
fn apply_event(view: &mut dyn View, event: &ChangeEvent, applied: &mut usize) {
    // A stale view stays stale; feeding it more events won't unstick it
    // until an explicit rebuild. Skip for cheapness.
    if view.is_stale() {
        return;
    }
    let view_id: String = String::from(view.id());
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| view.update(event)));
    match result {
        Ok(Ok(())) => {
            *applied += 1;
        }
        Ok(Err(ViewError::BudgetExceeded(_))) => {
            view.mark_stale();
        }
        Ok(Err(ViewError::Stale { .. })) => {
            // Idempotent: the view flipped stale between our is_stale check
            // and the update call. Nothing to do.
        }
        Ok(Err(other)) => {
            // PatternMismatch and similar are not fatal — log and keep
            // fanning out. TODO(phase-2): route to a telemetry channel
            // instead of stderr.
            log_view_error(&view_id, &other);
        }
        Err(_panic_payload) => {
            log_view_panic(&view_id);
            view.mark_stale();
        }
    }
}

/// Log a non-fatal view error. Kept in one place so a follow-up can swap
/// the sink (tracing, metrics, structured log) without touching the hot
/// path.
///
/// TODO(phase-2): route to `tracing` once the engine wires a subscriber.
/// For now we deliberately discard — PatternMismatch is the expected
/// "this view doesn't handle this query shape" signal, not an alert.
#[allow(
    clippy::print_stderr,
    reason = "stderr sink is the Phase 1 placeholder; Phase 2 wires tracing"
)]
fn log_view_error(view_id: &str, err: &dyn fmt::Display) {
    eprintln!("benten-ivm: view {view_id} returned non-fatal error: {err}");
}

/// Log a view panic. Paired with `apply_event`'s `catch_unwind` so a
/// panicking view marks stale and the fan-out continues.
#[allow(
    clippy::print_stderr,
    reason = "stderr sink is the Phase 1 placeholder; Phase 2 wires tracing"
)]
fn log_view_panic(view_id: &str) {
    eprintln!("benten-ivm: view {view_id} panicked during update; marking stale");
}

impl Default for Subscriber {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Subscriber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Subscriber")
            .field("view_count", &self.view_count())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ChangeSubscriber impl — the engine's cross-thread hook
// ---------------------------------------------------------------------------

impl ChangeSubscriber for Subscriber {
    fn on_change(&self, event: &ChangeEvent) {
        let mut guard = self
            .views
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut applied = 0usize;
        for view in guard.iter_mut() {
            apply_event(view.as_mut(), event, &mut applied);
        }
    }
}

// ---------------------------------------------------------------------------
// Alias: ChangeStreamSubscriber
// ---------------------------------------------------------------------------

/// Canonical G5-A name for the subscriber. Alias for [`Subscriber`] so the
/// "new module, new name" shape requested in the G5-A brief coexists with
/// the `Subscriber` name the R3 test suite already uses.
pub type ChangeStreamSubscriber = Subscriber;

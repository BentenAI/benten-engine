//! Per-view work-budget tracker.
//!
//! Every Phase-1 IVM view replicates the same `remaining/original/stale`
//! state machine for the "budget-exceeded → trip stale, rebuild restores the
//! original cap" contract (mini-review g5-cr-3). [`BudgetTracker`] extracts
//! that state into a single owned helper so the five view implementations
//! share one definition.
//!
//! ## Contract
//!
//! - `new(max)` records `max` as both the current and the original cap.
//!   `max == u64::MAX` models an unbounded budget — the common case for
//!   production views — and the per-update cost is still decremented so
//!   a pathological accumulator eventually saturates at zero and trips.
//! - `try_consume(cost)` decrements the remaining budget by `cost` (saturating
//!   at zero) and returns `Err(ViewError::BudgetExceeded)` when the budget
//!   was already zero at entry. The caller is expected to flip its view
//!   state to `Stale` on the error.
//! - `rebuild` restores the original cap and clears the stale flag. Matches
//!   the uniform-budget-on-rebuild contract (g5-cr-3).
//! - `mark_stale` / `is_stale` are idempotent flag getters/setters so view
//!   implementations can share the stale predicate.
//!
//! The tracker is deliberately `Copy` and `no_std`-friendly — views hold it
//! by value and every method is cheap.

use alloc::string::{String, ToString};

use crate::ViewError;

/// Per-view runtime budget + stale flag bundle.
///
/// See module docs for the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetTracker {
    remaining: u64,
    original: u64,
    stale: bool,
}

impl BudgetTracker {
    /// Construct with `max` as both the current and original cap.
    ///
    /// `max == u64::MAX` is the conventional "unlimited" sentinel used by
    /// the zero-argument view constructors. `max == 0` produces a tracker
    /// that trips on the very next `try_consume` — callers that want to
    /// reject that configuration up-front can do so at view-construction
    /// time.
    #[must_use]
    pub const fn new(max: u64) -> Self {
        Self {
            remaining: max,
            original: max,
            stale: false,
        }
    }

    /// Remaining budget. Primarily useful for diagnostics.
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Originally-configured cap (restored on `rebuild`).
    #[must_use]
    pub const fn original(&self) -> u64 {
        self.original
    }

    /// True once the view has flipped stale (either because `try_consume`
    /// surfaced `BudgetExceeded` or because the view explicitly called
    /// `mark_stale`).
    #[must_use]
    pub const fn is_stale(&self) -> bool {
        self.stale
    }

    /// Attempt to consume `cost` work units.
    ///
    /// Returns `Ok(())` on success; on failure the view is flipped stale
    /// and `ViewError::BudgetExceeded(view_id)` is returned. Subsequent
    /// calls against the stale tracker also return `BudgetExceeded` (no
    /// cost is charged once stale).
    ///
    /// Budget arithmetic is saturating — a `cost` larger than `remaining`
    /// leaves `remaining == 0` without panicking, and the next call trips.
    ///
    /// # Errors
    ///
    /// Returns [`ViewError::BudgetExceeded`] when the tracker is already
    /// stale or when `remaining` was zero at entry.
    pub fn try_consume(&mut self, cost: u64, view_id: &str) -> Result<(), ViewError> {
        if self.stale {
            return Err(ViewError::BudgetExceeded(view_id.to_string()));
        }
        if self.remaining == 0 {
            self.stale = true;
            return Err(ViewError::BudgetExceeded(view_id.to_string()));
        }
        self.remaining = self.remaining.saturating_sub(cost);
        Ok(())
    }

    /// Restore the tracker to the original cap and clear the stale flag.
    pub fn rebuild(&mut self) {
        self.remaining = self.original;
        self.stale = false;
    }

    /// Mark the view stale without charging against the budget. Used by
    /// views that detect staleness via an out-of-band signal (e.g. an
    /// explicit `View::mark_stale` call from the subscriber).
    pub fn mark_stale(&mut self) {
        self.stale = true;
    }

    /// Borrow the tracker's stale-id shape as a `ViewError::Stale` error.
    ///
    /// Convenience for the 5-view boilerplate where every read path leads
    /// with `if self.is_stale() { return Err(ViewError::Stale { view_id }) }`.
    ///
    /// # Errors
    ///
    /// Always returns `Err(ViewError::Stale { view_id })` — the method is a
    /// helper for call sites that already decided to refuse.
    pub fn stale_error(view_id: &str) -> ViewError {
        ViewError::Stale {
            view_id: String::from(view_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_consume_decrements_and_succeeds() {
        let mut b = BudgetTracker::new(3);
        assert!(b.try_consume(1, "t").is_ok());
        assert_eq!(b.remaining(), 2);
        assert!(!b.is_stale());
    }

    #[test]
    fn try_consume_trips_on_zero_remaining() {
        let mut b = BudgetTracker::new(1);
        assert!(b.try_consume(1, "t").is_ok());
        assert_eq!(b.remaining(), 0);
        let err = b.try_consume(1, "t").expect_err("should trip");
        assert!(matches!(err, ViewError::BudgetExceeded(_)));
        assert!(b.is_stale());
    }

    #[test]
    fn rebuild_restores_original_cap_and_clears_stale() {
        let mut b = BudgetTracker::new(2);
        let _ = b.try_consume(1, "t");
        let _ = b.try_consume(1, "t");
        let _ = b.try_consume(1, "t"); // trips
        assert!(b.is_stale());
        b.rebuild();
        assert!(!b.is_stale());
        assert_eq!(b.remaining(), 2);
    }

    #[test]
    fn mark_stale_does_not_affect_remaining() {
        let mut b = BudgetTracker::new(5);
        b.mark_stale();
        assert!(b.is_stale());
        assert_eq!(b.remaining(), 5);
    }
}

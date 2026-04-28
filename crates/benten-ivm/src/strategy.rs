//! IVM strategy enum + per-view strategy selection (G8-A).
//!
//! ## D8-RESOLVED EXPLICIT-OPT-IN
//!
//! The IVM crate ships three strategies for view maintenance, exposed via a
//! single closed enum so callers can match exhaustively. Strategy choice is
//! **explicit per view at construction time**: there is no auto-select, no
//! runtime adaptation, and no API to mutate a view's strategy after the
//! constructor runs.
//!
//! - [`Strategy::A`] — hand-written incremental maintenance. The 5 Phase-1
//!   views (capability grants, event dispatch, content listing, governance
//!   inheritance, version-current) all return [`Strategy::A`] from
//!   [`crate::View::strategy`]. This is the baseline the bench gate measures
//!   B against.
//! - [`Strategy::B`] — generalized Algorithm B (dependency-tracked
//!   incremental). Implemented by [`crate::algorithm_b::AlgorithmBView`].
//!   Phase 2b runs B alongside the 5 hand-written views (g8-clarity-1
//!   keep-all-parallel); B does not subsume A in 2b.
//! - [`Strategy::C`] — Z-set / DBSP cancellation. **Reserved-not-implemented
//!   in Phase 2b** (g8-concern-3). The variant exists so the catalog is
//!   stable; constructing a Strategy::C view returns
//!   [`crate::ViewError::StrategyNotImplemented`] with a deferral message
//!   pointing at Phase 3+.
//!
//! See `.addl/phase-2b/00-implementation-plan.md` §3 G8-A + §5 D8.

/// Per-view IVM maintenance strategy.
///
/// `#[non_exhaustive]` is **deliberately omitted** — D8 pins the closed
/// `{ A, B, C }` set. A future Phase-3+ algorithm would land as a NEW enum
/// (e.g. `StrategyV2`) so the runtime contract here stays stable. Adding a
/// fourth variant would be a breaking change; downstream matchers want the
/// compiler to flag missing arms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    /// Hand-written incremental maintenance. The 5 Phase-1 views default to
    /// this. Each view's `update`, `read`, and `rebuild` are bespoke code
    /// that the implementor reasons about line by line. `Strategy::A` is the
    /// correctness baseline the G8-A bench gate measures `Strategy::B`
    /// against.
    A,
    /// Generalized Algorithm B (dependency-tracked incremental). One
    /// algorithm replays change events against any view definition, tracking
    /// per-input-CID dependencies so unrelated input changes don't trigger
    /// recomputation. Implemented by [`crate::algorithm_b::AlgorithmBView`].
    B,
    /// Z-set / DBSP cancellation. **Reserved-not-implemented in Phase 2b**
    /// (g8-concern-3). The variant exists so the catalog of options is
    /// complete + stable. Constructing a `Strategy::C` view via
    /// [`crate::testing::try_construct_view_with_strategy`] returns
    /// [`crate::ViewError::StrategyNotImplemented`] with a deferral message
    /// naming the Phase-3+ target.
    C,
}

//! IVM strategy enum + per-view strategy selection (G8-A; G23-0a rename).
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
//!   B against. Per CLAUDE.md baked-in #2 + arch-r1-14, `Strategy::A` is
//!   reserved for engine-internal canonical-view shapes — user-defined
//!   subgraph-shaped views go through [`Strategy::B`] only.
//! - [`Strategy::B`] — generalized Algorithm B (dependency-tracked
//!   incremental). Implemented by [`crate::algorithm_b::AlgorithmBView`].
//!   `Strategy::B` IS the generalized Algorithm B — G23-0a generalizes the
//!   kernel to consume a `SubgraphSpec` as view definition without minting
//!   a new Strategy variant (CLAUDE.md baked-in #2).
//! - [`Strategy::Reserved`] — Z-set / DBSP cancellation. **Reserved-not-
//!   implemented** (g8-concern-3; renamed from the prior third-variant
//!   spelling at G23-0a per arch-r1-14 — closes CRATES-DEEP-DIVE §4
//!   named-but-deferred item). The variant exists so the catalog is
//!   stable; constructing a `Strategy::Reserved` view returns
//!   [`crate::ViewError::StrategyNotImplemented`] with a deferral message
//!   (the runtime payload string is a v1-error-catalog / #1084 decision).
//!
//! See `.addl/phase-2b/00-implementation-plan.md` §3 G8-A + §5 D8 and
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0a.

/// Per-view IVM maintenance strategy.
///
/// `#[non_exhaustive]` is **deliberately omitted** — D8 pins the closed
/// `{ A, B, Reserved }` set. A future maintenance algorithm would land as
/// a NEW enum (e.g. `StrategyV2`) so the runtime contract here stays stable.
/// Adding a fourth variant would be a breaking change; downstream matchers
/// want the compiler to flag missing arms.
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
    /// G23-0a generalizes the kernel to consume
    /// [`crate::subgraph_spec::SubgraphSpec`] as view definition; the
    /// classification remains `Strategy::B` (no new variant per CLAUDE.md
    /// baked-in #2 — `Strategy::B` IS the generalized Algorithm B).
    B,
    /// Z-set / DBSP cancellation. **Reserved-not-implemented** (renamed
    /// from the prior third-variant spelling at G23-0a per arch-r1-14 —
    /// closes CRATES-DEEP-DIVE §4 named-but-deferred item). The variant
    /// exists so the catalog of options is complete + stable.
    /// Constructing a `Strategy::Reserved` view via
    /// [`crate::testing::try_construct_view_with_strategy`] returns
    /// [`crate::ViewError::StrategyNotImplemented`] with a deferral message
    /// (runtime payload string is a v1-error-catalog / #1084 decision).
    Reserved,
}

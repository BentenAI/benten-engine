//! # benten-ivm — Incremental View Maintenance
//!
//! Phase 1 ships five hand-written IVM views:
//!
//! 1. Capability grants per entity
//! 2. Event handler dispatch
//! 3. Content listing (paginated, sorted by `createdAt`) — load-bearing for
//!    the `crud('post')` exit criterion
//! 4. Governance inheritance
//! 5. Version-chain CURRENT pointer resolution
//!
//! All views subscribe to the graph change stream from `benten-graph` and
//! maintain their state incrementally. The evaluator is deliberately ignorant
//! of IVM; IVM is a subscriber, not an engine-internal feature.
//!
//! ## Module layout
//!
//! - [`view`] — [`View`] trait, [`ViewError`], [`ViewBudget`],
//!   [`ViewDefinition`], and the shared query/result shapes.
//! - [`budget`] — [`BudgetTracker`], the shared `remaining/original/stale`
//!   state machine used by every Phase-1 view (r6-ref R-major-02).
//! - [`subscriber`] — [`Subscriber`] / [`ChangeStreamSubscriber`], the
//!   fan-out that dispatches change events to every registered view.
//! - [`views`] — the five Phase 1 hand-written view implementations.
//!
//! Beyond the five hand-written Phase-1 views, the crate also ships the
//! generalized Algorithm B (per-view strategy selection, user-registered
//! views via [`algorithm_b::AlgorithmBView`]); Z-set / DBSP cancellation
//! remains reserved-not-implemented behind [`Strategy::Reserved`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![allow(
    clippy::todo,
    reason = "Phase 1 leaves some per-view methods as todo!() for later groups"
)]

extern crate alloc;

pub mod algorithm_b;
pub mod budget;
pub mod strategy;
pub mod subgraph_spec;
pub mod subscriber;
pub mod testing;
pub mod view;

pub use algorithm_b::{
    Algorithm, AlgorithmBView, AlgorithmError, LabelPattern, Projection, dispatch_for,
    hardcoded_label_for_id, is_canonical_view_id,
};
pub use budget::BudgetTracker;
pub use strategy::Strategy;
pub use subgraph_spec::{KernelInput, KernelOutput, SubgraphSpec, TypedOutputProjection};
pub use subscriber::{ChangeStreamSubscriber, Subscriber};
pub use view::{
    IvmError, View, ViewBudget, ViewDefinition, ViewError, ViewQuery, ViewResult, ViewState,
};

// TODO(IVM criterion benchmarks): criterion benchmarks against
// RESULTS.md §1 targets — one target per view (capability lookup,
// event dispatch, content listing, governance traversal,
// version-current resolve). See mini-review g5-ivm-14. Pairs with §5
// IVM Algorithm B maturity work.
//
// TODO(IVM cascade-deletion integration test): construct a small
// graph, feed Create events through the subscriber, then the cascade
// of Delete events, and assert every view converges to empty
// (RESULTS.md §3). See mini-review g5-ivm-13.
//
// TODO(IVM rebuild-equivalence event-replay): 4 rebuild-equivalence
// tests in view1/2/3/5 are R3 defects — they construct an empty
// rebuilt view and assert equality with a populated incremental one.
// Fixing requires event-replay. Pairs with §5 IVM Algorithm B
// maturity work. See mini-review g5-ivm-12.
// ---------------------------------------------------------------------------
// The five Phase 1 views — each in its own submodule under `src/views/`.
// Normalized to a proper `src/views/mod.rs` (see R-minor-12 / g5-cr-1).
// ---------------------------------------------------------------------------

pub mod views;

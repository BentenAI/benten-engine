//! # benten-ivm — Incremental View Maintenance (STUB)
//!
//! Phase 1 stub. Full implementation lands in Phase 1 proper.
//!
//! Responsibilities (Phase 1 proper):
//!
//! - Subscribe to the graph change notification stream exposed by `benten-graph`
//!   via the SUBSCRIBE primitive.
//! - Per-view strategy selection (Algorithm B is the default; Algorithm A
//!   is used as a fallback for simple views).
//! - Pre-compute materialized views (capability grants, event handler dispatch,
//!   content listings) so reads are O(1).
//!
//! The evaluator is deliberately ignorant of IVM: IVM is a composable subscriber
//! to graph events, not an engine-internal feature. See
//! [`docs/ENGINE-SPEC.md`](../../../docs/ENGINE-SPEC.md) Section 8.
//!
//! The spike uses this crate only to validate that the 6-crate workspace
//! compiles cleanly and that `benten-engine` can depend on it.

#![forbid(unsafe_code)]

/// Marker for the current stub phase. Removed when real IVM lands.
pub const STUB_MARKER: &str = "benten-ivm::stub";

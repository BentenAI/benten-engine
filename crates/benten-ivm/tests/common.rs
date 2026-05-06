//! Shared test helpers for IVM Algorithm B drift-detector proptests.
//!
//! ## Purpose (r4-r2-ivm-8 closure)
//!
//! R4-R1 ivm-r4-10 named the helper-undefined producer-consumer
//! ambiguity for G15-B: `algorithm_b_drift_detector.rs` invokes
//! `build_incremental_view`, `build_full_view`, and `structured_diff`
//! at multiple sites without a defined home for the helpers. R4-R2
//! `r4-r2-ivm-8` MINOR carried the gap forward at HEAD 98280fe.
//!
//! This file pins the producer-consumer pair at the helper-signature
//! level so the G15-B implementer doesn't make architectural choices
//! silently while wiring un-ignore'd test bodies. The helper home is
//! `crates/benten-ivm/tests/common.rs` (NOT `benten-engine` fixtures
//! and NOT inlined per-proptest); the signatures pin the
//! `MaterializedView` / `StructuredDiff` types' shape such that
//! `algorithm_b_drift_detector.rs` (consumer) and the G15-B
//! implementer (producer) agree on the contract.
//!
//! ## RED-PHASE discipline
//!
//! All helpers are `unimplemented!()` stubs with explicit
//! producer-consumer pair documentation. G15-B implementer fills
//! bodies + drops the `unimplemented!()` per pim-2 §3.6b discipline.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::common::{build_incremental_view, build_full_view, structured_diff};
//! ```
//!
//! Test files include via `mod common;` at the top of the file.
//!
//! ## r4-r2-ivm-2 / r4-r2-ivm-8 cross-references
//!
//! - `algorithm_b_drift_detector.rs::prop_budget_trip_state_propagation_consistent`
//!   reads the wrapper's `is_stale()` + `read()` STATE-RESULT observables
//!   (NOT the execution-trace). Helper signature `build_incremental_view`
//!   returns the wrapper directly so consumer can probe state.
//! - `algorithm_b_drift_detector.rs::prop_drift_detector_reports_one_path_errors_other_succeeds`
//!   uses `try_build_incremental_view` + `try_build_full_view` (Result-
//!   returning siblings) + `asymmetric_path_diff` to surface the
//!   AsymmetricBudget structured-diff kind.

#![allow(dead_code, unused_variables, clippy::unwrap_used)]
#![allow(
    clippy::needless_pass_by_value,
    clippy::result_unit_err,
    reason = "RED-PHASE stubs; final signatures (including custom error type) determined at G15-B implementation"
)]

/// Result of materialising a view (incremental or from-scratch).
///
/// The shape is pinned at the producer-consumer pair: G15-B implementer
/// fills the inner type to match the kernel's wrapper API surface
/// (`is_stale`, `canonical_bytes`, `read`, `read_with`, `refresh`,
/// `materialised`).
#[derive(Debug)]
pub struct MaterializedView {
    // G15-B implementer fills:
    //   inner: benten_ivm::ViewWrapper<...>,
    _placeholder: (),
}

/// Structured diff between two materialisations.
///
/// G15-B implementer fills the inner type with the structured-diff
/// helper output (per `prop_drift_detector_reports_one_path_errors_other_succeeds`
/// the diff carries a `kind()` accessor that returns `DiffKind::AsymmetricBudget`
/// for the one-path-errors-other-succeeds asymmetry shape).
#[derive(Debug)]
pub struct StructuredDiff {
    _placeholder: (),
}

/// Diff kind enum — used by `prop_drift_detector_reports_one_path_errors_other_succeeds`
/// to assert the AsymmetricBudget structured-diff variant.
#[derive(Debug, PartialEq, Eq)]
pub enum DiffKind {
    /// Equal canonical_bytes — no drift.
    Equal,
    /// Different canonical_bytes — drift between incremental + from-scratch.
    Drift,
    /// One path errored (BudgetExceeded), other succeeded — asymmetric.
    AsymmetricBudget,
}

/// Build an incremental view by replaying writes through Algorithm B.
///
/// Producer: G15-B implementer wires this against the kernel's
/// incremental-update path (Strategy::B).
/// Consumer: `algorithm_b_drift_detector.rs::prop_*` proptests.
///
/// # Returns
///
/// A `MaterializedView` whose `is_stale()` is true if budget tripped
/// during apply, false otherwise.
pub fn build_incremental_view(
    _view_def: &(), /* G15-B fills: &benten_ivm::ViewDefinition */
    _writes: &[()], /* G15-B fills: &[benten_engine::Write] */
) -> MaterializedView {
    unimplemented!(
        "G15-B wires incremental-view materialisation via Algorithm B \
         (Strategy::B kernel path; budget-trip propagates to \
         MaterializedView::is_stale()) per r4-r2-ivm-8"
    )
}

/// Build a from-scratch view by full rebuild over all writes.
///
/// Producer: G15-B implementer wires this against the kernel's
/// from-scratch-rebuild path (the comparison baseline for the
/// drift-detector proptests).
/// Consumer: `algorithm_b_drift_detector.rs::prop_*` proptests.
///
/// # Returns
///
/// A `MaterializedView` whose `canonical_bytes()` is the
/// from-scratch materialisation.
pub fn build_full_view(
    _view_def: &(), /* G15-B fills: &benten_ivm::ViewDefinition */
    _writes: &[()], /* G15-B fills: &[benten_engine::Write] */
) -> MaterializedView {
    unimplemented!("G15-B wires from-scratch full-rebuild materialisation per r4-r2-ivm-8")
}

/// Result-returning sibling of `build_incremental_view` for the
/// asymmetric-budget probe. Returns `Err(BudgetExceeded)` instead of
/// producing a stale wrapper when budget trips.
pub fn try_build_incremental_view(_view_def: &(), _writes: &[()]) -> Result<MaterializedView, ()> /* G15-B fills: Result<_, benten_ivm::Error> */
{
    unimplemented!(
        "G15-B wires Result-returning incremental-view sibling for \
         AsymmetricBudget structured-diff probe per r4-r2-ivm-8 + r4-r2-ivm-2"
    )
}

/// Result-returning sibling of `build_full_view` for the
/// asymmetric-budget probe.
pub fn try_build_full_view(_view_def: &(), _writes: &[()]) -> Result<MaterializedView, ()> /* G15-B fills: Result<_, benten_ivm::Error> */
{
    unimplemented!(
        "G15-B wires Result-returning from-scratch sibling for \
         AsymmetricBudget structured-diff probe per r4-r2-ivm-8 + r4-r2-ivm-2"
    )
}

/// Compute a structured diff between two materialisations.
///
/// Producer: G15-B implementer wires this against a structural-diff
/// algorithm that reports kind + per-row drift.
/// Consumer: `algorithm_b_drift_detector.rs::prop_algorithm_b_incremental_equals_rebuild_for_arbitrary_label_pattern`
/// (uses the diff in the prop_assert_eq error message).
pub fn structured_diff(_a: &MaterializedView, _b: &MaterializedView) -> StructuredDiff {
    unimplemented!("G15-B wires structured-diff algorithm (per-row, kind-tagged) per r4-r2-ivm-8")
}

/// Compute the asymmetric-path diff for the one-path-errors-other-succeeds
/// proptest scenario.
///
/// Returns a `StructuredDiff` whose `kind()` is
/// `DiffKind::AsymmetricBudget`. The diff's `is_reported()` accessor
/// MUST return true (the diff is explicitly tracked, NOT silently
/// filtered via prop_assert_eq early-return per pim-2 §3.6b +
/// r4-r2-ivm-2 third-pin).
pub fn asymmetric_path_diff(
    _incremental_err: &Option<()>, /* G15-B: &Option<benten_ivm::Error> */
    _from_scratch: &MaterializedView,
) -> StructuredDiff {
    unimplemented!(
        "G15-B wires asymmetric-path structured-diff (DiffKind::AsymmetricBudget) \
         per ivm-major-3 (c) + r4-r2-ivm-2 third-pin"
    )
}

impl StructuredDiff {
    /// Returns the diff's kind (Equal / Drift / AsymmetricBudget).
    pub fn kind(&self) -> DiffKind {
        unimplemented!("G15-B wires StructuredDiff::kind accessor per r4-r2-ivm-8")
    }

    /// Returns true if the diff is explicitly reported (NOT a silent
    /// prop_assert_eq early-return pass).
    pub fn is_reported(&self) -> bool {
        unimplemented!("G15-B wires StructuredDiff::is_reported accessor per r4-r2-ivm-2 third-pin")
    }
}

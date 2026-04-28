//! Test-only helpers — strategy construction + Criterion estimates parser.
//!
//! Not gated behind `#[cfg(test)]` because consumers across crates
//! (integration tests in this crate, the bench-gate `tests/algorithm_b_within_20pct_gate.rs`)
//! need to call into them. The module is documented as test/dev surface;
//! production code paths construct views via the per-view `::new()` ctors.

use alloc::boxed::Box;
use alloc::string::{String, ToString};

use crate::Strategy;
use crate::algorithm_b::AlgorithmBView;
use crate::view::{View, ViewError};
use crate::views::{CapabilityGrantsView, ContentListingView};

/// Construct a [`Box<dyn View>`] for the requested [`Strategy`].
///
/// - [`Strategy::A`] → a hand-written view (capability grants, the simplest
///   shape). The 5 hand-written views are ALL `Strategy::A`; tests that
///   need a specific shape construct it directly via `XxxView::new`.
/// - [`Strategy::B`] → an [`AlgorithmBView`] for the `content_listing`
///   shape (the highest gate-risk view per `r1-ivm-algorithm.json`, so
///   the default test target).
///
/// # Panics
///
/// Panics on [`Strategy::C`] because the variant is reserved and not
/// implemented in Phase 2b. Tests that need to assert the typed-error
/// behavior should use [`try_construct_view_with_strategy`] instead.
#[must_use]
pub fn testing_construct_view_with_strategy(strategy: Strategy) -> Box<dyn View> {
    try_construct_view_with_strategy(strategy)
        .expect("strategy is reserved — use try_construct_view_with_strategy for the typed error")
}

/// Fallible variant of [`testing_construct_view_with_strategy`] — surfaces
/// the typed [`ViewError::StrategyNotImplemented`] for [`Strategy::C`]
/// rather than panicking.
///
/// # Errors
///
/// Returns [`ViewError::StrategyNotImplemented`] when `strategy` is
/// [`Strategy::C`] (Z-set / DBSP cancellation, deferred to Phase 3+).
/// Strategy::A + Strategy::B always succeed.
pub fn try_construct_view_with_strategy(strategy: Strategy) -> Result<Box<dyn View>, ViewError> {
    match strategy {
        Strategy::A => Ok(Box::new(CapabilityGrantsView::new())),
        Strategy::B => {
            let view = AlgorithmBView::for_id("content_listing", ContentListingView::definition())?;
            Ok(Box::new(view))
        }
        Strategy::C => Err(ViewError::StrategyNotImplemented {
            strategy: Strategy::C,
            deferred_to_phase: "Phase 3+".to_string(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Criterion estimates parser (G8-A bench gate helper)
// ---------------------------------------------------------------------------

/// Read a Criterion `estimates.json` and return the `mean.point_estimate`
/// in nanoseconds.
///
/// Path shape: `target/criterion/<group>/<view>/<axis>/<value>/estimates.json`.
/// E.g. `target/criterion/algorithm_b_vs_handwritten/content_listing/strategy/A/estimates.json`.
///
/// Centralized here so the bench-gate test + any future cross-bench gate
/// share one parser. Only available outside `#![no_std]` builds because
/// it touches `std::fs` + `serde_json`.
///
/// # Errors
///
/// Returns a string description on:
/// - missing file at the expected path (no Criterion run staged)
/// - file present but not valid JSON
/// - JSON missing the `mean.point_estimate` field
#[cfg(feature = "phase_2b_landed")]
pub fn criterion_estimates_mean_ns(
    group: &str,
    view: &str,
    axis: &str,
    value: &str,
) -> Result<f64, String> {
    use std::path::PathBuf;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|e| format!("CARGO_MANIFEST_DIR not set: {e}"))?;
    // Walk from the crate dir up to the workspace `target/`.
    let target_dir = PathBuf::from(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("target"))
        .ok_or_else(|| format!("could not derive workspace target/ from {manifest_dir}"))?;
    let path = target_dir
        .join("criterion")
        .join(group)
        .join(view)
        .join(axis)
        .join(value)
        .join("estimates.json");
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("failed to parse {} as JSON: {e}", path.display()))?;
    let mean = parsed
        .get("mean")
        .and_then(|m| m.get("point_estimate"))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| {
            format!(
                "estimates JSON at {} missing `mean.point_estimate`",
                path.display()
            )
        })?;
    Ok(mean)
}

/// Stub variant when `phase_2b_landed` is OFF. The bench gate test is
/// itself gated on the same feature so the real parser is only needed
/// when the gate runs.
#[cfg(not(feature = "phase_2b_landed"))]
pub fn criterion_estimates_mean_ns(
    _group: &str,
    _view: &str,
    _axis: &str,
    _value: &str,
) -> Result<f64, String> {
    Err(String::from(
        "criterion_estimates_mean_ns requires `phase_2b_landed` feature",
    ))
}

// Internal compile-time use to satisfy the unused-import warning when the
// feature flag flips. `String` is actually used in both branches but the
// combined lint sometimes mis-reports.
#[allow(dead_code)]
fn _force_string_use() -> String {
    String::new()
}

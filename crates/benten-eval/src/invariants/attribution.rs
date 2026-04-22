//! Phase 2a G5-B-ii / E12 / phil-1: Invariant-14 — every `TraceStep` carries
//! an `AttributionFrame`. Registration-time structural declaration lives
//! here; runtime threading lives in `evaluator/attribution.rs` (G5-B-ii).
//!
//! TODO(phase-2a-G5-B-ii): wire declaration-time checks + runtime threading
//! per plan §9.1 + phil-1 dual-surface resolution.

use crate::{EvalError, NullHost, Subgraph, TraceStep};

/// Registration-time declaration validator. Rejects a subgraph whose
/// primitives fail to declare their attribution source.
///
/// # Errors
/// Returns [`EvalError`] carrying `ErrorCode::InvAttribution`.
pub fn validate_registration(_subgraph: &Subgraph) -> Result<(), EvalError> {
    todo!("Phase 2a G5-B-ii: implement registration-time Inv-14 declaration check")
}

/// Backwards-compat shim for the `invariant_14_attribution` test.
///
/// # Errors
/// Forwards to [`validate_registration`].
pub fn validate(sg: &Subgraph) -> Result<(), EvalError> {
    validate_registration(sg)
}

/// Test harness: run a 5-step handler with attribution stamping.
///
/// # Errors
/// Returns [`EvalError`] on runtime failure.
pub fn run_with_attribution_for_test(
    _subgraph: &Subgraph,
    _host: &NullHost,
) -> Result<Vec<TraceStep>, EvalError> {
    todo!("Phase 2a G5-B-ii: test harness for `invariant_14_attribution_every_trace_step`")
}

/// Test harness: build a 5-step handler whose primitives declare their
/// attribution source.
#[must_use]
pub fn build_five_step_handler_for_test() -> Subgraph {
    todo!("Phase 2a G5-B-ii: test harness for Inv-14 positive case")
}

/// Test harness: build a subgraph whose primitives intentionally omit
/// attribution declaration (negative case — must reject at registration).
#[must_use]
pub fn build_subgraph_with_undeclared_attribution_for_test() -> Subgraph {
    todo!("Phase 2a G5-B-ii: test harness for Inv-14 negative case")
}

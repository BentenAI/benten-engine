//! Phase 2a G4-A / E9 / Code-as-graph Major #2: Invariant-8 multiplicative
//! cumulative budget. Stub — R5 G4-A lands the real check.
//!
//! TODO(phase-2a-G4-A): compute cumulative budget over DAG paths with
//! saturating arithmetic + honour `CALL { isolated }` reset semantics.

use benten_errors::ErrorCode;

use crate::{RegistrationError, Subgraph};

/// Thin newtype wrapping the cumulative bound.
#[derive(Debug, Clone, Copy)]
pub struct MultiplicativeBudget(u64);

impl MultiplicativeBudget {
    /// Construct a budget bound.
    #[must_use]
    pub fn new(limit: u64) -> Self {
        Self(limit)
    }

    /// Return the underlying bound.
    #[must_use]
    pub fn limit(self) -> u64 {
        self.0
    }
}

/// Budget-check error type surfaced by the multiplicative validator.
#[derive(Debug, Clone)]
pub struct BudgetError {
    code: ErrorCode,
    message: String,
}

// Ensure the `matches!(rejected, Err(e) ...)` pattern in tests compiles —
// the macro-expanded pattern binds `e` by value, which requires the
// `Result<_, BudgetError>` be movable-out. Since `Err` on a `Result<T, E>`
// already owns `e`, this is structural; no impl change needed beyond the
// `Clone` derive above.

impl BudgetError {
    /// Stable catalog code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        self.code.clone()
    }

    /// Human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.code.as_str(), self.message)
    }
}

impl std::error::Error for BudgetError {}

/// Validate the multiplicative cumulative budget for a subgraph.
///
/// # Errors
/// Fires `E_INV_ITERATE_BUDGET` via [`BudgetError`] when the worst-path
/// product exceeds `bound.limit()`.
pub fn validate_multiplicative(
    _subgraph: &Subgraph,
    _bound: MultiplicativeBudget,
) -> Result<(), BudgetError> {
    todo!(
        "Phase 2a G4-A: compute product-over-paths multiplicative budget per \
         plan §9.12 + Code-as-graph Major #2"
    )
}

/// Test harness: build a subgraph shape of `CALL → ITERATE → CALL → ITERATE → CALL`
/// with the three per-node maxes specified.
#[must_use]
pub fn build_chained_call_iterate_iterate_for_test(_m1: u64, _m2: u64, _m3: u64) -> Subgraph {
    todo!("Phase 2a G4-A: test harness for `invariant_8_multiplicative_through_call`")
}

/// Test harness: ITERATE(inner_max) nested inside ITERATE(outer_max).
#[must_use]
pub fn build_nested_iterate_for_test(_outer_max: u64, _inner_max: u64) -> Subgraph {
    todo!("Phase 2a G4-A: test harness for `invariant_8_multiplicative_through_iterate`")
}

/// Test harness: CALL into a callee with a declared callee-side budget.
/// Single-arg form used by `invariant_8_multiplicative.rs`.
#[must_use]
pub fn build_call_with_callee_budget_for_test(_callee_bound: u64) -> Subgraph {
    todo!("Phase 2a G4-A: test harness for `isolated_call_resets_to_callee_grant`")
}

/// Test harness: a DAG with two paths of different products; cumulative is
/// the MAX over paths. Takes the per-path ITERATE-max lists.
#[must_use]
pub fn build_two_path_dag_for_test(_path_a: &[u64], _path_b: &[u64]) -> Subgraph {
    todo!("Phase 2a G4-A: test harness for `multiplicative_product_over_path`")
}

/// Compute the cumulative Inv-8 budget for a subgraph; returns the
/// product-over-paths MAX value.
#[must_use]
pub fn compute_cumulative(_subgraph: &Subgraph) -> u64 {
    todo!("Phase 2a G4-A: implement cumulative-budget computation")
}

/// Integration with the structural `validate_subgraph` entry point.
///
/// # Errors
/// Converts [`BudgetError`] to [`RegistrationError`] carrying the Inv-8
/// context.
pub fn validate(_subgraph: &Subgraph) -> Result<(), RegistrationError> {
    // Phase 2a stub — Phase 1 validation still lives in `invariants/mod.rs`
    // under the nest-depth stopgap; G4-A activates this path.
    Ok(())
}

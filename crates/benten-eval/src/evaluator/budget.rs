//! Phase 2a G4-A: shared budget helper.
//!
//! This module is the single coordination point where the evaluator's
//! ITERATE + CALL primitives consult iteration-budget bookkeeping at
//! registration + run time. It is deliberately thin:
//!
//! - [`cumulative_budget_for_subgraph`] exposes the registration-time
//!   multiplicative product (delegating to
//!   [`crate::invariants::budget::compute_cumulative`]) so that
//!   `primitives/iterate.rs` and `primitives/call.rs` do not each re-walk
//!   the subgraph.
//! - [`check_per_iteration_budget`] is the shared run-time check both
//!   primitives call on each batch boundary; it composes the caller's
//!   remaining budget with the per-iteration cost and fires a
//!   `BudgetError` (stable code `E_INV_ITERATE_BUDGET`) when the limit is
//!   crossed.
//!
//! The file was introduced per plan §3 G4-A to resolve cr-r1-3's
//! file-ownership coordination concern: G9-A's wall-clock TOCTOU check at
//! batch boundaries and G4-A's multiplicative accumulation share one
//! entry point rather than duplicating the bookkeeping across two
//! primitive executors.

use crate::Subgraph;
use crate::invariants::budget::{self, BudgetError};

/// Compute the cumulative Inv-8 multiplicative budget for a subgraph.
///
/// Thin re-export of [`budget::compute_cumulative`]; lives here so the
/// evaluator-side call-sites have a co-located surface. The underlying
/// walker lives in `invariants/budget.rs` alongside the registration-time
/// validator (`validate_multiplicative` / `validate_snapshot`), which
/// keeps the DAG-walking state-machine in one place.
#[must_use]
pub fn cumulative_budget_for_subgraph(sg: &Subgraph) -> u64 {
    budget::compute_cumulative(sg)
}

/// Run-time check consulted at every ITERATE batch boundary (and at
/// every CALL-entry boundary when callers want pre-flight validation).
/// Returns `Err(BudgetError)` when `consumed > limit`.
///
/// The helper is intentionally simple — it does not touch any shared
/// state. Callers pass the per-frame counts (the iterative evaluator
/// holds them on the stack). Consolidation into a single helper is the
/// cr-r1-3 coordination fix so the budget check does not drift between
/// `primitives/iterate.rs` and `primitives/call.rs`.
///
/// # Errors
/// Returns a [`BudgetError`] carrying `E_INV_ITERATE_BUDGET` when the
/// running count crosses the declared limit.
pub fn check_per_iteration_budget(consumed: u64, limit: u64) -> Result<(), BudgetError> {
    if consumed > limit {
        // Delegate to the registration-time validator on a synthetic
        // single-node subgraph so the BudgetError wording + catalog code
        // stay in one place (the registration-time path owns the canonical
        // message). Performance is not a concern here — the helper only
        // fires when the evaluator has already crossed the limit.
        let bound = budget::MultiplicativeBudget::new(limit);
        let mut sg = Subgraph::new("budget_check");
        sg.nodes.push(
            crate::OperationNode::new("iter_budget_check", crate::PrimitiveKind::Iterate)
                .with_property(
                    "max",
                    benten_core::Value::Int(i64::try_from(consumed).unwrap_or(i64::MAX)),
                ),
        );
        return budget::validate_multiplicative(&sg, bound);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invariants::budget::build_chained_call_iterate_iterate_for_test;

    #[test]
    fn cumulative_shared_helper_matches_invariant_module() {
        let sg = build_chained_call_iterate_iterate_for_test(2, 3, 5);
        assert_eq!(cumulative_budget_for_subgraph(&sg), 30);
    }

    #[test]
    fn per_iteration_under_limit_ok() {
        assert!(check_per_iteration_budget(99, 100).is_ok());
        assert!(check_per_iteration_budget(100, 100).is_ok());
    }

    #[test]
    fn per_iteration_over_limit_fires() {
        let err = check_per_iteration_budget(101, 100).unwrap_err();
        use benten_errors::ErrorCode;
        assert_eq!(err.code(), ErrorCode::InvIterateBudget);
    }
}

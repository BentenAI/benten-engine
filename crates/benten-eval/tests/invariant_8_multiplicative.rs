//! R3 unit tests for G4-A / E9 / Code-as-graph Major #2: Invariant-8
//! multiplicative cumulative budget through CALL + ITERATE.
//!
//! Happy-path coverage:
//! - `invariant_8_multiplicative_through_call` (3*3*3=27; bound 26 rejects,
//!   28 accepts)
//! - `invariant_8_multiplicative_through_iterate` (ITERATE(5) nested inside
//!   ITERATE(4) = 20 cumulative; bound 19 rejects, 20 accepts)
//! - `invariant_8_multiplicative_product_over_path` — cumulative = product
//!   over each DAG path, MAX across paths
//! - CALL respects capability on budget
//!
//! Plus `prop_invariant_8_multiplicative_exact` — random DAG nesting;
//! static bound equals runtime max-over-paths product.
//!
//! TDD red-phase: the multiplicative budget check lives in
//! `benten_eval::invariants::budget` which does not yet exist in its Phase-2a
//! shape. Tests fail to compile until G4-A lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.4).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_errors::ErrorCode;
use benten_eval::invariants::budget::{self, MultiplicativeBudget};
use proptest::prelude::*;

/// Pre-register the callee that
/// [`budget::build_chained_call_iterate_iterate_for_test`] now embeds as a
/// real CALL node (G4-A mini-review C2). Callers must invoke this before
/// validating the resulting subgraph — the M1 fallback now rejects an
/// unregistered callee at registration time.
fn register_chained_callee(call_factor: u64) {
    benten_eval::register_test_callee(&budget::callee_name_for_factor(call_factor), call_factor);
}

#[test]
fn invariant_8_multiplicative_through_call() {
    // ITERATE(3) → CALL(non-isolated, callee-bound=3) → ITERATE(3) =
    // 3 * 3 * 3 = 27 cumulative iterations along the worst path.
    register_chained_callee(3);
    let subgraph = budget::build_chained_call_iterate_iterate_for_test(3, 3, 3);

    // With a bound of 26, registration must reject.
    let rejected = budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(26));
    assert!(
        matches!(&rejected, Err(e) if e.code() == ErrorCode::InvIterateBudget),
        "bound 26 below cumulative 27 must fire E_INV_ITERATE_BUDGET; got {rejected:?}"
    );

    // With a bound of 28, registration must accept.
    budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(28))
        .expect("bound 28 above cumulative 27 must accept");
}

#[test]
fn invariant_8_multiplicative_through_iterate() {
    // ITERATE(max=5) nested inside ITERATE(max=4) = 4 * 5 = 20 cumulative.
    let subgraph = budget::build_nested_iterate_for_test(4, 5);

    let rejected = budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(19));
    assert!(
        matches!(&rejected, Err(e) if e.code() == ErrorCode::InvIterateBudget),
        "bound 19 rejects; got {rejected:?}"
    );

    budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(20))
        .expect("bound 20 accepts exactly cumulative 20");
}

#[test]
fn invariant_8_multiplicative_product_over_path() {
    // Branching DAG: two paths through the handler.
    // Path A: ITERATE(3) -> ITERATE(2) = 6
    // Path B: ITERATE(5)              = 5
    // Cumulative = max(6, 5) = 6 (MAX across paths, NOT sum).
    let subgraph = budget::build_two_path_dag_for_test(&[3, 2], &[5]);
    let rejected = budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(5));
    assert!(
        matches!(&rejected, Err(e) if e.code() == ErrorCode::InvIterateBudget),
        "bound 5 below max-path 6 must reject; got {rejected:?}"
    );
    budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(6))
        .expect("bound 6 equal to max-path accepts");
}

#[test]
fn call_respecting_cap_on_budget() {
    // CALL into a callee whose grant-declared budget is 4. The caller cannot
    // use a tighter caller-declared budget to exceed the callee's.
    let subgraph = budget::build_call_with_callee_budget_for_test(4);
    // Any caller-declared bound ≥ 4 is fine; the callee's grant is what binds.
    budget::validate_multiplicative(&subgraph, MultiplicativeBudget::new(100))
        .expect("callee grant bound of 4 is honored regardless of caller bound");
}

// ---- Proptest: multiplicative exact across random DAGs ----

// Pre-seed the chained-callee registry across the proptest's `call_factor`
// range so every shrink target is resolvable. Bounds 1..=5 match the
// proptest strategy below. G4-A mini-review M1 added registration-time
// rejection for unknown callees; without these entries the walker would
// reject every proptest input rather than exercising the CALL factor
// computation.
fn preseed_proptest_callees() {
    for factor in 1u64..=5 {
        register_chained_callee(factor);
    }
}

proptest! {
    // G4-A mini-review C2: the harness the proptest drives now builds
    // ITERATE(m1) → CALL(non-isolated, callee-bound=call_factor) →
    // ITERATE(m3). This genuinely exercises `non_isolated_callee_factor`
    // + the `walk` CALL branch — the prior harness collapsed call_factor
    // into a third ITERATE and so had zero CALL-path coverage.
    #[test]
    fn prop_invariant_8_multiplicative_exact(
        // CALL factor plus two ITERATE maxes; shrink toward minimal
        // counterexample DAGs. Registry pre-seeded for 1..=5 via
        // `preseed_proptest_callees`.
        call_factor in 1u64..=5,
        iter_a in 1u64..=5,
        iter_b in 1u64..=5,
    ) {
        preseed_proptest_callees();
        let subgraph =
            budget::build_chained_call_iterate_iterate_for_test(iter_a, call_factor, iter_b);
        // Reference implementation: the path is a straight chain
        // ITERATE → CALL → ITERATE, so the worst-case MAX-over-paths
        // product equals iter_a * call_factor * iter_b.
        let expected = iter_a.saturating_mul(call_factor).saturating_mul(iter_b);
        let observed = budget::compute_cumulative(&subgraph);
        prop_assert_eq!(
            observed, expected,
            "static cumulative must equal product of nesting factors \
             (ITERATE × non-isolated-CALL × ITERATE)"
        );
    }
}

//! Edge-case tests: Invariant 8 (multiplicative iteration budget) behaviour
//! at the `CALL { isolated }` boundary.
//!
//! R2 landscape §2.5.4 row "Inv-8 isolated-CALL resets to callee grant"
//! (Code-as-graph Major #2).
//!
//! Concerns pinned:
//! - `CALL { isolated: true }` does NOT multiply the parent's remaining
//!   budget into the callee — the callee runs under its own grant's
//!   declared iteration bound, a fresh frame.
//! - `CALL { isolated: false }` (default, non-isolated) DOES inherit the
//!   parent's remaining budget — budget propagates multiplicatively along
//!   the call chain.
//! - A callee chain that would overflow the parent's budget under
//!   non-isolated CALL correctly fires `E_INV_ITERATE_BUDGET`.
//! - Under `isolated: true`, the same callee chain is accepted (budget
//!   reset at frame boundary).
//! - Boundary: saturating arithmetic at the `max × max × max` path product —
//!   cumulative `u64::MAX × 2` must NOT panic; must fire
//!   `E_INV_ITERATE_BUDGET`.
//!
//! R3 red-phase contract: R5 (G4-A) lands multiplicative Inv-8 + the
//! `isolated` boolean on CALL nodes. These tests compile; they fail because
//! the multiplicative check is not in place yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes per R1 triage."
)]

use benten_eval::{ErrorCode, Subgraph, SubgraphBuilder};
use benten_eval::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};

/// Register a callee handler with a declared iteration-budget bound `callee_bound`.
/// The caller subgraph under test invokes this handler.
fn register_callee_with_bound(bound: u64) -> String {
    // Placeholder registration hook used by the evaluator test harness.
    let name = format!("callee_bound_{bound}");
    benten_eval::register_test_callee(&name, bound);
    name
}

fn caller_subgraph_with_iterate_then_call(
    iter_max: u64,
    callee: &str,
    isolated: bool,
) -> Result<Subgraph, benten_eval::RegistrationError> {
    let mut sb = SubgraphBuilder::new(&format!("caller_iter{iter_max}_iso{isolated}"));
    let root = sb.read("input");
    let iter = sb.iterate(root, "iter_body", iter_max);
    let _call = sb.call_with_isolated(iter, callee, isolated);
    sb.respond(iter);
    sb.build_validated()
}

#[test]
fn invariant_8_isolated_call_resets_to_callee_grant() {
    // Parent: ITERATE(max=1000) → CALL {isolated:true} of a callee with
    // declared bound 5. Under isolated-CALL semantics the callee runs at
    // its own bound (5); the caller's 1000 does NOT multiply in.
    let callee = register_callee_with_bound(5);
    let sg = caller_subgraph_with_iterate_then_call(1000, &callee, true)
        .expect("isolated-CALL must accept since callee frame is fresh");

    // Explicit semantic pin: the cumulative-budget on the CALL's subgraph
    // for the callee-path is `callee_bound` (5), NOT `iter_max × callee`.
    let cumulative = sg
        .cumulative_budget_for_handle_for_test(sg.handle_of("call_2"))
        .expect("cumulative budget must be computed");
    assert_eq!(
        cumulative, 5,
        "isolated CALL must reset cumulative to callee bound; got {cumulative}"
    );
}

#[test]
fn invariant_8_non_isolated_call_inherits_parent_remaining_and_overflows() {
    // Parent: ITERATE(max=1000) → CALL {isolated:false} of a callee with
    // declared bound 1000. Cumulative = 1000 × 1000 = 1_000_000; if the
    // configured Inv-8 bound is 500_000 → registration must reject.
    let callee = register_callee_with_bound(1000);
    let err = caller_subgraph_with_iterate_then_call(1000, &callee, false)
        .expect_err("non-isolated × overflow must fail registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvIterateBudget,
        "non-isolated CALL overflow must fire E_INV_ITERATE_BUDGET, got {:?}",
        err.code()
    );
}

#[test]
fn invariant_8_non_isolated_call_inherits_parent_remaining_within_bound() {
    // Negative case: non-isolated CALL where the product stays under the
    // Inv-8 bound is accepted.
    let callee = register_callee_with_bound(2);
    let sg = caller_subgraph_with_iterate_then_call(3, &callee, false)
        .expect("product 3×2=6 must be within default Inv-8 bound");
    let cumulative = sg
        .cumulative_budget_for_handle_for_test(sg.handle_of("call_2"))
        .unwrap();
    assert_eq!(
        cumulative, 6,
        "non-isolated CALL product 3×2 must be 6, got {cumulative}"
    );
}

#[test]
fn invariant_8_multiplicative_saturating_arithmetic_does_not_panic() {
    // Boundary: `u64::MAX × 2` would overflow; the Inv-8 computation must
    // use saturating arithmetic + fire E_INV_ITERATE_BUDGET, NOT panic.
    let callee = register_callee_with_bound(u64::MAX);
    // caller iterate_max = 2 (non-isolated) — product would be 2 × u64::MAX.
    let err = caller_subgraph_with_iterate_then_call(2, &callee, false)
        .expect_err("u64-saturating overflow must fail registration");
    assert_eq!(
        err.code(),
        ErrorCode::InvIterateBudget,
        "saturating-overflow must fire E_INV_ITERATE_BUDGET (no panic)"
    );
}

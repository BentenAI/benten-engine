//! G4-A mini-review M2: isolated-CALL must not leak the callee's bound
//! into the caller's post-CALL path.
//!
//! Per plan §9.12, `CALL { isolated: true }` resets the cumulative budget
//! to the callee grant's declared bound — but that reset applies to the
//! CALLEE frame. The caller's nodes reached AFTER the isolated CALL
//! continue under the caller's grant, so their cumulative must compose
//! with the caller's PRE-CALL running product (not the callee's bound).
//!
//! A caller shape `ITERATE(10) → isolated-CALL(callee-bound=5) →
//! ITERATE(3)` must yield cumulative `10 * 3 = 30` at the trailing
//! ITERATE, not `5 * 3 = 15`. The prior `carry_forward = at_here`
//! implementation under-counted and silently let subgraphs past
//! registration that should fail.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};
use benten_eval::{Subgraph, SubgraphBuilder};

fn caller_iterate_isolated_call_iterate(
    outer_iter: u64,
    callee_bound: u64,
    trailing_iter: u64,
) -> Subgraph {
    // Register the callee so the Inv-8 walker can resolve its bound.
    let callee = format!("isolated_callee_bound_{callee_bound}");
    benten_eval::register_test_callee(&callee, callee_bound);

    let mut sb = SubgraphBuilder::new("caller_iso_post_boundary");
    let root = sb.read("input");
    let iter_outer = sb.iterate(root, "outer_body", outer_iter);
    let call = sb.call_with_isolated(iter_outer, &callee, true);
    let _iter_after = sb.iterate(call, "after_body", trailing_iter);
    sb.build_validated().expect(
        "ITERATE(outer) → isolated-CALL → ITERATE(trailing) must validate — \
         the post-CALL path inherits the CALLER's running product, not the \
         callee bound",
    )
}

#[test]
fn isolated_call_followed_by_iterate_does_not_inherit_callee_bound() {
    // Caller: ITERATE(10) → isolated-CALL(callee-bound=5) → ITERATE(3).
    // Caller's cumulative at the trailing ITERATE = 10 * 3 = 30.
    // The isolated CALL's OWN cumulative is the callee bound 5 (reset
    // semantics at the CALL boundary — the CALLEE frame runs at 5).
    // The caller's running product of 10 must be what propagates past
    // the CALL to the trailing ITERATE.
    let sg = caller_iterate_isolated_call_iterate(10, 5, 3);
    let handle_after = sg.handle_of("iterate_3");
    let cumulative = sg
        .cumulative_budget_for_handle_for_test(handle_after)
        .expect("trailing ITERATE cumulative must be computed");
    assert_eq!(
        cumulative, 30,
        "isolated-CALL must NOT leak its callee bound into the caller's \
         post-CALL path: expected 10 (outer) × 3 (trailing) = 30, got {cumulative}"
    );

    // The isolated-CALL node itself carries the callee bound (reset).
    let handle_call = sg.handle_of("call_2");
    let cum_call = sg
        .cumulative_budget_for_handle_for_test(handle_call)
        .expect("isolated-CALL cumulative must be computed");
    assert_eq!(
        cum_call, 5,
        "isolated-CALL node itself records the callee bound (reset); got {cum_call}"
    );
}

#[test]
fn isolated_call_rejects_when_post_call_path_overflows_default_bound() {
    // Pre-M2: a shape `ITERATE(100_000) → isolated-CALL(callee=1) →
    // ITERATE(100)` under-counted as 1 * 100 = 100, sneaking past the
    // 500_000 default. Post-M2: cumulative correctly computes to
    // 100_000 * 100 = 10_000_000, which the default Inv-8 bound rejects.
    let callee = "low_cost_callee_for_overflow_test";
    benten_eval::register_test_callee(callee, 1);

    let mut sb = SubgraphBuilder::new("iso_post_overflow");
    let root = sb.read("input");
    let outer = sb.iterate(root, "outer", 100_000);
    let call = sb.call_with_isolated(outer, callee, true);
    let _trail = sb.iterate(call, "trail", 100);
    let err = sb.build_validated().expect_err(
        "ITERATE(100_000) × ITERATE(100) = 1e7 should reject against the \
         500_000 default Inv-8 bound (M2 — caller's post-CALL path \
         inherits 100_000, not the callee's bound of 1)",
    );
    use benten_eval::ErrorCode;
    assert_eq!(
        err.code(),
        ErrorCode::InvIterateBudget,
        "post-isolated-CALL overflow must fire E_INV_ITERATE_BUDGET, got {:?}",
        err.code()
    );
}

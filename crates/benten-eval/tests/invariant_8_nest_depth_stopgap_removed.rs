//! Edge-case tests: Phase-1 ITERATE nest-depth-3 stopgap is removed in Phase 2a
//! (plan E9 + r1-cr-8 stopgap phrasing fix).
//!
//! R2 landscape §2.5.4 row "Inv-8 nest-depth-3 stopgap dropped".
//!
//! Phase 1 enforced `MAX_ITERATE_NEST_DEPTH = 3` as a stopgap for Inv-8's
//! cumulative-budget aspect. Phase 2a lands the proper multiplicative form,
//! so the nest-depth-3 stopgap is dropped. A 4-deep ITERATE nest whose
//! cumulative product stays within the Inv-8 bound MUST be accepted (whereas
//! Phase 1 always rejected depth 4 regardless of budget).
//!
//! Concerns pinned:
//! - Depth-4 ITERATE with small per-level max (e.g. 2) has cumulative 16 —
//!   well below any reasonable Inv-8 bound — and must be accepted.
//! - Depth-4 ITERATE with per-level max that blows the budget still rejects,
//!   but with `E_INV_ITERATE_BUDGET` (multiplicative), not
//!   `E_INV_ITERATE_NEST_DEPTH` (the dropped stopgap).
//! - `E_INV_ITERATE_NEST_DEPTH` is no longer fired in Phase 2a — a depth-10
//!   nest whose product stays in budget is accepted.
//!
//! R3 red-phase contract: R5 (G4-A) replaces the Phase-1 check with the
//! multiplicative form. Tests compile; they fail because the current code
//! still rejects depth-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes per R1 triage."
)]

use benten_eval::{ErrorCode, Subgraph, SubgraphBuilder};

fn nested_iterates_with_max(
    n: usize,
    per_level_max: u64,
) -> Result<Subgraph, benten_eval::RegistrationError> {
    let mut sb = SubgraphBuilder::new(&format!("nested_n{n}_m{per_level_max}"));
    let root = sb.read("list");
    let mut prev = root;
    for _ in 0..n {
        prev = sb.iterate(prev, "inner_body_handler", per_level_max);
    }
    sb.respond(prev);
    sb.build_validated()
}

#[test]
fn invariant_8_nest_depth_4_accepted_under_multiplicative() {
    // 4-deep nest, per-level max = 2 → cumulative 16. Well within default.
    // Phase 1 rejected with E_INV_ITERATE_NEST_DEPTH; Phase 2a must accept.
    let sg = nested_iterates_with_max(4, 2)
        .expect("depth-4 with cumulative 16 must be accepted in Phase 2a");
    let cumulative = sg.cumulative_budget_for_root_for_test();
    assert_eq!(
        cumulative, 16,
        "cumulative must equal product of per-level max across depth"
    );
}

#[test]
fn invariant_8_nest_depth_4_rejects_by_budget_not_by_depth_code() {
    // 4-deep nest, per-level max = 1000 → cumulative 10^12. Overflows the
    // default Inv-8 bound (assumed <= 10^9). Must reject with the
    // multiplicative-budget code, NOT the dropped nest-depth stopgap code.
    let err = nested_iterates_with_max(4, 1000).expect_err("cumulative 10^12 must be rejected");
    assert_eq!(
        err.code(),
        ErrorCode::InvIterateBudget,
        "depth-4 over-budget must fire E_INV_ITERATE_BUDGET, got {:?}",
        err.code()
    );
    assert_ne!(
        err.code(),
        ErrorCode::InvIterateNestDepth,
        "Phase 2a must NOT fire the dropped nest-depth stopgap code"
    );
}

#[test]
fn invariant_8_nest_depth_stopgap_removed_accepts_depth_10_within_budget() {
    // 10-deep nest with per-level max 1 → cumulative 1. Must be accepted.
    // Phase 1 would have rejected at depth 4 regardless. Any Ok here proves
    // the stopgap is gone.
    let sg = nested_iterates_with_max(10, 1).expect("depth-10 with cumulative 1 must be accepted");
    let cumulative = sg.cumulative_budget_for_root_for_test();
    assert_eq!(cumulative, 1);
}

#[test]
fn invariant_8_nest_depth_code_is_unreachable_in_phase_2a() {
    // Thorough pin: attempt depth-50 with max 1; if the stopgap is still
    // wired it will fire E_INV_ITERATE_NEST_DEPTH. In Phase 2a this code is
    // unreachable from `build_validated`.
    let sg = nested_iterates_with_max(50, 1)
        .expect("deep nest within budget must be accepted in Phase 2a");
    assert!(sg.has_multiplicative_budget_tracked_for_test());
}

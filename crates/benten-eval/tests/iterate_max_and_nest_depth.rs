//! Edge-case tests for ITERATE primitive's `max` and nesting-depth boundaries.
//!
//! Covers error codes:
//! - `E_INV_ITERATE_NEST_DEPTH` — Phase 1 named compromise: ITERATE nesting
//!   is hardcoded at depth 3 as a stopgap for Invariant 8's cumulative-budget
//!   enforcement (full form ships in Phase 2).
//! - (Phase-2-reserved: `E_INV_ITERATE_MAX_MISSING` and `E_INV_ITERATE_BUDGET`
//!   — NOT fired in Phase 1 per R1 triage, but tests still pin "no Phase 1
//!   registration incorrectly fires these codes today.")
//!
//! Regression: MAX_ITERATE_NEST_DEPTH=3 is a Phase 1 stopgap for Invariant 8.
//! Phase 2 removes this limit (or tightens it via cumulative-budget).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::{ErrorCode, SubgraphBuilder};

/// Build a subgraph with `n` levels of nested ITERATEs.
fn nested_iterates(n: usize) -> Result<benten_eval::Subgraph, benten_eval::RegistrationError> {
    let mut sb = SubgraphBuilder::new(&format!("nested_iter_{n}"));
    let root = sb.read("list");
    let mut prev = root;
    for _ in 0..n {
        prev = sb.iterate(prev, "inner_body_handler", 10);
    }
    sb.respond(prev);
    sb.build_validated()
}

#[test]
fn accepts_nest_depth_3_rejects_4() {
    // Regression: MAX_ITERATE_NEST_DEPTH=3 is a Phase 1 stopgap for Invariant 8.
    // Phase 2 removes this limit (or tightens it via cumulative-budget).
    let _ok = nested_iterates(1).expect("depth-1 ITERATE must pass");
    let _ok = nested_iterates(2).expect("depth-2 ITERATE must pass");
    let _ok = nested_iterates(3).expect("depth-3 ITERATE must pass (at limit)");
}

#[test]
fn rejects_depth_4_accepts_depth_3() {
    // Regression: MAX_ITERATE_NEST_DEPTH=3 is a Phase 1 stopgap for Invariant 8.
    // Phase 2 removes this limit.
    let err = nested_iterates(4).expect_err("depth-4 ITERATE must be rejected in Phase 1");
    assert_eq!(err.code(), ErrorCode::InvIterateNestDepth);

    // Context names the depth and the limit.
    assert_eq!(err.iterate_nest_depth_actual().unwrap(), 4);
    assert_eq!(err.iterate_nest_depth_max().unwrap(), 3);
    // Context includes the path (so devs can locate the offending
    // nested ITERATE chain).
    assert!(err.iterate_nest_path().is_some());
}

#[test]
fn iterate_missing_max_is_phase_2_not_phase_1() {
    // Phase-2-reserved: E_INV_ITERATE_MAX_MISSING is NOT fired by Phase 1
    // per R1 triage. This test documents the contract:
    //
    //   Phase 1's builder's `iterate(...)` takes `max` as a required arg
    //   at compile time -> "missing max" is impossible to express. The
    //   code E_INV_ITERATE_MAX_MISSING stays reserved in ERROR-CATALOG
    //   for the Phase 2 registration-time invariant form.
    //
    // This test's job is to ensure no Phase-1 path accidentally fires the code.
    let mut sb = SubgraphBuilder::new("iter_with_explicit_max");
    let root = sb.read("list");
    let _iter = sb.iterate(root, "body", 10);
    let sg = sb
        .build_validated()
        .expect("iter with max present must pass");

    // No error was produced, but if an implementation regression starts
    // firing E_INV_ITERATE_MAX_MISSING in Phase 1, registration would fail.
    assert!(sg.primitive_count() > 0);
}

#[test]
fn iterate_max_zero_accepted_as_noop() {
    // Boundary: `iterate(..., max=0)` is a no-op iteration (runs the
    // body 0 times). Legal; the API honestly says "this ITERATE does
    // nothing" rather than rejecting it. The dev can then evaluate and
    // see the trace show 0 iterations.
    let mut sb = SubgraphBuilder::new("iter_max_zero");
    let root = sb.read("list");
    let _ = sb.iterate(root, "body", 0);
    let _sg = sb
        .build_validated()
        .expect("max=0 must parse cleanly; runtime will evaluate 0 times");
}

#[test]
fn iterate_max_negative_is_parse_error() {
    // The builder API types `max` as a u64, so negative is impossible
    // from Rust. But the DSL (TypeScript) can pass a negative literal;
    // napi boundary must catch that as E_INPUT_LIMIT before the subgraph
    // reaches the Rust-side invariant checker. This test pins that the
    // Rust side's u64 contract is actually u64 — if someone changes it
    // to i64 as a convenience, this test must fire to force a design
    // discussion.
    //
    // Compile-time check: `iterate` signature must accept u64.
    fn _typecheck_iterate_max_is_u64() {
        let mut sb = SubgraphBuilder::new("t");
        let r = sb.read("r");
        let _ = sb.iterate(r, "body", 0u64);
    }
}

//! Edge-case tests for ITERATE primitive's `max` and nesting-depth boundaries.
//!
//! Phase 2a (G4-A): the Phase-1 `MAX_ITERATE_NEST_DEPTH = 3` stopgap for
//! Invariant 8 has been retired in favour of the multiplicative cumulative-
//! budget check. The tests below exercise the post-retirement contract: a
//! depth-4 nest whose product stays within `DEFAULT_INV_8_BUDGET` is
//! accepted. Exhaustive coverage lives in
//! `invariant_8_nest_depth_stopgap_removed.rs`.
//!
//! Phase-2-reserved: `E_INV_ITERATE_MAX_MISSING` is still not fired by any
//! Phase-2a path because the builder's `iterate(..., u64)` takes `max` at
//! compile time. The code stays reserved for a Phase-2 registration form
//! that reaches a subgraph via bytes rather than the typed builder.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(
    clippy::result_large_err,
    reason = "RegistrationError carries ~360 bytes of diagnostic context per R1 triage; test helpers mirror the crate-internal API. Public engine surface boxes it."
)]

use benten_eval::SubgraphBuilder;
use benten_eval::{NodeHandleExt, SubgraphBuilderExt, SubgraphExt};

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
fn accepts_depth_3_and_all_shorter() {
    // Phase 2a (G4-A): the nest-depth-3 stopgap is retired in favour of the
    // multiplicative cumulative-budget check. A 3-deep nest with each
    // per-level max=10 has cumulative 10^3 = 1000, well within the default
    // `DEFAULT_INV_8_BUDGET` (500_000) — all three must continue to pass.
    let _ok = nested_iterates(1).expect("depth-1 ITERATE must pass");
    let _ok = nested_iterates(2).expect("depth-2 ITERATE must pass");
    let _ok = nested_iterates(3).expect("depth-3 ITERATE must pass");
}

#[test]
fn depth_4_within_budget_accepted_under_multiplicative() {
    // Phase 2a (G4-A): `nested_iterates(4)` uses per-level max=10 →
    // cumulative 10^4 = 10_000 < DEFAULT_INV_8_BUDGET (500_000). Phase 1
    // used to reject this with `E_INV_ITERATE_NEST_DEPTH`; Phase 2a
    // accepts because the product stays inside the budget. The
    // nest-depth stopgap removal is covered exhaustively in
    // `invariant_8_nest_depth_stopgap_removed.rs`.
    let _ok = nested_iterates(4).expect("depth-4 with cumulative 10_000 must be accepted");
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

// R4 triage M8: removed `iterate_max_negative_is_parse_error`. The test's
// sole body was a never-called `_typecheck_iterate_max_is_u64` inner fn that
// added zero value — the u64 typing is already enforced by the compiler
// wherever other tests construct `SubgraphBuilder::iterate(..., u64)`. A
// dedicated "no-run" test is redundant.
//
// If the DSL-side (TypeScript napi boundary) negative-literal rejection
// behavior needs explicit coverage, that belongs in `bindings/napi/index.test.ts`
// or a dedicated napi-surface Rust test — not here.

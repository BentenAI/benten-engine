//! Phase 2a R3 security — Inv-8 isolated-CALL budget bypass attempt
//! (code-as-graph Major #2 / §9.12 addendum).
//!
//! **Attack class.** Inv-8 multiplicative cumulative budget composes
//! through ITERATE + CALL: the static registration-time ceiling is the
//! product of declared ITERATE.max values along every DAG path, multiplied
//! through CALL. But CALL has two semantics: `isolated: true` (default,
//! per validated decision #4 — callee runs under its own grant) and
//! `isolated: false` (callee inherits caller's context).
//!
//! Code-as-graph Major #2 locks:
//! - `isolated: true` CALL RESETS the budget to the callee grant's bound.
//! - `isolated: false` CALL INHERITS (multiplies through).
//!
//! The bypass attempt: if an isolated CALL accidentally INHERITS the
//! parent's remaining budget (or leaks it back on return), a compromised
//! outer handler could exhaust a privileged callee's budget by calling it
//! repeatedly in an ITERATE. The per-call isolation's whole point is that
//! the callee is bounded by its OWN grant's budget; attack tests that the
//! parent cannot leak budget INTO or OUT OF the isolation frame.
//!
//! **Prerequisite.** Two handlers: outer wraps an ITERATE around a CALL
//! (isolated:true) to inner. Inner has its own grant with a declared
//! cumulative budget.
//!
//! **Attack sequence.**
//!  1. Outer handler: ITERATE(100) { CALL(inner, isolated:true) }.
//!  2. Inner handler: ITERATE(50) { WRITE }. Declared budget via inner
//!     grant = 50.
//!  3. Without proper isolation, the cumulative budget seen by inner =
//!     max(100 * 50 = 5000) — inner can write up to 5000 times.
//!  4. With proper isolation (code-as-graph Major #2 fix): inner's budget
//!     per CALL is 50; outer's budget is 100 CALLs; inner cannot exceed
//!     its own 50 per call.
//!
//! **Impact.** Privileged callee's budget exhaustion via parent looping;
//! budget-bounded cost-of-service model breaks.
//!
//! **Recommended mitigation.** §9.12 addendum: `isolated: true` resets
//! to callee grant's declared bound; `isolated: false` inherits. G4-A's
//! `invariants/budget.rs` owns the logic; registration-time check
//! computes cumulative per CALL-isolation boundary.
//!
//! **Red-phase contract.** G4-A lands multiplicative cumulative + isolated-
//! CALL reset semantics. Today Inv-8 fires only the Phase-1 scalar + nest-
//! depth-3 stopgap. Test asserts the bypass is blocked; fails today.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// Major #2: isolated:true CALL MUST reset multiplicative budget to the
/// callee grant's declared bound. Parent looping cannot leak budget.
#[test]
#[ignore = "phase-2a-pending: Inv-8 multiplicative + isolated-CALL reset semantics land in G4-A per plan §9.12 + code-as-graph Major #2. Drop #[ignore] once invariants/budget.rs wires the isolation-boundary reset."]
fn inv_8_isolated_call_resets_to_callee_grant() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Target API path (G4-A):
    //
    //     let inner = SubgraphSpec::builder()
    //         .handler_id("inner")
    //         .iterate(50, |b| b.write(|w| w.label("post")))
    //         .respond()
    //         .build();
    //     let inner_id = engine.register_subgraph(inner).unwrap();
    //
    //     let outer = SubgraphSpec::builder()
    //         .handler_id("outer")
    //         .iterate(100, |b| {
    //             b.call(|c| c.handler_id(&inner_id).isolated(true))
    //         })
    //         .respond()
    //         .build();
    //
    //     // Inner grant declares cumulative budget = 50.
    //     // Attack: outer tries to run 100 * 50 iterations.
    //     //
    //     // Under code-as-graph Major #2 fix: isolated CALL resets the
    //     // budget to inner grant's 50 per call. Overall: outer makes 100
    //     // CALLs, each inner call does <=50 writes → 100 * 50 = 5000
    //     // total writes BUT the INDIVIDUAL budget ceiling for the inner
    //     // body remains 50, not inflated via multiplicative composition.
    //     //
    //     // The registration-time check must not compute inner's static
    //     // ceiling as outer * inner (= 5000); it must cap at inner's
    //     // own declared bound.
    //     let outer_id = engine.register_subgraph(outer).unwrap();
    //
    //     // Drive a call that would exceed inner's 50-iter bound if the
    //     // budget were inherited. Expect success up to 50 per inner
    //     // call, NOT 5000 accumulated-in-inner.
    //     let outcome = engine.call(&outer_id, "default", Node::empty()).unwrap();
    //     // The structural check fires at registration if inner's
    //     // declared budget is inadequate for its body — catching a
    //     // mis-configured inner. For the multiplicative-through-CALL
    //     // bypass: assertion is that TraceStep budget accounting
    //     // resets at each CALL isolation boundary.
    //     assert_eq!(outcome.completed_iterations(), Some(100),
    //         "outer ITERATE completes 100 CALLs; inner does NOT accumulate \
    //          budget across calls");

    let _ = engine;
    panic!(
        "red-phase: Inv-8 multiplicative + isolated-CALL reset not yet \
         present. G4-A to land per plan §9.12 + code-as-graph Major #2."
    );
}

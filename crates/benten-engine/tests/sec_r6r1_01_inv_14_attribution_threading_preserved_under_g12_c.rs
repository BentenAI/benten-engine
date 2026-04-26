//! G12-C sec-pre-r1-13 carry: assert the Phase-2a `sec-r6r1-01` Inv-14
//! attribution-threading wiring (live, not dead-coded) is NOT regressed
//! during the `Subgraph` type relocation from `benten-eval` to `benten-core`.
//!
//! Per `r1-security-auditor.json` sec-pre-r1-13 + line 148: "sec-r6r1-01
//! (Inv-14 dead-coded wiring closed) — D7 hybrid recommendation requires
//! per-host-fn live check that emits TraceStep with AttributionFrame inherited
//! from dispatching SANDBOX primitive. sec-pre-r1-03 deliverable +
//! sec-pre-r1-13 G12-A non-regression note both pin this. R1 ratifies."
//!
//! G12-C touches the Subgraph type (used by SubgraphSpec which is used by
//! handler dispatch), so a careless migration could re-detach the Inv-14
//! AttributionFrame threading. This test pins the wiring is live post-G12-C.
//!
//! TDD red-phase. Owner: R5 G12-C (qa-r4-02 sec-pre-r1-13 carry; R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-C red-phase: Inv-14 wiring liveness check post-migration not yet wired"]
fn inv_14_attribution_frame_threaded_through_subgraph_dispatch_post_g12_c() {
    // Drive: register a handler with a SANDBOX primitive (or any primitive
    // that emits a TraceStep); execute via Engine::call; inspect the emitted
    // TraceStep's AttributionFrame; assert it carries the dispatching
    // primitive's frame (NOT default / NOT empty).
    //
    // Phase-2a `attribution_non_regression.rs` already pins the wiring; this
    // file pins the SAME property after G12-C migration.
    todo!(
        "R5 G12-C: replicate the attribution_non_regression.rs assertion \
         against a benten_core::Subgraph-dispatched handler"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: AttributionFrame inherited across primitive boundary not yet asserted"]
fn attribution_frame_inherits_correctly_across_subgraph_primitive_boundary_post_migration() {
    // Pin the per-host-fn live check (sec-r6r1-01 D7 hybrid): emitted TraceStep
    // carries inherited AttributionFrame from dispatcher; not regressed by
    // G12-C type relocation.
    todo!(
        "R5 G12-C: drive a 2-primitive subgraph (READ -> RESPOND); collect TraceSteps; \
         assert RESPOND's AttributionFrame matches READ's parent frame"
    )
}

#[test]
#[ignore = "R5 G12-C red-phase: G12-A budget-exhausted non-regression carry not yet wired"]
fn g12a_budget_exhausted_does_not_bypass_attribution_frame_post_g12_c() {
    // sec-pre-r1-13 G12-A non-regression note: "BudgetExhausted runtime
    // emission wiring does not bypass the AttributionFrame routing path."
    // G12-C migration must preserve this — the Subgraph type relocation
    // should not detach the BudgetExhausted emission from AttributionFrame.
    todo!(
        "R5 G12-C: drive a budget-exhausting handler; assert the BudgetExhausted \
         TraceStep carries a non-empty AttributionFrame post-migration"
    )
}

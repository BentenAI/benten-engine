//! Phase 2b R3-B — Inv-4 sandbox-depth registration-time check (G7-B).
//!
//! Pin source: plan §3 G7-B.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-B pending — Inv-4 registration-time"]
fn invariant_4_sandbox_nest_depth_rejected_at_registration() {
    // Plan §3 G7-B — STATIC SubgraphSpec analysis: a SubgraphSpec
    // declaring 5 nested SANDBOX primitives (depth > max_nest_depth=4)
    // is rejected at registration with E_INV_SANDBOX_DEPTH.
    //
    // Use `testing_install_handler_with_sandbox_depth(engine, 5)` helper
    // (R2 §9 anticipated, G7-B scope).
    //
    // Distinct from runtime: registration-time analysis is purely
    // structural (counts SANDBOX nodes in the SubgraphSpec call-graph);
    // runtime analysis (separate test) handles TRANSFORM-computed
    // SANDBOX targets where the static depth is unknowable.
    todo!("R5 G7-B — testing_install_handler_with_sandbox_depth + rejection assert");
}

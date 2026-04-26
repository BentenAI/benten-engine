//! Phase 2b R3-B — Inv-7 sandbox-output registration-time check (G7-B).
//!
//! Pin source: plan §4.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-B pending — Inv-7 registration-time"]
fn invariant_7_output_declared_max_rejected_at_registration() {
    // Plan §4 — STATIC SubgraphSpec analysis: a SANDBOX node that
    // declares `output_max_bytes` outside the engine's allowed range
    // (e.g., 0 or > engine_max_output_bytes) is rejected at
    // registration with E_INV_SANDBOX_OUTPUT.
    //
    // Test:
    //   1. SubgraphSpec with SANDBOX node + output_max_bytes = 0
    //      → E_INV_SANDBOX_OUTPUT at registration.
    //   2. SubgraphSpec with SANDBOX node + output_max_bytes
    //      = engine_max_output_bytes + 1 → E_INV_SANDBOX_OUTPUT.
    //   3. SubgraphSpec with SANDBOX node + output_max_bytes = 1024
    //      (within range) → registers cleanly.
    todo!("R5 G7-B — registration-time output_max_bytes range check");
}

//! Phase 2b R3-B — Inv-4 sandbox-depth across CALL boundary integration
//! tests (G7-B).
//!
//! Pin sources: wsa-D20, D20 + Phase-2a Inv-14 carry.
//!
//! These tests exercise the depth-inheritance pattern (D20) at the
//! cross-crate integration level (engine + eval together), not at the
//! eval unit level (those tests live in invariant_4_runtime.rs).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-B pending — Inv-4 cross-CALL boundary integration"]
fn invariant_4_sandbox_depth_crosses_call_boundary() {
    // wsa-D20 — engine-level integration. Handler A (registered with
    // engine) SANDBOXes → CALLs handler B (registered) → SANDBOXes.
    // Through the engine.call surface, the cumulative
    // sandbox_depth crosses the CALL boundary.
    //
    // Assertion: engine.call("A") evaluates to depth-2 SANDBOX inside
    // B; depth-3 SANDBOX inside C (called from B) trips the configured
    // max=2 boundary with E_INV_SANDBOX_DEPTH (or
    // E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED).
    todo!("R5 G7-B — register A/B/C handlers + engine.call + depth-3 trap");
}

#[test]
#[ignore = "Phase 2b G7-B pending — D20 + Inv-14 carry integration"]
fn invariant_4_end_to_end_with_attribution_frame() {
    // D20 + Phase-2a Inv-14 carry — the depth counter rides on
    // AttributionFrame (Phase-2a sec-r6r1-01 closure shape).
    //
    // Test:
    //   1. Engine.call("A_with_actor", input) — actor X dispatches A.
    //   2. A SANDBOXes (depth 1; frame.actor=X, frame.handler=A,
    //      frame.sandbox_depth=1).
    //   3. A CALLs B with attenuated cap (Inv-14 attribution check).
    //   4. B's frame inherits actor=X, handler=B, sandbox_depth=1.
    //   5. B SANDBOXes (frame.sandbox_depth=2).
    //   6. White-box assertion: engine's audit log captures the chain
    //      (X, A, depth=1) → (X, B, depth=1 inherited) → (X, B, depth=2);
    //      the cap-attenuation chain (Inv-14) is preserved.
    todo!("R5 G7-B — full attribution chain + depth + Inv-14 audit");
}

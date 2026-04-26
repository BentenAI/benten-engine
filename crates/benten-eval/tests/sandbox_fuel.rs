//! Phase 2b R3-B — SANDBOX fuel-axis unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A (fuel limit), dx-r1-2b-5 (default 1_000_000).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — fuel exhaustion routing"]
fn sandbox_fuel_exhausts_routes_e_sandbox_fuel_exhausted() {
    // Plan §3 G7-A — module enters tight `loop ... br 0 ... end` with
    // budget = N; assert E_SANDBOX_FUEL_EXHAUSTED fires within N
    // wasmtime fuel-units (NOT wallclock-bounded).
    //
    // Distinct from the wallclock axis (D21 priority FUEL > OUTPUT,
    // WALLCLOCK > FUEL — wallclock would mask fuel if both eligible).
    todo!("R5 G7-A — fixture infinite_loop.wat + budget=N assertion");
}

#[test]
#[ignore = "Phase 2b G7-A pending — dx-r1-2b-5 default budget"]
fn sandbox_fuel_default_1_000_000_runs_canonical_fixture() {
    // dx-r1-2b-5 — default fuel budget = 1_000_000 wasmtime units;
    // canonical echo fixture completes well within this default
    // (consumed << 1M for a constant-return).
    //
    // Test: `engine.sandbox_call(echo_cid, ManifestRef::default(), input)`
    // succeeds with `result.fuel_consumed < 1_000_000`.
    todo!("R5 G7-A — assert echo fixture under default fuel budget");
}

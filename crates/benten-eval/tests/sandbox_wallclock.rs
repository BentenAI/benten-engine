//! Phase 2b R3-B — SANDBOX wallclock-axis unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A, D24-RESOLVED (30s default / 5min max),
//! D6 + D24 (per-handler override via SubgraphSpec.primitives).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — wallclock exhaustion routing"]
fn sandbox_wallclock_kills_routes_e_sandbox_wallclock_exceeded() {
    // Plan §3 G7-A — module enters tight loop with sufficient fuel that
    // wallclock fires before fuel does. Assertion:
    // `E_SANDBOX_WALLCLOCK_EXCEEDED` fires within `SandboxConfig.wallclock_ms`
    // bounded by epoch-deadline-async-yield (D27 async-support enabled).
    //
    // D21: WALLCLOCK > FUEL — if both eligible at the trap callback,
    // WALLCLOCK is selected. Use a configuration where fuel is generous
    // so the priority isn't tested here (separate test in
    // sandbox_severity_priority.rs).
    todo!("R5 G7-A — wallclock=100ms + fuel=u64::MAX + infinite_loop fixture");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — D24-RESOLVED defaults"]
fn sandbox_wallclock_default_30s_max_5min() {
    // D24-RESOLVED — `SandboxConfig.wallclock_ms` defaults to 30000 (30s);
    // values above 300_000 (5min) rejected at SubgraphSpec validation OR
    // saturated to 5min at SandboxOptions construction.
    //
    // Test 1: `SandboxConfig::default().wallclock_ms == 30_000`.
    // Test 2: per-handler override of 60_000 accepted (within 5min).
    // Test 3: per-handler override of 600_000 rejected (>5min cap) with
    //         typed error `E_SANDBOX_WALLCLOCK_INVALID` or saturated.
    todo!("R5 G7-A — assert default + per-handler override + cap rejection");
}

#[test]
#[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — D24 + D6 per-handler override"]
fn sandbox_wallclock_per_handler_override_via_subgraphspec_primitives() {
    // D24 + D6 — `SubgraphSpec.primitives` widening (G12-D) carries
    // per-primitive `wallclock_ms` config. SANDBOX primitive config:
    //   { kind: "sandbox", wallclock_ms: 5000, ... }
    //
    // Test: register a SubgraphSpec where one SANDBOX node sets
    //   wallclock_ms=5000; engine honors it (overrides default 30s);
    //   fixture that runs 6s trips wallclock at ~5s.
    //
    // Cross-test: SubgraphSpec.primitives must carry the value through
    // canonical-bytes round-trip (covered by G12-D R3-E test).
    todo!("R5 G7-A + G12-D — wire per-handler wallclock_ms via primitives");
}

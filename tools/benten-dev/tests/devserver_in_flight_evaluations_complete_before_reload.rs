//! G12-B red-phase: re-validate Phase-2a in-flight-evaluation semantics
//! against the **real Engine** post-routing-refactor.
//!
//! Per plan §3.2 G12-B must-pass tests: "devserver_in_flight_evaluations_complete_before_reload
//! (re-validated)."
//!
//! Property pin: when devserver receives a hot-reload signal mid-flight, any
//! in-progress `Engine::call(handler_id, ...)` against the OLD handler version
//! completes (via the existing CallGuard / ReloadCoordinator) BEFORE the new
//! handler version replaces it via `Engine::register_subgraph`.
//!
//! The Phase-2a test pinned this against the in-memory stub. Routing through
//! the real engine (G12-B) must preserve the property end-to-end.
//!
//! TDD red-phase. Owner: R5 G12-B (qa-r4-01 R3-followup).

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "R5 G12-B red-phase: in-flight completion through real engine not yet wired"]
fn devserver_inflight_call_completes_against_v1_before_engine_register_subgraph_swaps_to_v2() {
    // Drive:
    //   1. Spawn devserver with handler-v1; start a long-running Engine::call(v1).
    //   2. Trigger hot-reload to v2 mid-call.
    //   3. Assert the in-flight call completes against the v1 SubgraphSpec
    //      (NOT v2) — even though the file mutation already happened.
    //   4. Assert subsequent calls hit v2.
    todo!(
        "R5 G12-B: spawn devserver; issue blocking call; trigger reload; \
           assert call result reflects v1 SubgraphSpec; subsequent call reflects v2"
    )
}

#[test]
#[ignore = "R5 G12-B red-phase: ReloadCoordinator preservation not yet wired"]
fn devserver_reload_coordinator_remains_responsible_for_concurrency_under_engine_routing() {
    // Pin: ReloadCoordinator + CallGuard surfaces are PRESERVED by G12-B
    // refactor (per plan §3.2 G12-B "preserve the Phase-2a ReloadCoordinator
    // / CallGuard (concurrency coordination, not storage)").
    todo!("R5 G12-B: assert ReloadCoordinator surface present + behaviorally identical to Phase-2a")
}

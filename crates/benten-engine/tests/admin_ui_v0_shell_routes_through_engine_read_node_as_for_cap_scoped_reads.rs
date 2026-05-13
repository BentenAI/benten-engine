//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! routing through `Engine::read_node_as` for cap-scoped reads.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 2 (substantive runtime trace); closes CLAUDE.md baked-in #18
//! (Class B β `read_node_as` consumer; admin UI v0 is the first
//! non-trusted-principal consumer of the seam shipped at PR #184).
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #18: admin UI v0 runs under its own per-plugin
//! DID (admin-UI-DID). Reads MUST flow through `Engine::read_node_as`
//! with the active walk-time principal threaded — the public Class B β
//! seam (`crates/benten-engine/src/engine_wait.rs:1082`). The
//! `pub(crate) Engine::read_node` surface is RESERVED for engine
//! internals (IVM, sync, audit) per baked-in #18 — admin UI plugin code
//! must NEVER reach for it.
//!
//! This pin is the **runtime-trace half** of the §3.6f SHAPE-not-SUBSTANCE
//! pair. The grep-assert companion is
//! `admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs`.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 canary wires this. Pin source: r2-test-landscape.md §2.6 row 2 + CLAUDE.md baked-in #18 Class B β. Substantive runtime trace: actually invoke an admin UI subgraph + capture trace events + assert read calls hit read_node_as branch with admin-UI-DID principal."]
fn admin_ui_v0_shell_routes_through_engine_read_node_as_for_cap_scoped_reads() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Invoke the admin UI subgraph that renders the "Workflows"
    //   // route — a real evaluator walk against a real engine.
    //   let trace = harness.trace_capture(|h| {
    //       h.dispatch_admin_ui_route("workflows")
    //   });
    //
    //   // Per pim-18 §3.6f: runtime trace MUST show read_node_as
    //   // invocations, not read_node:
    //   let read_calls = trace.calls_to("Engine::read_node_as");
    //   assert!(
    //       !read_calls.is_empty(),
    //       "Admin UI workflows-route render MUST invoke \
    //        Engine::read_node_as (Class B β seam); trace shows ZERO \
    //        invocations of that surface — admin UI is bypassing cap-policy"
    //   );
    //
    //   // Each read call MUST thread the admin-UI-DID as the active
    //   // principal, NOT the engine's internal trusted-principal handle:
    //   for call in &read_calls {
    //       assert_eq!(
    //           call.principal_arg,
    //           admin_ui_did,
    //           "Engine::read_node_as principal arg MUST be \
    //            admin-UI-DID per CLAUDE.md baked-in #18; saw {:?}",
    //           call.principal_arg,
    //       );
    //   }
    //
    //   // Defense-in-depth: zero invocations of the pub(crate) seam:
    //   assert_eq!(
    //       trace.calls_to("Engine::read_node").len(),
    //       0,
    //       "Admin UI MUST NEVER call pub(crate) Engine::read_node — \
    //        that surface is engine-internal-only per CLAUDE.md \
    //        baked-in #18. Trace shows {:?} invocations.",
    //       trace.calls_to("Engine::read_node"),
    //   );
    //
    // OBSERVABLE consequence: cap-policy fires on every admin UI read.
    // Would FAIL if read pathway short-circuited the public seam.
    unimplemented!(
        "G24-A wires admin UI routes-through-read_node_as runtime-trace \
         pin. Pair with grep-assert in \
         admin_ui_v0_source_never_calls_engine_read_node_only_engine_read_node_as.rs"
    );
}

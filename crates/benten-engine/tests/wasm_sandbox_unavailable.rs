//! R3-A + R4-FP RED-PHASE pins: wasm32-unknown-unknown SANDBOX
//! unavailable path is observable across ALL 4 entry points (G13-C +
//! G14-C + G14-D + G16-D wave 3+; br-r1-3 + br-r4-r1-2 + Ben's D3
//! LOAD-BEARING decision).
//!
//! Pin sources:
//!
//! - r2-test-landscape §2.1 G13-C row
//!   `wasm32_unknown_unknown_browser_backend_e_sandbox_unavailable_on_wasm_path_observable`
//!   (br-r1-3; ALREADY pinned at R3-A wave-3) — install_module entry point
//! - br-r1-3 fix-brief item (2) — register_module_bytes entry point (NEW at R4-FP)
//! - br-r1-3 fix-brief item (3) — call→sandbox-handler dispatch entry point (NEW at R4-FP)
//! - br-r1-3 fix-brief item (4) — atrium-replicated sandbox invocation entry point (NEW at R4-FP)
//!
//! ## D3 LOAD-BEARING decision (Ben 2026-05-04 R4-FP)
//!
//! "SANDBOX uniformity = pin ALL 4 entry points (not just 1)." The
//! original R3-A landed pin (1) — the install_module path. R4-FP adds
//! pins (2)/(3)/(4) covering the remaining 3 SANDBOX entry points so
//! a R5 implementer who wires the install_module arm + silently no-ops
//! on the other 3 entry points fires THIS file's pins, not just the
//! single primitive_host.rs:1022-1045 dispatch site.
//!
//! ## What this pins
//!
//! On wasm32-unknown-unknown (browser), wasmtime is unavailable
//! (wasmtime cannot recursively host itself in a browser-tab WASM
//! runtime). SANDBOX primitive execution on this target must surface
//! `E_SANDBOX_UNAVAILABLE_ON_WASM` typed error from EVERY production
//! entry point that can reach SANDBOX dispatch:
//!
//! 1. **install_module** (G13-C wave 3) — DSL-driven SANDBOX manifest
//!    install at module registration.
//! 2. **register_module_bytes** (G14-C wave 4b) — direct module-bytes
//!    registration carrying a SANDBOX handler.
//! 3. **call→SANDBOX-handler** (G14-D wave 5a / G19) — runtime CALL
//!    primitive dispatching into a registered SANDBOX handler-id.
//! 4. **atrium-replicated SANDBOX invocation** (G16-D wave 5+) — sync-
//!    replica receives Atrium-replicated SANDBOX-bearing data + the
//!    receiver dispatches into local SANDBOX execution.
//!
//! Each entry point must surface the SAME typed error
//! (`E_SANDBOX_UNAVAILABLE_ON_WASM`), uniformly. Defends against the
//! failure shape "fix landed at one entry point but the other 3
//! silently no-op or panic" — exactly the structural shape that
//! produced 24 cumulative producer/consumer drift instances in
//! Phase-2b (per `feedback_3_plus_recurrence_deep_sweep`).
//!
//! Per pim-2 §3.6b end-to-end: each pin drives a distinct production
//! entry point + asserts an observable consequence (typed error
//! reaching the caller; not panic; not silent success).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-C wave 3 wires the wasm-target SANDBOX unavailable path at install_module entry point"]
fn wasm32_unknown_unknown_browser_backend_e_sandbox_unavailable_on_wasm_path_observable() {
    // br-r1-3 pin (entry point 1 of 4 — install_module).
    // G13-C implementer wires this:
    //
    // Option A — runtime test under wasm-bindgen-test (requires wasm32 target):
    //
    //   #[cfg(target_arch = "wasm32")]
    //   {
    //       let engine = browser_engine_with_browser_backend();
    //       let module_with_sandbox = build_subgraph_with_sandbox_node();
    //       let result = engine.execute(module_with_sandbox);
    //       assert!(matches!(
    //           result.unwrap_err(),
    //           benten_engine::EngineError::Sandbox(
    //               benten_eval::SandboxError::UnavailableOnWasm
    //           ),
    //       ));
    //   }
    //
    // Option B — native source-cite assertion:
    //
    //   // The host-side primitive dispatch arm in
    //   // `crates/benten-engine/src/primitive_host.rs::PrimitiveHost`
    //   // must contain the wasm-arch-conditional SANDBOX-unavailable
    //   // error path (per br-r1-3 + §3.5b HARDENED point 3 — symbol-form
    //   // for high-churn surface).
    //   let src = std::fs::read_to_string("crates/benten-engine/src/primitive_host.rs").unwrap();
    //   assert!(src.contains("UnavailableOnWasm")
    //         || src.contains("E_SANDBOX_UNAVAILABLE_ON_WASM"),
    //       "primitive_host.rs MUST surface the wasm32 SANDBOX-unavailable typed error per br-r1-3");
    //
    // OBSERVABLE consequence: a browser-side SANDBOX call returns a
    // typed `UnavailableOnWasm` error rather than panic or generic
    // failure. Defends against the regression where wasm32 builds
    // silently skip SANDBOX primitives (which would expose the
    // browser tab to "module loaded but did not enforce its limits"
    // failure shape).
    //
    // This pin is companion to G19-D's `ts_surface_parity_meta_test`
    // (which checks the TS DSL exposes the same error code) — both
    // sides pin the same browser-side observable.
    unimplemented!(
        "G13-C wires runtime/source-cite assertion for E_SANDBOX_UNAVAILABLE_ON_WASM browser-side observability at install_module"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-C wave 4b wires fail-loud E_SANDBOX_UNAVAILABLE_ON_WASM at register_module_bytes entry point (br-r1-3 + br-r4-r1-2 + D3)"]
fn wasm32_unknown_unknown_browser_register_module_bytes_with_sandbox_handler_returns_e_sandbox_unavailable_on_wasm()
 {
    // br-r1-3 fix-brief item (2) + br-r4-r1-2 + D3 LOAD-BEARING pin —
    // entry point 2 of 4. G14-C implementer wires this:
    //
    // Option A — runtime test under wasm-bindgen-test:
    //
    //   #[cfg(target_arch = "wasm32")]
    //   {
    //       use benten_napi::wasm_browser::browser_runtime_available;
    //       assert!(browser_runtime_available(),
    //           "this test runs in browser context");
    //
    //       let engine = browser_engine_with_browser_backend();
    //       let bytes = test_fixture_module_bytes_with_sandbox_handler();
    //
    //       // register_module_bytes carrying a SANDBOX manifest in the
    //       // module table fails with the typed UnavailableOnWasm error
    //       // — NOT a generic "module rejected" or panic, NOT silent
    //       // success-but-no-effect:
    //       let result = engine.register_module_bytes("compute:safe-default", &bytes);
    //       assert!(matches!(
    //           result.unwrap_err(),
    //           benten_engine::EngineError::Sandbox(
    //               benten_eval::SandboxError::UnavailableOnWasm
    //           ),
    //       ));
    //   }
    //
    // Option B — native source-cite + module-bytes registry surface:
    //
    //   let src = std::fs::read_to_string(
    //       "crates/benten-engine/src/module_registry.rs"  // or wherever G14-C
    //                                                      // wires register_module_bytes
    //   ).unwrap();
    //   assert!(src.contains("UnavailableOnWasm")
    //         || src.contains("E_SANDBOX_UNAVAILABLE_ON_WASM"),
    //       "register_module_bytes wasm32 arm MUST surface the typed UnavailableOnWasm \
    //        error per br-r1-3 fix-brief item (2) + D3 LOAD-BEARING (uniformity across \
    //        all 4 SANDBOX entry points)");
    //
    // OBSERVABLE consequence: a browser-side `engine.registerModuleBytes(...)`
    // call carrying a SANDBOX-bearing module surfaces the typed
    // UnavailableOnWasm error rather than silently registering a
    // module that will fail at execution time (or worse, succeed at
    // registration but not execution and never observe the gate).
    //
    // Defends against the failure shape "G14-C wires register_module_bytes
    // for browser bundles but forgets to gate SANDBOX-bearing modules
    // — the gate fires only at execute time (entry point 3) and the
    // user can register apparently-good module bytes that silently
    // bypass SANDBOX limits."
    unimplemented!(
        "G14-C wires E_SANDBOX_UNAVAILABLE_ON_WASM at register_module_bytes entry point per br-r1-3 + br-r4-r1-2 + D3"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D / G19 wave 5+ wires fail-loud E_SANDBOX_UNAVAILABLE_ON_WASM at CALL→SANDBOX-handler dispatch entry point (br-r1-3 + br-r4-r1-2 + D3)"]
fn wasm32_unknown_unknown_browser_call_primitive_into_sandbox_handler_returns_e_sandbox_unavailable_on_wasm()
 {
    // br-r1-3 fix-brief item (3) + br-r4-r1-2 + D3 LOAD-BEARING pin —
    // entry point 3 of 4. G14-D / G19 implementer wires this:
    //
    // Shape — runtime test under wasm-bindgen-test (the CALL primitive
    // is the production dispatch surface for handler invocation):
    //
    //   #[cfg(target_arch = "wasm32")]
    //   {
    //       let engine = browser_engine_with_browser_backend();
    //
    //       // Pre-register a SANDBOX handler (assume install_module
    //       // earlier returned UnavailableOnWasm, so this would not
    //       // succeed in a real browser; for the test, we synthesize
    //       // the registry state to drive the CALL→SANDBOX dispatch
    //       // arm directly):
    //       seed_sandbox_handler_registration_for_test(&engine, "h:compute");
    //
    //       // CALL primitive routed at handler-id-router ⇒ SANDBOX:
    //       let sg = subgraph_with_call_into_sandbox_handler("h:compute");
    //       let result = engine.run(sg, /* input */).await;
    //
    //       // Dispatch returns the typed UnavailableOnWasm error —
    //       // NOT a generic "handler not found" + NOT a silent no-op:
    //       assert!(matches!(
    //           result.unwrap_err(),
    //           benten_engine::EngineError::Sandbox(
    //               benten_eval::SandboxError::UnavailableOnWasm
    //           ),
    //       ));
    //   }
    //
    // Source-cite alternative: assert the handler-id-router (G14-D /
    // G19 dispatch site) contains the wasm32 conditional + typed-error
    // emission for the CALL→SANDBOX arm.
    //
    // OBSERVABLE consequence: a browser-side `engine.run()` that
    // routes through CALL→SANDBOX dispatch surfaces the typed error
    // rather than panic or skip-silently. Defends against the failure
    // shape "register_module_bytes refuses SANDBOX modules + the CALL
    // dispatch arm has no defensive gate, so a path that bypasses
    // register_module_bytes (e.g. via a module imported through an
    // alternate route) silently dispatches into wasmtime that doesn't
    // exist on wasm32 + panics at the wasmtime::Engine::new call site."
    unimplemented!(
        "G14-D / G19 wires E_SANDBOX_UNAVAILABLE_ON_WASM at CALL→SANDBOX-handler dispatch per br-r1-3 + br-r4-r1-2 + D3"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-D wave 5+ wires fail-loud E_SANDBOX_UNAVAILABLE_ON_WASM at atrium-replicated SANDBOX invocation receive (br-r1-3 + br-r4-r1-2 + D3)"]
fn wasm32_unknown_unknown_browser_atrium_replicated_sandbox_handler_returns_e_sandbox_unavailable_on_wasm()
 {
    // br-r1-3 fix-brief item (4) + br-r4-r1-2 + D3 LOAD-BEARING pin —
    // entry point 4 of 4 (the most subtle case). G16-D implementer
    // wires this:
    //
    // Shape — sync-replica receive of Atrium-replicated SANDBOX-bearing
    // data on the browser thin-client side. Per CLAUDE.md baked-in #17,
    // the browser is a thin-client view; the full peer is the source
    // of truth for SANDBOX execution. But the thin-client may receive
    // a Loro-merged Version Node whose payload includes a SANDBOX
    // invocation event (e.g. a logged invocation in an audit trail).
    // The browser MUST NOT attempt to dispatch into local SANDBOX —
    // it MUST surface the typed UnavailableOnWasm error if any code
    // path tries.
    //
    //   #[cfg(target_arch = "wasm32")]
    //   {
    //       let engine = browser_engine_with_browser_backend();
    //
    //       // Synthesize an Atrium-replicated SANDBOX-bearing event
    //       // arriving on the thin-client (the production path is
    //       // benten-sync's sync-replica receive arm; on browser this
    //       // would be the WebSocket/data-channel receive path that
    //       // routes through to the engine):
    //       let event = atrium_replicated_event_carrying_sandbox_dispatch();
    //
    //       // The receiver dispatches into local SANDBOX execution
    //       // (production code path that processes Atrium events that
    //       // reference handler invocations). On wasm32 this MUST
    //       // surface the typed error rather than dispatch into a
    //       // wasmtime that doesn't exist:
    //       let result = engine.process_atrium_event(event);
    //       assert!(matches!(
    //           result.unwrap_err(),
    //           benten_engine::EngineError::Sandbox(
    //               benten_eval::SandboxError::UnavailableOnWasm
    //           ),
    //       ));
    //   }
    //
    // Source-cite alternative: assert benten-sync's sync-replica
    // receive arm (or the engine's atrium-event-processor) contains
    // the wasm32-conditional gate before any SANDBOX dispatch.
    //
    // OBSERVABLE consequence: a browser thin-client receiving
    // Atrium-replicated data that references SANDBOX invocation
    // surfaces the typed error rather than attempting to recreate
    // the invocation locally on wasm32. Defends against the failure
    // shape "the thin-client receives an attribution-frame that
    // names a SANDBOX handler and helpfully tries to re-execute it
    // locally on the browser, panicking at wasmtime construction."
    //
    // This is the most subtle of the 4 entry points because the
    // thin-client commitment per CLAUDE.md baked-in #17 SHOULD
    // already prevent such dispatch attempts — but the architectural
    // commitment must be PINNED at the production code path, not just
    // documented.
    unimplemented!(
        "G16-D wires E_SANDBOX_UNAVAILABLE_ON_WASM at atrium-replicated SANDBOX invocation receive per br-r1-3 + br-r4-r1-2 + D3"
    );
}

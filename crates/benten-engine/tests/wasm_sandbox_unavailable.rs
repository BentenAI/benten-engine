//! R3-A RED-PHASE pin: wasm32-unknown-unknown SANDBOX unavailable path
//! is observable (G13-C wave 3; br-r1-3).
//!
//! Pin source: r2-test-landscape §2.1 G13-C row
//! `wasm32_unknown_unknown_browser_backend_e_sandbox_unavailable_on_wasm_path_observable`;
//! br-r1-3.
//!
//! ## What this pins
//!
//! On wasm32-unknown-unknown (browser), wasmtime is unavailable
//! (wasmtime cannot recursively host itself in a browser-tab WASM
//! runtime). SANDBOX primitive execution on this target must surface
//! `E_SANDBOX_UNAVAILABLE_ON_WASM` typed error from a specific code
//! path in `crates/benten-engine/src/primitive_host.rs::PrimitiveHost`
//! impl arm (per br-r1-3; symbol-form per §3.5b HARDENED point 3 for
//! high-churn surface; G13-C implementer points cite at the precise
//! arm site when un-ignoring this test).
//!
//! The pin asserts the path is OBSERVABLE end-to-end on the browser
//! target — not just that the typed-error variant exists in the enum,
//! but that a SANDBOX call from a browser bundle actually reaches the
//! variant (per pim-2 §3.6b end-to-end test pin requirement).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-C wave 3 wires the wasm-target SANDBOX unavailable path"]
fn wasm32_unknown_unknown_browser_backend_e_sandbox_unavailable_on_wasm_path_observable() {
    // br-r1-3 pin. G13-C implementer wires this:
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
        "G13-C wires runtime/source-cite assertion for E_SANDBOX_UNAVAILABLE_ON_WASM browser-side observability"
    );
}

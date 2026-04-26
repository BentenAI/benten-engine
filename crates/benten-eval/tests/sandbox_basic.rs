//! Phase 2b R3-B — SANDBOX core unit tests (G7-A).
//!
//! Red-phase TDD: these tests reference the future SANDBOX API that R5
//! implementation lands. Until then they remain `#[ignore]`d.
//!
//! Pin sources: plan §3 G7-A, wsa-15 (rename), wsa-20 (Engine singleton +
//! Module cache), D3-RESOLVED (per-call instance lifecycle), D22-precondition.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 G7-A surfaces consumed by these tests:
//   benten_eval::sandbox::{Sandbox, SandboxConfig, SandboxResult,
//                          ManifestRef, host_fns}
//   benten_eval::sandbox::instance::{shared_engine, module_cache}
//
// The entire module is gated #[cfg(not(target_arch = "wasm32"))] per
// sec-pre-r1-05; tests run native only.

#[test]
#[ignore = "Phase 2b G7-A pending — SANDBOX surface not yet landed"]
fn sandbox_end_to_end() {
    // Plan §3 G7-A must-pass test #1 — minimal echo module, default caps,
    // returns the input verbatim through the SANDBOX primitive executor.
    //
    // R5 wires:
    //   1. Compile fixtures/sandbox/echo.wasm (D26 — pre-built committed bytes).
    //   2. `engine.sandbox_call(echo_cid, ManifestRef::Named("compute-basic"),
    //                           input_bytes)` returns `Ok(SandboxResult { ... })`.
    //   3. fuel/memory/wallclock/output budgets default per D24 + plan §3 G7-A.
    todo!("R5 G7-A — wire echo fixture + sandbox_call public surface");
}

#[test]
#[ignore = "Phase 2b G7-A pending — wsa-15 rename"]
fn sandbox_no_state_persists_across_calls() {
    // wsa-15 — pin the PROPERTY (no cross-call retention) not the
    // implementation (per-call instance). D3-RESOLVED clarified:
    //   - wasmtime::Engine: shared singleton (created once)
    //   - wasmtime::Module: content-CID-cached (compiled once per CID)
    //   - wasmtime::Store + wasmtime::Instance: per-call (constructed
    //     fresh, dropped at completion)
    //
    // Test: invoke a module that writes to module-global memory in call 1;
    // call 2 of the same module observes the global at its initial value
    // (not the call-1 written value).
    todo!("R5 G7-A — assert module global memory is reset between calls");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D3-RESOLVED + wsa-20"]
fn sandbox_engine_singleton_lifetime() {
    // wsa-20 + D3-RESOLVED — `wasmtime::Engine` constructed ONCE per
    // benten Engine open (not per primitive call).
    //
    // White-box test: `benten_eval::sandbox::instance::shared_engine()`
    // returns the same Arc<wasmtime::Engine> on every call within a
    // benten Engine's lifetime.
    todo!("R5 G7-A — assert shared_engine() returns same Arc across calls");
}

#[test]
#[ignore = "Phase 2b G7-A pending — wsa-20 + D22 precondition"]
fn sandbox_module_cache_avoids_recompilation_on_repeated_call() {
    // wsa-20 — `wasmtime::Module` is content-CID-cached. The cold-start
    // budget (D22 ≤2ms p95 Linux x86_64) is unmeetable if Module
    // recompiles per call.
    //
    // Test: invoke same module twice; assert the second call's
    // module-compilation timestamp == the first call's (i.e., the
    // cached entry is reused). White-box via
    // `benten_eval::sandbox::instance::module_cache().get_compile_ts(cid)`.
    todo!("R5 G7-A — assert module_cache hits on repeated CID");
}

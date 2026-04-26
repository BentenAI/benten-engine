//! Phase 2b R3-B — SANDBOX compile-time-disabled-on-wasm32 integration
//! test (G7-C, eval-crate-internal symbol-absence assertion).
//!
//! R2 file path: `crates/benten-eval/tests/integration/sandbox_wasm32_disabled.rs`
//! R2 test name: `sandbox_compile_time_disabled_on_wasm32_executor`.
//!
//! Pin sources: sec-pre-r1-05, wasm-r1-3.
//!
//! NOTE — the engine-side companion test
//! (`crates/benten-engine/tests/integration/sandbox_compile_time_disabled_on_wasm32.rs`)
//! is R3-E territory (G7-C engine surface absence pin). This file
//! covers the eval-crate-internal absence of the SANDBOX executor
//! symbol — the wasmtime-touching code in `crates/benten-eval/src/sandbox/`
//! must not compile into wasm32 targets.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-C pending — eval-side wasm32 absence pin"]
fn sandbox_compile_time_disabled_on_wasm32_executor() {
    // sec-pre-r1-05 + wasm-r1-3 — assert that the wasmtime-touching
    // SANDBOX executor in `benten-eval` is compile-time excluded from
    // wasm32 builds. The crate-level cfg-gate is on
    // `crates/benten-eval/src/sandbox/mod.rs` and `primitives/sandbox.rs`.
    //
    // Strategy:
    //   - Run `cargo check --target wasm32-wasip1 -p benten-eval`.
    //   - Assert: the build succeeds.
    //   - Assert: the resulting rmeta does NOT contain
    //     `benten_eval::sandbox::execute_sandbox` symbol.
    //   - Optionally: a `compile_fail` doctest under
    //     `#[cfg(target_arch = "wasm32")]` proving the symbol cannot
    //     be referenced.
    //
    // Distinct from the engine-side R3-E test which checks the
    // higher-level `Engine::sandbox_*` surface absence; this is the
    // eval-internal layer.
    todo!("R5 G7-C — wasm32 build + symbol absence assertion at eval layer");
}

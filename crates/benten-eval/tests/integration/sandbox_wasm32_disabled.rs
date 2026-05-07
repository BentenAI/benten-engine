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
//!
//! **G20-A1 wave-8a** (Phase 3): body un-ignored. Source-grep approach
//! (the cfg-gate decoration on `src/primitives/sandbox.rs` + the
//! `src/sandbox/mod.rs` module-level gate) is the canonical pin
//! shape; the actual wasm32 build cell in CI is the runtime
//! verification.

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[test]
fn sandbox_compile_time_disabled_on_wasm32_executor() {
    // sec-pre-r1-05 + wasm-r1-3 — the wasmtime-touching SANDBOX
    // executor in `benten-eval/src/sandbox/` MUST be cfg-gated out of
    // wasm32 builds. The structural pin: source-grep at the executor
    // module + the sandbox subsystem mod.rs to confirm the
    // `#[cfg(not(target_arch = "wasm32"))]` decoration is present.
    //
    // The runtime wasm32-build verification lives in CI's
    // `wasm-browser.yml` workflow (which builds the napi crate
    // against wasm32-unknown-unknown + asserts the bundle does NOT
    // pull in wasmtime symbols).

    // Audit 1: sandbox/mod.rs carries the module-level wasm32 cut.
    let mod_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("sandbox")
        .join("mod.rs");
    let mod_src = std::fs::read_to_string(&mod_path)
        .expect("benten-eval/src/sandbox/mod.rs must be readable");
    assert!(
        mod_src.contains("#![cfg(not(target_arch = \"wasm32\"))]"),
        "sandbox/mod.rs MUST carry `#![cfg(not(target_arch = \"wasm32\"))]` \
         to enforce the SANDBOX subsystem cut on wasm32 (sec-pre-r1-05 \
         + wasm-r1-3)"
    );

    // Audit 2: the executor primitives/sandbox.rs carries its own
    // wasm32 cut.
    let exec_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("primitives")
        .join("sandbox.rs");
    let exec_src = std::fs::read_to_string(&exec_path)
        .expect("benten-eval/src/primitives/sandbox.rs must be readable");
    assert!(
        exec_src.contains("#![cfg(not(target_arch = \"wasm32\"))]")
            || exec_src.contains("#[cfg(not(target_arch = \"wasm32\"))]"),
        "primitives/sandbox.rs MUST carry the wasm32 cut decoration \
         to enforce sec-pre-r1-05 + wasm-r1-3"
    );
}

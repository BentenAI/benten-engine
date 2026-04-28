//! Phase 2b R3 (R3-E) — sec-pre-r1-05 + wasm-r1-3: SANDBOX is
//! compile-time-disabled on wasm32 targets.
//!
//! TDD red-phase. Pin source: plan §3 G7-C + sec-pre-r1-05 (compile-time
//! gate) + wasm-r1-3 (browser target ships without SANDBOX executor;
//! DSL builder still defined so user code calling `sandbox(...)` gets
//! a typed registration-time error rather than a missing symbol).
//!
//! The SANDBOX primitive runs wasmtime, which itself does not target
//! wasm32. So the engine MUST cfg-gate the SANDBOX executor module out
//! of the wasm32 build. Per plan §3 G10-A-browser the gate is a
//! `#[cfg(not(target_arch = "wasm32"))]` on the executor; per the same
//! finding, the DSL stays defined on wasm32 so the surface compiles
//! and surfaces a typed `E_SANDBOX_DISABLED_ON_WASM32` at registration
//! time.
//!
//! This integration test confirms the gate at the *engine* boundary by
//! exercising `register_subgraph` against a SANDBOX-bearing
//! `SubgraphSpec`; on wasm32 the registration call MUST surface the
//! typed error; on native targets it MUST proceed normally.
//!
//! **Status:** RED-PHASE (Phase 2b G7-C + G10-A pending). The cfg-gate
//! plus typed-error surface live in `crates/benten-eval/src/primitives/sandbox.rs`
//! (executor) and `crates/benten-engine/src/subgraph_spec.rs`
//! (registration-time error path).
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `sandbox_compile_time_disabled_on_wasm32_executor` — R2 §2.3.
///
/// On wasm32 targets, the SANDBOX executor module is NOT compiled in.
/// Registering a SANDBOX-bearing handler MUST fail at registration time
/// with the typed `E_SANDBOX_DISABLED_ON_WASM32` error. On native
/// targets, the same registration MUST succeed (the executor is present).
///
/// This test confirms BOTH halves of the cfg-gate so a future regression
/// (e.g. dropping the `#[cfg(not(target_arch = "wasm32"))]` decoration
/// on the executor module) is caught immediately.
#[test]
#[ignore = "pending G10-A wasm32 build target; tracks G10-A's phase-2b/g10/a-wasip1 (wave-5). G7-C delivers the napi cfg-gate; G10-A delivers the wasm32 build that exercises the negative half of sec-pre-r1-05."]
fn sandbox_compile_time_disabled_on_wasm32_executor() {
    let (_dir, mut engine) = fresh_engine();

    // Build a SANDBOX-bearing SubgraphSpec via the testing helper
    // (G7-A owns the helper signature; this is a §9 backdoor).
    let spec = benten_engine::testing::testing_make_minimal_sandbox_spec();

    let result = engine.register_subgraph("sandbox.test", spec);

    #[cfg(target_arch = "wasm32")]
    {
        let err = result.expect_err(
            "registration of a SANDBOX-bearing handler MUST fail on wasm32 \
             with E_SANDBOX_DISABLED_ON_WASM32 (sec-pre-r1-05 compile-time gate)",
        );
        let rendered = err.to_string();
        assert!(
            rendered.contains("E_SANDBOX_DISABLED_ON_WASM32"),
            "error must be the typed E_SANDBOX_DISABLED_ON_WASM32 code, got: {}",
            rendered
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        result.expect(
            "registration of a SANDBOX-bearing handler MUST succeed on native \
             targets (the executor is present; the cfg-gate is the negative \
             half of sec-pre-r1-05)",
        );
    }
}

/// Companion drift detector: assert the cfg-gate decoration is present
/// in the executor module source so a future refactor that drops it is
/// caught at the source level too (defense in depth against the R2-noted
/// "test-passes-by-accident" anti-pattern).
#[test]
#[ignore = "pending G7-A executor; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30). G7-A owns `crates/benten-eval/src/primitives/sandbox.rs`; the source-grep drift detector runs once that module lands."]
fn sandbox_executor_module_carries_wasm32_cfg_gate_in_source() {
    let exec_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/benten-eval/src/primitives/sandbox.rs");

    let src = std::fs::read_to_string(&exec_path).unwrap_or_else(|e| {
        panic!(
            "SANDBOX executor source not found at {} ({}) — G7-A owns the file",
            exec_path.display(),
            e
        );
    });

    assert!(
        src.contains("#[cfg(not(target_arch = \"wasm32\"))]")
            || src.contains("#![cfg(not(target_arch = \"wasm32\"))]"),
        "sandbox.rs must carry a `#[cfg(not(target_arch = \"wasm32\"))]` \
         decoration to enforce sec-pre-r1-05 + wasm-r1-3 (compile-time \
         disable on wasm32)"
    );
}

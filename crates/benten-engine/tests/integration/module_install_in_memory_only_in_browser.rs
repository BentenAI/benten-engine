//! Phase 2b R4-FP B-3 — G10-B browser-target in-memory-only constraint.
//!
//! TDD red-phase. Pin sources:
//!   - `r1-wasm-target.json` — G10-B in-memory-only on
//!     wasm32-unknown-unknown in Phase 2b; persistence (IndexedDB,
//!     OPFS, etc.) defers to Phase 3.
//!   - `r2-test-landscape.md` §2.4
//!     `module_install_in_memory_only_in_browser_session`.
//!
//! Owned by R4-FP B-3 (R3-followup); R5 owners G10-B + G10-A-browser.

#![cfg(feature = "phase_2b_landed")]
#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_engine::Engine (browser-target constructor)
//   benten_engine::Engine::install_module
//   benten_engine::Engine::is_module_installed
//   benten_engine::testing::testing_make_minimal_manifest
//   benten_engine::testing::testing_compute_manifest_cid

#[test]
#[ignore = "Phase 2b G10-B + G10-A-browser pending — in-memory-only install on wasm32"]
fn module_install_in_memory_only_in_browser_session() {
    // r1-wasm-target.json — Phase-2b browser engines run with an
    // in-memory KV backend ONLY (no IndexedDB persistence). Installs
    // succeed within a session but MUST NOT survive a page reload.
    //
    // R5 G10-B + G10-A-browser wires:
    //   1. Construct an Engine with the in-memory browser backend
    //      (Engine::open(":memory:") or the browser-specific builder).
    //   2. Build manifest M; install with matching CID.
    //   3. ASSERT engine.is_module_installed(&cid_m) == true within the
    //      same session.
    //   4. Drop the engine (simulating a page reload).
    //   5. Construct a fresh Engine with the same in-memory backend.
    //   6. ASSERT engine.is_module_installed(&cid_m) == false (the
    //      install did NOT persist).
    //   7. Pin documentation: a Phase-3 IndexedDB backend SHOULD make
    //      this assertion FLIP — the test then needs to be re-scoped
    //      to the in-memory-backend variant explicitly.
    //
    // Test gating: this case may need #[cfg(target_arch = "wasm32")] OR
    // a runtime backend-kind check; R5 picks the gating shape. The
    // wasm32 + native split likely lives in benten-graph's backend
    // abstraction.
    todo!(
        "R5 G10-B + G10-A-browser — assert in-memory-only install lifecycle \
         on wasm32-unknown-unknown (no IndexedDB in Phase 2b)"
    );
}

#[test]
#[ignore = "Phase 2b G10-B + G10-A-browser pending — uninstall mid-session takes effect"]
fn module_uninstall_during_session_takes_effect() {
    // R2 §2.4 — uninstall MUST take effect within the same session
    // (don't have to wait for a session restart) so that operators can
    // hot-swap modules during devserver iteration.
    //
    // R5 wires:
    //   1. Install M; assert is_module_installed == true.
    //   2. Uninstall M.
    //   3. ASSERT is_module_installed == false IMMEDIATELY (without
    //      destroying + recreating the engine).
    //   4. ASSERT a subsequent register_subgraph referencing the
    //      manifest by name fails with E_SANDBOX_MANIFEST_UNKNOWN.
    todo!(
        "R5 G10-B — assert uninstall takes effect within same session \
         (no engine restart required)"
    );
}

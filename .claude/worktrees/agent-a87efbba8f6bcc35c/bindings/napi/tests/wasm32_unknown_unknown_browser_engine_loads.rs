//! Phase 2b G10-A-browser must-pass — `wasm32_unknown_unknown_browser_engine_loads`.
//!
//! Pin source: plan §3 G10-A-browser must-pass list +
//! `.addl/phase-2b/00-implementation-plan.md` G10-A-browser row.
//!
//! Asserts that the browser-target runtime surface
//! (`benten_napi::wasm_browser`) compiles + exposes the
//! `BrowserManifestStore` and `browser_runtime_available` symbols
//! that the wasm32-unknown-unknown bundle build relies on.
//!
//! ## What "loads" means here
//!
//! The cargo test binary cannot itself execute under wasm32-unknown-unknown
//! (libtest's harness needs `std::process` etc.). The integration-test
//! shape that satisfies the must-pass entry is:
//!
//!   1. **Native arm** (this file, default-cargo-test path): exercise
//!      the `BrowserManifestStore` + `browser_runtime_available()` API
//!      shapes that the browser bundle depends on. If these compile
//!      and run correctly on the native test target, the same module
//!      compiles cleanly on wasm32 (the cfg-split lives entirely
//!      inside `wasm_browser.rs`).
//!   2. **Wasm32 arm** (executed by `wasm-browser.yml` headless
//!      browser smoke job): the bundle build itself loads the
//!      compiled module into a browser context; the smoke job asserts
//!      `engine.browserRuntimeAvailable() === true` post-load.
//!
//! Both arms together pin the must-pass; this file is arm 1.
//!
//! Cross-target build invariant: this file MUST also compile under
//! `cargo check --target wasm32-unknown-unknown -p benten-napi --tests`
//! once the bundle build is wired (the integration-test compile step
//! is a precondition of the bundle's smoke harness landing). For now
//! the cfg-honest test below covers the native arm; the bundle smoke
//! harness in `wasm-browser.yml` covers the wasm32 arm.
//!
//! Owned by G10-A-browser per plan §3.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_napi::wasm_browser::{BrowserManifestStore, browser_runtime_available};

/// Plan-pinned must-pass. The "loads" predicate decomposes into:
///   1. The store type is constructable (the constructor symbol is
///      present in the browser-target build).
///   2. The target-availability probe is callable + returns the
///      target-honest answer.
///   3. The store is non-persistent (Compromise #N+8 storage-contract
///      half — the higher-level enforcement test is the sister
///      `wasm32_unknown_unknown_module_manifest_in_memory_only_no_indexeddb_persistence`).
#[test]
fn wasm32_unknown_unknown_browser_engine_loads() {
    let store = BrowserManifestStore::new();

    // Storage-contract symbol presence
    assert!(store.is_empty(), "fresh store must be empty");
    assert_eq!(store.len(), 0, "fresh store length must be 0");
    assert!(
        store.installed_cids().is_empty(),
        "fresh store must list no installed CIDs"
    );

    // G18-A wave-5a HONEST DISCLOSURE — Compromise #19 PARTIALLY
    // CLOSED. The IndexedDB schema + handler scaffolding landed at
    // `crate::browser_indexeddb`; the wasm32 `web-sys` / `js-sys`
    // / `wasm-bindgen-futures` plumbing is deferred to G18-A-followup
    // wave per `docs/future/phase-3-backlog.md` §4.3. The flag stays
    // `false` honestly until the wasm32 IDB calls wire — flipping it
    // to `true` ahead of the plumbing would lie about durability to
    // operators branching on the flag (br-r1-8 honest-disclosure
    // principle). The scope per CLAUDE.md baked-in #17 remains
    // thin-client cache + manifest-store ONLY.
    assert!(
        !store.is_persistent(),
        "G18-A: BrowserManifestStore::is_persistent stays false until \
         G18-A-followup wires the wasm32 IDB plumbing per CLAUDE.md \
         baked-in #17 thin-client cache scope + br-r1-8 honest-disclosure"
    );

    // Target-availability probe: cfg-honest answer.
    //   - native build: false (this test runs natively in the cargo-test
    //     pipeline — the wasm32 bundle smoke harness in `wasm-browser.yml`
    //     verifies `true` post-load on wasm32).
    //   - wasm32 build: true.
    #[cfg(not(target_arch = "wasm32"))]
    assert!(
        !browser_runtime_available(),
        "browser_runtime_available must be false on native target builds"
    );

    #[cfg(target_arch = "wasm32")]
    assert!(
        browser_runtime_available(),
        "browser_runtime_available must be true on wasm32 builds (the \
         wasm-browser.yml headless-browser smoke loads this same probe)"
    );
}

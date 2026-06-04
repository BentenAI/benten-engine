//! Phase-3 G17-C wave-5b — `engine.registerModuleBytes` napi method
//! source-cite diagnostic (phase-3-backlog §6.6).
//!
//! Pin source: r2-test-landscape §2.5 G17-C
//! `engine_register_module_bytes_napi_source_cite_diagnostic` +
//! `engine_register_module_bytes_napi_method_present`.
//!
//! ## r4-r2-napi-3 framing (carried forward)
//!
//! This file is a SOURCE-CITE DIAGNOSTIC, NOT a load-bearing end-to-end
//! pin per pim-2 §3.6b. The grep-against-source-text shape verifies
//! the method's PRESENCE on the napi surface but does NOT drive the
//! production-grade entry point with observable behavioral consequence
//! — that contract lives at the LOAD-BEARING end-to-end pin in
//! `crates/benten-engine/tests/manifest_unknown.rs` (registration-
//! time validation walk drives `Engine::register_subgraph` with
//! observable rejection of unresolved manifest names) +
//! `packages/engine/test/install_module.test.ts::"engine.uninstallModule(cid) clean release"`
//! (Vitest end-to-end DSL → napi → engine path through
//! `engine.registerSubgraph` after uninstall).
//!
//! Per pim-2 §3.6b: source-cite diagnostics are useful scaffolding
//! pins but do NOT satisfy the end-to-end pin requirement on their
//! own — the load-bearing pins live at the entry-point-driven sites
//! above. This diagnostic catches a regression where the napi method
//! is renamed or accidentally removed from the source surface
//! (cheap-to-run signal that fires at unit-test cadence rather than
//! Vitest cadence).
//!
//! ## Method-presence diagnostic shape
//!
//! G17-C ships the `register_module_bytes` napi method at
//! `bindings/napi/src/lib.rs::register_module_bytes` (the napi crate's
//! Engine class definition lives in lib.rs; there is no separate
//! `engine.rs` despite the plan-row text — verified at G17-C dispatch
//! pre-flight 2026-05-06). It carries the WRITE side of named-manifest
//! module-bytes registration:
//!
//! 1. TS DSL caller passes module bytes + caller-supplied CID.
//! 2. napi handler validates bytes (BLAKE3 recompute against the CID).
//! 3. Bytes are persisted via the durable `RedbBlobBackend`
//!    (`Engine::register_module_bytes`) so SANDBOX dispatch can
//!    resolve `module: "<cid>"` references at execution time.
//!
//! Pairs with `crates/benten-engine/tests/manifest_unknown.rs`
//! (READ-AND-VALIDATE side at `register_subgraph` time).

#![allow(clippy::unwrap_used, clippy::expect_used)]

#[test]
fn engine_register_module_bytes_napi_method_present() {
    // Phase-3-backlog §6.6 source-cite diagnostic. Reads the napi
    // crate's lib.rs and verifies the `register_module_bytes` napi
    // method is exposed on the Engine class (with the correct
    // js_name annotation so JS callers see `registerModuleBytes`).
    let napi_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("bindings/napi/src/lib.rs MUST exist (G17-C source-cite diagnostic)");

    // Method present (snake_case Rust source side):
    assert!(
        napi_src.contains("fn register_module_bytes"),
        "bindings/napi/src/lib.rs MUST expose `fn register_module_bytes` per phase-3-backlog §6.6"
    );

    // js_name annotation surfaces it as `registerModuleBytes` to JS:
    assert!(
        napi_src.contains("js_name = \"registerModuleBytes\""),
        "register_module_bytes MUST carry the js_name = \"registerModuleBytes\" annotation \
         so the TS Engine wrapper sees the camelCase JS surface"
    );

    // The method dispatches through the inner engine's
    // `register_module_bytes` (not just sentinel-construct a parsed
    // CID + return — verifies the actual dispatch line is present):
    assert!(
        napi_src.contains(".register_module_bytes(&parsed"),
        "registerModuleBytes napi method MUST dispatch through the inner Engine's \
         register_module_bytes(&Cid, &[u8]) Rust API per phase-3-backlog §6.6"
    );

    // TS-side surface mirror (camelCase in DSL):
    let ts_src = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("packages")
            .join("engine")
            .join("src")
            .join("engine.ts"),
    )
    .expect("packages/engine/src/engine.ts MUST exist (G17-C source-cite diagnostic)");
    assert!(
        ts_src.contains("registerModuleBytes"),
        "packages/engine/src/engine.ts MUST expose registerModuleBytes per phase-3-backlog \
         §6.6 + §3.5b doc-coupling discipline"
    );
    // The TS method dispatches through `this.inner.registerModuleBytes`
    // (the napi-shim accessor); verify the dispatch line is present:
    assert!(
        ts_src.contains("this.inner.registerModuleBytes(cid, bytes)"),
        "Engine.registerModuleBytes MUST dispatch through the napi shim's \
         registerModuleBytes accessor"
    );
}

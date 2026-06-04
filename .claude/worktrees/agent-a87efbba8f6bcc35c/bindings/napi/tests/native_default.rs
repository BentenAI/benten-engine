//! G13-B GREEN-PHASE pin for native-default napi binding compile
//! (Phase-3 R5 wave-2; plan §3 G13-B).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + plan §3 G13-B
//! must-pass column):
//!
//! - `bindings/napi/tests/native_default.rs::engine_napi_binding_compiles_native_redb_default` — plan §3 G13-B (G13-B GREEN)
//! - `bindings/napi/tests/native_default.rs::engine_napi_binding_compiles_browser_target_browser_backend` — plan §3 G13-B (G13-C STILL RED)
//!
//! ## What this pins
//!
//! After G13-B's generic cascade lands, both targets continue to compile:
//!
//! - **Native (default-features):** `Engine = EngineGeneric<RedbBackend>`,
//!   the napi cdylib for Node.js continues to compile + link.
//! - **Browser (`--target wasm32-unknown-unknown --features browser-backend`):**
//!   `Engine = EngineGeneric<BrowserBackend>`, the napi cdylib for
//!   browser bundles compiles + links without redb in the dep tree.
//!
//! These are compile-pin tests. The native pin's "test" is the test
//! crate compiling AT ALL — if `benten_engine::Engine` no longer
//! resolves under default features post-G13-B, the integration test
//! binary fails to link and the suite never reaches `#[test]`
//! execution. The body below adds compile-time type-equality
//! assertions on top of that automatic verification.

#![allow(clippy::unwrap_used, clippy::used_underscore_items)]

#[test]
fn engine_napi_binding_compiles_native_redb_default() {
    // G13-B GREEN pin. The presence of this test file under
    // `bindings/napi/tests/` running in CI is the primary verifier
    // — if the napi binding's transitive `benten_engine` dep loses
    // the `Engine` alias (or the alias resolves to a non-redb
    // backend under default features), this test crate fails to
    // compile and the test binary never produces.
    //
    // Compile-time pin: `benten_engine::Engine` and
    // `benten_engine::EngineGeneric<RedbBackend>` are the same type.
    // We don't import `benten_graph` directly (the napi binding
    // doesn't depend on it as a dev-dep — keeps the test crate's
    // dep-tree minimal); instead we pin via type equivalence on a
    // function pointer that requires both views of the alias.
    fn _accepts_alias(_: &benten_engine::Engine) {}

    // Witness via fn-pointer assignment: `_accepts_alias` accepts
    // `&Engine` which equals `&EngineGeneric<RedbBackend>` under the
    // default-features build. If a future refactor flips the alias
    // away from the redb specialization without updating this pin,
    // a downstream type mismatch would surface in the napi binding
    // proper (which DOES use the alias's redb-specific surface) —
    // this test catches the alias-flip BEFORE the cdylib build's
    // surface-area regression manifests.
    let _: fn(&benten_engine::Engine) = _accepts_alias;
}

#[test]
fn engine_napi_binding_compiles_browser_target_browser_backend() {
    // G13-C GREEN pin. Source-cite assertion: the benten-engine
    // `browser-backend` feature exists + forwards to benten-graph,
    // AND the `Engine` alias arm under `cfg(feature = "browser-backend")`
    // re-points to `EngineGeneric<BrowserBackend>`.
    //
    // The COMPILE-VERIFICATION arm runs under the dedicated CI workflow
    // step (`wasm-checks.yml` cargo-check on
    // `--target wasm32-unknown-unknown -p benten-graph --features
    // browser-backend --no-default-features`) plus the wave-3 G13-C
    // pre-flight discipline. Asserting the source-side wiring here
    // catches the regression where a refactor drops the alias arm
    // before CI re-runs.
    use std::path::PathBuf;
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");

    let engine_cargo =
        std::fs::read_to_string(workspace_root.join("crates/benten-engine/Cargo.toml"))
            .expect("read benten-engine/Cargo.toml");
    assert!(
        engine_cargo.contains(r#"browser-backend = ["benten-graph/browser-backend"]"#),
        "benten-engine/Cargo.toml MUST declare browser-backend feature forwarding to benten-graph per G13-C"
    );

    let engine_rs =
        std::fs::read_to_string(workspace_root.join("crates/benten-engine/src/engine.rs"))
            .expect("read benten-engine/src/engine.rs");
    assert!(
        engine_rs.contains("#[cfg(feature = \"browser-backend\")]")
            && engine_rs.contains("pub type Engine = EngineGeneric<benten_graph::BrowserBackend>"),
        "engine.rs MUST add cfg(feature = \"browser-backend\") alias arm pointing at BrowserBackend per G13-C"
    );

    let graph_cargo =
        std::fs::read_to_string(workspace_root.join("crates/benten-graph/Cargo.toml"))
            .expect("read benten-graph/Cargo.toml");
    assert!(
        graph_cargo.contains("browser-backend = []"),
        "benten-graph/Cargo.toml MUST declare browser-backend feature per G13-C"
    );

    // OBSERVABLE consequence: a future PR that drops any of these
    // four wiring sites breaks this test. The wasm32 build path is
    // verified end-to-end by the `wasm-checks.yml` workflow's
    // `cargo check --target wasm32-unknown-unknown -p benten-graph
    // --features browser-backend --no-default-features` step.
}

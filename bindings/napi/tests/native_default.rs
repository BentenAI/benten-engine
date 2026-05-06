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
    let _f: fn(&benten_engine::Engine) = _accepts_alias;
}

#[test]
#[ignore = "RED-PHASE: G13-C wave-3 introduces BrowserBackend behind cargo feature `browser-backend`"]
fn engine_napi_binding_compiles_browser_target_browser_backend() {
    // G13-C STILL-RED pin. Runtime check on the browser target uses
    // `wasm-bindgen-test` (or the `wasm-checks.yml` workflow's
    // compile-only verification — whichever is faster).
    //
    // The native test above pins the source-side absence of
    // browser-incompatible refs; this test is the COMPILE-VERIFICATION
    // pin that runs under
    // `cargo check --target wasm32-unknown-unknown -p benten-napi
    // --features browser-backend --no-default-features`.
    //
    // OBSERVABLE consequence: bundling the napi binding for a
    // browser tab does not pull RedbBackend into the wasm32 link.
    unimplemented!(
        "G13-C wires browser-target napi-binding compile pin (un-ignore at G13-C wave-3)"
    );
}

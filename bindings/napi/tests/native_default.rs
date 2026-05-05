//! R3-A RED-PHASE pins for native-default napi binding compile
//! (G13-B wave-2; plan §3 G13-B).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + plan §3 G13-B
//! must-pass column):
//!
//! - `tests/engine_napi_binding_compiles_native_redb_default` — plan §3 G13-B
//! - `tests/engine_napi_binding_compiles_browser_target_browser_backend` — plan §3 G13-B
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
//! These are compile-pin tests — runtime verification happens via
//! `wasm-checks.yml` (browser target compile) + the existing native
//! `cargo nextest run` (native compile). The Rust-side pins assert
//! that the napi wrapper ITSELF compiles against both Engine
//! specializations.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B — native default-features napi binding"]
fn engine_napi_binding_compiles_native_redb_default() {
    // G13-B implementer wires this:
    //   #[cfg(not(target_arch = "wasm32"))]
    //   #[cfg(not(feature = "browser-backend"))]
    //   {
    //       // Compile-time check: the napi binding's Engine alias resolves
    //       // to EngineGeneric<RedbBackend>. The presence of this test
    //       // file under `bindings/napi/tests/` running in CI is the
    //       // verifier — if the binding fails to compile, the whole
    //       // test crate fails to compile.
    //       let _: benten_napi::Engine = unsafe { std::mem::zeroed() };
    //   }
    //
    // OBSERVABLE consequence: native compile gate is the dependency
    // graph integrity check. Stays useful even after the binding is
    // un-`#[ignore]`'d at G13-B landing because the `Engine` alias
    // is then dereferenced on the test path.
    unimplemented!("G13-B wires native default-features napi-binding compile pin");
}

#[test]
#[ignore = "RED-PHASE: G13-C — wasm32 browser-backend target napi binding"]
fn engine_napi_binding_compiles_browser_target_browser_backend() {
    // G13-C implementer wires this. Runtime check on the browser
    // target uses `wasm-bindgen-test` (or the `wasm-checks.yml`
    // workflow's compile-only verification — whichever is faster).
    //
    // The native test file pins the source-side absence of
    // browser-incompatible refs (mirroring `wasm_no_redb.rs`); this
    // test is the COMPILE-VERIFICATION pin that runs under
    // `cargo check --target wasm32-unknown-unknown -p benten-napi
    // --features browser-backend --no-default-features`.
    //
    // OBSERVABLE consequence: bundling the napi binding for a
    // browser tab does not pull RedbBackend into the wasm32 link.
    unimplemented!(
        "G13-C wires browser-target napi-binding compile pin (verified via wasm-checks.yml)"
    );
}

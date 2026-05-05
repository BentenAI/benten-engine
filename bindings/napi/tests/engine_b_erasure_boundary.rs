//! R3-A RED-PHASE pin: napi binding erasure boundary at the cdylib edge
//! (G13-B wave 2; D-PHASE-3-1a + arch-r1-1).
//!
//! Pin source: r2-test-landscape §2.1 G13-B row
//! `engine_b_napi_binding_erases_at_cdylib_boundary_only`; D-PHASE-3-1a;
//! arch-r1-1.
//!
//! ## What this pins
//!
//! The generic-cascade `EngineGeneric<B>` lives inside the engine crate.
//! At the napi cdylib boundary (`bindings/napi/src/lib.rs`), the engine
//! type is erased back to a concrete shape (e.g. `Engine` =
//! `EngineGeneric<RedbBackend>` for the native cdylib;
//! `EngineGeneric<BrowserBackend>` for the browser-target cdylib). The
//! `Box<dyn std::error::Error + Send + Sync>` boundary erasure happens
//! AT the napi layer, not deeper inside the engine.
//!
//! This pin asserts:
//!
//! 1. The cdylib (`bindings/napi/src/lib.rs`) ships with EXACTLY ONE
//!    type alias for `Engine` per cargo feature combination
//!    (default → RedbBackend; `browser-backend` → BrowserBackend).
//! 2. NO `<B: GraphBackend>` generic-cascade leaks into the napi
//!    public surface (since napi-rs cannot expose generic types over
//!    the napi-v3 ABI; concrete-only).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B wave 2 — napi cdylib erasure boundary"]
fn engine_b_napi_binding_erases_at_cdylib_boundary_only() {
    // G13-B implementer wires this:
    //   // Read bindings/napi/src/lib.rs and verify:
    //   //   - At least one `pub type Engine = EngineGeneric<...>;`
    //   //     concrete alias is present (gated by cargo feature for
    //   //     default vs browser-backend builds).
    //   //   - No `pub fn ... <B: GraphBackend>` signature in the
    //   //     `#[napi]`-annotated surface — generic-bound functions
    //   //     cannot cross the napi-v3 ABI.
    //
    //   let napi_src = std::fs::read_to_string("bindings/napi/src/lib.rs").unwrap();
    //   let alias_count = napi_src.lines()
    //       .filter(|l| {
    //           let t = l.trim_start();
    //           !t.starts_with("//") && t.contains("type Engine =")
    //       })
    //       .count();
    //   assert!(alias_count >= 1,
    //       "bindings/napi/src/lib.rs MUST declare a concrete Engine type alias \
    //        per D-PHASE-3-1a / arch-r1-1");
    //
    //   // Generic-bound napi function regression scan:
    //   for line in napi_src.lines() {
    //       let t = line.trim_start();
    //       if t.starts_with("#[napi]") || t.contains("#[napi") {
    //           // Approximate scan: a `fn name<B: GraphBackend>`
    //           // appearing within a few lines after a #[napi] attr
    //           // is the failure mode.
    //       }
    //   }
    //
    // OBSERVABLE consequence: a future PR that tries to expose
    // `EngineGeneric<B>` directly through napi (instead of erasing
    // to a concrete alias) fails this test.
    unimplemented!(
        "G13-B wires bindings/napi/src/lib.rs grep assertion for cdylib erasure boundary"
    );
}

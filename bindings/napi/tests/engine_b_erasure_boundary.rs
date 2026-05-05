//! R3-A RED-PHASE pin: napi binding erasure boundary at the cdylib edge
//! (G13-B wave 2; D-PHASE-3-1a + arch-r1-1).
//!
//! Pin source: r2-test-landscape §2.1 G13-B row
//! `engine_b_napi_binding_erases_at_cdylib_boundary_only`; D-PHASE-3-1a;
//! arch-r1-1; sharpened by R4-R1 napi-r4-r1-3 to use a compile-time
//! witness instead of grep-against-source-text.
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
//! This pin asserts (post-napi-r4-r1-3 reshape):
//!
//! 1. The `benten_napi::Engine` symbol resolves to a concrete (NOT
//!    generic-bound) type at compile time. The compile-time witness
//!    pattern (`std::marker::PhantomData<Engine>`) succeeds only if
//!    `Engine` is a concrete alias — a `<B>`-generic type cannot be
//!    instantiated without a type parameter and the test would fail
//!    to compile, providing the load-bearing assertion.
//! 2. The companion compile-fail pin
//!    (`compile_fail_attempting_to_use_engine_with_explicit_backend.rs`)
//!    asserts that callers cannot write `Engine<RedbBackend>` directly
//!    (the alias is concrete; supplying a type argument is a hard
//!    compile error).
//!
//! Per pim-2 §3.6b end-to-end requirement: this pin drives the actual
//! `benten_napi::Engine` type symbol at compile time + observable
//! consequence (compile-failure if the symbol is generic). Replaces the
//! prior grep-against-source-text shape that R4-R1 napi-r4-r1-3 named
//! as a contract violation (text scanning could pass with comment-only
//! aliases or miss generic bounds in sibling files).

#![allow(clippy::unwrap_used, dead_code)]

/// Compile-time witness that `benten_napi::Engine` is a CONCRETE type
/// alias (not a generic-bound type). If `Engine` were declared as
/// `pub struct Engine<B>` (generic-bound), this `PhantomData` reference
/// would fail to compile because no type argument is supplied — making
/// the witness load-bearing per pim-2 §3.6b (would FAIL if the napi
/// surface silently widened to a generic-bound shape).
///
/// G13-B implementer un-ignores the test below; the witness shape stays
/// permanently as a compile-time gate.
#[allow(dead_code)]
struct EngineErasureWitness {
    // Once G13-B lands, uncomment the next line. It compiles only when
    // `benten_napi::Engine` resolves to a concrete alias:
    //
    //     _engine: std::marker::PhantomData<benten_napi::Engine>,
    _placeholder: (),
}

#[test]
#[ignore = "RED-PHASE: G13-B wave 2 — napi cdylib erasure boundary (compile-time witness per napi-r4-r1-3)"]
fn engine_b_napi_binding_erases_at_cdylib_boundary_only() {
    // G13-B implementer wires this (post-napi-r4-r1-3 reshape):
    //
    //   // Compile-time witness: `benten_napi::Engine` MUST resolve to
    //   // a concrete type alias. The PhantomData reference compiles
    //   // only if Engine is concrete (not generic-bound).
    //   let _witness: std::marker::PhantomData<benten_napi::Engine>
    //       = std::marker::PhantomData;
    //
    //   // Defense-in-depth — runtime sanity assertion that a public
    //   // surface fn taking the concrete Engine type is callable
    //   // (drives the production-grade entry point per pim-2 §3.6b):
    //   let _ = benten_napi::testing::open_in_memory_engine();
    //
    //   // The companion compile-fail doctest at the module-level
    //   // rustdoc asserts that `benten_napi::Engine<RedbBackend>` is
    //   // a compile error (the alias is concrete; supplying a type
    //   // argument should fail compilation):
    //   //
    //   // ```compile_fail
    //   // fn _attempt() {
    //   //     let _: benten_napi::Engine<benten_graph::RedbBackend> = unreachable!();
    //   // }
    //   // ```
    //
    // OBSERVABLE consequence: a future PR that widens the napi public
    // surface to expose `EngineGeneric<B>` directly (instead of erasing
    // to a concrete alias) fails this test AT COMPILE TIME — there is
    // no missed-grep-pattern attack surface. Per pim-2 §3.6b the test
    // drives the actual `benten_napi::Engine` symbol resolution; the
    // compile-fail pin asserts the negative case.
    //
    // ALTERNATIVE: if napi-r4-r1-3 RECOMMEND (b) is preferred — delete
    // this file as redundant with `bindings/napi/tests/native_default.rs`
    // which already pins compile-success of the napi binding for both
    // native + wasm32 targets. The compile-time witness here is a
    // symbol-level check; the native_default test is a build-level
    // check. Defense-in-depth keeps both.
    unimplemented!(
        "G13-B wires compile-time witness for benten_napi::Engine concrete-alias resolution per napi-r4-r1-3"
    );
}

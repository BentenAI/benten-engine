//! G13-B GREEN pin: napi binding erasure boundary at the cdylib edge
//! (Phase-3 R5 wave-2; D-PHASE-3-1a + arch-r1-1).
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
//! type is erased back to a concrete shape:
//! `Engine` = `EngineGeneric<RedbBackend>` for the native cdylib;
//! `EngineGeneric<BrowserBackend>` for the browser-target cdylib.
//!
//! This pin is the **engine-side** compile-time witness reachable from
//! the napi integration-test crate (`benten_engine::Engine` is the
//! concrete alias `napi_surface::Engine` wraps). The napi cdylib's
//! internal `napi_surface::Engine` struct is private to the crate and
//! cannot be referenced from an integration test — but the underlying
//! engine alias IS the load-bearing concrete-alias surface, and witnessing
//! IT closes the structural contract: if `benten_engine::Engine` were
//! widened to `Engine<B>` (generic-bound), this test fails to compile
//! AND the napi cdylib (which uses `Engine` everywhere internally)
//! fails to compile alongside.
//!
//! Per pim-2 §3.6b: drives the actual `benten_engine::Engine` symbol at
//! compile time + observable consequence (compile-failure if the alias
//! widens to a generic-bound shape). The native_default.rs sibling pins
//! the same alias via `fn(&benten_engine::Engine)` fn-pointer assignment;
//! this pin uses the PhantomData form for defense-in-depth (different
//! syntactic position; either alone catches the regression but both
//! together survive a future refactor that breaks one syntactic form).
//!
//! ## Disposition note (pre-v1 Class A un-ignore, 2026-05-10)
//!
//! Original RED-PHASE pin asked for `PhantomData<benten_napi::Engine>`.
//! `benten_napi::Engine` is NOT publicly exported (the `napi_surface`
//! module that defines it is private; only `pub use policy::PolicyKind`
//! crosses the napi crate boundary). Reshaped to drive
//! `benten_engine::Engine` directly — the engine-side concrete alias the
//! napi `napi_surface::Engine` wraps — which IS publicly reachable from
//! the integration test and carries the same structural contract.

#![allow(clippy::unwrap_used, dead_code)]

#[test]
fn engine_b_napi_binding_erases_at_cdylib_boundary_only() {
    // Compile-time witness: `benten_engine::Engine` MUST resolve to a
    // concrete type alias. The PhantomData reference compiles only when
    // `Engine` is concrete (not generic-bound) — the engine-side alias
    // the napi `napi_surface::Engine` wraps. If a future refactor flips
    // `Engine` away from the concrete `EngineGeneric<RedbBackend>`
    // specialization toward an exposed `<B>`-generic shape, the line
    // below fails to compile + this test never produces — surfacing the
    // alias regression at the integration-test crate's compile step
    // before the cdylib build catches the surface-area mismatch.
    let _witness: std::marker::PhantomData<benten_engine::Engine> = std::marker::PhantomData;

    // Defense-in-depth — runtime sanity that the alias resolves cleanly
    // to a value-level position (not just a type-position witness). A
    // function that accepts `&benten_engine::Engine` is the concrete-
    // type contract surface the napi cdylib's `JsEngine.inner: Arc<...>`
    // field consumes internally.
    fn _accepts_concrete_alias(_engine: &benten_engine::Engine) {}
    let _: fn(&benten_engine::Engine) = _accepts_concrete_alias;

    // OBSERVABLE consequence: a future PR that widens the engine alias
    // to `Engine<B>` (or relocates the alias) fails this test AT COMPILE
    // TIME — no missed-grep-pattern attack surface. Per pim-2 §3.6b the
    // test drives the alias symbol resolution at TWO syntactic positions
    // (PhantomData type + fn-pointer assignment). Companion to
    // `bindings/napi/tests/native_default.rs::engine_napi_binding_compiles_native_redb_default`.
}

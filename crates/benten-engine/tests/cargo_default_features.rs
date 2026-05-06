//! G13-B GREEN-PHASE pin: cargo feature × default-Engine-alias mapping
//! (Phase-3 R5 wave-2; arch-r1-9).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + arch-r1-9):
//!
//! - `crates/benten-engine/tests/cargo_default_features.rs::cargo_workspace_default_features_yields_engine_generic_redbbackend` — arch-r1-9 (G13-B GREEN)
//! - `crates/benten-engine/tests/cargo_default_features.rs::cargo_browser_backend_feature_yields_engine_generic_browserbackend` — arch-r1-9 (G13-C STILL RED)
//!
//! ## What arch-r1-9 pins
//!
//! Default-features build: `Engine = EngineGeneric<RedbBackend>`.
//! `--features browser-backend` build: `Engine = EngineGeneric<BrowserBackend>`.
//!
//! Workspace-wide `cargo check --workspace --all-targets` runs DEFAULT
//! features only — that's the canonical CI surface and defines what
//! "an engine" means for downstream consumers absent explicit feature
//! opt-in.
//!
//! G13-B lands the redb default-alias pin; the browser-backend pin
//! stays `#[ignore]`'d until G13-C wave-3 introduces the
//! `BrowserBackend` impl + the `browser-backend` cargo feature on
//! benten-engine that re-points the alias.

#![allow(clippy::unwrap_used, clippy::used_underscore_items)]

use benten_engine::{Engine, EngineGeneric};
use benten_graph::RedbBackend;

#[test]
fn cargo_workspace_default_features_yields_engine_generic_redbbackend() {
    // arch-r1-9 G13-B GREEN pin. Compile-time type-equality witness
    // under default features: the public `Engine` alias resolves to
    // `EngineGeneric<RedbBackend>`.
    //
    // The witness uses an `assert_eq_type` helper with
    // `PhantomData<T>` so the type-equality is enforced at compile
    // time. The test body never actually constructs an Engine —
    // failing to compile is the assertion shape.
    fn assert_alias_equals<T1, T2>()
    where
        T1: Sized,
        T2: Sized,
        for<'a> &'a T1: Into<&'a T2>,
    {
    }
    // Default-features build: alias resolves to redb specialization.
    assert_alias_equals::<Engine, EngineGeneric<RedbBackend>>();

    // Direct compile-time pin via fn-pointer transmutation: a function
    // taking `&EngineGeneric<RedbBackend>` accepts a `&Engine` value
    // (and vice-versa) iff they're the same type.
    fn _accepts_redb_specialization(_: &EngineGeneric<RedbBackend>) {}
    fn _accepts_alias(e: &Engine) {
        _accepts_redb_specialization(e);
    }

    // Smoke at runtime: open via the default alias compiles + runs.
    // Defends against a regression where the default-alias line is
    // dropped or flipped to a different backend.
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("g13_b_default_features.redb");
    let engine: Engine = Engine::open(&db_path).unwrap();
    assert!(!engine.is_read_only_snapshot());
}

#[test]
#[ignore = "RED-PHASE: G13-C wave-3 introduces BrowserBackend behind cargo feature `browser-backend`"]
fn cargo_browser_backend_feature_yields_engine_generic_browserbackend() {
    // arch-r1-9 G13-C STILL-RED pin. The `BrowserBackend` type does
    // not exist on main `624bf54` — G13-C wave-3 introduces it under
    // cargo feature `browser-backend` on `benten-graph`. G13-B's
    // engine-cascade scope explicitly excludes the BrowserBackend
    // landing per plan §3 G13-C row.
    //
    // G13-C implementer wires this:
    //   #[cfg(feature = "browser-backend")]
    //   {
    //       use benten_engine::{Engine, EngineGeneric};
    //       use benten_graph::BrowserBackend;
    //       fn _accepts_browser_specialization(_: &EngineGeneric<BrowserBackend>) {}
    //       fn _accepts_alias(e: &Engine) { _accepts_browser_specialization(e); }
    //   }
    //
    // OBSERVABLE consequence: under `cargo check --features
    // browser-backend`, the engine alias resolves to the browser
    // specialization. Defends against the regression where the
    // feature flag is added but the alias forgets to re-point.
    unimplemented!(
        "G13-C wires browser-backend feature-gated type-alias assertion (un-ignore at G13-C wave-3)"
    );
}

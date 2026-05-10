//! G13-B GREEN-PHASE pin: cargo feature × default-Engine-alias mapping
//! (Phase-3 R5 wave-2; arch-r1-9).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + arch-r1-9):
//!
//! - `crates/benten-engine/tests/cargo_default_features.rs::cargo_workspace_default_features_yields_engine_generic_redbbackend` — arch-r1-9 (G13-B GREEN)
//! - `crates/benten-engine/tests/cargo_default_features.rs::cargo_browser_backend_feature_yields_engine_generic_browserbackend` — arch-r1-9 (G13-C GREEN)
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
//! G13-B shipped the redb default-alias pin; G13-C wave-3 shipped the
//! `BrowserBackend` impl + the `browser-backend` cargo feature on
//! benten-engine that re-points the alias. Both pins run by default;
//! the browser-backend type-equality witness is feature-gated.

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
fn cargo_browser_backend_feature_yields_engine_generic_browserbackend() {
    // arch-r1-9 G13-C GREEN pin. G13-C wave-3 introduces `BrowserBackend`
    // under cargo feature `browser-backend` on `benten-graph`, and adds a
    // matching feature on `benten-engine` that re-points the default
    // `Engine` alias from `EngineGeneric<RedbBackend>` to
    // `EngineGeneric<BrowserBackend>`.
    //
    // The test crate runs against the default-features build of
    // benten-engine, so the feature-gated compile-time assertion below
    // is conditionally compiled; on the default-features run it checks
    // source-side wiring, and on a `--features browser-backend` test
    // run it ALSO drives the type-equality witness.
    #[cfg(feature = "browser-backend")]
    {
        use benten_engine::{Engine, EngineGeneric};
        use benten_graph::BrowserBackend;
        fn _accepts_browser_specialization(_: &EngineGeneric<BrowserBackend>) {}
        fn _accepts_alias(e: &Engine) {
            _accepts_browser_specialization(e);
        }
        // Function-pointer transmutation pin: `&Engine` and
        // `&EngineGeneric<BrowserBackend>` are the same type under the
        // browser-backend feature.
        let _: fn(&EngineGeneric<BrowserBackend>) = _accepts_browser_specialization;
        let _: fn(&Engine) = _accepts_alias;
    }

    // Source-cite assertion always runs (independent of feature flag):
    // the Cargo.toml feature flag exists + forwards to the
    // benten-graph-side feature, AND the engine.rs alias has BOTH a
    // `cfg(feature = "browser-backend")` arm AND a
    // `cfg(not(feature = "browser-backend"))` arm so the alias flips
    // cleanly between the two. Defends against the regression where
    // the feature flag is added but the alias forgets to re-point.
    let cargo_toml = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .expect("read benten-engine/Cargo.toml");
    assert!(
        cargo_toml.contains(r#"browser-backend = ["benten-graph/browser-backend"]"#),
        "benten-engine/Cargo.toml MUST declare `browser-backend = [\"benten-graph/browser-backend\"]` per G13-C wave-3"
    );

    let engine_rs = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/engine.rs"))
        .expect("read benten-engine/src/engine.rs");
    assert!(
        engine_rs.contains("#[cfg(not(feature = \"browser-backend\"))]")
            && engine_rs.contains("pub type Engine = EngineGeneric<benten_graph::RedbBackend>"),
        "engine.rs MUST gate the redb alias arm with cfg(not(feature = \"browser-backend\")) per G13-C"
    );
    assert!(
        engine_rs.contains("#[cfg(feature = \"browser-backend\")]")
            && engine_rs.contains("pub type Engine = EngineGeneric<benten_graph::BrowserBackend>"),
        "engine.rs MUST add the browser-backend alias arm gated on cfg(feature = \"browser-backend\") per G13-C"
    );
}

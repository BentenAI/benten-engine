//! R3-A RED-PHASE pins: cargo feature × default-Engine-alias mapping
//! (G13-B wave 2 / G13-C wave 3; arch-r1-9).
//!
//! Pin sources (per r2-test-landscape §2.1 G13-B + arch-r1-9):
//!
//! - `tests/cargo_workspace_default_features_yields_engine_generic_redbbackend` — arch-r1-9
//! - `tests/cargo_browser_backend_feature_yields_engine_generic_browserbackend` — arch-r1-9
//!
//! ## What arch-r1-9 pins
//!
//! Default-features build:  `Engine = EngineGeneric<RedbBackend>`.
//! `--features browser-backend` build: `Engine = EngineGeneric<BrowserBackend>`.
//!
//! Workspace-wide `cargo check --workspace --all-targets` runs
//! DEFAULT features only — that's the canonical CI surface and
//! defines what "an engine" means for downstream consumers absent
//! explicit feature opt-in.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B introduces the type alias under default features"]
fn cargo_workspace_default_features_yields_engine_generic_redbbackend() {
    // arch-r1-9 pin. G13-B implementer wires this as a compile-time
    // type-equality assertion under default features:
    //
    //   #[cfg(not(feature = "browser-backend"))]
    //   fn assert_default() {
    //       fn _eq<T1, T2>() where T1: Sized, T2: Sized {}
    //       // Phantom alias-equality assertion: rely on type-inference
    //       // failing if the alias resolves to a different B.
    //       let _: benten_engine::Engine = unsafe { std::mem::zeroed() };
    //       // (Or the more rigorous shape: a const-fn comparing TypeId
    //       // of EngineGeneric<RedbBackend> vs EngineGeneric<BrowserBackend>.)
    //   }
    //
    // OBSERVABLE consequence: dropping the default-alias assignment
    // (or flipping default to BrowserBackend) fails this test.
    unimplemented!("G13-B wires default-features type-alias assertion");
}

#[test]
#[ignore = "RED-PHASE: G13-C introduces BrowserBackend behind cargo feature"]
fn cargo_browser_backend_feature_yields_engine_generic_browserbackend() {
    // arch-r1-9 pin. G13-C implementer wires this as a feature-gated
    // type-alias assertion:
    //
    //   #[cfg(feature = "browser-backend")]
    //   fn assert_browser() {
    //       fn _eq<T1, T2>() where T1: Sized, T2: Sized {}
    //       let _: benten_engine::Engine = unsafe { std::mem::zeroed() };
    //       // Pin: under --features browser-backend, the alias
    //       // resolves to EngineGeneric<BrowserBackend>.
    //   }
    //
    // OBSERVABLE consequence: under `cargo check --features
    // browser-backend`, the engine alias resolves to the browser
    // specialization. Defends against the regression where the
    // feature flag is added but the alias forgets to re-point.
    unimplemented!("G13-C wires browser-backend feature-gated type-alias assertion");
}

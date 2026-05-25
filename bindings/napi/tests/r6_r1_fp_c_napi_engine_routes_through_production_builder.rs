//! R6 R1 FP-F4 §S4 (CRITIC-1 FIX-3) — napi Engine construction
//! routing through `ProductionEngineBuilder`.
//!
//! At v1-beta the napi binding's `Engine::new(path)` and
//! `Engine::open_with_policy(path, policy)` routes go through
//! `EngineBuilder::new()` directly — NOT through
//! `ProductionEngineBuilder`. Per the Δv3-10 + CRITIC-1 FIX-3 brief
//! disposition this pin is the forensic anchor for the v1-beta
//! posture: the napi binding does NOT YET install the substantive
//! `ProductionManifestEnvelopeRechecker` automatically. The
//! migration is a deliberate v1-beta-to-v1-GM window decision (a
//! Phase-4-Meta-Composing wave per Row D-4 narrative); the napi
//! binding migration coupled to the broader plugin-lifecycle wiring
//! lands together.
//!
//! This file is a compile-test only: it verifies that the
//! `ProductionEngineBuilder` type IS accessible at the engine-side
//! public surface (so a future napi binding migration is a one-line
//! change in `bindings/napi/src/lib.rs`).
//!
//! ## Why a separate compile-test pin?
//!
//! Per CRITIC-1 FIX-3: a future wave that adds the napi migration
//! should be one-line + verifiable. This pin asserts the surface
//! is available; the migration is the actual wiring.

#![cfg(not(target_arch = "wasm32"))]

use benten_engine::production_engine_builder::ProductionEngineBuilder;

#[test]
fn production_engine_builder_surface_is_accessible_from_napi_binding_crate() {
    // The napi binding's `bindings/napi/src/lib.rs:Engine::new` +
    // `Engine::open_with_policy` can be migrated to:
    //
    //   let inner = ProductionEngineBuilder::new().open(&path)?;
    //
    // (instead of `InnerEngine::open(&path)?` + raw EngineBuilder).
    //
    // This pin asserts the surface exists; the migration itself is
    // a Phase-4-Meta-Composing wave per Row D-4 narrative.
    let _b = ProductionEngineBuilder::new();
}

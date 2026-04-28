//! Phase 2b R3-B — SANDBOX fuel-axis unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A (fuel limit), dx-r1-2b-5 (default 1_000_000).
//!
//! Wave-8b: bodies wired against the live wasmtime invocation pipeline
//! (`primitives::sandbox::execute`).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

fn dummy_attribution() -> AttributionFrame {
    let zero = Cid::from_blake3_digest([0u8; 32]);
    AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    }
}

#[test]
fn sandbox_fuel_exhausts_routes_e_sandbox_fuel_exhausted() {
    // Plan §3 G7-A — module enters tight `loop ... br 0 ... end` with
    // a small fuel budget; assert E_SANDBOX_FUEL_EXHAUSTED fires.
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) (loop $L br $L) i32.const 0))")
            .unwrap();
    let registry = ManifestRegistry::new();
    // Wallclock generous so fuel fires first (D21: WALLCLOCK > FUEL,
    // but we want fuel as the active limiter).
    let cfg = SandboxConfig {
        fuel: 10_000,
        wallclock_ms: 60_000,
        ..SandboxConfig::default()
    };
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        cfg,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxFuelExhausted);
}

#[test]
fn sandbox_fuel_default_1_000_000_runs_canonical_fixture() {
    // dx-r1-2b-5 — default fuel budget = 1_000_000 wasmtime units;
    // canonical fixture (constant-return) completes well within that
    // default with `result.fuel_consumed < 1_000_000`.
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))").unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let res = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .expect("constant-return runs under default fuel");
    assert!(
        res.fuel_consumed < 1_000_000,
        "fuel_consumed {} must be < default 1_000_000",
        res.fuel_consumed
    );
}

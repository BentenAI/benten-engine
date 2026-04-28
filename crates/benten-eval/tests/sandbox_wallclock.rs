//! Phase 2b R3-B — SANDBOX wallclock-axis unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A, D24-RESOLVED (30s default / 5min max),
//! D6 + D24 (per-handler override via SubgraphSpec.primitives).
//!
//! Wave-8b: wired against the live wasmtime epoch-interruption pipeline.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    ManifestRef, ManifestRegistry, SandboxConfig, WALLCLOCK_DEFAULT_MS, WALLCLOCK_MAX_MS, execute,
};

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
fn sandbox_wallclock_kills_routes_e_sandbox_wallclock_exceeded() {
    // Wallclock = 50ms; fuel generous so wallclock fires first.
    // (D21 priority verifies wallclock > fuel; the priority test lives
    // in sandbox_severity_priority.rs.)
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) (loop $L br $L) i32.const 0))")
            .unwrap();
    let registry = ManifestRegistry::new();
    let cfg = SandboxConfig {
        fuel: u64::MAX / 2, // effectively infinite — wallclock should fire
        wallclock_ms: 50,
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
    assert_eq!(err.code(), ErrorCode::SandboxWallclockExceeded);
}

#[test]
fn sandbox_wallclock_default_30s_max_5min() {
    // D24-RESOLVED defaults pinned at the type level.
    assert_eq!(SandboxConfig::default().wallclock_ms, 30_000);
    assert_eq!(WALLCLOCK_DEFAULT_MS, 30_000);
    assert_eq!(WALLCLOCK_MAX_MS, 5 * 60_000);

    // Within ceiling — accepted.
    let _ok = SandboxConfig::default().with_wallclock_ms(60_000).unwrap();

    // Above ceiling — rejected.
    let err = SandboxConfig::default()
        .with_wallclock_ms(600_000)
        .unwrap_err();
    assert_eq!(err, ErrorCode::SandboxWallclockInvalid);
}

#[test]
#[ignore = "Wave-8b: per-handler override via SubgraphSpec.primitives requires the engine-side wire-through (paired 8c sub-track) to thread the primitive properties into SandboxConfig at dispatch time. This file's `sandbox::execute` surface accepts the override directly; the missing layer is `Engine::execute_sandbox_native` reading the SANDBOX node's `wallclock_ms` property from the operation node. Lifted in 8c."]
fn sandbox_wallclock_per_handler_override_via_subgraphspec_primitives() {
    // Pin: per-handler override is ENGINE-layer property propagation;
    // Wave-8b's primitive-level execute() takes a SandboxConfig directly
    // and respects whatever ms-budget the caller passes. The
    // engine-side wire-through is paired 8c work.
}

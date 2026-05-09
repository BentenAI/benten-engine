//! R6FP-Group-1 (r6-cr-1 / r6-mpc-4 / r6-wsa-1) regression pin —
//! Inv-4 / D20 SANDBOX runtime arm fires when
//! `AttributionFrame.sandbox_depth > SandboxConfig.max_nest_depth`.
//!
//! Pre-R6FP-G1 the runtime arm was dormant: the engine-side production
//! override hardcoded `sandbox_depth: 1` literally at every SANDBOX
//! entry, so the chain handler→CALL→handler→...-with-SANDBOX could not
//! deepen past 1. The 3-lens convergent finding (R6 code-reviewer +
//! metadata-producer-vs-consumer + wasmtime-sandbox-auditor) named the
//! threading lift as the load-bearing fix.
//!
//! This test exercises the eval-side `sandbox::execute` directly with a
//! crafted `AttributionFrame.sandbox_depth` that exceeds the configured
//! `SandboxConfig.max_nest_depth`. The runtime arm at the top of
//! `execute` body MUST fire `SandboxError::NestedDispatchDepthExceeded`
//! before any wasmtime work runs. The companion engine-side threading
//! pin (in `crates/benten-engine/tests/...`) verifies the engine
//! override constructs the AttributionFrame with the cumulative depth
//! `parent.sandbox_depth + 1`.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError};

mod sandbox {
    pub use benten_eval::sandbox::execute;
}

fn trivial_run_module_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))")
        .expect("trivial run module compiles")
}

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

/// R6FP-G1 regression pin: when the dispatching attribution frame's
/// `sandbox_depth` exceeds the configured `max_nest_depth`, the
/// runtime arm fires BEFORE wasmtime runs. The test sets
/// `max_nest_depth = 2` and an attribution depth of `3` — the arm
/// surfaces `SandboxError::NestedDispatchDepthExceeded { max: 2 }`.
#[test]
fn inv_4_runtime_arm_fires_when_depth_exceeds_max() {
    let bytes = trivial_run_module_bytes();
    let registry = ManifestRegistry::new();
    let manifest_ref = ManifestRef::Inline(CapBundle::new(Vec::new(), None));

    let mut config = SandboxConfig::default();
    config.max_nest_depth = 2;

    let attribution = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        // Depth 3 > max 2 — runtime arm trips.
        sandbox_depth: 3,
        ..Default::default()
    };

    let result = sandbox::execute(&bytes, manifest_ref, &registry, config, &[], &attribution);

    match result {
        Err(SandboxError::NestedDispatchDepthExceeded { max }) => {
            assert_eq!(
                max, 2,
                "the runtime arm surfaces the configured max in the error \
                 payload so operators can correlate against their config"
            );
        }
        other => panic!(
            "expected SandboxError::NestedDispatchDepthExceeded {{ max: 2 }}, \
             got {other:?} — pre-R6FP-G1 (r6-cr-1) the runtime arm was \
             dormant because the engine-side override hardcoded \
             sandbox_depth: 1 literally and the eval-side execute body \
             never compared the depth to max_nest_depth at entry"
        ),
    }
}

/// Boundary pin: depth EQUAL to `max_nest_depth` is the FINAL admitted
/// level — the arm fires only when depth strictly exceeds the max.
/// Guards against an off-by-one that would reject legitimate deepest
/// allowed nesting.
#[test]
fn inv_4_runtime_arm_admits_depth_equal_to_max() {
    let bytes = trivial_run_module_bytes();
    let registry = ManifestRegistry::new();
    let manifest_ref = ManifestRef::Inline(CapBundle::new(Vec::new(), None));

    let mut config = SandboxConfig::default();
    config.max_nest_depth = 4;

    let attribution = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        // Depth 4 == max 4 — admitted. The next level (5) would trip.
        sandbox_depth: 4,
        ..Default::default()
    };

    let result = sandbox::execute(&bytes, manifest_ref, &registry, config, &[], &attribution);

    // Must NOT be NestedDispatchDepthExceeded. The trivial module
    // compiles + runs, returning Ok. (If a non-depth-related error
    // surfaced from wasmtime we still pass the depth check — the arm
    // is the load-bearing assertion, not the wasmtime success.)
    match result {
        Err(SandboxError::NestedDispatchDepthExceeded { .. }) => {
            panic!(
                "depth equal to max_nest_depth must NOT trip the runtime \
                 arm — the arm should only fire when depth strictly \
                 exceeds the max"
            );
        }
        _ => { /* arm did not fire — pass */ }
    }
}

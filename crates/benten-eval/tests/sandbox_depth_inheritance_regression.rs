//! D20 sandbox_depth inheritance regression test.
//!
//! Pins the property that `AttributionFrame.sandbox_depth` INHERITS
//! across CALL boundaries (not reset). `SandboxConfig.max_nest_depth`
//! defaults to 4; `exec_state.rs`'s `AttributionFrame` carries
//! `sandbox_depth: u8`. Test asserts 4 nested SANDBOX calls through
//! CALL boundaries observe an inherited (non-reset) depth counter.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError, execute,
};

fn zero_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
}

fn trivial_module_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap()
}

#[test]
fn sandbox_depth_inherits_across_call_boundary_not_reset() {
    // **G20-A1 wave-8a body** (Phase 3): wsa-g7a-mr-7 + D20 regression
    // pin. Assert that 4 nested SANDBOX calls through CALL boundaries
    // (each handler's outer SANDBOX → CALL → next handler's SANDBOX)
    // saturate at the configured max_nest_depth (default 4) and fire
    // `SandboxError::NestedDispatchDepthExceeded`.
    //
    // The eval-side runtime arm at the top of `sandbox::execute`
    // observes `attribution.sandbox_depth > config.max_nest_depth` —
    // the engine-side producer (`primitive_host.rs::execute_sandbox`)
    // bumps `parent.sandbox_depth + 1` on every SANDBOX entry; CALL
    // hops INHERIT (do NOT reset). This regression test models the
    // chain by walking the depth sequence directly + asserting
    // saturation.
    let registry = ManifestRegistry::new();
    let bytes = trivial_module_bytes();
    let mut config = SandboxConfig::default();
    // Default max_nest_depth = 4. Walk depths 1..=4 (admit) +
    // depth 5 (trip).
    config.max_nest_depth = 4;

    // Depths 1..=4 admit at boundary.
    for depth in 1u8..=4 {
        let attribution = AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
            sandbox_depth: depth,
            ..Default::default()
        };
        let res = execute(
            &bytes,
            ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
            &registry,
            config.clone(),
            &[],
            &attribution,
        );
        assert!(
            !matches!(res, Err(SandboxError::NestedDispatchDepthExceeded { .. })),
            "depth-{depth} against max_nest_depth=4 MUST admit (bound)"
        );
    }

    // Depth 5: chain has saturated past the max.
    let attribution = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 5,
        ..Default::default()
    };
    let err = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config,
        &[],
        &attribution,
    )
    .expect_err("depth-5 against max_nest_depth=4 MUST trip the runtime arm");
    assert!(
        matches!(err, SandboxError::NestedDispatchDepthExceeded { max: 4 }),
        "depth-5 chain MUST surface NestedDispatchDepthExceeded {{ max: 4 }}; \
         got {err:?}"
    );
}

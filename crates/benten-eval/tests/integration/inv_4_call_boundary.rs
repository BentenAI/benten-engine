//! Phase 2b R3-B — Inv-4 sandbox-depth across CALL boundary integration
//! tests (G7-B).
//!
//! Pin sources: wsa-D20, D20 + Phase-2a Inv-14 carry.
//!
//! These tests exercise the depth-inheritance pattern (D20) at the
//! cross-crate integration level (engine + eval together), not at the
//! eval unit level (those tests live in invariant_4_runtime.rs).
//!
//! **G20-A1 wave-8a** (Phase 3): bodies un-ignored. The eval-side
//! integration drives `sandbox::execute` directly with crafted
//! AttributionFrames simulating the engine-side
//! `primitive_host.rs::execute_sandbox` chain producer (which bumps
//! `parent.sandbox_depth + 1` per R6FP-Group-1). The engine-level
//! integration with full `engine.call` chain coverage is at
//! `crates/benten-engine/tests/integration/engine_sandbox.rs` (the
//! G20-A1 un-ignored body) + the runtime-arm pin at
//! `crates/benten-eval/tests/inv_4_runtime_arm_fires_at_max_depth.rs`.

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
fn invariant_4_sandbox_depth_crosses_call_boundary() {
    // wsa-D20 — eval-side integration of the depth-inheritance arm.
    // Models the chain handler1 → SANDBOX (depth=1) → CALL → handler2
    // → SANDBOX (depth=2) → CALL → handler3 → SANDBOX (depth=3) by
    // walking the cumulative depth sequence directly.
    //
    // With max_nest_depth=2, depth=2 admits at boundary; depth=3 trips.
    let registry = ManifestRegistry::new();
    let bytes = trivial_module_bytes();
    let mut config = SandboxConfig::default();
    config.max_nest_depth = 2;

    // depth=1 admits.
    let f1 = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 1,
        ..Default::default()
    };
    let res1 = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config.clone(),
        &[],
        &f1,
    );
    assert!(
        !matches!(res1, Err(SandboxError::NestedDispatchDepthExceeded { .. })),
        "depth-1 admits"
    );

    // depth=2 admits at boundary.
    let f2 = AttributionFrame {
        sandbox_depth: 2,
        ..f1.clone()
    };
    let res2 = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config.clone(),
        &[],
        &f2,
    );
    assert!(
        !matches!(res2, Err(SandboxError::NestedDispatchDepthExceeded { .. })),
        "depth-2 admits at boundary"
    );

    // depth=3 trips (the CALL chain has saturated past max=2).
    let f3 = AttributionFrame {
        sandbox_depth: 3,
        ..f1.clone()
    };
    let err = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config,
        &[],
        &f3,
    )
    .expect_err("depth-3 against max=2 MUST trip");
    assert!(
        matches!(err, SandboxError::NestedDispatchDepthExceeded { max: 2 }),
        "engine-level chain end-to-end: depth-3 fires NestedDispatchDepthExceeded"
    );
}

#[test]
fn invariant_4_end_to_end_with_attribution_frame() {
    // D20 + Phase-2a Inv-14 carry — the depth counter rides on
    // AttributionFrame (Phase-2a sec-r6r1-01 closure shape). This
    // pin covers the full attribution-chain integrity:
    //   1. Distinct (actor, handler, grant) per chain hop are
    //      preserved canonically (CIDs depend on values).
    //   2. depth=0 frames preserve the Phase-2a fixture CID
    //      (Inv-14 carry).
    //   3. Non-zero depth modifies the canonical bytes (D20 extension
    //      load-bearing).
    //   4. The chain CIDs are distinguishable across different
    //      handler/actor/grant combinations + different depths
    //      (no canonicalisation collisions).

    let actor_a = Cid::from_blake3_digest(*blake3::hash(b"actor:A").as_bytes());
    let handler_a = Cid::from_blake3_digest(*blake3::hash(b"handler:A").as_bytes());
    let grant_a = Cid::from_blake3_digest(*blake3::hash(b"grant:A").as_bytes());

    let actor_x = Cid::from_blake3_digest(*blake3::hash(b"actor:X").as_bytes());
    let handler_b = Cid::from_blake3_digest(*blake3::hash(b"handler:B").as_bytes());

    // Frame at depth=1 for handler_a / actor_a.
    let frame_outer = AttributionFrame {
        actor_cid: actor_a,
        handler_cid: handler_a,
        capability_grant_cid: grant_a,
        sandbox_depth: 1,
        ..Default::default()
    };
    // After CALL inheritance: actor stays actor_a, handler shifts to
    // handler_b, grant stays grant_a (cap-attenuation chain), depth
    // bumps to 2 at the next SANDBOX.
    let frame_inner = AttributionFrame {
        actor_cid: actor_a,
        handler_cid: handler_b,
        capability_grant_cid: grant_a,
        sandbox_depth: 2,
        ..Default::default()
    };
    // Frame for a different actor (X dispatches a DIFFERENT chain).
    let frame_other_actor = AttributionFrame {
        actor_cid: actor_x,
        handler_cid: handler_a,
        capability_grant_cid: grant_a,
        sandbox_depth: 1,
        ..Default::default()
    };

    let cid_outer = frame_outer.cid().expect("outer encodes");
    let cid_inner = frame_inner.cid().expect("inner encodes");
    let cid_other = frame_other_actor.cid().expect("other encodes");

    // All three CIDs are distinct: chains on different (actor,
    // handler, depth) combinations are content-distinguishable.
    assert_ne!(cid_outer, cid_inner, "outer vs inner CID distinct");
    assert_ne!(cid_outer, cid_other, "outer vs other-actor CID distinct");
    assert_ne!(cid_inner, cid_other, "inner vs other-actor CID distinct");

    // Inv-14 carry: depth-0 frames preserve the Phase-2a fixture.
    const PHASE_2A_FIXTURE: &str = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";
    let frame_default = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
        ..Default::default()
    };
    assert_eq!(
        frame_default.cid().unwrap().to_base32(),
        PHASE_2A_FIXTURE,
        "Inv-14 carry — Phase-2a-pinned fixture preserved across D20 \
         extension"
    );
}

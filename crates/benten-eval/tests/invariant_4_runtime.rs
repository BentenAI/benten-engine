//! Phase 2b R3-B — Inv-4 sandbox-depth runtime + D20 inheritance unit
//! tests (G7-B).
//!
//! D20-RESOLVED:
//!   - `AttributionFrame.sandbox_depth: u8` — counter on the evaluator
//!     frame (NOT on the SANDBOX executor — per-call instance lifecycle
//!     would discard a Store-resident counter).
//!   - INHERITED across CALL boundaries (NOT reset). Handler A SANDBOXes
//!     → CALLs handler B → SANDBOXes is depth-2, not two depth-1s.
//!   - Default max_nest_depth = 4. Saturates to typed error
//!     E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED on overflow.
//!
//! Pin sources: plan §3 G7-B, D20-RESOLVED, sec-pre-r1-03 (frame
//! threading), wsa-6 suggested fix.
//!
//! **G20-A1 wave-8a** (Phase 3): `#[ignore]` removed. Bodies cover three
//! orthogonal Inv-4 facets the engine-side runtime arm depends on:
//!   1. Direct depth trap from a freshly-constructed frame at the
//!      eval-side `execute` boundary (companion pin to
//!      `inv_4_runtime_arm_fires_at_max_depth.rs`).
//!   2. Inheritance through CALL: the chain
//!      `outer_sandbox(depth=1) -> CALL -> inner_sandbox(depth=2)`
//!      observes cumulative depth (engine-side producer at
//!      `primitive_host.rs::execute_sandbox` lines 966-1000).
//!   3. AttributionFrame canonical-bytes schema integrity under
//!      depth-extension (Inv-14 carry per sec-pre-r1-13).

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
fn invariant_4_sandbox_runtime_depth_traps() {
    // Plan §3 G7-B — runtime depth check fires when the dispatching
    // attribution.sandbox_depth STRICTLY exceeds max_nest_depth, BEFORE
    // any wasmtime instantiation work runs. The eval-side runtime arm
    // at the top of `sandbox::execute` body is the load-bearing check.
    let registry = ManifestRegistry::new();
    let manifest_ref = ManifestRef::Inline(CapBundle::new(Vec::new(), None));
    let mut config = SandboxConfig::default();
    config.max_nest_depth = 4;
    let attribution = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        // Depth 5 strictly exceeds max 4 — runtime arm trips.
        sandbox_depth: 5,
    };

    let bytes = trivial_module_bytes();
    let err = execute(&bytes, manifest_ref, &registry, config, &[], &attribution)
        .expect_err("depth-5 against max_nest_depth=4 MUST trip the runtime arm");

    match err {
        SandboxError::NestedDispatchDepthExceeded { max } => {
            assert_eq!(max, 4, "the runtime arm carries the configured max");
        }
        other => panic!(
            "expected NestedDispatchDepthExceeded {{ max: 4 }}; got {other:?} \
             — runtime arm dormant if this fails"
        ),
    }
}

#[test]
fn invariant_4_depth_inherited_across_call_boundary() {
    // D20 inherit-not-reset — the security claim. Handler A SANDBOXes
    // → CALLs handler B → SANDBOXes is depth-2 cumulative (NOT two
    // separate depth-1s).
    //
    // The engine-side producer is
    // `crates/benten-engine/src/primitive_host.rs::execute_sandbox`
    // (per the comment trail at lines 966-1000 calling out
    // R6FP-Group-1: bumps the parent ActiveCall's sandbox_depth on
    // every SANDBOX entry, then a subsequent CALL pushes a child
    // frame inheriting parent.sandbox_depth via
    // `engine.rs::dispatch_call_inner`). The eval-side consumer is
    // the runtime arm at the top of `sandbox::execute`.
    //
    // Here we model that producer behaviour directly: walk through
    // each frame in the inheritance chain and assert the eval-side
    // arm observes the cumulative depth correctly.
    let registry = ManifestRegistry::new();
    let bytes = trivial_module_bytes();
    let mut config = SandboxConfig::default();
    config.max_nest_depth = 2;

    let base_frame = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
    };

    // Outer SANDBOX entry — depth bumped to 1. Admits.
    let frame_outer = AttributionFrame {
        sandbox_depth: 1,
        ..base_frame
    };
    let res_outer = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config.clone(),
        &[],
        &frame_outer,
    );
    assert!(
        !matches!(
            res_outer,
            Err(SandboxError::NestedDispatchDepthExceeded { .. })
        ),
        "depth-1 against max_nest_depth=2 MUST admit"
    );

    // After CALL inherits depth=1; inner SANDBOX bumps to depth=2.
    // Admits at the boundary.
    let frame_inner = AttributionFrame {
        sandbox_depth: 2,
        ..base_frame
    };
    let res_inner = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config.clone(),
        &[],
        &frame_inner,
    );
    assert!(
        !matches!(
            res_inner,
            Err(SandboxError::NestedDispatchDepthExceeded { .. })
        ),
        "depth-2 against max_nest_depth=2 MUST admit (boundary)"
    );

    // Depth-3: the chain HAS deepened past the max.
    let frame_too_deep = AttributionFrame {
        sandbox_depth: 3,
        ..base_frame
    };
    let err = execute(
        &bytes,
        ManifestRef::Inline(CapBundle::new(Vec::new(), None)),
        &registry,
        config,
        &[],
        &frame_too_deep,
    )
    .expect_err("depth-3 against max_nest_depth=2 MUST trip the inheritance arm");
    assert!(
        matches!(err, SandboxError::NestedDispatchDepthExceeded { max: 2 }),
        "expected NestedDispatchDepthExceeded {{ max: 2 }}; got {err:?}"
    );
}

#[test]
fn invariant_4_depth_inherited_through_attribution_frame() {
    // D20 white-box — assert the AttributionFrame schema slot for
    // `sandbox_depth` is the load-bearing inheritance carrier. The
    // schema-fixture pattern asserts:
    //   1. depth=0 frames produce the Phase-2a-pinned schema CID
    //      (stability — Inv-14 carry per sec-pre-r1-13).
    //   2. Non-zero depth frames produce a DISTINCT CID (security
    //      pin: a SANDBOX-bearing chain is content-distinguishable).
    //   3. Two frames at the same non-zero depth produce the same CID
    //      (canonicalisation pin).
    //   4. Two frames at different non-zero depths produce DIFFERENT
    //      CIDs (the field is load-bearing in canonical bytes).

    // Phase-2a-pinned schema CID for a default (depth=0) frame.
    const PHASE_2A_FIXTURE: &str = "bafyr4ig26oo2jmvq47wewho4sdpiscjpluvpzev3uerleuj2rtl63r7c5a";

    let frame_zero = AttributionFrame {
        actor_cid: zero_cid(),
        handler_cid: zero_cid(),
        capability_grant_cid: zero_cid(),
        sandbox_depth: 0,
    };
    let cid_zero = frame_zero.cid().expect("default frame encodes");
    assert_eq!(
        cid_zero.to_base32(),
        PHASE_2A_FIXTURE,
        "depth-0 frame must produce the Phase-2a-pinned schema CID; \
         the `sandbox_depth` slot is omitted from canonical bytes \
         when zero per exec_state.rs::AttributionFrame::cid"
    );

    let frame_one = AttributionFrame {
        sandbox_depth: 1,
        ..frame_zero
    };
    let cid_one = frame_one.cid().expect("depth-1 frame encodes");
    assert_ne!(
        cid_one, cid_zero,
        "non-zero sandbox_depth MUST produce a distinct CID — security pin"
    );

    let frame_one_again = AttributionFrame {
        sandbox_depth: 1,
        ..frame_zero
    };
    assert_eq!(
        frame_one_again.cid().unwrap(),
        cid_one,
        "two frames at the same depth canonicalise to the same CID"
    );

    let frame_two = AttributionFrame {
        sandbox_depth: 2,
        ..frame_zero
    };
    assert_ne!(
        frame_two.cid().unwrap(),
        cid_one,
        "different depths produce different CIDs — `sandbox_depth` \
         is load-bearing in canonical bytes"
    );
}

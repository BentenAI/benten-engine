//! Phase 2b R3-B — SANDBOX core unit tests (G7-A).
//!
//! **cr-g7a-mr-1 fix-pass:** 2 of 4 tests FLIPPED from `#[ignore]`
//! `todo!()` to live assertions against the G7-A-landed surface.
//! The remaining 2 (`sandbox_end_to_end`, `sandbox_no_state_persists_across_calls`)
//! need G7-C engine integration to fire — markers re-pointed to PR #33.
//!
//! Pin sources: plan §3 G7-A, wsa-15 (rename), wsa-20 (Engine singleton +
//! Module cache), D3-RESOLVED (per-call instance lifecycle), D22-precondition.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_eval::sandbox::instance::{module_cache_size, module_for_bytes, shared_engine};
use std::sync::Arc;

#[test]
fn sandbox_end_to_end() {
    // Wave-8b: minimal echo-shaped module returning a constant via the
    // primitive-level `sandbox::execute` surface.
    use benten_core::Cid;
    use benten_eval::AttributionFrame;
    use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))").unwrap();
    let registry = ManifestRegistry::new();
    let zero = Cid::from_blake3_digest([0u8; 32]);
    let attribution = AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    };
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
    .unwrap();
    // i32::42 little-endian.
    assert_eq!(res.output, vec![42, 0, 0, 0]);
    assert!(res.fuel_consumed > 0);
}

#[test]
fn sandbox_no_state_persists_across_calls() {
    // wsa-15 — module-global memory MUST reset across primitive calls
    // (per-call Store+Instance lifecycle, D3-RESOLVED).
    use benten_core::Cid;
    use benten_eval::AttributionFrame;
    use benten_eval::sandbox::{ManifestRef, ManifestRegistry, SandboxConfig, execute};

    // Module: increments a global on each `run` call, returns its value.
    let bytes = wat::parse_str(
        "(module
           (global $g (mut i32) (i32.const 0))
           (func (export \"run\") (result i32)
             global.get $g
             i32.const 1
             i32.add
             global.set $g
             global.get $g
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let zero = Cid::from_blake3_digest([0u8; 32]);
    let attribution = AttributionFrame {
        actor_cid: zero,
        handler_cid: zero,
        capability_grant_cid: zero,
        sandbox_depth: 0,
    };
    let grant = vec![
        "host:compute:log".to_string(),
        "host:compute:time".to_string(),
    ];
    // Call 1 — global ends at 1.
    let r1 = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &grant,
        &attribution,
    )
    .unwrap();
    assert_eq!(r1.output, vec![1, 0, 0, 0]);
    // Call 2 — fresh Store+Instance, global resets to 0, ends at 1
    // again.
    let r2 = execute(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        SandboxConfig::default(),
        &grant,
        &attribution,
    )
    .unwrap();
    assert_eq!(
        r2.output, r1.output,
        "wsa-15 — global memory MUST reset across calls"
    );
}

#[test]
fn sandbox_engine_singleton_lifetime() {
    // wsa-20 + D3-RESOLVED — `wasmtime::Engine` constructed ONCE per
    // benten Engine open (not per primitive call). White-box test:
    // `benten_eval::sandbox::instance::shared_engine()` returns the
    // same `&'static Engine` reference on every call within a benten
    // Engine's lifetime.
    let a = shared_engine();
    let b = shared_engine();
    assert!(
        std::ptr::eq(a, b),
        "wsa-20 — shared_engine MUST return the same singleton reference"
    );
}

#[test]
fn sandbox_module_cache_avoids_recompilation_on_repeated_call() {
    // wsa-20 — `wasmtime::Module` is content-CID-cached. The cold-start
    // budget (D22 ≤2ms p95 Linux x86_64) is unmeetable if Module
    // recompiles per call.
    let bytes = wat::parse_str("(module)").unwrap();
    let initial_size = module_cache_size();
    let m1 = module_for_bytes(&bytes).unwrap();
    let m2 = module_for_bytes(&bytes).unwrap();
    // Arc pointer equality — second call returns the cached entry.
    assert!(
        Arc::ptr_eq(&m1, &m2),
        "wsa-20 — Module cache MUST reuse the compiled artifact"
    );
    // Cache must have grown by at most 1 entry (the new fixture if
    // not previously cached).
    let after_size = module_cache_size();
    assert!(
        after_size <= initial_size + 1,
        "module cache must contain at most one new entry per CID"
    );
}

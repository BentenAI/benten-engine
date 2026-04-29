//! Phase 2b R3-B — `kv:read` host-fn unit tests (G7-A).
//!
//! D1 + sec-pre-r1-06 §2.4 — per-call cap-check via D18 + 1000-read
//! default budget per primitive call.
//!
//! Test surface:
//!   1. Per-grant budget enforcement (sandbox_host_fn_kv_read_respects_per_grant_budget_1000).
//!   2. Per-call cap-recheck after revoke (sandbox_host_fn_kv_read_per_call_cap_check_after_revoke).
//!      — pin source: D18 + ESC-9.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

//! Wave-8b: budget enforcement wired in the live trampoline; the cap-
//! revoke path remains 8c (paired engine integration).

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
fn sandbox_host_fn_kv_read_respects_per_grant_budget_1000() {
    // 1001 invocations under default per_call_read_cap=1000.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"kv_read\"
             (func $kv (param i32 i32 i32 i32) (result i32)))
           (memory (export \"memory\") 1)
           (func (export \"run\") (result i32)
             (local $i i32)
             (loop $L
               i32.const 0 i32.const 4 i32.const 0 i32.const 0
               call $kv
               drop
               local.get $i
               i32.const 1
               i32.add
               local.tee $i
               i32.const 1001
               i32.lt_s
               br_if $L
             )
             local.get $i
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::named("compute-with-kv"),
        &registry,
        SandboxConfig::default(),
        &[
            "host:compute:kv:read".to_string(),
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxHostFnDenied);
}

#[test]
#[ignore = "Phase 3 — testing_revoke_cap_mid_call helper for kv:read deferred per docs/future/phase-3-backlog.md §7.3.A.7 (security-critical SANDBOX-escape pin; cross-ref SECURITY-POSTURE.md ESC matrix entry for ESC-9 + Compromise #4)"]
fn sandbox_host_fn_kv_read_per_call_cap_check_after_revoke() {
    // D18 + ESC-9 — `kv:read` is `cap_recheck = "per_call"` (sensitive).
    //
    // Test:
    //   1. Grant module `host:compute:kv:read` cap.
    //   2. Module invokes kv:read once → SUCCESS.
    //   3. `testing_revoke_cap_mid_call(engine, &kv_read_scope)`.
    //   4. Module invokes kv:read again → FAILS with
    //      E_SANDBOX_HOST_FN_DENIED.
    //
    // Mirrors ESC-9 escape vector. The umbrella ESC-9 driver (R3-C
    // territory) batches this into the security-class suite; this is
    // the surface-level unit-test for D18's per_call enforcement on
    // kv:read specifically.
    todo!("R5 G7-A — testing_revoke_cap_mid_call + per_call kv:read denial");
}

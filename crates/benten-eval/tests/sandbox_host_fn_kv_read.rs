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
        ..Default::default()
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

/// **G20-A1 wave-8a body** (Phase 3): D18 + ESC-9 — kv:read declared
/// `cap_recheck = "per_call"`; mid-call revocation trips the second
/// host-fn invocation. Drive `execute_with_live_cap_check` with a
/// 2-call kv_read module + flip-flag callback.
#[test]
fn sandbox_host_fn_kv_read_per_call_cap_check_after_revoke() {
    use benten_eval::sandbox::{LiveCapCheck, SandboxError, execute_with_live_cap_check};
    use std::sync::{Arc, Mutex};

    let bytes = wat::parse_str(
        r#"(module
            (import "host" "kv_read"
                (func $kvread (param i32 i32 i32 i32) (result i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                i32.const 0 i32.const 0 i32.const 0 i32.const 0
                call $kvread
                drop
                i32.const 0 i32.const 0 i32.const 0 i32.const 0
                call $kvread
            )
        )"#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let mut config = SandboxConfig::default();
    config.fuel = 10_000_000;

    let revoked = Arc::new(Mutex::new(false));
    let revoked_clone = Arc::clone(&revoked);
    let live_cap_check: LiveCapCheck = Arc::new(move |cap: &str| -> bool {
        if cap != "host:compute:kv:read" {
            return false;
        }
        let mut g = revoked_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *g {
            return false;
        }
        *g = true;
        true
    });

    let err = execute_with_live_cap_check(
        &bytes,
        ManifestRef::named("compute-with-kv"),
        &registry,
        config,
        &[
            "host:compute:kv:read".to_string(),
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
        Some(live_cap_check),
    )
    .expect_err("ESC-9 / D18 per_call: revocation MUST trip second call");
    assert!(
        matches!(err, SandboxError::HostFnDenied { ref cap } if cap == "host:compute:kv:read"),
        "kv:read PerCall recheck MUST surface HostFnDenied(kv:read); got {err:?}"
    );
}

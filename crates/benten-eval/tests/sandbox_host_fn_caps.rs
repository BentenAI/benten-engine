//! Phase 2b R3-B — SANDBOX host-fn capability intersection + D18 hybrid
//! cap-recheck cadence unit tests (G7-A).
//!
//! Pin sources: plan §3 G7-A, D7 (both-layers — init + per-invocation),
//! D18-RESOLVED (per-host-fn `cap_recheck = "per_call" | "per_boundary"`,
//! default `per_call` fail-secure), wsa D18 codegen drift, sec-r1 D7.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapBundle, LiveCapCheck, ManifestRef, ManifestRegistry, SandboxConfig, SandboxError, execute,
    execute_with_live_cap_check,
};
use std::sync::{Arc, Mutex};

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
fn sandbox_host_fn_capability_intersection_at_init() {
    // D7 init-snapshot intersection: grant has time+log; manifest
    // requires time+kv:read; init fails with SandboxHostFnDenied.
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(
        vec![
            "host:compute:kv:read".to_string(),
            "host:compute:time".to_string(),
        ],
        None,
    );
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap();
    let attribution = dummy_attribution();
    let err = execute(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        SandboxConfig::default(),
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .unwrap_err();
    assert_eq!(err.code(), ErrorCode::SandboxHostFnDenied);
}

/// **G20-A1 wave-8a body** (Phase 3): D18-RESOLVED — `kv:read`
/// declared `cap_recheck = "per_call"` in host-functions.toml; the
/// PerCall recheck cadence catches mid-call revocation. Drive
/// `execute_with_live_cap_check` with a 2-call kv_read module +
/// flip-flag callback; assert the second call surfaces HostFnDenied.
#[test]
fn sandbox_host_fn_per_call_recheck_after_revoke_for_kv_read() {
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
    .expect_err("D18 per_call: revocation between calls MUST trip");
    assert!(
        matches!(err, SandboxError::HostFnDenied { ref cap } if cap == "host:compute:kv:read"),
        "D18 per_call cadence MUST observe revocation; got {err:?}"
    );
}

/// **G20-A1 wave-8a body** (Phase 3): D18-RESOLVED per_boundary —
/// `log` is per_boundary; mid-call would-revoke does NOT trip the
/// log call (the boundary snapshot is the authority). Drive a 2-log
/// module with a flip-flag callback; both log calls succeed.
#[test]
fn sandbox_host_fn_per_boundary_recheck_for_time_log() {
    let bytes = wat::parse_str(
        r#"(module
            (import "host" "log" (func $log (param i32 i32)))
            (memory (export "memory") 1)
            (func (export "run") (result i32)
                i32.const 0 i32.const 4
                call $log
                i32.const 0 i32.const 4
                call $log
                i32.const 0
            )
        )"#,
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let attribution = dummy_attribution();
    let config = SandboxConfig::default();

    let invocation_count = Arc::new(Mutex::new(0u32));
    let invocation_count_clone = Arc::clone(&invocation_count);
    let live_cap_check: LiveCapCheck = Arc::new(move |cap: &str| -> bool {
        if cap != "host:compute:log" {
            return false;
        }
        let mut g = invocation_count_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *g += 1;
        // First call returns true; subsequent calls return false.
        *g == 1
    });

    let res = execute_with_live_cap_check(
        &bytes,
        ManifestRef::named("compute-basic"),
        &registry,
        config,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
        Some(live_cap_check),
    );
    assert!(
        res.is_ok(),
        "per_boundary host-fn (log) MUST use init-snapshot; both \
         log calls succeed despite mid-call revoke; got {res:?}"
    );
}

#[test]
fn sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call() {
    use benten_eval::sandbox::CapRecheckPolicy;
    assert_eq!(CapRecheckPolicy::default(), CapRecheckPolicy::PerCall);
}

/// **G20-A1 wave-8a body** (Phase 3): wsa D18 fail-secure default —
/// the rust-side `CapRecheckPolicy::default() == PerCall` is the
/// type-level fail-secure encoding. The TOML schema's `#[serde(default)]`
/// drift detector lives at `sandbox_named_manifest_codegen_drift.rs`;
/// this assertion pins the type-level default.
#[test]
fn _sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call_old() {
    use benten_eval::sandbox::CapRecheckPolicy;
    assert_eq!(
        CapRecheckPolicy::default(),
        CapRecheckPolicy::PerCall,
        "fail-secure: undeclared cap_recheck defaults to PerCall"
    );
}

/// **G20-A1 wave-8a body** (Phase 3): sec-r1 D7 — host-fn cap denial
/// surfaces as typed `SandboxError::HostFnDenied` (NOT a wasmtime
/// trap that would corrupt Store state). Drive `execute` with an
/// inline manifest claiming a cap not in the dispatcher's grant; the
/// returned error code is `E_SANDBOX_HOST_FN_DENIED` (the typed
/// path).
#[test]
fn sandbox_host_fn_denied_routes_typed_error_not_trap() {
    let bytes =
        wat::parse_str("(module (func (export \"run\") (result i32) i32.const 0))").unwrap();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:kv:read".to_string()], None);
    let attribution = dummy_attribution();
    // Grant lacks kv:read; init-snapshot intersection trips with
    // typed E_SANDBOX_HOST_FN_DENIED, NOT a wasmtime Trap.
    let err = execute(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        SandboxConfig::default(),
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
    )
    .expect_err("cap denial MUST fire typed error");
    assert_eq!(
        err.code(),
        ErrorCode::SandboxHostFnDenied,
        "sec-r1 D7: host-fn cap denial routes E_SANDBOX_HOST_FN_DENIED \
         (typed path; wasmtime trap path reserved for engine-side \
         enforcement); got {:?}",
        err.code()
    );
}

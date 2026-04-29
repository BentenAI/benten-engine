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
use benten_eval::sandbox::{CapBundle, ManifestRef, ManifestRegistry, SandboxConfig, execute};

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

#[test]
#[ignore = "Phase 3 — testing_revoke_cap_mid_call helper deferred per docs/future/phase-3-backlog.md §7.3.A.7 (security-critical SANDBOX-escape pin; cross-ref SECURITY-POSTURE.md ESC matrix entry for ESC-9 + Compromise #4 honest disclosure)"]
fn sandbox_host_fn_per_call_recheck_after_revoke_for_kv_read() {
    // D18-RESOLVED — `kv:read` declared `cap_recheck = "per_call"` in
    // host-functions.toml (sensitive — mutation/network/cross-tenant
    // surface).
    //
    // Test:
    //   1. Grant module `host:compute:kv:read` cap.
    //   2. SANDBOX call: module invokes kv:read → SUCCESS.
    //   3. Mid-call: orchestrator revokes the cap via
    //      `testing_revoke_cap_mid_call(engine, &kv_read_scope)`.
    //   4. Module invokes kv:read again → FAILS with
    //      E_SANDBOX_HOST_FN_DENIED (D18 per_call check sees revoked cap).
    todo!("R5 G7-A — testing_revoke_cap_mid_call helper + per_call enforcement");
}

#[test]
#[ignore = "Phase 3 — per_boundary mid-call revoke positive-test deferred per docs/future/phase-3-backlog.md §7.3.A.7 (testing_revoke_cap_mid_call helper; cross-ref SECURITY-POSTURE.md ESC matrix)"]
fn sandbox_host_fn_per_boundary_recheck_for_time_log() {
    // D18-RESOLVED — `time` and `log` declared `cap_recheck = "per_boundary"`
    // in host-functions.toml (cheap, output-bounded, idempotent reads
    // tolerate boundary granularity).
    //
    // Test:
    //   1. Grant module `host:compute:time` + `host:compute:log` caps.
    //   2. SANDBOX call: module invokes log → SUCCESS.
    //   3. Mid-call: orchestrator revokes `host:compute:log` cap.
    //   4. Module invokes log AGAIN → STILL SUCCEEDS (boundary snapshot
    //      taken at SANDBOX entry; revocation visible only at next
    //      primitive boundary).
    //
    // Positive test for the per_boundary semantics — the snapshot is
    // load-bearing for D22 ≤2ms cold-start (per-call check would add
    // policy-evaluation overhead per host-fn invocation).
    todo!("R5 G7-A — per_boundary uses init snapshot regardless of mid-call revoke");
}

#[test]
fn sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call() {
    use benten_eval::sandbox::CapRecheckPolicy;
    assert_eq!(CapRecheckPolicy::default(), CapRecheckPolicy::PerCall);
}

#[test]
#[ignore = "Phase 3 — typed-error-not-trap integration-shape pin deferred per docs/future/phase-3-backlog.md §7.3.A.7 (unit-level pin in trap_to_typed::tests::host_fn_denial_marker_round_trips_cap_denied; cross-ref SECURITY-POSTURE.md ESC matrix)"]
fn _sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call_old() {
    // wsa D18 — UNDECLARED `cap_recheck` field defaults to `per_call`
    // (fail-secure). Regression guard: a host-fn TOML entry without
    // explicit `cap_recheck = ...` MUST behave as if it declared
    // `per_call`.
    //
    // White-box test: parse a host-functions.toml fixture containing
    // an entry WITHOUT `cap_recheck`; assert the codegen-emitted
    // CapRecheckPolicy variant for that entry is `PerCall`.
    todo!("R5 G7-A — assert codegen default = PerCall for undeclared field");
}

#[test]
#[ignore = "Phase 3 — typed-error-not-trap full-ABI round-trip integration body deferred per docs/future/phase-3-backlog.md §7.3.A.7 (unit pin in trap_to_typed::tests; cross-ref SECURITY-POSTURE.md ESC matrix)"]
fn sandbox_host_fn_denied_routes_typed_error_not_trap() {
    // sec-r1 D7 — when a host-fn cap check fails, the engine surfaces
    // E_SANDBOX_HOST_FN_DENIED as a typed error THROUGH the host-fn
    // ABI (NOT as a wasmtime trap that would corrupt module state).
    //
    // The module receives an error-shaped return value from the host-fn
    // call; the trap path is reserved for engine-side enforcement
    // (memory/wallclock/fuel/output) where the module has no chance to
    // recover.
    //
    // Test: deny a cap; module's host-fn-return-value is the typed error
    // payload (engine consumes it on the way back out as
    // E_SANDBOX_HOST_FN_DENIED).
    todo!("R5 G7-A — assert host-fn cap denial routes typed error");
}

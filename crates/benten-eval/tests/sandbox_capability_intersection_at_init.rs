//! Phase 2b R3-C — D7 init-snapshot path: capability intersection at SANDBOX
//! entry (G7-A).
//!
//! Pin sources: D7-RESOLVED hybrid (init-snapshot + per-call live policy);
//! sec-pre-r1-02 Option-D; r1-security-auditor.json D7 recommendation;
//! r1-wasmtime-sandbox-auditor D18 init-snapshot allowlist;
//! r2-test-landscape.md §1.3 `sandbox_host_fn_capability_intersection_at_init`.
//!
//! Pairs with `sandbox_capability_check_per_call_after_revoke.rs` (D18
//! per-call live policy path). Together: D7 hybrid is fully exercised.
//!
//! **cr-g7a-mr-1 fix-pass:** 2 of 4 tests FLIPPED (cap-recheck-default +
//! init-snapshot intersection via the executor's `execute()` surface).
//! Other 2 (per_boundary mid-call revoke + codegen drift) need
//! G7-C engine integration + build.rs codegen pipeline.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(not(target_arch = "wasm32"))]

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_eval::AttributionFrame;
use benten_eval::sandbox::{
    CapAllowlist, CapBundle, CapRecheckPolicy, LiveCapCheck, ManifestRef, ManifestRegistry,
    SandboxConfig, default_host_fns, execute, execute_with_live_cap_check,
};
use std::sync::{Arc, Mutex};

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
    // Plan §3 G7-A — at SANDBOX entry, the executor intersects the
    // host-fn manifest declarations against the dispatching grant's
    // cap-set; the resulting allowlist is the manifest membership lens.
    //
    // G7-A surface check via execute(): manifest CLAIMS kv:read but the
    // live grant doesn't. Init-snapshot intersection produces an
    // allowlist of just {time, log}; the executor fires
    // SandboxHostFnDenied on the missing kv:read claim.
    let registry = ManifestRegistry::new();
    let module_bytes = wat::parse_str("(module)").unwrap();
    let attribution = dummy_attribution();
    let err = execute(
        &module_bytes,
        ManifestRef::named("compute-with-kv"),
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

    // sec-pre-r1-02 Option-D — companion CapAllowlist::intersect
    // assertion: the structural allowlist for this combination is just
    // {time, log}; kv:read absent.
    let allow = CapAllowlist::intersect(
        &[
            "host:compute:kv:read".to_string(),
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
    );
    assert_eq!(
        allow.allowed,
        vec![
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ]
    );
    assert!(!allow.contains("host:compute:kv:read"));
}

#[test]
fn sandbox_host_fn_per_boundary_cap_uses_init_snapshot() {
    // **G20-A1 wave-8a body** (Phase 3): D18-RESOLVED — per_boundary
    // host-fns (`time`, `log`) take the init-snapshot at SANDBOX entry
    // and continue to serve through the call even if the cap is
    // revoked mid-call (revocation visible only at the next primitive
    // boundary).
    //
    // The live_cap_check callback consults a shared revoke flag.
    // First call observes cap present + flips the flag; second call
    // would see the flag set, BUT for a per_boundary host-fn the
    // trampoline does NOT consult live_cap_check on every invocation
    // (only PerCall does). So a `log` call that comes AFTER the
    // mid-call revoke STILL succeeds — the per_boundary semantic is
    // load-bearing for D22 ≤2ms cold-start.
    //
    // The drive: a guest module that calls `log` twice. The
    // live_cap_check callback would-revoke `host:compute:log` mid-
    // call. If per_boundary correctly took init-snapshot, both calls
    // succeed (Ok). If per_boundary regressed to per-call cadence,
    // the second call would fail.
    let bytes = wat::parse_str(
        "(module
           (import \"host\" \"log\" (func $log (param i32 i32)))
           (memory (export \"memory\") 1)
           (func (export \"run\") (result i32)
             ;; first call
             i32.const 0 i32.const 4
             call $log
             ;; second call (post-mock-revoke)
             i32.const 0 i32.const 4
             call $log
             i32.const 0
           )
         )",
    )
    .unwrap();
    let registry = ManifestRegistry::new();
    let inline = CapBundle::new(vec!["host:compute:log".to_string()], None);
    let attribution = dummy_attribution();
    let config = SandboxConfig::default();

    // Callback: returns true on first invocation, then would-revoke.
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
        // Return true on first ever call; subsequent calls return
        // false (mock revocation). For a per_boundary host-fn the
        // trampoline does NOT consult this callback — so this
        // counter SHOULD remain at 0 (or at most 1, depending on
        // the boundary check schedule), and BOTH log calls should
        // succeed.
        *g == 1
    });

    let res = execute_with_live_cap_check(
        &bytes,
        ManifestRef::Inline(inline),
        &registry,
        config,
        &[
            "host:compute:log".to_string(),
            "host:compute:time".to_string(),
        ],
        &attribution,
        Some(live_cap_check),
    );

    // Per-boundary semantic: BOTH log calls succeed. (If the executor
    // regressed log to PerCall, the second call would fail with
    // SandboxHostFnDenied.)
    assert!(
        res.is_ok(),
        "per_boundary host-fn (log) MUST use init-snapshot — both calls \
         succeed even with mock revocation between them; got {res:?}"
    );
}

#[test]
fn sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call() {
    // wsa D18 fail-secure regression — `CapRecheckPolicy::default() ==
    // PerCall` is the type-level encoding of the fail-secure default.
    // Any host-fn entry that omits `cap_recheck` in the source TOML
    // gets PerCall via `#[serde(default)]`.
    assert_eq!(CapRecheckPolicy::default(), CapRecheckPolicy::PerCall);

    // Companion check: every D1-default entry that DOES declare
    // cap_recheck matches its TOML declaration (positive coverage).
    let table = default_host_fns();
    assert_eq!(
        table["kv:read"].cap_recheck,
        CapRecheckPolicy::PerCall,
        "D1 — kv:read MUST be per_call (sensitive surface)"
    );
}

#[test]
fn sandbox_host_fn_cap_recheck_policy_codegen_drift() {
    // **G20-A1 wave-8a body** (Phase 3): closes the manifest-vs-codegen
    // drift class (D2 sibling concern). The deeper drift detector
    // lives in `sandbox_named_manifest_codegen_drift.rs`; this test
    // pins the per-host-fn cap_recheck cadence policy in
    // codegen-emitted state.
    //
    // Specifically asserts the D1 surface declarations from
    // host-functions.toml match the codegen table.
    let table = default_host_fns();
    // per_boundary on time + log (cheap, output-bounded, idempotent).
    assert_eq!(
        table["time"].cap_recheck,
        CapRecheckPolicy::PerBoundary,
        "D1 — `time` MUST be per_boundary (cheap snapshot)"
    );
    assert_eq!(
        table["log"].cap_recheck,
        CapRecheckPolicy::PerBoundary,
        "D1 — `log` MUST be per_boundary (output-bounded)"
    );
    // per_call on kv:read + random (sensitive surfaces).
    assert_eq!(
        table["kv:read"].cap_recheck,
        CapRecheckPolicy::PerCall,
        "D1 — `kv:read` MUST be per_call (sensitive)"
    );
    assert_eq!(
        table["random"].cap_recheck,
        CapRecheckPolicy::PerCall,
        "D1 — `random` MUST be per_call (sensitive)"
    );
    // No surplus host-fns leaked through codegen — the table at
    // wave-8a time has exactly the D1 surface (time / log / kv:read /
    // random). A future host-fn addition flips this assertion +
    // forces an update of the cap_recheck pin above (intended:
    // adding a host-fn requires an explicit cap_recheck declaration
    // be pinned here).
    let names: Vec<&String> = table.keys().collect();
    assert_eq!(
        names.len(),
        4,
        "D1 surface MUST be exactly 4 host-fns at G20-A1 close; got \
         {}: {:?}",
        names.len(),
        names
    );

    // Companion smoke check that the CapBundle constructor is in
    // scope (was previously the only live line in this test body).
    let _ = CapBundle::new(vec![], None);
}

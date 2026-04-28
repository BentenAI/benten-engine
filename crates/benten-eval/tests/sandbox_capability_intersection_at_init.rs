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
    CapAllowlist, CapBundle, CapRecheckPolicy, ManifestRef, ManifestRegistry, SandboxConfig,
    default_host_fns, execute,
};

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
#[ignore = "Phase 2b G7-C pending — testing_revoke_cap_mid_call helper requires the live wasmtime trampoline G7-C wires (PR #33)."]
fn sandbox_host_fn_per_boundary_cap_uses_init_snapshot() {
    todo!("G7-C PR #33 — assert per_boundary log() still serves after mid-call revoke");
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
#[ignore = "Phase 2b G7-C pending — TOML codegen drift detector exercises the build.rs pipeline G7-C wires (PR #33). G7-A ships the static table inline (no build.rs); the drift surface lands in G7-C."]
fn sandbox_host_fn_cap_recheck_policy_codegen_drift() {
    // Closes the manifest-vs-codegen drift class (a D2 sibling concern).
    // Body lands when build.rs codegen pipeline lands in G7-C.
    let _ = CapBundle::new(vec![], None);
    todo!("G7-C PR #33 — walk host-functions.toml + assert codegen agreement");
}

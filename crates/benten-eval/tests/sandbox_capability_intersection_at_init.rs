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
//! Cross-territory: per R2 §10, R3-C owns cap-related security tests; this
//! file covers the init-snapshot leg of D7 hybrid (the per-boundary surface
//! per D18 — the `time`/`log` defaults).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef, host_fns}
//   benten_eval::sandbox::host_fns::CapRecheckPolicy::{PerCall, PerBoundary}
//   benten_caps::{CapabilityPolicy, CapScope}
//   benten_errors::ErrorCode::SandboxHostFnDenied

#[test]
#[ignore = "Phase 2b G7-A pending — D7 hybrid init-snapshot path"]
fn sandbox_host_fn_capability_intersection_at_init() {
    // Plan §3 G7-A — at SANDBOX entry, the executor intersects the
    // host-fn manifest declarations against the dispatching grant's
    // cap-set; the resulting allowlist is the manifest membership lens.
    //
    // R5 wires:
    //   1. Engine open with CapabilityPolicy granting host:compute:time
    //      + host:compute:log (no kv:read).
    //   2. sandbox_call(echo_cid, ManifestRef::Named("compute-with-kv"))
    //      — manifest CLAIMS kv:read but the live grant doesn't.
    //   3. Init-snapshot intersection produces an allowlist of just
    //      {time, log}; kv_read is NOT linkable. Subsequent kv_read call
    //      from inside the module fires SandboxHostFnDenied (or
    //      SandboxHostFnNotFound at link time).
    //   4. White-box assert: the per-call SandboxContext records the
    //      init-snapshot allowlist == {time, log} — kv:read absent.
    //
    // sec-pre-r1-02 Option-D recommendation — init-snapshot is consulted
    // ONLY for manifest-membership / link-time decisions; per-call live
    // check is the separate cap-string check (covered in companion
    // `_per_call_after_revoke.rs`).
    todo!("R5 G7-A — assert init-snapshot intersects manifest ∩ live grant");
}

#[test]
#[ignore = "Phase 2b G7-A pending — D18 per_boundary positive (init snapshot serves)"]
fn sandbox_host_fn_per_boundary_cap_uses_init_snapshot() {
    // Per D18 hybrid: host-fns declared `cap_recheck = "per_boundary"`
    // (D1 default for `time`/`log`) consult ONLY the init-snapshot per
    // host-fn invocation; revocations during the SANDBOX call don't
    // affect them until the next primitive boundary.
    //
    // R5 wires:
    //   1. Engine grants host:compute:time + host:compute:log at entry.
    //   2. sandbox_call begins; module calls log() (boundary-cap).
    //   3. Driver mid-call revokes host:compute:log via
    //      `testing_revoke_cap_mid_call(engine, &CapScope::host_compute_log())`.
    //   4. Module's NEXT log() call STILL succeeds — per_boundary semantics
    //      mean no live recheck during the call. The revocation takes
    //      effect at the NEXT SANDBOX primitive entry.
    //
    // wsa D18 positive test (`per_boundary_cap_uses_init_snapshot`).
    // Asymmetric pair to the per-call deny test.
    todo!("R5 G7-A — assert per_boundary log() still serves after mid-call revoke");
}

#[test]
#[ignore = "Phase 2b G7-A pending — fail-secure default cap_recheck"]
fn sandbox_host_fn_undeclared_cap_recheck_defaults_to_per_call() {
    // wsa D18 fail-secure regression — host-fn entries lacking the
    // explicit `cap_recheck` field in host-functions.toml MUST default
    // to per_call (the safer bound).
    //
    // R5 wires:
    //   1. Construct a host-fn fixture entry omitting cap_recheck.
    //   2. Codegen emits CapRecheckPolicy::PerCall for that entry.
    //   3. Test invocation revokes the cap mid-call; subsequent host-fn
    //      call denies with SandboxHostFnDenied.
    //
    // Pin: sec-pre-r1-02 fail-secure — auditors reading the manifest
    // know that anything not explicitly relaxed gets the tightest TOCTOU
    // window.
    todo!("R5 G7-A — assert undeclared cap_recheck defaults to PerCall");
}

#[test]
#[ignore = "Phase 2b G7-A pending — codegen drift detector (D2 + D18)"]
fn sandbox_host_fn_cap_recheck_policy_codegen_drift() {
    // wsa D18 codegen-drift detector — TOML schema and generated Rust
    // CapRecheckPolicy enum MUST agree.
    //
    // R5 wires (mirrors `error_code_drift_test` Phase-1 pattern):
    //   1. Walk every entry in `host-functions.toml`.
    //   2. For each entry, assert that the codegen-emitted CapRecheckPolicy
    //      matches the declared TOML field (default per_call when absent).
    //   3. Assert no orphan codegen variants exist beyond TOML entries.
    //
    // Closes the manifest-vs-codegen drift class (a D2 sibling concern).
    todo!("R5 G7-A — walk host-functions.toml + assert codegen agreement");
}

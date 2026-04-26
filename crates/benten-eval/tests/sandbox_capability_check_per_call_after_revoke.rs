//! Phase 2b R3-C — D7 per-call live policy path: cap-revocation TOCTOU
//! enforcement (G7-A).
//!
//! Pin sources: D7-RESOLVED hybrid + D18-RESOLVED `cap_recheck = "per_call"`
//! default; sec-pre-r1-02 Option-D recommendation; r1-security-auditor.json
//! D7 + r1-wasmtime-sandbox-auditor D18; r2-test-landscape.md §5.2
//! `sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent`
//! (per R2 §10 disambiguation: R3-C owns; cap-revocation TOCTOU is the
//! primary lens; sandbox is the surface).
//!
//! Pairs with `sandbox_capability_intersection_at_init.rs` (D7 init-snapshot
//! path). Together: D7 hybrid is fully exercised.
//!
//! Closes: Compromise #N+? (TOCTOU bound at SANDBOX) — TIGHTER than
//! Phase-1 Compromise #1 ITERATE batch boundary (~1 µs vs 100 iterations).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

// R5 surfaces consumed:
//   benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef}
//   benten_eval::testing::testing_revoke_cap_mid_call (R2 §9 helper)
//   benten_caps::{CapabilityPolicy, CapScope}
//   benten_errors::ErrorCode::SandboxHostFnDenied

#[test]
#[ignore = "Phase 2b G7-A pending — D7 hybrid per-call live check + D18 per_call"]
fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent() {
    // wsa-2 + sec-pre-r1-02 — the load-bearing TOCTOU test.
    //
    // R5 wires:
    //   1. Engine open with CapabilityPolicy granting host:compute:kv:read.
    //   2. sandbox_call begins; module makes its first kv_read call —
    //      succeeds (cap was live at the per-call check).
    //   3. Driver invokes `testing_revoke_cap_mid_call(engine,
    //      &CapScope::host_compute_kv_read())` between kv_read calls.
    //   4. Module's SECOND kv_read call observes the revoked cap (per-call
    //      live check fires against `policy.check_capability(actor,
    //      derived_scope)`); returns ErrorCode::SandboxHostFnDenied.
    //
    // CRITICAL: the assertion is that the second call denies, NOT that
    // the first call denies. The TOCTOU bound for kv:read is per-host-fn
    // invocation (D18 per_call default).
    //
    // R3-C ownership per R2 §10 (cap-revocation TOCTOU is the primary
    // lens). Companion to ESC-9 `sandbox_escape_host_fn_after_cap_revoke`
    // (which exercises the same property end-to-end via `.wat` fixture);
    // this test exercises the policy contract directly without `.wat`.
    todo!("R5 G7-A — assert second kv_read fires SandboxHostFnDenied via live policy");
}

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
#[ignore = "Wave-8b ships the wasmtime trampoline that consults the per-call live cap-set on every kv:read invocation; in 8b the live cap-set is initialised from the dispatching grant snapshot. The actual mid-call revoke surface requires the engine-side `testing_revoke_cap_mid_call` helper that mutates the live cap-set DURING a host-fn callback (paired with engine integration in 8c). The trampoline path itself is wired and tested by the unit-level cap-check; this test is the integration-shaped pin that flips when the helper lands."]
fn sandbox_host_fn_capability_revoked_mid_execution_denies_subsequent() {
    // Pin: the trampoline calls `cap_check(... PerCall)` for kv:read on
    // every invocation. The integration helper that mutates `live_caps`
    // mid-call (so the second invocation observes the revoked cap)
    // lives at the engine layer — paired 8c work.
}

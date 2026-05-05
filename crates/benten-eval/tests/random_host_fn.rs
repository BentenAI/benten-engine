//! R3-D RED-PHASE pins for `random` host-fn workspace CSPRNG
//! (G17-A2 wave 5b; D-PHASE-3-11 RESOLVED-at-R1 + r1-wsa-8 +
//! CLAUDE.md baked-in #16 closure).
//!
//! Pin sources (per r2-test-landscape §2.5 G17-A2):
//!
//! - `tests/random_host_fn_csprng_round_trip` — D-PHASE-3-11
//! - `tests/random_host_fn_capability_gated_entropy_budget` — D-PHASE-3-11
//! - `tests/random_host_fn_per_manifest_budget_override_via_module_manifest_field`
//!   — r1-wsa-8
//! - `tests/sandbox_host_fn_random_no_longer_returns_deferred_error`
//!   — plan §3 G17-A2
//!
//! ## D-PHASE-3-11 RESOLVED shape
//!
//! Phase-2b's `random` host-fn returned a `Deferred("phase-3 — workspace
//! CSPRNG decision pending")` typed error. Phase-3 G17-A2 wires the
//! resolved decision:
//!
//! - **Workspace CSPRNG:** `getrandom` direct (NOT `rand` / NOT
//!   re-implemented).
//! - **Capability-gated entropy budget:** the `host:random:read` cap
//!   carries a per-call budget in BYTES.
//! - **Default budget per call:** 4096 bytes (per r1-wsa-8).
//! - **Per-manifest override:** module manifest exposes additive
//!   optional `host_fns.random.budget_bytes_per_call` field.
//!
//! ## Compromise #16 closure
//!
//! `host-functions.toml` flips `random` from `IMPLEMENTED = false` to
//! `IMPLEMENTED = true`; SECURITY-POSTURE.md Compromise #16 marked
//! CLOSED-IN-PHASE-3-G17-A2.
//!
//! Pin file is shared with `random_constant_time.rs` (G17-A1) by
//! topic; the constant-time pin is about TIMING + per-r1-wsa-3+sec-r1-3
//! co-shape, while this file pins FUNCTIONAL surface.

#![allow(clippy::unwrap_used)]
#![cfg(not(target_arch = "wasm32"))]

#[test]
#[ignore = "RED-PHASE: G17-A2 wave 5b wires getrandom-backed random host-fn (D-PHASE-3-11 RESOLVED)"]
fn random_host_fn_csprng_round_trip() {
    // D-PHASE-3-11 pin. G17-A2 implementer wires this:
    //
    //   use benten_eval::sandbox::{Sandbox, SandboxConfig, ManifestRef};
    //
    //   // Build a SANDBOX whose manifest grants host:random:read
    //   // with default budget (4096 bytes/call):
    //   let sandbox = build_sandbox_with_cap("host:random:read");
    //
    //   // Guest calls random(64 bytes):
    //   let result = sandbox.execute(/* fixture: random_round_trip */);
    //
    //   let bytes: Vec<u8> = result.unwrap().output();
    //   assert_eq!(bytes.len(), 64);
    //
    //   // CSPRNG-shape assertion: two successive calls produce different
    //   // bytes (overwhelming probability — flake budget < 1/2^256):
    //   let bytes_2 = sandbox.execute(/* same fixture */).unwrap().output();
    //   assert_ne!(bytes, bytes_2,
    //       "random host-fn must produce CSPRNG entropy per D-PHASE-3-11 \
    //        (getrandom-direct workspace decision)");
    //
    // OBSERVABLE consequence: random host-fn returns 64 bytes drawn
    // from `getrandom`. Defends D-PHASE-3-11 RESOLVED + closes the
    // Phase-2b deferral.
    unimplemented!("G17-A2 wires getrandom-backed random host-fn + round-trip assertion");
}

#[test]
#[ignore = "RED-PHASE: G17-A2 wave 5b enforces capability-gated entropy budget (D-PHASE-3-11 + r1-wsa-8)"]
fn random_host_fn_capability_gated_entropy_budget() {
    // D-PHASE-3-11 + r1-wsa-8 pin. G17-A2 implementer:
    //
    //   let sandbox = build_sandbox_with_cap_and_budget("host:random:read", 4096);
    //
    //   // Guest's first call within budget — succeeds:
    //   let r1 = sandbox.invoke_random_host_fn(/* 100 bytes */);
    //   assert!(r1.is_ok());
    //
    //   // Guest's second call also within budget — succeeds:
    //   let r2 = sandbox.invoke_random_host_fn(/* 4000 bytes (cumulative 4100, but per-CALL budget) */);
    //   // Per-CALL budget interpretation per r1-wsa-8: each individual
    //   // call must fit; cumulative-per-frame is a separate cap.
    //
    //   // Guest's third call exceeds the per-call budget — fails:
    //   let r3 = sandbox.invoke_random_host_fn(/* 8192 bytes */);
    //   assert!(matches!(
    //       r3.unwrap_err(),
    //       benten_eval::SandboxError::HostFnDenied { code, .. }
    //         if code == benten_errors::ErrorCode::SandboxHostFnRandomBudgetExceeded
    //   ));
    //
    // OBSERVABLE consequence: a guest cannot drain entropy via a
    // single oversized call; the per-call ceiling is enforced.
    // Defends r1-wsa-8 budget-bytes default.
    unimplemented!("G17-A2 wires capability-gated per-call entropy budget enforcement");
}

#[test]
#[ignore = "RED-PHASE: G17-A2 wave 5b adds per-manifest budget override via module_manifest field (r1-wsa-8)"]
fn random_host_fn_per_manifest_budget_override_via_module_manifest_field() {
    // r1-wsa-8 pin. G17-A2 implementer:
    //
    //   // Module manifest sets a custom budget:
    //   let manifest_toml = r#"
    //       [host_fns.random]
    //       budget_bytes_per_call = 1024  # tighter than default 4096
    //   "#;
    //
    //   // Build SANDBOX from that manifest:
    //   let sandbox = build_sandbox_from_manifest(manifest_toml);
    //
    //   // 1024-byte call: succeeds.
    //   let ok = sandbox.invoke_random_host_fn(/* 1024 bytes */);
    //   assert!(ok.is_ok());
    //
    //   // 2048-byte call: rejected (exceeds the per-manifest override):
    //   let denied = sandbox.invoke_random_host_fn(/* 2048 bytes */);
    //   assert!(denied.is_err());
    //
    //   // Source-cite for the field name:
    //   let manifest_src = std::fs::read_to_string("docs/MODULE-MANIFEST.md").unwrap();
    //   assert!(manifest_src.contains("budget_bytes_per_call"),
    //       "MODULE-MANIFEST.md must document host_fns.random.budget_bytes_per_call per r1-wsa-8 + §3.5b doc-coupling");
    //
    // OBSERVABLE consequence: a manifest author can tighten (or
    // permissively widen) the entropy budget per-module. Defends
    // r1-wsa-8 — additive optional field.
    unimplemented!("G17-A2 wires manifest.host_fns.random.budget_bytes_per_call override");
}

#[test]
#[ignore = "RED-PHASE: G17-A2 wave 5b drops Deferred error variant for random (CLAUDE.md baked-in #16 closure)"]
fn sandbox_host_fn_random_no_longer_returns_deferred_error() {
    // plan §3 G17-A2 pin. G17-A2 implementer:
    //
    //   // Existing Phase-2b test in this crate:
    //   //   crates/benten-eval/tests/sandbox_host_fn_random_deferred.rs
    //   // is RETIRED at G17-A2 (asserted Deferred error fires; that
    //   // path is now removed).
    //
    //   // This pin verifies the deferred-error path is GONE from the
    //   // dispatch tree:
    //   let dispatch_src = std::fs::read_to_string(
    //       "crates/benten-eval/src/sandbox/host_fns.rs"
    //   ).unwrap();
    //
    //   // The validate-time deferral guard is dropped per plan §3 G17-A2:
    //   assert!(!dispatch_src.contains("Deferred(\"phase-3"),
    //       "Phase-2b deferred-error guard for random host-fn must be removed at G17-A2 \
    //        per CLAUDE.md baked-in #16 closure (Compromise #16)");
    //   assert!(!dispatch_src.contains("phase-3 — workspace CSPRNG decision pending"),
    //       "Phase-2b 'workspace CSPRNG decision pending' rationale must be removed at G17-A2");
    //
    //   // The real implementation is wired:
    //   assert!(dispatch_src.contains("getrandom") || dispatch_src.contains("OsRng"),
    //       "host_fns.rs must invoke a CSPRNG (getrandom direct per D-PHASE-3-11)");
    //
    //   // host-functions.toml flag flipped:
    //   let toml = std::fs::read_to_string("host-functions.toml").unwrap();
    //   // Find the [host_fns.random] section + assert IMPLEMENTED = true:
    //   //   (implementer pins exact key spelling)
    //   assert!(toml.contains("random"),
    //       "host-functions.toml must declare random host-fn (G17-A2 retire Compromise #16)");
    //
    // OBSERVABLE consequence: the deferred-error path is gone; the
    // real CSPRNG path is wired; Compromise #16 closes. Defends
    // pim-2 — closure of "TODO: phase-3" without leaving the legacy
    // arm in place.
    unimplemented!("G17-A2 wires deferred-error retirement + CSPRNG wiring source-cite assertion");
}

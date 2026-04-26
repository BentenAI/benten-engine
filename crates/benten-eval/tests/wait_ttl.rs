#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G12-E
// WAIT TTL surface (`ttl_hours` field, validation, `E_WAIT_TTL_INVALID`,
// `E_WAIT_TTL_EXPIRED` error codes).
//
//! Phase 2b R4-FP (B-1) — D12 WAIT TTL registration + resume unit tests.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D12-RESOLVED
//!     (optional `ttl_hours: NonZeroU32` + 24h default + 720h max +
//!     hybrid GC + new `E_WAIT_TTL_EXPIRED` + `E_WAIT_TTL_INVALID`).
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.10 + §8.1 rows 514-518.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-06.
//!   - r1-streaming-systems D12 implementation hint (TTL field in
//!     `SubgraphSpec.primitives` per-WAIT property; GC in
//!     `crates/benten-eval/src/primitives/wait.rs`).
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use benten_engine::Engine;
use std::time::Duration;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// `wait_ttl_default_24h_applies_when_omitted` — D12 + R2 §1.10 row 514.
///
/// A WAIT primitive whose `args` does NOT carry a `ttl_hours` key MUST
/// receive the 24h default at registration time. The deadline at
/// suspend-time is `now + 24h`.
#[test]
#[ignore = "Phase 2b G12-E pending — ttl_hours default unimplemented"]
fn wait_ttl_default_24h_applies_when_omitted() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   // Build a SubgraphSpec with a WAIT node carrying NO ttl_hours.
    //   let spec = benten_engine::testing::testing_make_wait_spec_default_ttl();
    //   engine.register_subgraph("test.default_ttl", spec).unwrap();
    //
    //   let envelope = benten_engine::testing::testing_call_to_suspend(
    //       &mut engine, "test.default_ttl",
    //   ).expect("WAIT suspends cleanly");
    //
    //   let metadata = benten_engine::testing::testing_inspect_wait_metadata(
    //       &engine, &envelope,
    //   ).expect("metadata in SuspensionStore");
    //   assert_eq!(metadata.ttl_hours.get(), 24,
    //       "omitted ttl_hours MUST default to 24 (D12 default)");
    todo!("R5 G12-E — assert 24h default when ttl_hours omitted");
}

/// `wait_ttl_explicit_overrides_default` — D12 + R2 §1.10 row 515.
///
/// Explicit `ttl_hours: 48` MUST override the 24h default end-to-end.
#[test]
#[ignore = "Phase 2b G12-E pending — explicit ttl_hours override unimplemented"]
fn wait_ttl_explicit_overrides_default() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(48);
    //   engine.register_subgraph("test.explicit_ttl", spec).unwrap();
    //   let envelope = benten_engine::testing::testing_call_to_suspend(
    //       &mut engine, "test.explicit_ttl",
    //   ).unwrap();
    //   let metadata = benten_engine::testing::testing_inspect_wait_metadata(
    //       &engine, &envelope,
    //   ).unwrap();
    //   assert_eq!(metadata.ttl_hours.get(), 48,
    //       "explicit ttl_hours: 48 MUST override the 24h default");
    todo!("R5 G12-E — assert explicit ttl_hours overrides default");
}

/// `wait_ttl_zero_rejected_at_registration` — D12 + R2 §1.10 row 516
/// (depends on new `E_WAIT_TTL_INVALID`).
///
/// `ttl_hours: 0` is nonsensical (would expire immediately on suspend);
/// the spec MUST be rejected at `register_subgraph` with the typed
/// `E_WAIT_TTL_INVALID` error. This pin enforces the
/// `NonZeroU32`-style validation independently of the type system
/// (TS-side spec carries `number`, not `NonZeroU32`).
#[test]
#[ignore = "Phase 2b G12-E pending — ttl_hours: 0 validation unimplemented"]
fn wait_ttl_zero_rejected_at_registration() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   let bad_spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours_unchecked(0);
    //   let err = engine
    //       .register_subgraph("test.zero_ttl", bad_spec)
    //       .expect_err("ttl_hours: 0 MUST be rejected at registration");
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_WAIT_TTL_INVALID"),
    //       "expected E_WAIT_TTL_INVALID, got: {rendered}"
    //   );
    todo!("R5 G12-E — assert ttl_hours: 0 rejected with E_WAIT_TTL_INVALID");
}

/// `wait_ttl_exceeds_max_rejected` — D12 + R2 §1.10 row 517.
///
/// 720h is the documented max (30 days). `ttl_hours: 721` MUST be
/// rejected with `E_WAIT_TTL_INVALID`. 720 itself MUST be accepted
/// (boundary inclusive).
#[test]
#[ignore = "Phase 2b G12-E pending — 720h max validation unimplemented"]
fn wait_ttl_exceeds_max_rejected() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   // Boundary inclusive: 720 accepted.
    //   let ok_spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(720);
    //   engine.register_subgraph("test.max_ttl", ok_spec)
    //       .expect("ttl_hours: 720 MUST be accepted (max inclusive)");
    //
    //   // Just over: 721 rejected.
    //   let bad_spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours_unchecked(721);
    //   let err = engine
    //       .register_subgraph("test.over_max_ttl", bad_spec)
    //       .expect_err("ttl_hours: 721 MUST be rejected");
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_WAIT_TTL_INVALID"),
    //       "expected E_WAIT_TTL_INVALID for ttl_hours > 720, got: {rendered}"
    //   );
    todo!("R5 G12-E — assert 720 accepted + 721 rejected");
}

/// `wait_resume_after_expiry_fires_typed_error` — D12 + R2 §1.10 row 518.
///
/// Companion to the integration test
/// `wait_ttl_expires_via_suspension_store` (R3-E landed). This
/// unit-level pin asserts the error variant exists at the registration/
/// resume API boundary in eval-side, not just at the end-to-end level.
#[test]
#[ignore = "Phase 2b G12-E pending — E_WAIT_TTL_EXPIRED resume path unimplemented"]
fn wait_resume_after_expiry_fires_typed_error() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //   engine.register_subgraph("test.expiry_typed", spec).unwrap();
    //   let envelope = benten_engine::testing::testing_call_to_suspend(
    //       &mut engine, "test.expiry_typed",
    //   ).unwrap();
    //
    //   // Advance well past the deadline.
    //   benten_engine::testing::testing_advance_wait_clock(
    //       &mut engine, Duration::from_secs(2 * 3600),
    //   );
    //
    //   let err = engine
    //       .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
    //       .expect_err("expired resume MUST fail closed");
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_WAIT_TTL_EXPIRED"),
    //       "expected E_WAIT_TTL_EXPIRED in error rendering, got: {rendered}"
    //   );
    todo!("R5 G12-E — assert E_WAIT_TTL_EXPIRED on expired resume");
}

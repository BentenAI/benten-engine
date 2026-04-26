#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G12-E
// WAIT TTL surface (ttl_hours + E_WAIT_TTL_EXPIRED + clock advancement
// helper).
//
//! Phase 2b R4-FP (B-1) — D12 WAIT TTL property test.
//!
//! Property: for any (ttl, resume_offset) pair with non-zero ttl ≤ 720h,
//! `(resume_offset > ttl) ↔ E_WAIT_TTL_EXPIRED`. No silent expiry, no
//! permissive-Complete fallback.
//!
//! Pin source:
//!   - `.addl/phase-2b/r1-streaming-systems.json` `must_pass_tests_for_r3`
//!     entry `prop_wait_ttl_no_silent_expiry_in_resume`.
//!   - `.addl/phase-2b/r2-test-landscape.md` §3 row 333.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-06 (only proptest in
//!     §3 not landed).
//!
//! Iterations: 10k (per R2 §3).
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// `prop_wait_ttl_no_silent_expiry_in_resume` — D12 + R2 §3 + R1
    /// streaming-systems must-pass.
    ///
    /// Property: register a WAIT with `ttl_hours = ttl`; suspend; advance
    /// the wait-clock by `offset_hours`; attempt resume. Outcome MUST be:
    ///
    ///   (offset_hours > ttl)  →  resume errors with E_WAIT_TTL_EXPIRED
    ///   (offset_hours <= ttl) →  resume completes cleanly
    ///
    /// No silent expiry (resume succeeds despite past-deadline) and no
    /// false-positive expiry (resume fails despite within-deadline) is
    /// permissible.
    #[test]
    #[ignore = "Phase 2b G12-E pending — TTL property; depends on ttl_hours + E_WAIT_TTL_EXPIRED + testing_advance_wait_clock"]
    fn prop_wait_ttl_no_silent_expiry_in_resume(
        ttl in 1u32..=720,
        offset_hours in 0u32..=2000,
    ) {
        // R5 G12-E pseudo:
        //   let dir = tempfile::tempdir().unwrap();
        //   let mut engine = benten_engine::Engine::builder()
        //       .path(dir.path().join("benten.redb"))
        //       .build()
        //       .unwrap();
        //
        //   let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(ttl);
        //   engine.register_subgraph("test.prop_ttl", spec).unwrap();
        //   let envelope = benten_engine::testing::testing_call_to_suspend(
        //       &mut engine, "test.prop_ttl",
        //   ).unwrap();
        //
        //   benten_engine::testing::testing_advance_wait_clock(
        //       &mut engine,
        //       std::time::Duration::from_secs(u64::from(offset_hours) * 3600),
        //   );
        //
        //   let result = engine.resume_with_meta(
        //       &envelope, benten_engine::ResumePayload::None,
        //   );
        //
        //   if offset_hours > ttl {
        //       let err = result.expect_err("offset > ttl MUST fire E_WAIT_TTL_EXPIRED");
        //       prop_assert!(
        //           err.to_string().contains("E_WAIT_TTL_EXPIRED"),
        //           "offset_hours={} ttl={} → expected E_WAIT_TTL_EXPIRED, got: {}",
        //           offset_hours, ttl, err,
        //       );
        //   } else {
        //       prop_assert!(
        //           result.is_ok(),
        //           "offset_hours={} ttl={} → expected clean resume, got: {:?}",
        //           offset_hours, ttl, result.err(),
        //       );
        //   }
        let _ = (ttl, offset_hours);
        // Force fail-fast in red phase — R5 G12-E replaces with the property body.
        prop_assume!(false);
    }
}

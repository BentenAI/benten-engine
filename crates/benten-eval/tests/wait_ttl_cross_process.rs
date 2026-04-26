#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G12-E
// cross-process WAIT TTL via persistent SuspensionStore.
//
//! Phase 2b R4-FP (B-1) — D12 cross-process WAIT TTL unit tests.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D12 + §3.2 G12-E
//!     (SuspensionStore generalized; TTL state survives Engine::open
//!     across process boundary; TTL clock is wall-clock-relative,
//!     NOT process-uptime-relative).
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.10 + §8.1 rows 523-524.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-06.
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use benten_engine::Engine;
use std::time::Duration;

/// `wait_ttl_cross_process_survives_restart` — D12 + G12-E + R2 row 523.
///
/// Suspend with `ttl_hours: 1` in process A. Drop engine A. Open
/// engine B against the same path. Advance B's wait-clock past the
/// deadline. Resume — MUST surface `E_WAIT_TTL_EXPIRED`. The TTL state
/// (deadline timestamp) is in the persistent SuspensionStore, NOT in
/// process-local memory.
#[test]
#[ignore = "Phase 2b G12-E pending — TTL persistence across Engine::open boundary"]
fn wait_ttl_cross_process_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // R5 G12-E pseudo:
    //   let envelope = {
    //       let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
    //       let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //       engine_a.register_subgraph("test.crossproc_ttl", spec).unwrap();
    //       benten_engine::testing::testing_call_to_suspend(
    //           &mut engine_a, "test.crossproc_ttl",
    //       ).unwrap()
    //       // engine_a drops here — process A boundary.
    //   };
    //
    //   let mut engine_b = Engine::builder().path(&db_path).build().unwrap();
    //   assert!(
    //       benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
    //       "TTL metadata MUST survive cross-process boundary"
    //   );
    //
    //   benten_engine::testing::testing_advance_wait_clock(
    //       &mut engine_b, Duration::from_secs(2 * 3600),
    //   );
    //
    //   let err = engine_b
    //       .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
    //       .expect_err("resume after cross-process TTL expiry MUST fail closed");
    //   let rendered = err.to_string();
    //   assert!(
    //       rendered.contains("E_WAIT_TTL_EXPIRED"),
    //       "expected E_WAIT_TTL_EXPIRED across process boundary, got: {rendered}"
    //   );
    todo!("R5 G12-E — assert TTL survives Engine::open + cross-process expiry fires");
}

/// `wait_ttl_does_NOT_apply_during_suspend_pause` — D12 wall-clock-
/// relative pin + R2 row 524.
///
/// The TTL deadline is wall-clock-relative (e.g. `now + 24h`), NOT
/// "24h of engine-runtime". If an engine sleeps mid-suspend (or the
/// host laptop is closed), the wall-clock continues advancing; the
/// TTL deadline is unaffected by whether the engine is "ticking".
///
/// Equivalent: simulate engine A sleeping by suspending, then opening
/// engine B much later — the deadline anchored at suspend-time MUST
/// still apply correctly relative to wall-clock-now in engine B.
#[test]
#[ignore = "Phase 2b G12-E pending — wall-clock-relative TTL semantics unimplemented"]
fn wait_ttl_does_not_apply_during_suspend_pause() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // R5 G12-E pseudo:
    //   let (envelope, suspend_wallclock) = {
    //       let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
    //       let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //       engine_a.register_subgraph("test.pause_ttl", spec).unwrap();
    //       let env = benten_engine::testing::testing_call_to_suspend(
    //           &mut engine_a, "test.pause_ttl",
    //       ).unwrap();
    //       let stamp = benten_engine::testing::testing_inspect_wait_metadata(
    //           &engine_a, &env,
    //       ).unwrap().deadline_wallclock;
    //       (env, stamp)
    //       // engine_a drops; simulated "laptop closed for 2h" by advancing
    //       // the wall-clock baseline before engine_b opens.
    //   };
    //
    //   let mut engine_b = Engine::builder().path(&db_path).build().unwrap();
    //   // Pretend wall-clock advanced 2h while engine A was paused / dropped.
    //   benten_engine::testing::testing_set_wall_clock_baseline(
    //       &mut engine_b,
    //       suspend_wallclock + Duration::from_secs(2 * 3600),
    //   );
    //
    //   // Resume MUST fire E_WAIT_TTL_EXPIRED — the engine being "off" did
    //   // NOT pause the deadline.
    //   let err = engine_b
    //       .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
    //       .expect_err("wall-clock TTL must NOT freeze during suspend pause");
    //   assert!(
    //       err.to_string().contains("E_WAIT_TTL_EXPIRED"),
    //       "TTL deadline is wall-clock-anchored — engine downtime MUST NOT \
    //        extend it (D12 wall-clock-relative semantics)"
    //   );
    todo!("R5 G12-E — assert TTL is wall-clock-relative, not process-uptime-relative");
}

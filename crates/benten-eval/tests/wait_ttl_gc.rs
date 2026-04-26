#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G12-E
// hybrid GC (event-driven sweep + interval backstop + drop-final).
//
//! Phase 2b R4-FP (B-1) — D12 hybrid-GC unit tests.
//!
//! Pin source:
//!   - `.addl/phase-2b/00-implementation-plan.md` §5 D12 hybrid GC:
//!     event-driven sweep on suspend/resume + 1h interval backstop +
//!     final sweep on Engine::drop.
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.10 + §8.1 rows 519-522.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-06.
//!
//! Three GC paths under test:
//!   1. event-driven on resume (covered by integration
//!      wait_ttl_expires_via_suspension_store; Rust-side unit pin
//!      added here as `wait_gc_event_driven_on_suspend_sweeps_expired_siblings`).
//!   2. event-driven on suspend (sibling sweep).
//!   3. interval backstop (1h periodic sweep on idle engine).
//!   4. drop-final (final sweep on Engine::drop).
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

/// `wait_gc_event_driven_suspend_sweeps_expired_siblings` — D12 + R2 row 519.
///
/// Setup: suspend two waits, A (ttl_hours: 1) + B (ttl_hours: 24).
/// Advance the wait-clock past A's deadline. Suspend a third wait C —
/// the suspend operation MUST opportunistically sweep A from the
/// SuspensionStore (event-driven GC on suspend).
#[test]
#[ignore = "Phase 2b G12-E pending — event-driven sweep on suspend unimplemented"]
fn wait_gc_event_driven_suspend_sweeps_expired_siblings() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   let spec_a = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //   let spec_b = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(24);
    //   let spec_c = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(24);
    //   engine.register_subgraph("test.gc_a", spec_a).unwrap();
    //   engine.register_subgraph("test.gc_b", spec_b).unwrap();
    //   engine.register_subgraph("test.gc_c", spec_c).unwrap();
    //
    //   let env_a = benten_engine::testing::testing_call_to_suspend(&mut engine, "test.gc_a").unwrap();
    //   let env_b = benten_engine::testing::testing_call_to_suspend(&mut engine, "test.gc_b").unwrap();
    //
    //   // Past A's 1h deadline, before B's 24h.
    //   benten_engine::testing::testing_advance_wait_clock(
    //       &mut engine, Duration::from_secs(2 * 3600),
    //   );
    //
    //   // Suspend C — should sweep expired A as a side effect.
    //   let _env_c = benten_engine::testing::testing_call_to_suspend(&mut engine, "test.gc_c").unwrap();
    //
    //   assert!(
    //       !benten_engine::testing::testing_suspension_store_has_wait(&engine, &env_a),
    //       "A's expired entry MUST be GC'd by the suspend-time sweep"
    //   );
    //   assert!(
    //       benten_engine::testing::testing_suspension_store_has_wait(&engine, &env_b),
    //       "B is unexpired; MUST remain in store"
    //   );
    todo!("R5 G12-E — assert event-driven sweep on suspend GCs expired siblings");
}

/// `wait_gc_interval_backstop_sweeps_idle_engine` — D12 + R2 row 520.
///
/// On a fully-idle engine (no suspend, no resume events firing), the
/// 1h interval-backstop GC MUST still catch expired waits. Test
/// simulates by advancing both the wait-clock AND the interval-clock,
/// then asserts the entry is gone.
#[test]
#[ignore = "Phase 2b G12-E pending — 1h interval backstop GC unimplemented"]
fn wait_gc_interval_backstop_sweeps_idle_engine() {
    let (_dir, mut engine) = fresh_engine();

    // R5 G12-E pseudo:
    //   let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //   engine.register_subgraph("test.gc_interval", spec).unwrap();
    //   let envelope = benten_engine::testing::testing_call_to_suspend(
    //       &mut engine, "test.gc_interval",
    //   ).unwrap();
    //
    //   benten_engine::testing::testing_advance_wait_clock(
    //       &mut engine, Duration::from_secs(2 * 3600),
    //   );
    //
    //   // Trigger the interval backstop without firing a suspend/resume.
    //   benten_engine::testing::testing_run_gc_interval_tick(&mut engine);
    //
    //   assert!(
    //       !benten_engine::testing::testing_suspension_store_has_wait(&engine, &envelope),
    //       "interval backstop MUST sweep the expired entry on an idle engine"
    //   );
    todo!("R5 G12-E — assert interval backstop GC on idle engine");
}

/// `wait_gc_disabled_event_driven_still_works_via_interval` — D12 + R2 row 521.
///
/// If event-driven GC is disabled (config knob), the interval backstop
/// MUST still ensure no entry survives indefinitely past expiry.
#[test]
#[ignore = "Phase 2b G12-E pending — event-driven-disabled config unimplemented"]
fn wait_gc_disabled_event_driven_still_works_via_interval() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        // .gc_event_driven(false)  // R5 G12-E builder option
        .build()
        .unwrap();

    // R5 G12-E pseudo:
    //   let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //   engine.register_subgraph("test.no_eventdriven", spec).unwrap();
    //   let envelope = benten_engine::testing::testing_call_to_suspend(
    //       &mut engine, "test.no_eventdriven",
    //   ).unwrap();
    //
    //   benten_engine::testing::testing_advance_wait_clock(
    //       &mut engine, Duration::from_secs(2 * 3600),
    //   );
    //
    //   // Suspend another wait — under event-driven-disabled this would NOT sweep.
    //   let _other = benten_engine::testing::testing_call_to_suspend(&mut engine, "test.no_eventdriven").unwrap();
    //
    //   // The entry MAY still be present here (event-driven off).
    //   // Run the interval backstop:
    //   benten_engine::testing::testing_run_gc_interval_tick(&mut engine);
    //
    //   assert!(
    //       !benten_engine::testing::testing_suspension_store_has_wait(&engine, &envelope),
    //       "with event-driven GC disabled the interval backstop is the SOLE \
    //        GC mechanism; expired entries MUST still be swept"
    //   );
    todo!("R5 G12-E — assert interval backstop sufficient when event-driven disabled");
}

/// `wait_gc_engine_drop_runs_final_sweep` — D12 + R2 row 522.
///
/// `Engine::drop` MUST perform a final GC sweep before releasing the
/// SuspensionStore handle. Test: suspend an expired wait, drop the
/// engine, then re-open against the same path and assert the entry is
/// absent (proving the drop-time sweep removed it from durable storage).
#[test]
#[ignore = "Phase 2b G12-E pending — Engine::drop final sweep unimplemented"]
fn wait_gc_engine_drop_runs_final_sweep() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // R5 G12-E pseudo:
    //   let envelope = {
    //       let mut engine = Engine::builder().path(&db_path).build().unwrap();
    //       let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
    //       engine.register_subgraph("test.drop_sweep", spec).unwrap();
    //       let envelope = benten_engine::testing::testing_call_to_suspend(
    //           &mut engine, "test.drop_sweep",
    //       ).unwrap();
    //       benten_engine::testing::testing_advance_wait_clock(
    //           &mut engine, Duration::from_secs(2 * 3600),
    //       );
    //       envelope
    //       // Engine::drop MUST run a final sweep here.
    //   };
    //
    //   // Re-open and assert the entry is gone.
    //   let engine_b = Engine::builder().path(&db_path).build().unwrap();
    //   assert!(
    //       !benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
    //       "Engine::drop MUST run final GC sweep — expired entry MUST NOT \
    //        survive across the drop boundary"
    //   );
    todo!("R5 G12-E — assert Engine::drop final sweep removes expired entries");
}

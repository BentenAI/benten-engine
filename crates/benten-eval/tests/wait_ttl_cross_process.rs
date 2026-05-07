// Phase-3 G20-A2 (D12 wave-8a) — D12 cross-process WAIT TTL tests.
//
// `phase_2b_landed` cfg gate retired at G20-A2 wave-8a.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::duration_suboptimal_units)]

use std::time::Duration;

use benten_engine::Engine;

/// `wait_ttl_cross_process_survives_restart` — D12 + G12-E + R2 row 523.
///
/// Suspend with `ttl_hours: 1` in process A. Drop engine A. Open
/// engine B against the same path. Advance B's wait-clock past the
/// deadline. Resume — MUST surface `E_WAIT_TTL_EXPIRED`. The TTL state
/// (deadline timestamp) is in the persistent SuspensionStore, NOT in
/// process-local memory.
#[test]
fn wait_ttl_cross_process_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let envelope = {
        let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
        let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
        let handler_id = engine_a.register_subgraph(spec).unwrap();
        benten_engine::testing::testing_call_to_suspend(&mut engine_a, &handler_id).unwrap()
        // engine_a drops here — process A boundary.
    };

    let engine_b = Engine::builder().path(&db_path).build().unwrap();
    assert!(
        benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "TTL metadata MUST survive cross-process boundary"
    );

    benten_engine::testing::testing_advance_wait_clock(&engine_b, Duration::from_secs(2 * 3600));

    let err = engine_b
        .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
        .expect_err("resume after cross-process TTL expiry MUST fail closed");
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_TTL_EXPIRED"),
        "expected E_WAIT_TTL_EXPIRED across process boundary, got: {rendered}"
    );
}

/// `wait_ttl_does_NOT_apply_during_suspend_pause` — D12 wall-clock-
/// relative pin + R2 row 524.
///
/// The TTL deadline is wall-clock-anchored (`suspend_wallclock_ms +
/// ttl_hours * 3_600_000`). If an engine sleeps mid-suspend (or the
/// host laptop is closed), the wall-clock continues advancing; the TTL
/// deadline is unaffected by whether the engine is "ticking".
#[test]
fn wait_ttl_does_not_apply_during_suspend_pause() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let envelope = {
        let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
        let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(1);
        let handler_id = engine_a.register_subgraph(spec).unwrap();
        benten_engine::testing::testing_call_to_suspend(&mut engine_a, &handler_id).unwrap()
    };

    let engine_b = Engine::builder().path(&db_path).build().unwrap();
    // Confirm metadata survived the cross-process boundary first.
    let meta = benten_engine::testing::testing_inspect_wait_metadata(&engine_b, &envelope)
        .expect("metadata must survive cross-process boundary");
    let suspend_wallclock_ms = meta
        .suspend_wallclock_ms
        .expect("wall-clock anchor recorded at suspend time");

    // Pretend wall-clock advanced 2h while engine A was paused / dropped.
    // The deadline anchored at suspend-time MUST still apply correctly
    // relative to wall-clock-now in engine B.
    benten_engine::testing::testing_set_wall_clock_baseline(
        &engine_b,
        Duration::from_millis(suspend_wallclock_ms) + Duration::from_secs(2 * 3600),
    );

    let err = engine_b
        .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
        .expect_err("wall-clock TTL must NOT freeze during suspend pause");
    assert!(
        err.to_string().contains("E_WAIT_TTL_EXPIRED"),
        "TTL deadline is wall-clock-anchored — engine downtime MUST NOT \
         extend it (D12 wall-clock-relative semantics)"
    );
}

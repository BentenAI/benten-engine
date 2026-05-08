// Phase-3 G20-A2 (D12 wave-8a) — D12 hybrid-GC unit tests.
//
// `phase_2b_landed` cfg gate retired at G20-A2 wave-8a.
//
// Three GC paths under test:
//   1. event-driven on suspend (sibling sweep).
//   2. interval backstop (1h periodic sweep on idle engine).
//   3. event-driven-disabled config (interval still sweeps).
//   4. drop-final (final sweep on Engine::drop).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::duration_suboptimal_units)]

use std::time::Duration;

use benten_engine::Engine;

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
/// Setup: register two TTL-bearing handlers + suspend through them.
/// Advance the wait-clock past the first one's deadline. Suspend a
/// third — the suspend MUST opportunistically sweep the expired entry
/// (event-driven GC on suspend).
#[test]
fn wait_gc_event_driven_suspend_sweeps_expired_siblings() {
    let (_dir, mut engine) = fresh_engine();

    // Register three distinct handlers — handler_id is property-derived
    // so we mutate the spec property to keep handler ids distinct.
    let spec_a = make_named_ttl_spec("phase_3_g20_a2_gc_a", 1);
    let spec_b = make_named_ttl_spec("phase_3_g20_a2_gc_b", 24);
    let spec_c = make_named_ttl_spec("phase_3_g20_a2_gc_c", 24);
    let id_a = engine.register_subgraph(spec_a).unwrap();
    let id_b = engine.register_subgraph(spec_b).unwrap();
    let id_c = engine.register_subgraph(spec_c).unwrap();

    let env_a = benten_engine::testing::testing_call_to_suspend(&mut engine, &id_a).unwrap();
    let env_b = benten_engine::testing::testing_call_to_suspend(&mut engine, &id_b).unwrap();

    // Past A's 1h deadline, before B's 24h.
    benten_engine::testing::testing_advance_wait_clock(&engine, Duration::from_secs(2 * 3600));

    // Suspend C — should sweep expired A as a side effect.
    let _env_c = benten_engine::testing::testing_call_to_suspend(&mut engine, &id_c).unwrap();

    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine, &env_a),
        "A's expired entry MUST be GC'd by the suspend-time sweep"
    );
    assert!(
        benten_engine::testing::testing_suspension_store_has_wait(&engine, &env_b),
        "B is unexpired; MUST remain in store"
    );
}

/// `wait_gc_interval_backstop_sweeps_idle_engine` — D12 + R2 row 520.
#[test]
fn wait_gc_interval_backstop_sweeps_idle_engine() {
    let (_dir, mut engine) = fresh_engine();
    let spec = make_named_ttl_spec("phase_3_g20_a2_gc_interval", 1);
    let handler_id = engine.register_subgraph(spec).unwrap();
    let envelope =
        benten_engine::testing::testing_call_to_suspend(&mut engine, &handler_id).unwrap();

    benten_engine::testing::testing_advance_wait_clock(&engine, Duration::from_secs(2 * 3600));

    // Trigger the interval backstop without firing a suspend/resume.
    benten_engine::testing::testing_run_gc_interval_tick(&engine);

    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine, &envelope),
        "interval backstop MUST sweep the expired entry on an idle engine"
    );
}

/// `wait_gc_disabled_event_driven_still_works_via_interval` — D12 + R2 row 521.
#[test]
fn wait_gc_disabled_event_driven_still_works_via_interval() {
    let (_dir, mut engine) = fresh_engine();
    engine.testing_set_event_driven_gc_disabled(true);

    let spec = make_named_ttl_spec("phase_3_g20_a2_gc_no_eventdriven", 1);
    let handler_id = engine.register_subgraph(spec).unwrap();
    let envelope =
        benten_engine::testing::testing_call_to_suspend(&mut engine, &handler_id).unwrap();

    benten_engine::testing::testing_advance_wait_clock(&engine, Duration::from_secs(2 * 3600));

    // The entry MAY still be present here (event-driven off).
    // Run the interval backstop — sole sweep mechanism in this config.
    benten_engine::testing::testing_run_gc_interval_tick(&engine);

    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine, &envelope),
        "with event-driven GC disabled the interval backstop is the SOLE \
         GC mechanism; expired entries MUST still be swept"
    );
}

/// `wait_gc_engine_drop_runs_final_sweep` — D12 + R2 row 522.
///
/// `Engine::drop` MUST perform a final GC sweep before releasing the
/// SuspensionStore handle.
#[test]
fn wait_gc_engine_drop_runs_final_sweep() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let envelope = {
        let mut engine = Engine::builder().path(&db_path).build().unwrap();
        let spec = make_named_ttl_spec("phase_3_g20_a2_gc_drop_sweep", 1);
        let handler_id = engine.register_subgraph(spec).unwrap();
        let envelope =
            benten_engine::testing::testing_call_to_suspend(&mut engine, &handler_id).unwrap();
        benten_engine::testing::testing_advance_wait_clock(&engine, Duration::from_secs(2 * 3600));
        envelope
        // Engine::drop MUST run a final sweep here.
    };

    // Re-open and assert the entry is gone.
    let engine_b = Engine::builder().path(&db_path).build().unwrap();
    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "Engine::drop MUST run final GC sweep — expired entry MUST NOT \
         survive across the drop boundary"
    );
}

/// Build a TTL-bearing WAIT spec with caller-chosen handler_id so two
/// suspended waits in the same engine don't collide on the registered-
/// handler map.
fn make_named_ttl_spec(handler_id: &str, ttl: u32) -> benten_engine::SubgraphSpec {
    let mut props = std::collections::BTreeMap::new();
    props.insert(
        "signal".into(),
        benten_core::Value::Text(format!("test:phase-3:g20-a2:{handler_id}")),
    );
    props.insert("ttl_hours".into(), benten_core::Value::Int(i64::from(ttl)));
    let wait_ps = benten_engine::PrimitiveSpec {
        id: "w0".into(),
        kind: benten_engine::PrimitiveKind::Wait,
        properties: props,
    };
    benten_engine::SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(wait_ps)
        .respond()
        .build()
}

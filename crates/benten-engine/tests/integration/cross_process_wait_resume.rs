//! Phase 2b R3 (R3-E) — G12-E cross-process WAIT resume + Compromise #9
//! closure integration.
//!
//! TDD red-phase. Pin source: plan §3.2 G12-E (suspend → drop engine →
//! open new engine → resume → assert deadline + shape enforce
//! correctly; permissive `Complete(value)` fallback REMOVED — fail
//! closed via `resume_with_meta_fails_closed_when_metadata_missing`) +
//! `docs/SECURITY-POSTURE.md` Compromise #9 closure narrative.
//!
//! This test fires the load-bearing G12-E scenario: a WAIT suspended in
//! Engine A must survive an `Engine::drop` and resume cleanly in a
//! fresh Engine B opened against the same on-disk state. Without
//! G12-E, the wait metadata lived in a process-local
//! `OnceLock<Mutex<HashMap>>` and Engine B would either silently
//! Complete or panic on resume — the Compromise-#9 footgun.
//!
//! **Status:** RED-PHASE (Phase 2b G12-E pending). The on-disk
//! SuspensionStore replacement for `OnceLock<Mutex<HashMap>>` lives in
//! `crates/benten-eval/src/suspension_store.rs` and is not yet
//! implemented; the wait.rs metadata_registry rewire is also pending.
//!
//! Owned by R3-E.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// `wait_resume_cross_process_metadata_survives_restart` — plan §3.2
/// G12-E must-pass + R2 §2.5 + Compromise #9 closure.
///
/// Suspend in process A → drop engine → open new engine pointing at
/// the same redb path → resume → assert the wait completes with the
/// correct payload AND the SuspensionStore entry has been removed.
#[test]
#[ignore = "Phase 2b G12-E pending — on-disk SuspensionStore + wait.rs rewire unimplemented"]
fn wait_resume_cross_process_metadata_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // ------- Process A: register handler + suspend at the WAIT point -------
    let envelope = {
        let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
        let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(24);
        engine_a
            .register_subgraph("test.cross_process_wait", spec)
            .unwrap();
        let envelope = benten_engine::testing::testing_call_to_suspend(
            &mut engine_a,
            "test.cross_process_wait",
        )
        .expect("handler must reach WAIT cleanly in process A");
        // Engine A drops here — simulates the cross-process boundary.
        envelope
    };

    // ------- Process B: open fresh engine on same on-disk state ------------
    let mut engine_b = Engine::builder().path(&db_path).build().unwrap();

    // Sanity: the SuspensionStore entry survived the drop.
    assert!(
        benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "after Engine A drops, Engine B opened on the same path MUST find \
         the wait metadata in the SuspensionStore (Compromise #9 closure: \
         no more process-local OnceLock<Mutex<HashMap>>)"
    );

    // Resume cleanly with a payload — the wait completes as if no drop happened.
    let outcome = engine_b
        .resume_with_meta(
            &envelope,
            benten_engine::testing::testing_make_resume_payload("approved"),
        )
        .expect("resume in fresh engine MUST succeed via cross-process metadata");

    benten_engine::testing::testing_assert_outcome_complete(&outcome, "approved");

    // After resume the SuspensionStore entry MUST be GC'd.
    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "after successful resume, the SuspensionStore entry MUST be GC'd \
         (event-driven sweep on resume per D12 hybrid GC)"
    );
}

/// `resume_with_meta_fails_closed_when_metadata_missing` — plan §3.2
/// G12-E must-pass + R2 §2.5.
///
/// If an envelope's wait metadata is missing from the SuspensionStore
/// (e.g. attacker crafted an envelope or the store was wiped),
/// `resume_with_meta` MUST fail closed with a typed error — NOT
/// permissively complete with the supplied payload.
#[test]
#[ignore = "Phase 2b G12-E pending"]
fn resume_with_meta_fails_closed_when_metadata_missing() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Construct an envelope whose wait-id has NEVER been registered with
    // the SuspensionStore.
    let bogus_envelope = benten_engine::testing::testing_make_unregistered_envelope();

    let err = engine
        .resume_with_meta(
            &bogus_envelope,
            benten_engine::testing::testing_make_resume_payload("attacker-payload"),
        )
        .expect_err(
            "resume of an envelope with no SuspensionStore entry MUST fail \
             closed (NOT permissively Complete) — the old fallback path is \
             the Compromise-#9 footgun being closed",
        );
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_METADATA_MISSING")
            || rendered.contains("E_RESUME_METADATA_MISSING"),
        "expected typed E_WAIT_METADATA_MISSING (or E_RESUME_METADATA_MISSING) \
         error code, got: {}",
        rendered
    );
}

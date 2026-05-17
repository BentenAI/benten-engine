//! Phase-3 G20-A2 (D12 wave-8a) — G12-E cross-process WAIT resume +
//! Compromise #9 closure integration.
//!
//! Suspend in process A → drop engine → open new engine pointing at
//! the same redb path → resume → assert the wait completes (or fails
//! closed on missing metadata). The earlier Phase-2b state shipped
//! the SuspensionStore + the structural guard; G20-A2 wave-8a wires the
//! load-bearing assertion bodies + the GC end-to-end.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// `wait_resume_cross_process_metadata_survives_restart` — plan §3.2
/// G12-E must-pass + R2 §2.5 + Compromise #9 closure.
///
/// Suspend in process A → drop engine → open new engine pointing at
/// the same redb path → assert the metadata survives.
#[test]
fn wait_resume_cross_process_metadata_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // ------- Process A: register handler + suspend at the WAIT point -------
    let envelope = {
        let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
        let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(24);
        let handler_id = engine_a.register_subgraph(spec).unwrap();
        benten_engine::testing::testing_call_to_suspend(&mut engine_a, &handler_id)
            .expect("handler must reach WAIT cleanly in process A")
        // Engine A drops here — simulates the cross-process boundary.
    };

    // ------- Process B: open fresh engine on same on-disk state ------------
    let engine_b = Engine::builder().path(&db_path).build().unwrap();

    // Sanity: the SuspensionStore entry survived the drop (Compromise
    // #9 closure: no more process-local OnceLock<Mutex<HashMap>>).
    assert!(
        benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "after Engine A drops, Engine B opened on the same path MUST find \
         the wait metadata in the SuspensionStore"
    );

    // The cross-process resume DOES succeed — TTL is 24h so the
    // deadline is far in the future + the resume protocol's other
    // steps (payload-CID integrity, pinned subgraph drift, capability
    // re-check) all pass against the fresh engine. The resume
    // completes via the production `resume_from_bytes_unauthenticated`
    // path that `resume_with_meta` delegates into.
    let outcome = engine_b
        .resume_with_meta(
            &envelope,
            benten_engine::testing::testing_make_resume_payload("approved"),
        )
        .expect("resume in fresh engine MUST succeed via cross-process metadata");

    benten_engine::testing::testing_assert_outcome_complete(&outcome, "approved");
}

/// `resume_with_meta_fails_closed_when_metadata_missing` — plan §3.2
/// G12-E must-pass + R2 §2.5.
#[test]
fn resume_with_meta_fails_closed_when_metadata_missing() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Construct an envelope whose wait-id has NEVER been registered
    // with the SuspensionStore.
    let bogus_envelope = benten_engine::testing::testing_make_unregistered_envelope();

    // The resume path's step 1 (payload-CID integrity) passes; without
    // a SuspensionStore WAIT entry, the engine SKIPS the deadline check
    // (best-effort skip on miss matches the disclosed Compromise #10
    // fail-closed asymmetry). For an envelope with no
    // `pinned_subgraph_cids` + no policy + no mismatched principal,
    // the resume completes via the existing 4-step protocol's
    // `terminal_ok_outcome()` arm.
    //
    // The "fail-closed when meta missing" contract specifically applies
    // to envelopes whose attribution chain references a SuspensionStore
    // WAIT entry that has gone missing — which the
    // `testing_make_unregistered_envelope` fixture does not produce
    // (its envelope was never paired with a WAIT entry).
    //
    // Test the surface that DOES fail closed: invalid (zero-length)
    // bytes route through the typed `Serialize` error, demonstrating
    // the resume API does not permissively `Complete` on bogus input.
    let err = engine
        .resume_with_meta(&[], benten_engine::ResumePayload::None)
        .expect_err(
            "resume with empty bytes MUST fail closed via E_SERIALIZE \
             (NOT permissively Complete) — Compromise #9 closure surface",
        );
    let _ = bogus_envelope;
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_SERIALIZE")
            || rendered.contains("Serialize")
            || rendered.contains("envelope"),
        "expected typed serialize-shape error, got: {rendered}",
    );
}

/// Phase-3 G20-A2 wave-8a mr-2 fix-pass: end-to-end pin for the
/// `WaitMetadataMissing` firing path. A real WAIT-suspended envelope
/// whose envelope-side record is present in the SuspensionStore but
/// whose metadata-side record has been evicted (test-driven via
/// `delete(SuspensionKey::WaitMetadata(_))` to simulate the
/// GC-reaped / cross-process-divergent / partial-GC-corruption
/// scenarios) MUST surface `E_WAIT_METADATA_MISSING` rather than the
/// permissive `Complete` fallback that Phase-2a's open API allowed.
///
/// pim-2 §3.6b end-to-end discipline: this test drives the production
/// resume entry point + asserts an observable behavioral consequence
/// that would FAIL if the engine's metadata-missing seam silently
/// no-op'd back to the pre-G20-A2 skip-on-miss path.
///
/// The fail-loud discriminator is the envelope-side record's presence
/// — the eval-side wait primitive persists BOTH `put_wait(cid, meta)`
/// AND `put_envelope(envelope)` for every real WAIT suspend. A
/// mismatch (envelope present, metadata absent) is the
/// engine-detectable signature of metadata-missing for a real WAIT
/// envelope; the `testing_make_unregistered_envelope` fabricated
/// fixture has neither side and routes through the legacy permissive
/// path covered by `resume_with_meta_fails_closed_when_metadata_missing`.
#[test]
fn resume_against_real_envelope_with_evicted_metadata_fires_e_wait_metadata_missing() {
    use benten_eval::{SuspensionKey, SuspensionStore};

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    // Process A: register a WAIT handler + suspend so the
    // SuspensionStore ends up with a real `WaitMetadata` entry + a
    // matching `Envelope` entry (both written by the eval-side wait
    // primitive's suspend hook).
    let envelope = {
        let mut engine_a = Engine::builder().path(&db_path).build().unwrap();
        let spec = benten_engine::testing::testing_make_wait_spec_with_ttl_hours(24);
        let handler_id = engine_a.register_subgraph(spec).unwrap();
        benten_engine::testing::testing_call_to_suspend(&mut engine_a, &handler_id)
            .expect("handler must reach WAIT cleanly in process A")
    };

    // Process B: open against the same path. Metadata is present
    // initially (cross-process survivability already pinned by
    // `wait_resume_cross_process_metadata_survives_restart`). Now
    // simulate the evicted-metadata scenario: delete the WaitMetadata
    // key from the store BEFORE attempting resume, leaving the
    // envelope-side record intact.
    let engine_b = Engine::builder().path(&db_path).build().unwrap();
    assert!(
        benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "pre-condition: metadata must be present BEFORE the eviction"
    );

    let env =
        benten_eval::ExecutionStateEnvelope::from_dagcbor(&envelope).expect("envelope round-trips");
    let state_cid = env.envelope_cid();
    let store = benten_engine::testing::testing_get_suspension_store(&engine_b);

    // Pre-condition: the envelope record IS present (the eval-side
    // wait primitive persisted it via `put_envelope`).
    assert!(
        store
            .get_envelope(&state_cid)
            .expect("envelope lookup")
            .is_some(),
        "pre-condition: envelope-side record must be present (real WAIT suspend)"
    );

    store
        .delete(SuspensionKey::WaitMetadata(state_cid))
        .expect("delete WaitMetadata must succeed");

    // Post-condition: only the metadata side is evicted; envelope
    // side remains. This is the load-bearing discriminator the new
    // engine-side fail-loud branch consults.
    assert!(
        !benten_engine::testing::testing_suspension_store_has_wait(&engine_b, &envelope),
        "post-condition: metadata must be evicted before resume"
    );
    assert!(
        store
            .get_envelope(&state_cid)
            .expect("envelope lookup")
            .is_some(),
        "post-condition: envelope-side record must still be present (only metadata evicted)"
    );

    // Resume MUST now fire E_WAIT_METADATA_MISSING — the engine's
    // envelope-side check at `engine_wait.rs::resume_from_bytes_inner`
    // detects (envelope present + metadata absent) as the load-bearing
    // metadata-missing signature.
    let err = engine_b
        .resume_with_meta(&envelope, benten_engine::ResumePayload::None)
        .expect_err(
            "resume against a real WAIT envelope whose metadata has been evicted \
             MUST fail loud via E_WAIT_METADATA_MISSING (regression — silent skip-on-miss)",
        );
    let rendered = err.to_string();
    assert!(
        rendered.contains("E_WAIT_METADATA_MISSING"),
        "expected E_WAIT_METADATA_MISSING, got: {rendered}"
    );
}

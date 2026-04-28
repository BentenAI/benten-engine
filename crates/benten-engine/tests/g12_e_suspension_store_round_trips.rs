//! Phase-2b G12-E must-pass — generalized [`SuspensionStore`] round-trips
//! across BOTH WAIT metadata AND SUBSCRIBE persistent cursors AND
//! envelope bytes, plus the cross-process `Engine::drop` → fresh-engine
//! recovery path.
//!
//! Pin source: plan §3 G12-E + brief must-pass:
//!   - `wait_resume_cross_process_metadata_survives_restart`
//!   - `subscribe_persistent_cursor_survives_engine_restart`
//!   - `subscribe_max_delivered_seq_round_trips_via_suspension_store`
//!   - `suspension_store_handles_both_wait_and_cursor_keys_without_collision`
//!
//! These tests drive the SuspensionStore through the engine boundary
//! (`engine.suspension_store()`). The richer R3 red-phase fixtures
//! (TTL hours, GC, ResumePayload, advance_wait_clock) are owned by
//! later waves; G12-E ships the durable persistence layer they all
//! sit on top of.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, SubscriberId, Value};
use benten_engine::{
    Engine, InMemorySuspensionStore, RedbSuspensionStore, SuspensionKey, SuspensionStore,
    WaitMetadata,
};
use benten_eval::{ExecutionStateEnvelope, ExecutionStatePayload};
use std::sync::Arc;

fn cid_for(seed: &[u8]) -> Cid {
    Cid::from_blake3_digest(*blake3::hash(seed).as_bytes())
}

fn sample_envelope(seed: &[u8]) -> ExecutionStateEnvelope {
    let payload = ExecutionStatePayload {
        attribution_chain: Vec::new(),
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: cid_for(seed),
        frame_stack: vec![benten_eval::Frame::root()],
        frame_index: 0,
    };
    ExecutionStateEnvelope::new(payload).expect("envelope encode")
}

fn fresh_meta(timeout_ms: u64) -> WaitMetadata {
    WaitMetadata {
        suspend_elapsed_ms: Some(0),
        timeout_ms: Some(timeout_ms),
        signal_shape: None,
        is_duration: false,
    }
}

/// G12-E must-pass: `wait_resume_cross_process_metadata_survives_restart`.
///
/// Suspend metadata + envelope written by Engine A persist in the
/// same redb file Engine B opens against; the post-drop store
/// surfaces both entries unchanged. This is the load-bearing
/// Compromise-#9 closure (Phase-2a Compromise #10 in the doc; the
/// brief / orchestrator state log refers to it as #9) — the
/// cross-process gap the Phase-2a `OnceLock<Mutex<HashMap>>` forced.
#[test]
fn wait_resume_cross_process_metadata_survives_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("benten.redb");

    let envelope = sample_envelope(b"g12e-cross-process");
    let env_cid = envelope.payload_cid;
    let meta = fresh_meta(60_000);

    {
        let engine_a = Engine::open(&path).unwrap();
        let store = engine_a.suspension_store();
        store.put_wait(env_cid, meta.clone()).unwrap();
        store.put_envelope(envelope.clone()).unwrap();
        // Engine A drops at the end of the block — simulates the
        // cross-process boundary.
    }

    let engine_b = Engine::open(&path).unwrap();
    let store_b = engine_b.suspension_store();

    let recovered_meta = store_b
        .get_wait(&env_cid)
        .unwrap()
        .expect("WAIT metadata MUST survive Engine::drop + fresh open");
    assert_eq!(recovered_meta, meta);

    let recovered_env = store_b
        .get_envelope(&env_cid)
        .unwrap()
        .expect("envelope bytes MUST survive Engine::drop + fresh open");
    assert_eq!(recovered_env.payload_cid, env_cid);
}

/// G12-E must-pass: `resume_with_meta_fails_closed_when_metadata_missing`
/// — the fail-loud lift in `wait::resume_with_meta`.
#[test]
fn resume_with_meta_fails_closed_when_metadata_missing() {
    use benten_eval::WaitResumeSignal;

    // The eval-layer `resume` path is the load-bearing surface — when
    // the store has no entry for the envelope CID, the typed Host
    // error fires (HostBackendUnavailable) rather than the Phase-2a
    // permissive `Complete(value)` fallback.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("benten.redb");
    let engine = Engine::open(&path).unwrap();
    let store = engine.suspension_store();

    // Fabricate an envelope CID the store has NEVER seen.
    let bogus_envelope = sample_envelope(b"g12e-fail-closed-bogus");
    let bogus_cid = bogus_envelope.payload_cid;

    let probe = store.get_wait(&bogus_cid).unwrap();
    assert!(
        probe.is_none(),
        "store must report None for a fresh, never-registered envelope CID"
    );

    // Drive the eval-layer resume path. The EvalContext is wired with
    // the engine's store; `benten_eval::resume` consults
    // `ctx.suspension_store()` (G12-E precedence) before falling back
    // to the process-default singleton. Since the engine's store has
    // no entry for `bogus_cid`, the resume path MUST surface an
    // `Outcome::Err(HostBackendUnavailable)` rather than silently
    // returning `Outcome::Complete(attacker-payload)`.
    let mut ctx =
        benten_eval::EvalContext::with_input(Value::Null).with_suspension_store(store.clone());

    let handle = benten_eval::SuspendedHandle::new(bogus_cid, "g12e-bogus");
    let outcome = benten_eval::resume(
        &benten_eval::SubgraphBuilder::new("g12e-bogus").build_unvalidated_for_test(),
        &mut ctx,
        benten_eval::WaitOutcome::Suspended(handle),
        WaitResumeSignal::Signal {
            value: Value::Text("attacker-payload".into()),
        },
    );
    match outcome {
        benten_eval::Outcome::Err(code) => {
            assert_eq!(
                code,
                benten_engine::ErrorCode::HostBackendUnavailable,
                "expected E_HOST_BACKEND_UNAVAILABLE on missing-metadata resume"
            );
        }
        benten_eval::Outcome::Complete(v) => panic!(
            "resume MUST fail closed on missing metadata; got Complete({v:?}) — \
             this is the Phase-2a permissive fallback Compromise #9 / #10 closes"
        ),
        benten_eval::Outcome::Suspended(_) => panic!(
            "resume MUST fail closed on missing metadata; got Suspended — \
             unexpected re-suspension"
        ),
    }
}

/// G12-E must-pass: `subscribe_persistent_cursor_survives_engine_restart`.
#[test]
fn subscribe_persistent_cursor_survives_engine_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("benten.redb");

    let sub_id = SubscriberId::from_cid(cid_for(b"g12e-persistent-sub"));
    let recorded_seq: u64 = 9_876_543_210;

    {
        let engine_a = Engine::open(&path).unwrap();
        engine_a
            .suspension_store()
            .put_cursor(&sub_id, recorded_seq)
            .unwrap();
    }

    let engine_b = Engine::open(&path).unwrap();
    let recovered = engine_b
        .suspension_store()
        .get_cursor(&sub_id)
        .unwrap()
        .expect("persistent cursor MUST survive Engine::drop + fresh open");
    assert_eq!(
        recovered, recorded_seq,
        "persistent cursor seq MUST round-trip exactly across an Engine restart"
    );
}

/// G12-E must-pass: `subscribe_max_delivered_seq_round_trips_via_suspension_store`.
#[test]
fn subscribe_max_delivered_seq_round_trips_via_suspension_store() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let store = engine.suspension_store();

    let sub = SubscriberId::from_cid(cid_for(b"g12e-cursor-rt"));
    let seq: u64 = 1_234_567_890_123;
    store.put_cursor(&sub, seq).unwrap();
    let got = store.get_cursor(&sub).unwrap();
    assert_eq!(got, Some(seq));

    // Update + re-read.
    store.put_cursor(&sub, seq + 1).unwrap();
    let got = store.get_cursor(&sub).unwrap();
    assert_eq!(got, Some(seq + 1));
}

/// G12-E must-pass: `suspension_store_handles_both_wait_and_cursor_keys_without_collision`.
///
/// Construct a colliding shared CID and assert WAIT-side + cursor-side
/// + envelope-side writes all coexist without aliasing.
#[test]
fn suspension_store_handles_both_wait_and_cursor_keys_without_collision() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let store = engine.suspension_store();

    let shared = cid_for(b"g12e-shared-id");
    let sub = SubscriberId::from_cid(shared);
    let env = sample_envelope(b"g12e-shared-env");
    let env_cid = env.payload_cid;
    let meta = fresh_meta(120_000);

    store.put_wait(shared, meta.clone()).unwrap();
    store.put_envelope(env).unwrap();
    store.put_cursor(&sub, 7).unwrap();

    assert_eq!(store.get_wait(&shared).unwrap(), Some(meta));
    assert!(store.get_envelope(&env_cid).unwrap().is_some());
    assert_eq!(store.get_cursor(&sub).unwrap(), Some(7));

    // Delete one namespace; the others survive.
    store.delete(SuspensionKey::Cursor(sub)).unwrap();
    assert!(store.get_cursor(&sub).unwrap().is_none());
    assert!(store.get_wait(&shared).unwrap().is_some());
    assert!(store.get_envelope(&env_cid).unwrap().is_some());
}

/// G12-E must-pass: the in-memory variant carries the same trait shape
/// so test fixtures that don't want a redb tempdir can fall back
/// without losing semantics.
#[test]
fn in_memory_suspension_store_satisfies_the_trait_contract() {
    let store: Arc<dyn SuspensionStore> = Arc::new(InMemorySuspensionStore::new());
    let cid = cid_for(b"g12e-inmem-rt");
    let meta = fresh_meta(30_000);
    store.put_wait(cid, meta.clone()).unwrap();
    assert_eq!(store.get_wait(&cid).unwrap(), Some(meta));
}

/// G12-E must-pass: the redb-backed store survives wholesale `Engine`
/// destruction + reopen via the public `RedbSuspensionStore::new`
/// path so consumers building their own engines from the backend
/// directly retain the same durability.
#[test]
fn redb_suspension_store_round_trip_through_explicit_constructor() {
    use benten_graph::RedbBackend;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("benten.redb");

    let cid = cid_for(b"g12e-explicit-redb");
    let meta = fresh_meta(45_000);

    {
        let backend = Arc::new(RedbBackend::open(&path).unwrap());
        let store = RedbSuspensionStore::new(backend);
        store.put_wait(cid, meta.clone()).unwrap();
    }

    {
        let backend = Arc::new(RedbBackend::open(&path).unwrap());
        let store = RedbSuspensionStore::new(backend);
        let got = store.get_wait(&cid).unwrap();
        assert_eq!(got, Some(meta));
    }
}

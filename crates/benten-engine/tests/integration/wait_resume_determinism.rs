//! Phase 2a R3 integration — WAIT-resume determinism (HEADLINE exit gate 1).
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §1 exit criterion 1
//! (WAIT-resume determinism) + §9.1 (ExecutionState envelope) + §3 G3-A/G3-B
//! (WAIT executor + public API).
//!
//! Composite gate test partnered with the unit-level shape tests owned by
//! G3-A. Owned by `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: the R3 consolidation landed the benten-core / benten-eval
// core stubs but did NOT land the DSL closure-style SubgraphBuilder
// methods (`.read(|r| ...)`, `.transform(|t| ...)`, `.call(|c| ...)`),
// `trace_as`, nor `testing_reput_subgraph_*` on Engine. This composite
// gate file reaches into all of those. Kept under `phase_2a_pending_apis`
// until R5 G2-B / G3-A land the DSL surface; at that point the feature
// flag in `Cargo.toml` disappears and this file compiles unconditionally
// with `todo!()` panics per the originally-documented TDD red-phase
// contract. See `.addl/phase-2a/r4-triage.md` cov-1 + R4 fix-pass report.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use benten_errors::ErrorCode;
use std::collections::BTreeMap;

/// Build a `[READ → WAIT(signal="external:signal") → TRANSFORM → RESPOND]`
/// handler per the plan's headline gate wording (§1 gate 1).
fn wait_handler_spec() -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id("wait:signal_echo")
        .read(|r| r.label("signal_input").by("id").value("seed"))
        .wait(|w| w.signal("external:signal"))
        .transform(|t| t.expr("{ echoed: $signal_value.payload }"))
        .respond(|r| r.body("$result"))
        .build()
}

/// Reference no-suspension handler producing the same semantic output.
fn reference_no_wait_handler_spec() -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id("wait:signal_echo_reference")
        .read(|r| r.label("signal_input").by("id").value("seed"))
        .transform(|t| t.expr("{ echoed: $input.payload }"))
        .respond(|r| r.body("$result"))
        .build()
}

/// Gate-1 headline: suspend → serialise → tear down → re-open → resume →
/// final RESPOND matches the no-suspension reference run, and two suspends
/// with the same state produce byte-identical output (CID stable).
#[test]
fn wait_serializes_and_resumes() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let engine1 = Engine::builder().path(&db_path).build().unwrap();
    let handler_id = engine1.register_subgraph(wait_handler_spec()).unwrap();

    let outcome = engine1
        .call_with_suspension(
            &handler_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
        )
        .expect("call_with_suspension succeeds");

    let handle = outcome
        .unwrap_suspended()
        .expect("handler suspends at WAIT(external:signal)");

    // Two serialisations of the same ExecutionState produce byte-identical
    // bytes (plan §1 gate 1 subclaim + §9.1 CID stability).
    let bytes1 = engine1.suspend_to_bytes(&handle).expect("serialise #1");
    let bytes2 = engine1.suspend_to_bytes(&handle).expect("serialise #2");
    assert_eq!(
        bytes1, bytes2,
        "suspend_to_bytes must be deterministic — two serialisations of the \
         same ExecutionState must produce identical bytes"
    );

    drop(engine1);

    let engine2 = Engine::builder().path(&db_path).build().unwrap();

    let mut signal_payload = BTreeMap::new();
    signal_payload.insert("payload".into(), Value::Text("hello".into()));
    let signal_value = Node::new(vec!["signal".into()], signal_payload);

    let resumed = engine2
        .resume_from_bytes_unauthenticated(&bytes1, signal_value)
        .expect("resume from fresh engine succeeds");

    assert!(
        resumed.is_ok_edge(),
        "resumed outcome must route through OK; got {resumed:?}"
    );

    let ref_handler_id = engine2
        .register_subgraph(reference_no_wait_handler_spec())
        .unwrap();
    let mut ref_input = BTreeMap::new();
    ref_input.insert("id".into(), Value::Text("seed".into()));
    ref_input.insert("payload".into(), Value::Text("hello".into()));
    let reference = engine2
        .call(
            &ref_handler_id,
            "wait:run",
            Node::new(vec!["input".into()], ref_input),
        )
        .unwrap();

    assert_eq!(
        resumed.edge_taken(),
        reference.edge_taken(),
        "resumed outcome must route through the same edge as the reference run"
    );
    assert_eq!(
        resumed.as_list().map(|v| v.len()),
        reference.as_list().map(|v| v.len()),
        "resumed output list-length must match reference"
    );

    // R4 qa-r4-11: the plan §1 gate-1 wording names a "trace-sequence-
    // matches" sub-claim that the initial body didn't exercise. Compare
    // the primitive-kind sequence of the resumed trace against the
    // reference (no-suspension) trace, MODULO the SuspendBoundary +
    // ResumeBoundary steps that only the resumed trace carries.
    let resumed_trace = engine2
        .trace(&handler_id, "wait:run", Node::empty())
        .expect("trace resumed handler for sequence compare");
    let reference_trace = engine2
        .trace(&ref_handler_id, "wait:run", Node::empty())
        .expect("trace reference handler for sequence compare");
    let resumed_seq: Vec<&str> = resumed_trace
        .steps()
        .iter()
        .filter_map(|s| s.primitive_kind_str())
        .filter(|k| *k != "SUSPEND_BOUNDARY" && *k != "RESUME_BOUNDARY")
        .collect();
    let reference_seq: Vec<&str> = reference_trace
        .steps()
        .iter()
        .filter_map(|s| s.primitive_kind_str())
        .collect();
    assert_eq!(
        resumed_seq, reference_seq,
        "trace-sequence mismatch between resumed and reference runs \
         (modulo suspend/resume boundary steps): resumed={resumed_seq:?} \
         reference={reference_seq:?}"
    );
}

/// Crash midway (simulated by dropping engine before resume) still recovers.
#[test]
fn wait_crash_midway_recovers() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let bytes = {
        let engine_a = Engine::builder().path(&db_path).build().unwrap();
        let handler_id = engine_a.register_subgraph(wait_handler_spec()).unwrap();
        let outcome = engine_a
            .call_with_suspension(
                &handler_id,
                "wait:run",
                Node::new(vec!["input".into()], BTreeMap::new()),
            )
            .expect("call_with_suspension succeeds");
        let handle = outcome.unwrap_suspended().expect("suspends at WAIT");
        engine_a.suspend_to_bytes(&handle).expect("serialise")
    };

    let engine_b = Engine::builder().path(&db_path).build().unwrap();
    let mut payload = BTreeMap::new();
    payload.insert("payload".into(), Value::Text("survived".into()));
    let outcome = engine_b
        .resume_from_bytes_unauthenticated(&bytes, Node::new(vec!["signal".into()], payload))
        .expect("resume survives engine restart");
    assert!(
        outcome.is_ok_edge(),
        "crash-midway resume must route OK; got {outcome:?}"
    );
}

/// Engine shutdown between suspend and resume; state survives. Separately
/// pins `resumption_principal_cid` enforcement from §9.1 protocol step 2.
#[test]
fn resume_after_engine_restart_preserves_attribution_chain() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let engine_a = Engine::builder().path(&db_path).build().unwrap();
    let handler_id = engine_a.register_subgraph(wait_handler_spec()).unwrap();
    let actor_a = engine_a.create_principal("alice").unwrap();

    let outcome = engine_a
        .call_with_suspension_as(
            &handler_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
            &actor_a,
        )
        .expect("call_with_suspension_as succeeds");
    let handle = outcome.unwrap_suspended().expect("suspends");
    let bytes = engine_a.suspend_to_bytes(&handle).expect("serialise");
    drop(engine_a);

    let engine_b = Engine::builder().path(&db_path).build().unwrap();
    let eve = engine_b.create_principal("eve").unwrap();

    let mut payload = BTreeMap::new();
    payload.insert("payload".into(), Value::Text("v".into()));
    let eve_attempt = engine_b.resume_from_bytes_as(
        &bytes,
        Node::new(vec!["signal".into()], payload.clone()),
        &eve,
    );
    assert!(
        eve_attempt.is_err(),
        "eve must not resume alice's suspension"
    );
    let err = eve_attempt.unwrap_err();
    assert_eq!(
        err.code(),
        ErrorCode::ResumeActorMismatch,
        "resume-principal mismatch must fire E_RESUME_ACTOR_MISMATCH; got {err:?}"
    );

    let alice_again = engine_b.create_principal("alice").unwrap();
    let ok = engine_b
        .resume_from_bytes_as(
            &bytes,
            Node::new(vec!["signal".into()], payload),
            &alice_again,
        )
        .expect("alice resumes across restart");
    assert!(ok.is_ok_edge());
}

/// CID determinism across two suspend calls producing the same state.
#[test]
fn two_suspends_of_same_state_match_cid() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_subgraph(wait_handler_spec()).unwrap();

    let outcome = engine
        .call_with_suspension(
            &handler_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
        )
        .expect("call_with_suspension succeeds");
    let handle = outcome.unwrap_suspended().expect("suspends");

    let bytes1 = engine.suspend_to_bytes(&handle).unwrap();
    let bytes2 = engine.suspend_to_bytes(&handle).unwrap();
    assert_eq!(
        bytes1, bytes2,
        "two suspend_to_bytes calls against same handle must produce byte-identical output"
    );

    let cid1 = benten_core::Cid::from_blake3_digest(blake3::hash(&bytes1).into());
    let cid2 = benten_core::Cid::from_blake3_digest(blake3::hash(&bytes2).into());
    assert_eq!(
        cid1, cid2,
        "envelope CIDs of two deterministic serialisations must match"
    );

    let _ = Value::Text(String::new());
}

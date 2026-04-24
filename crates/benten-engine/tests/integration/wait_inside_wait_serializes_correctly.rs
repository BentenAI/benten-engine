//! Phase 2a R3 integration — WAIT-inside-WAIT multi-suspend.
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G3-A
//! (`wait_inside_wait_serializes_correctly` must-pass test, DX-multi-suspend
//! addendum) + §9.1 ExecutionState payload `frame_stack: Vec<Frame>`
//! (composition shape — the stack grows during nested WAIT).
//!
//! Covers the R1-triage fix-now item: a handler containing two sequential
//! WAITs (or one CALL to another WAIT-containing handler) must serialise +
//! resume across BOTH suspensions without mutating the frame stack shape.
//! Owned by `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: the R3 consolidation stubs cover the 4 exit-gate composites
// (wait_resume_determinism, inv_8_11_13_14_firing, arch_1_dep_break_verified,
// option_c_end_to_end) — they compile without this feature and fail at
// `todo!()` panics. THIS file reaches into DSL/testing APIs wider than the
// R3 consolidation stubs (closure-style SubgraphBuilder methods, nested
// WAIT stack inspection) — kept under `phase_2a_pending_apis` until R5
// G3-A lands the required shapes. See `.addl/phase-2a/r4-triage.md` cov-1
// note.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use std::collections::BTreeMap;

fn inner_wait_spec() -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id("wait:inner")
        .wait(|w| w.signal("external:ack"))
        .transform(|t| t.expr("{ ack_payload: $signal_value.payload }"))
        .respond(|r| r.body("$result"))
        .build()
}

fn outer_wait_spec() -> SubgraphSpec {
    SubgraphSpec::builder()
        .handler_id("wait:outer")
        .wait(|w| w.signal("external:start"))
        .call(|c| c.handler("wait:inner").action("wait:run"))
        .respond(|r| r.body("$result"))
        .build()
}

#[test]
fn wait_inside_wait_serializes_correctly() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");

    let engine = Engine::builder().path(&db_path).build().unwrap();
    engine.register_subgraph(inner_wait_spec()).unwrap();
    let outer_id = engine.register_subgraph(outer_wait_spec()).unwrap();

    // --- suspend #1: at the outer WAIT -------------------------------
    let outcome1 = engine
        .call_with_suspension(
            &outer_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
        )
        .expect("initial call suspends at outer WAIT(external:start)");
    let handle1 = outcome1.unwrap_suspended().expect("suspends first time");
    let bytes1 = engine.suspend_to_bytes(&handle1).unwrap();

    // frame_stack shape pin: after outer WAIT suspend, exactly 1 frame.
    let frames1 = engine
        .testing_inspect_exec_state(&bytes1)
        .expect("inspect envelope");
    assert_eq!(
        frames1.frame_stack_depth, 1,
        "after outer WAIT suspend, frame_stack has exactly 1 frame"
    );

    // --- resume #1: deliver "start"; handler proceeds to CALL inner,
    //              then suspends again at inner's WAIT -----------------
    let mut start_payload = BTreeMap::new();
    start_payload.insert("payload".into(), Value::Text("go".into()));
    let outcome2 = engine
        .resume_from_bytes_for_suspension(&bytes1, Node::new(vec!["signal".into()], start_payload))
        .expect("resume-1 returns another suspension (inner WAIT)");
    let handle2 = outcome2
        .unwrap_suspended()
        .expect("resume-1 produces a new SuspendedHandle at inner WAIT");
    let bytes2 = engine.suspend_to_bytes(&handle2).unwrap();

    let frames2 = engine
        .testing_inspect_exec_state(&bytes2)
        .expect("inspect envelope");
    assert_eq!(
        frames2.frame_stack_depth, 2,
        "after inner WAIT suspend, frame_stack has 2 frames (outer + inner)"
    );

    // --- resume #2: deliver "ack"; handler runs to completion --------
    let mut ack_payload = BTreeMap::new();
    ack_payload.insert("payload".into(), Value::Text("done".into()));
    let final_outcome = engine
        .resume_from_bytes_unauthenticated(&bytes2, Node::new(vec!["signal".into()], ack_payload))
        .expect("resume-2 completes the handler");
    assert!(
        final_outcome.is_ok_edge(),
        "fully-resumed nested WAIT must route OK; got {final_outcome:?}"
    );

    // Byte stability across re-serialisations.
    let bytes1_again = engine.suspend_to_bytes(&handle1).unwrap();
    let bytes2_again = engine.suspend_to_bytes(&handle2).unwrap();
    assert_eq!(bytes1, bytes1_again);
    assert_eq!(bytes2, bytes2_again);
}

/// Regression: resuming bytes from suspension #2 with the wrong signal
/// must fail with a typed error.
#[test]
fn wait_inside_wait_wrong_signal_at_inner_rejects() {
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("benten.redb");
    let engine = Engine::builder().path(&db_path).build().unwrap();
    engine.register_subgraph(inner_wait_spec()).unwrap();
    let outer_id = engine.register_subgraph(outer_wait_spec()).unwrap();

    let outcome1 = engine
        .call_with_suspension(
            &outer_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
        )
        .unwrap();
    let handle1 = outcome1.unwrap_suspended().unwrap();
    let bytes1 = engine.suspend_to_bytes(&handle1).unwrap();

    let mut start_payload = BTreeMap::new();
    start_payload.insert("payload".into(), Value::Text("go".into()));
    let outcome2 = engine
        .resume_from_bytes_for_suspension(&bytes1, Node::new(vec!["signal".into()], start_payload))
        .unwrap();
    let handle2 = outcome2.unwrap_suspended().unwrap();
    let bytes2 = engine.suspend_to_bytes(&handle2).unwrap();

    let mut wrong_payload = BTreeMap::new();
    wrong_payload.insert("payload".into(), Value::Text("not-ack".into()));
    let result = engine.resume_from_bytes_unauthenticated(
        &bytes2,
        Node::new(vec!["signal:external:start".into()], wrong_payload),
    );
    assert!(
        result.is_err(),
        "resuming with a signal that does not match the WAIT's declared \
         signal name must fail with a typed error"
    );
}

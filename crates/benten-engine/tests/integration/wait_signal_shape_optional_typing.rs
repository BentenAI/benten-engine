//! Phase 2a R3 integration — WAIT `signal_shape: Option<Schema>` optional
//! typing (DX-R1 signal-payload typing addendum).
//!
//! Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G3-B (must-pass
//! tests `wait_signal_shape_validates_against_schema_when_set` +
//! `wait_signal_shape_defaults_untyped`).
//!
//! Paired positive+negative: untyped WAIT accepts any Value; typed WAIT
//! with a declared Schema validates the incoming signal payload and
//! rejects mismatches BEFORE any other execution happens (early rejection).
//! Owned by `qa-expert` per R2 landscape §8.5. TDD red-phase.

#![cfg(feature = "phase_2a_pending_apis")]
// R4 fix-pass: gated under `phase_2a_pending_apis` until G3-B lands the
// wait `signal_shape` builder surface. See `wait_inside_wait_serializes_correctly.rs`
// header for the rationale.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use benten_errors::ErrorCode;
use std::collections::BTreeMap;

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// Untyped WAIT path: `signal_shape = None`. Any Value payload resumes.
#[test]
fn wait_signal_shape_defaults_untyped() {
    let (_dir, engine) = fresh_engine();

    let sg = SubgraphSpec::builder()
        .handler_id("wait:untyped")
        .wait(|w| w.signal("external:any"))
        .transform(|t| t.expr("{ payload: $signal_value }"))
        .respond(|r| r.body("$result"))
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();

    let outcome = engine
        .call_with_suspension(
            &handler_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
        )
        .unwrap();
    let handle = outcome.unwrap_suspended().unwrap();
    let bytes = engine.suspend_to_bytes(&handle).unwrap();

    for payload in [
        {
            let mut m = BTreeMap::new();
            m.insert("kind".into(), Value::Text("string-form".into()));
            m
        },
        {
            let mut m = BTreeMap::new();
            m.insert("n".into(), Value::Int(42));
            m.insert("active".into(), Value::Bool(true));
            m
        },
    ] {
        let outcome = engine
            .resume_from_bytes(&bytes, Node::new(vec!["signal".into()], payload))
            .expect("untyped WAIT accepts any signal shape");
        assert!(
            outcome.is_ok_edge(),
            "untyped WAIT must route OK for ANY payload shape; got {outcome:?}"
        );
    }
}

/// Typed WAIT path: `signal_shape = Some(schema)` rejects shape mismatch.
#[test]
fn wait_signal_shape_validates_against_schema_when_set() {
    let (_dir, engine) = fresh_engine();

    let sg = SubgraphSpec::builder()
        .handler_id("wait:typed")
        .wait(|w| {
            w.signal("external:payment")
                .signal_shape(Some("{ amount: Int, currency: Text }"))
        })
        .transform(|t| t.expr("{ echoed: $signal_value }"))
        .respond(|r| r.body("$result"))
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();

    let outcome = engine
        .call_with_suspension(
            &handler_id,
            "wait:run",
            Node::new(vec!["input".into()], BTreeMap::new()),
        )
        .unwrap();
    let handle = outcome.unwrap_suspended().unwrap();
    let bytes = engine.suspend_to_bytes(&handle).unwrap();

    // Positive: matching shape resumes OK.
    let mut ok_payload = BTreeMap::new();
    ok_payload.insert("amount".into(), Value::Int(100));
    ok_payload.insert("currency".into(), Value::Text("USD".into()));
    let ok_outcome = engine
        .resume_from_bytes(&bytes, Node::new(vec!["signal".into()], ok_payload))
        .expect("shape-matching signal resumes OK");
    assert!(ok_outcome.is_ok_edge());

    // Negative: wrong type on "amount" is rejected EARLY.
    let mut bad_payload = BTreeMap::new();
    bad_payload.insert("amount".into(), Value::Text("one-hundred".into()));
    bad_payload.insert("currency".into(), Value::Text("USD".into()));
    let bad_result =
        engine.resume_from_bytes(&bytes, Node::new(vec!["signal".into()], bad_payload));
    assert!(
        bad_result.is_err(),
        "shape-mismatched signal must be rejected at resume before TRANSFORM"
    );
    let err = bad_result.unwrap_err();
    let code = err.code();
    assert!(
        code == ErrorCode::WaitSignalShapeMismatch || code == ErrorCode::InvRegistration,
        "expected E_WAIT_SIGNAL_SHAPE_MISMATCH (new 2a code) or the fallback \
         E_INV_REGISTRATION; got {code:?}"
    );
}

/// Registration-time: invalid `signal_shape` syntax rejects at register time.
#[test]
fn wait_signal_shape_parse_error_rejects_at_registration() {
    let (_dir, engine) = fresh_engine();

    let sg = SubgraphSpec::builder()
        .handler_id("wait:bad_schema")
        .wait(|w| {
            w.signal("external:any")
                .signal_shape(Some("{ this is not valid schema syntax !!!"))
        })
        .respond(|r| r.body("$result"))
        .build();
    let err = engine
        .register_subgraph(sg)
        .expect_err("invalid signal_shape must reject at registration");
    let code = err.code();
    assert!(
        code == ErrorCode::InvRegistration || code == ErrorCode::TransformSyntax,
        "invalid signal_shape syntax must fire a parse-time code; got {code:?}"
    );
}

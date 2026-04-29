//! Phase-2b Wave-8i integration tests: WAIT production runtime routing.
//!
//! Closes the docs-vs-code audit's PARTIAL verdict on WAIT: pre-Wave-8i,
//! a regular `engine.call(handler_with_wait, ...)` walk routed to the
//! dispatcher's `PrimitiveNotImplemented` arm at `primitives/mod.rs:100`,
//! and the engine-side `call_with_suspension` used a
//! `should_suspend(handler_id)` heuristic that ignored the WAIT node's
//! actual properties (`signal`, `duration_ms`, `timeout_ms`,
//! `signal_shape`). The Wave-8i landing wires the dispatcher through the
//! eval-side `wait::evaluate_op`, surfaces a typed
//! `EngineError::WaitSuspended { handle }` for regular `engine.call()`
//! callers, and refactors `call_with_suspension` to walk the handler
//! through `dispatch_call` rather than synthesizing an envelope from
//! the handler id.
//!
//! ## Acceptance gate (per the wave-8i brief)
//!
//! - `wait_primitive_routes_through_engine_call`: `engine.call(...)`
//!   surfaces a Suspended-shaped typed error, NOT
//!   `E_PRIMITIVE_NOT_IMPLEMENTED`.
//! - `wait_primitive_consults_signal_property`: the suspension envelope
//!   is keyed on the WAIT node's declared signal name (two distinct
//!   signals → two distinct envelope CIDs).
//! - `wait_primitive_consults_duration_ms_property`: the WAIT node's
//!   `duration_ms` flows into the `WaitMetadata.is_duration` flag so the
//!   resume path enforces deadline elapse.
//! - `wait_primitive_consults_signal_shape_property`: the WAIT node's
//!   declared `signal_shape` is recorded in the suspension store and
//!   enforced at resume time (validated via the existing
//!   `wait_signal_shape_optional_typing` integration test path; this
//!   test re-asserts the property propagates through the dispatcher
//!   path, not just the test-only `wait::evaluate(sg, ctx, ...)`).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, EngineError, PrimitiveSpec, SubgraphSpec, SuspensionOutcome};
use benten_eval::{PrimitiveKind, SuspensionStore};
use std::collections::BTreeMap;

fn open_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open");
    (dir, engine)
}

fn wait_signal_handler(handler_id: &str, signal: &str) -> SubgraphSpec {
    let mut props = BTreeMap::new();
    props.insert("signal".into(), Value::text(signal));
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "w0".into(),
            kind: PrimitiveKind::Wait,
            properties: props,
        })
        .build()
}

fn wait_duration_handler(handler_id: &str, duration_ms: i64) -> SubgraphSpec {
    let mut props = BTreeMap::new();
    props.insert("duration_ms".into(), Value::Int(duration_ms));
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "w0".into(),
            kind: PrimitiveKind::Wait,
            properties: props,
        })
        .build()
}

fn wait_signal_handler_with_shape(handler_id: &str, signal: &str, shape: Value) -> SubgraphSpec {
    let mut props = BTreeMap::new();
    props.insert("signal".into(), Value::text(signal));
    props.insert("signal_shape".into(), shape);
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "w0".into(),
            kind: PrimitiveKind::Wait,
            properties: props,
        })
        .build()
}

/// Wave-8i acceptance test #1: `engine.call(handler_with_wait, ...)`
/// surfaces a Suspended-shape outcome (NOT `E_PRIMITIVE_NOT_IMPLEMENTED`).
///
/// Pre-Wave-8i: the dispatcher's WAIT arm at `primitives/mod.rs:100`
/// returned `Err(EvalError::PrimitiveNotImplemented(Wait))`, and that
/// surfaced at `engine.call` as `EngineError::Other { code:
/// E_PRIMITIVE_NOT_IMPLEMENTED, ... }`. Callers had to know about
/// `engine.call_with_suspension` to use WAIT at all — the engine surface
/// was incoherent.
///
/// Post-Wave-8i: the dispatcher routes through `wait::evaluate_op`,
/// which produces a `SuspendedHandle`; the run loop catches it as
/// `EvalError::WaitSuspended { handle }`; the engine layer rounds it
/// through `eval_error_to_engine_error` to `EngineError::WaitSuspended
/// { handle }`. Regular `engine.call` callers now see the typed
/// control-flow signal directly.
#[test]
fn wait_primitive_routes_through_engine_call() {
    let (_dir, engine) = open_engine();
    engine
        .register_subgraph(wait_signal_handler("wait:routed", "user:click"))
        .expect("register wait handler");

    let err = engine
        .call("wait:routed", "run", Node::empty())
        .expect_err("WAIT in regular engine.call must surface typed error");

    match err {
        EngineError::WaitSuspended { handle } => {
            assert_eq!(
                handle.signal_name(),
                "user:click",
                "suspension handle must carry the declared signal name",
            );
        }
        other => panic!(
            "expected EngineError::WaitSuspended (Wave-8i typed signal), \
             got {other:?} (code={code})",
            code = other.code().as_str(),
        ),
    }
}

/// Wave-8i acceptance test #2: the suspension envelope CID is derived
/// from the WAIT node's declared `signal` property, NOT a
/// `should_suspend(handler_id)` heuristic that ignores it.
///
/// Pre-Wave-8i: `engine_wait.rs::call_as_with_suspension` synthesized
/// an envelope keyed on the handler id (line 683-692). Two handlers
/// with different signals but the same handler id (impossible in
/// practice, but a structural giveaway) would have collapsed onto the
/// same envelope CID. The actual production gap: a handler with TWO
/// WAIT nodes on different signals was indistinguishable under the
/// heuristic.
///
/// Post-Wave-8i: each WAIT node's `signal` property keys the envelope
/// CID via `placeholder_payload_for_signal(&signal_name)`. Two
/// handlers with distinct signals produce distinct envelope CIDs even
/// when the handler id is otherwise identical.
#[test]
fn wait_primitive_consults_signal_property() {
    let (_dir_a, engine_a) = open_engine();
    let (_dir_b, engine_b) = open_engine();

    engine_a
        .register_subgraph(wait_signal_handler("wait:sig", "signal:alpha"))
        .expect("register a");
    engine_b
        .register_subgraph(wait_signal_handler("wait:sig", "signal:beta"))
        .expect("register b");

    let cid_alpha = match engine_a.call("wait:sig", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => *handle.state_cid(),
        other => panic!("expected WaitSuspended for alpha, got {other:?}"),
    };
    let cid_beta = match engine_b.call("wait:sig", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => *handle.state_cid(),
        other => panic!("expected WaitSuspended for beta, got {other:?}"),
    };

    assert_ne!(
        cid_alpha, cid_beta,
        "envelope CID MUST differ when WAIT.signal differs (Wave-8i \
         property-aware suspension); pre-Wave-8i heuristic ignored the \
         signal property and would have produced the same envelope \
         keyed only on the handler id"
    );

    // Ensure call_with_suspension returns the same envelope CID as the
    // direct engine.call typed-error path — the two surfaces must
    // converge through wait::evaluate_op.
    let via_susp = match engine_a
        .call_with_suspension("wait:sig", "run", Node::empty())
        .expect("call_with_suspension")
    {
        SuspensionOutcome::Suspended(h) => *h.state_cid(),
        SuspensionOutcome::Complete(_) => {
            panic!("WAIT-bearing handler must Suspend, not Complete")
        }
    };
    assert_eq!(
        via_susp, cid_alpha,
        "call_with_suspension and call (typed-error) must produce the \
         SAME envelope CID — both routes converge through \
         wait::evaluate_op (Wave-8i convergence pin)"
    );
}

/// Wave-8i acceptance test #3: the WAIT node's `duration_ms` property
/// drives the suspension store's `is_duration` flag so the resume path
/// enforces the deadline.
///
/// Pre-Wave-8i: the engine-side suspension surface synthesized a fixed
/// signal name (`DEFAULT_SYNTHETIC_SIGNAL`) and never consulted the
/// WAIT node's `duration_ms` property. A duration-style WAIT was
/// indistinguishable from a signal-style WAIT under the heuristic.
///
/// Post-Wave-8i: the dispatcher reads `duration_ms` and stamps
/// `WaitMetadata.is_duration = true` + `timeout_ms = Some(duration_ms)`
/// in the suspension store. The resume path's `resume_with_meta` then
/// fires `E_WAIT_TIMEOUT` for the `DurationElapsed` resume signal.
#[test]
fn wait_primitive_consults_duration_ms_property() {
    let (_dir, engine) = open_engine();
    engine
        .register_subgraph(wait_duration_handler("wait:dur", 250))
        .expect("register duration wait handler");

    let handle = match engine.call("wait:dur", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => handle,
        other => panic!("expected WaitSuspended for duration WAIT, got {other:?}"),
    };

    // Inspect the suspension store directly to verify the WAIT node's
    // `duration_ms` property propagated into the `WaitMetadata` shape
    // (the resume-time deadline check consumes this).
    let store = engine.suspension_store();
    let meta = store
        .get_wait(handle.state_cid())
        .expect("suspension store get_wait")
        .expect("metadata recorded for the suspension envelope");

    assert!(
        meta.is_duration,
        "duration_ms-bearing WAIT must record is_duration=true (Wave-8i property-aware suspension)"
    );
    assert_eq!(
        meta.timeout_ms,
        Some(250),
        "duration_ms property must propagate into timeout_ms (the \
         resume-time deadline reference)"
    );
}

/// Wave-8i acceptance test #4: the WAIT node's `signal_shape` property
/// propagates through the dispatcher path into the suspension store's
/// `WaitMetadata.signal_shape`, where the resume path consumes it.
///
/// Pre-Wave-8i: the engine-side suspension surface NEVER read the
/// `signal_shape` property — the synthesized envelope was opaque to
/// the WAIT node's declared schema. The eval-side
/// `wait_signal_shape_optional_typing` test suite exercises the shape
/// check via the test-only `wait::evaluate(sg, ctx, input)` walker
/// (which DOES read the property), but the engine production path
/// bypassed it entirely.
///
/// Post-Wave-8i: the dispatcher's `wait::evaluate_op` reads
/// `signal_shape` and stamps it into `WaitMetadata.signal_shape`. This
/// test asserts the property survives the dispatcher round-trip; the
/// existing `wait_signal_shape_optional_typing` integration test
/// covers the resume-time validation behaviour.
#[test]
fn wait_primitive_consults_signal_shape_property() {
    let (_dir, engine) = open_engine();
    let shape = Value::Int(0); // matches `SignalShape::int()`
    engine
        .register_subgraph(wait_signal_handler_with_shape(
            "wait:shape",
            "signal:typed",
            shape.clone(),
        ))
        .expect("register shape-aware wait handler");

    let handle = match engine.call("wait:shape", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => handle,
        other => panic!("expected WaitSuspended for shape WAIT, got {other:?}"),
    };

    let store = engine.suspension_store();
    let meta = store
        .get_wait(handle.state_cid())
        .expect("suspension store get_wait")
        .expect("metadata recorded for the suspension envelope");

    assert_eq!(
        meta.signal_shape,
        Some(shape),
        "signal_shape property MUST propagate through the dispatcher \
         path into WaitMetadata.signal_shape (Wave-8i convergence): \
         the resume path's shape-validation reads this slot"
    );
    assert_eq!(handle.signal_name(), "signal:typed");
}

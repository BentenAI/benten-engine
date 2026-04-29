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
use benten_eval::{PrimitiveKind, SuspensionStore, WaitOutcome, WaitResumeSignal};
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

// ---------------------------------------------------------------------------
// Wave-8i fix-pass regression tests (against code-as-graph mini-review)
// ---------------------------------------------------------------------------

/// Wave-8i fix-pass acceptance test (w8i-wait-cag-01): the caller-named
/// principal threads through `call_as_with_suspension` into the
/// suspension envelope's `resumption_principal_cid`, so
/// `resume_from_bytes_as` step 2 binds correctly for real WAIT
/// handlers (NOT just for `SubgraphSpec::empty(...)` fixtures).
///
/// **The bug.** Pre-fix-pass, the Wave-8i regular-walk path dropped the
/// caller's principal arg (`let _ = principal;` at engine_wait.rs:782)
/// and the eval-side `placeholder_payload_for_signal` set
/// `resumption_principal_cid = BLAKE3(signal_name)` regardless of who
/// the caller said they were. A subsequent
/// `resume_from_bytes_as(_, _, &alice_cid)` then compared `alice_cid`
/// against `BLAKE3("user:click")` and fired `E_RESUME_ACTOR_MISMATCH`
/// for any non-trivial principal — a silent semantic regression of the
/// Phase-2a principal-binding contract.
///
/// **What the existing tests missed.** Every existing principal-binding
/// test (e.g. `resume_with_substituted_principal_rejects`) uses
/// `minimal_wait_handler() = SubgraphSpec::empty(...)` which routes
/// through the preserved legacy `empty_spec_should_suspend` branch in
/// `call_as_with_suspension`, so the fix-pass-relevant path was
/// untested.
///
/// **What this test asserts.**
///  1. Alice suspends a real (non-empty-spec) WAIT-bearing handler.
///  2. Alice's own resume succeeds (principal binds correctly).
///  3. Eve's resume against Alice's bytes fires
///     `E_RESUME_ACTOR_MISMATCH`.
#[test]
fn wait_principal_binding_threads_through_real_handler() {
    let (_dir, engine) = open_engine();
    engine
        .register_subgraph(wait_signal_handler("wait:bind", "user:click"))
        .expect("register real wait handler");

    let alice = benten_engine::testing::principal_cid("alice");
    let bob = benten_engine::testing::principal_cid("bob");
    assert_ne!(alice, bob, "principals must hash distinct");

    // Alice suspends a real WAIT-bearing handler under her principal.
    // Pre-fix-pass: this routed through dispatch_call(_, _, _, None)
    // and the envelope's resumption_principal_cid was BLAKE3("user:click").
    // Post-fix-pass: dispatch_call(_, _, _, Some(alice)) -> active_call
    // stack -> Engine::suspending_principal() -> wait::evaluate_op
    // overrides resumption_principal_cid = alice.
    let outcome = engine
        .call_as_with_suspension("wait:bind", "run", Node::empty(), &alice)
        .expect("alice's call_as_with_suspension succeeds");
    let handle = match outcome {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => {
            panic!("real WAIT handler must Suspend, not Complete")
        }
    };

    let bytes = engine
        .suspend_to_bytes(&handle)
        .expect("serialise alice's suspension envelope");

    // Step-2 principal binding: alice's own resume succeeds. Pre-fix-pass
    // this would have failed with E_RESUME_ACTOR_MISMATCH because the
    // envelope carried BLAKE3("user:click") instead of alice's CID.
    let alice_resume = engine
        .resume_from_bytes_as(&bytes, Value::text("ok"), &alice)
        .expect("alice's own resume must succeed (envelope binds her CID)");
    assert!(
        alice_resume.is_ok_edge(),
        "alice's own resume must route OK; got {alice_resume:?}"
    );

    // Step-2 principal binding: bob's resume must be rejected. This
    // confirms the binding actually fires — the envelope is not
    // permissive against any caller, but specifically bound to alice.
    let err = engine
        .resume_from_bytes_as(&bytes, Value::text("attack"), &bob)
        .expect_err("bob must not resume alice's suspension");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::ResumeActorMismatch,
        "bob's resume must fire E_RESUME_ACTOR_MISMATCH (step 2 of the \
         4-step protocol); got {err:?}"
    );
}

/// Wave-8i fix-pass acceptance test (w8i-wait-cag-02): the production
/// engine's `PrimitiveHost::elapsed_ms` override stamps a real
/// monotonic-clock reading into `WaitMetadata.suspend_elapsed_ms`,
/// enabling the resume-time deadline check.
///
/// **The bug.** Pre-fix-pass, `impl PrimitiveHost for Engine` overrode
/// `suspension_store` but NOT `elapsed_ms`. The trait default returned
/// `None`, so the dispatcher passed `None` into `wait::evaluate_op` and
/// `WaitMetadata.suspend_elapsed_ms` was always `None` on the regular-
/// walk path. The eval-side `resume_with_meta`'s deadline check
/// `if let (Some(timeout), Some(start), Some(now)) = ...` then silently
/// never fired against a production engine — resume-time deadline
/// enforcement was disabled for any WAIT reached via `engine.call()`.
///
/// **What this test asserts.**
///  1. After suspending a `timeout_ms`-bearing WAIT through
///     `engine.call()`, `WaitMetadata.suspend_elapsed_ms` is `Some(_)`
///     (not `None`) — the engine's monotonic clock was consulted.
///  2. After a real elapsed delay, the eval-side `resume_with_meta`
///     deadline check (which now has a real start reference) fires
///     `E_WAIT_TIMEOUT` for a Signal resume that arrives past the
///     deadline. (The engine surface `Engine::resume_with_meta` does
///     not currently consult the eval-side deadline check at the time
///     of this fix-pass — that wiring is a separate gap. The eval-side
///     `benten_eval::resume(_, _, _, _)` IS the surface that uses the
///     metadata, and is what this assertion targets.)
#[test]
fn wait_resume_deadline_fires_against_real_clock() {
    let (_dir, engine) = open_engine();
    // duration_ms makes is_duration=true; timeout_ms < duration_ms so the
    // resume-time deadline check fires before any duration-elapsed path
    // would. Both properties propagate to WaitMetadata via Wave-8i.
    let mut props = BTreeMap::new();
    props.insert("duration_ms".into(), Value::Int(100));
    props.insert("timeout_ms".into(), Value::Int(50));
    let spec = SubgraphSpec::builder()
        .handler_id("wait:deadline")
        .primitive_with_props(PrimitiveSpec {
            id: "w0".into(),
            kind: PrimitiveKind::Wait,
            properties: props,
        })
        .build();
    engine
        .register_subgraph(spec)
        .expect("register deadline wait handler");

    let handle = match engine.call("wait:deadline", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => handle,
        other => panic!("expected WaitSuspended, got {other:?}"),
    };

    // (1) The engine's monotonic clock reading was stamped — fix-pass
    //     w8i-wait-cag-02 acceptance gate.
    let store = engine.suspension_store();
    let meta = store
        .get_wait(handle.state_cid())
        .expect("store get_wait")
        .expect("metadata recorded");
    assert!(
        meta.suspend_elapsed_ms.is_some(),
        "Engine::elapsed_ms() override MUST stamp suspend_elapsed_ms; \
         pre-fix-pass this was None and the deadline check silently \
         never fired"
    );
    assert_eq!(
        meta.timeout_ms,
        Some(50),
        "timeout_ms property must propagate into WaitMetadata"
    );
    assert!(
        meta.is_duration,
        "duration_ms-bearing WAIT must record is_duration=true"
    );

    // (2) Drive the eval-side resume_with_meta path through the public
    //     `benten_eval::resume(...)` alias with a `MockTimeSource`
    //     positioned PAST the engine-stamped suspend time + timeout.
    //     The deadline check at wait.rs:453 fires E_WAIT_TIMEOUT when
    //     (now - start) >= timeout. We construct the resume-time ctx
    //     clock at `start + 1000ms`, well past the 50ms deadline.
    //
    //     This is the load-bearing acceptance gate for the
    //     `elapsed_ms()` Engine override: the metadata's start
    //     reference is real (sourced from the engine's monotonic
    //     clock), so a resume-time clock advanced past `start + timeout`
    //     fires the deadline. Pre-fix-pass, `start = None` made the
    //     deadline check silently skip regardless of resume-time clock.
    let start_ms = meta.suspend_elapsed_ms.expect("start stamped");
    let resume_time_ms = start_ms.saturating_add(1000);
    let clock = benten_eval::MockTimeSource::at(std::time::Duration::from_millis(resume_time_ms));
    let mut ctx = benten_eval::EvalContext::with_clock(clock);
    ctx = ctx.with_suspension_store(engine.suspension_store());

    let outcome = WaitOutcome::Suspended(handle);
    let signal = WaitResumeSignal::signal("user:click", Value::Int(0));
    let result = benten_eval::resume(&dummy_subgraph_for_resume(), &mut ctx, outcome, signal);
    use benten_eval::Outcome as EvalOutcome;
    match result {
        EvalOutcome::Err(e) => assert_eq!(
            e.code(),
            benten_errors::ErrorCode::WaitTimeout,
            "post-deadline resume must fire E_WAIT_TIMEOUT (Wave-8i \
             elapsed_ms() override stamps a real start reference; \
             pre-fix-pass start=None made the deadline check silently \
             skip); got {e:?}"
        ),
        other => panic!(
            "expected E_WAIT_TIMEOUT, got {other:?} (suggests the deadline \
             check did not fire — verify suspend_elapsed_ms threading)"
        ),
    }
}

/// Helper: a minimal subgraph used only to satisfy the `&Subgraph`
/// signature of `benten_eval::resume`. The resume function currently
/// ignores the `_sg` arg (it routes by handle.state_cid + suspension
/// store), but the public signature requires one.
fn dummy_subgraph_for_resume() -> benten_eval::Subgraph {
    use benten_eval::{SubgraphBuilder, SubgraphBuilderExt, SubgraphExt};
    let mut sb = SubgraphBuilder::new("wait:deadline:resume_dummy");
    let r = sb.read("ignored");
    sb.respond(r);
    sb.build_unvalidated_for_test()
}

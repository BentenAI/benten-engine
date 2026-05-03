//! R6-R5 r6-r5-pcds-2 (23rd producer/consumer drift) — Rust-side §3.6b
//! end-to-end pin verifying the post-fix DSL spread shape routes
//! correctly through the production runtime.
//!
//! ## Why this test exists
//!
//! The R6-R5 producer/consumer deep-sweep at HEAD `a73aeee` found that
//! the TypeScript DSL `subgraph(...).wait({ duration: "5m" })` spread
//! wrote the OperationNode property bag as `{ duration: Text("5m") }`,
//! while `crates/benten-eval/src/primitives/wait.rs::evaluate_op_with_handler_id`
//! reads `properties.get("duration_ms")` (Int). NO translation layer
//! existed at the napi boundary (`bindings/napi/src/subgraph.rs::json_to_props`
//! only converts JSON-int → `Value::Int` verbatim — no key rename / no
//! string-duration parser). The duration-variant WAIT therefore
//! suspended without a deadline + never auto-resumed.
//!
//! The TS-side fix (R6-R5 FP) translates at the DSL spread:
//! `dsl.ts::translateWaitArgs` parses `duration: "5m"` into
//! `duration_ms: 300_000` (Int). After napi round-trip through
//! `json_to_props`, the eval-side reader sees the canonical key shape
//! and stamps `WaitMetadata { is_duration: true, timeout_ms: Some(300_000), ... }`.
//!
//! ## Pin shape
//!
//! This Rust test mirrors the EXACT post-DSL-spread JSON the TS DSL
//! emits — `{ duration_ms: <Int> }` (NOT `{ duration: <Text> }`) — and
//! drives the production `engine.call(handler, ...)` entry point. It
//! asserts:
//!   1. The runtime routes through the typed-error WAIT path.
//!   2. The suspension store records `is_duration = true` (proves the
//!      `duration_ms` property reached `evaluate_op_with_handler_id`).
//!   3. The suspension store records `timeout_ms = Some(300_000)` (the
//!      deadline reference the resume-time `WaitResumeSignal::DurationElapsed`
//!      branch consumes).
//!
//! Combined with the TS-side spread tests at
//! `packages/engine/test/wait_duration_dsl_translation.test.ts` (which
//! assert the spread emits `duration_ms` instead of `duration`), this
//! test closes the §3.6b end-to-end gap that hid the 23rd
//! producer/consumer drift through 5 prior deep-sweeps.
//!
//! Reference: `.addl/phase-2b/r6-r5-producer-consumer-deep-sweep.json`
//! finding `r6-r5-pcds-2`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, EngineError, PrimitiveSpec, SubgraphSpec};
use benten_eval::{PrimitiveKind, SuspensionStore};
use std::collections::BTreeMap;

fn open_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().expect("tempdir");
    let engine = Engine::open(dir.path().join("engine.redb")).expect("open");
    (dir, engine)
}

/// Builds a SubgraphSpec with the EXACT property shape the post-R6-R5-FP
/// TS DSL spread emits for `subgraph(...).wait({ duration: "5m" })` —
/// `duration_ms: Value::Int(300_000)`. Pre-fix the DSL spread wrote
/// `duration: Value::Text("5m")` instead, which this test would
/// detect via the `is_duration: false` assertion failure if the spread
/// regressed.
fn dsl_post_spread_duration_handler(handler_id: &str, duration_ms: i64) -> SubgraphSpec {
    let mut props = BTreeMap::new();
    // Mirror EXACTLY the post-fix DSL spread shape:
    // `dsl.ts::translateWaitArgs` writes `{ duration_ms: <int> }` for
    // the bare-duration form (NOT `duration: <string>`).
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

/// §3.6b LOAD-BEARING end-to-end pin: the DSL spread emits the shape
/// the eval-side reader needs.
///
/// Drives the production `engine.call(handler, ...)` entry point with
/// a SubgraphSpec mirroring the post-R6-R5-FP DSL spread output.
/// Asserts the suspension store records `is_duration: true` +
/// `timeout_ms: Some(300_000)` — the runtime invariants the
/// resume-time `WaitResumeSignal::DurationElapsed` branch consumes.
///
/// Pre-R6-R5-FP regression detection: if the DSL spread regressed
/// back to writing `{ duration: Text(...) }`, this test (combined
/// with the TS-side `wait_duration_dsl_translation.test.ts`
/// spread-shape pins) would FAIL — the TS test's
/// `expect(args.duration_ms).toBe(300_000)` assertion would FAIL
/// directly, AND a Rust-side equivalent test using the regressed
/// shape would observe `is_duration: false` here.
#[test]
fn dsl_post_spread_duration_routes_through_engine_call_with_is_duration_true() {
    let (_dir, engine) = open_engine();
    engine
        .register_subgraph(dsl_post_spread_duration_handler(
            "wait:dsl-dur-pin",
            300_000,
        ))
        .expect("register dsl-shape duration wait handler");

    let handle = match engine.call("wait:dsl-dur-pin", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => handle,
        other => panic!(
            "expected EngineError::WaitSuspended for DSL-shape duration WAIT (post-R6-R5-FP \
             spread shape `duration_ms: Int`), got {other:?}"
        ),
    };

    let store = engine.suspension_store();
    let meta = store
        .get_wait(handle.state_cid())
        .expect("suspension store get_wait")
        .expect("metadata recorded for the suspension envelope");

    // Load-bearing assertion 1: `duration_ms` propagated into
    // `WaitMetadata.is_duration` (pre-R6-R5-FP this would have been
    // `false` because the spread wrote `duration: Text` and the eval
    // reader's `properties.get("duration_ms")` returned None).
    assert!(
        meta.is_duration,
        "DSL-spread `duration_ms: Int` MUST stamp `WaitMetadata.is_duration = true` \
         (R6-R5-FP r6-r5-pcds-2 end-to-end pin) — observation `is_duration = false` \
         signals the DSL spread regressed back to writing `duration: Text(<string>)` \
         which the eval reader ignores"
    );

    // Load-bearing assertion 2: the deadline reference the
    // resume-time `WaitResumeSignal::DurationElapsed` branch consumes
    // is set to the parsed millisecond value.
    assert_eq!(
        meta.timeout_ms,
        Some(300_000),
        "DSL-spread `duration_ms: 300_000` MUST propagate into \
         `WaitMetadata.timeout_ms` so the resume-time deadline check fires"
    );
}

/// Companion pin for the signal-with-deadline form: the DSL spread
/// emits `{ signal: <Text>, timeout_ms: <Int> }` (NOT
/// `{ signal, duration: Text }`) for `wait({ signal, duration: "1h" })`.
/// Verifies the napi → eval round-trip stamps `WaitMetadata.timeout_ms`
/// from the `timeout_ms` property (the signal-variant deadline
/// reference).
#[test]
fn dsl_post_spread_signal_with_timeout_routes_with_timeout_ms_set() {
    let (_dir, engine) = open_engine();

    let mut props = BTreeMap::new();
    // Mirror EXACTLY the post-fix DSL spread shape for
    // `wait({ signal: "external:ack", duration: "1h" })`:
    // `dsl.ts::translateWaitArgs` writes
    // `{ signal: Text(...), timeout_ms: Int(3_600_000) }`.
    props.insert("signal".into(), Value::text("external:ack"));
    props.insert("timeout_ms".into(), Value::Int(3_600_000));
    let spec = SubgraphSpec::builder()
        .handler_id("wait:dsl-sig-deadline-pin")
        .primitive_with_props(PrimitiveSpec {
            id: "w0".into(),
            kind: PrimitiveKind::Wait,
            properties: props,
        })
        .build();
    engine
        .register_subgraph(spec)
        .expect("register signal-with-deadline wait handler");

    let handle = match engine.call("wait:dsl-sig-deadline-pin", "run", Node::empty()) {
        Err(EngineError::WaitSuspended { handle }) => handle,
        other => panic!("expected EngineError::WaitSuspended, got {other:?}"),
    };

    let store = engine.suspension_store();
    let meta = store
        .get_wait(handle.state_cid())
        .expect("get_wait")
        .expect("metadata recorded");

    // Signal-variant: `is_duration` must remain `false` (the suspend
    // is keyed on the signal name, not a deadline-only timer).
    assert!(
        !meta.is_duration,
        "signal-variant WAIT must record is_duration=false; the deadline is a fallback \
         on the signal-style suspend, not a duration-keyed envelope"
    );
    // The `timeout_ms` property MUST propagate so the resume-time
    // deadline check can fire `E_WAIT_TIMEOUT` if the signal doesn't
    // arrive in time.
    assert_eq!(
        meta.timeout_ms,
        Some(3_600_000),
        "DSL-spread `timeout_ms: 3_600_000` (from `wait({{ signal, duration: \"1h\" }})` \
         DSL form) MUST propagate into `WaitMetadata.timeout_ms`"
    );
    assert_eq!(handle.signal_name(), "external:ack");
}

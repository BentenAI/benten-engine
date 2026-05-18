//! G19-C2 wave-7 (§7.1.5 + stream-r1-3 + stream-r1-9) — STREAM ESC
//! defenses per-handler config tests.
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C2 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C2 must-pass column):
//!
//! - `tests/stream_chunk_count_per_handler_override` — §7.1.5
//! - `tests/stream_wallclock_budget_per_handler_override` — §7.1.5
//! - `tests/stream_per_handler_esc_defense_fires_observable_consequence_via_eval_side_stream_execute` — stream-r1-3
//! - `tests/stream_per_handler_config_exceeds_grant_ceiling_fires_e_inv_stream_config` — stream-r1-9
//! - `tests/stream_per_handler_config_consumed_by_build_stream_handle_not_dead_execute_arm` — stream-r1-3
//!
//! ## What G19-C2 establishes (§7.1.5 + stream-r1-3 + stream-r1-9)
//!
//! Per-handler `chunkCountCap` + `wallclockBudgetMs` properties on the
//! STREAM `PrimitiveSpec` get consumed by
//! `crates/benten-engine/src/engine_stream.rs::build_stream_handle`
//! (the production runtime path).
//!
//! Per stream-r1-9: per-handler config NARROWS but cannot WIDEN the
//! workspace default grant ceilings
//! (`STREAM_GRANT_CEILING_CHUNK_COUNT` / `STREAM_GRANT_CEILING_WALLCLOCK_MS`)
//! — fail-loud on widen-attempt with `E_INV_STREAM_CONFIG`.
//!
//! Per stream-r1-3: tests pin the production-grade entry point
//! (`engine.call_stream`) NOT the dead `stream::execute` arm — defends
//! against the AST-cache (G19-E) wrapper accidentally resurrecting the
//! deceptive-sentinel pattern. The dead-arm preservation is verified
//! end-to-end in `stream_per_handler_config_consumed_by_build_stream_handle_not_dead_execute_arm`.

#![cfg(not(target_arch = "wasm32"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_errors::ErrorCode;
use benten_eval::PrimitiveKind;

/// Build a STREAM SubgraphSpec carrying per-handler config props.
///
/// `chunk_count_cap` / `wallclock_budget_ms` are optional — `None`
/// means "do not set the property" so the engine resolves to the
/// workspace defaults.
fn stream_spec_with_per_handler_config(
    handler_id: &str,
    source_value: Value,
    chunk_count_cap: Option<i64>,
    wallclock_budget_ms: Option<i64>,
) -> SubgraphSpec {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    // The stream resolver expects `source` to be a single-token expr;
    // we drive chunk-count via `$input.upTo` so the test can control
    // exact chunk count via `Engine::call_stream`'s input Node.
    props.insert("source".into(), Value::Text("$input.upTo".to_string()));
    props.insert("chunkSize".into(), Value::Int(1));
    if let Some(cap) = chunk_count_cap {
        props.insert("chunkCountCap".into(), Value::Int(cap));
    }
    if let Some(ms) = wallclock_budget_ms {
        props.insert("wallclockBudgetMs".into(), Value::Int(ms));
    }
    // Keep the source value unused on the spec — it's an
    // implementation hint for documentation. The actual driver
    // value comes from the input Node at call time.
    let _ = source_value;
    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "stream0".into(),
            kind: PrimitiveKind::Stream,
            properties: props,
        })
        .respond()
        .build()
}

fn open_engine() -> Engine {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("benten.redb");
    // Leak the tempdir so the redb file outlives this fn while still
    // staying scoped to the test process (tempdir cleans up on
    // process exit). Tempdir cleanup at test scope-end is incompatible
    // with returning the Engine — we'd have a use-after-cleanup race.
    std::mem::forget(dir);
    Engine::open(path).unwrap()
}

fn input_with_up_to(n: i64) -> Node {
    let mut props = BTreeMap::new();
    props.insert("upTo".to_string(), Value::Int(n));
    Node::new(vec!["test_input".to_string()], props)
}

/// G19-C2 (§7.1.5) — per-handler chunk_count_cap NARROWS the producer's
/// chunk emission via `ChunkProducerConfig::max_chunks` enforced by
/// `BoundedSink`.
#[test]
fn stream_chunk_count_per_handler_override() {
    let engine = open_engine();
    // Cap chunks at 5 (well below ceiling); request 10 chunks via
    // `upTo: 10`. Producer must emit at most 5 chunks before
    // `BoundedSink` kills it via the chunk-count budget.
    let spec = stream_spec_with_per_handler_config(
        "stream.chunk_count_per_handler",
        Value::Int(10),
        Some(5),
        None,
    );
    let handler_id = engine.register_subgraph(spec).unwrap();

    let mut handle = engine
        .call_stream(&handler_id, "main", input_with_up_to(10))
        .expect("call_stream should succeed when per-handler config narrows ceiling");

    let mut received = 0usize;
    loop {
        match handle.next_chunk() {
            Ok(Some(_)) => received += 1,
            Ok(None) => break,
            // BoundedSink terminates with ChunkCountExceeded once the
            // cap is hit — surfaces here through next_chunk's
            // EngineError translation.
            Err(_) => break,
        }
        assert!(
            received <= 50,
            "per-handler chunk_count_cap must enforce a bound; saw {received} chunks"
        );
    }
    assert!(
        received <= 5,
        "per-handler chunk_count_cap=5 must enforce the cap (got {received})"
    );
}

/// G19-C2 (§7.1.5) — per-handler wallclock_budget_ms NARROWS the
/// producer-thread wallclock budget. Smoke test: registration + a
/// successful call_stream return; the actual fall-through to
/// `wallclock_budget` enforcement is exercised by the BoundedSink
/// unit tests in `crates/benten-eval/tests/stream_producer_wallclock.rs`.
#[test]
fn stream_wallclock_budget_per_handler_override() {
    let engine = open_engine();
    let spec = stream_spec_with_per_handler_config(
        "stream.wallclock_budget_per_handler",
        Value::Int(3),
        None,
        Some(5_000), // 5s — well below the 30s ceiling
    );
    let handler_id = engine.register_subgraph(spec).unwrap();

    let mut handle = engine
        .call_stream(&handler_id, "main", input_with_up_to(3))
        .expect("call_stream with per-handler wallclock_budget_ms must succeed");
    let mut received = 0usize;
    while let Ok(Some(_)) = handle.next_chunk() {
        received += 1;
        if received > 10 {
            break;
        }
    }
    // Observable consequence: the call returned a usable handle the
    // caller drained without the wallclock budget firing
    // (the producer naturally completes well within 5s).
    assert!(
        received <= 3,
        "expected <=3 chunks for upTo=3; got {received}"
    );
}

/// G19-C2 (stream-r1-3) — per-handler ESC defense fires through the
/// production runtime path (`build_stream_handle`), not the dead
/// `stream::execute` arm.
///
/// The pin asserts that overage of the per-handler chunk_count_cap is
/// observable through the production entry point: the consumer drains
/// at most `cap` chunks (bounded by the producer-side BoundedSink) even
/// when the source would emit far more.
#[test]
fn stream_per_handler_esc_defense_fires_observable_consequence_via_eval_side_stream_execute() {
    let engine = open_engine();
    let spec = stream_spec_with_per_handler_config(
        "stream.esc_defense_fires_observably",
        Value::Int(100),
        Some(7),
        None,
    );
    let handler_id = engine.register_subgraph(spec).unwrap();

    let mut handle = engine
        .call_stream(&handler_id, "main", input_with_up_to(100))
        .expect("ESC defense path must yield a stream handle");
    let mut received = 0usize;
    loop {
        match handle.next_chunk() {
            Ok(Some(_)) => received += 1,
            _ => break,
        }
        if received > 50 {
            break;
        }
    }
    assert!(
        received <= 7,
        "ESC defense (chunk_count_cap=7) must fire on production path; got {received} chunks"
    );
}

/// G19-C2 (stream-r1-9) — per-handler config widening the workspace
/// grant ceiling fires `E_INV_STREAM_CONFIG`.
#[test]
fn stream_per_handler_config_exceeds_grant_ceiling_fires_e_inv_stream_config() {
    let engine = open_engine();

    // Try to widen chunk_count_cap above the workspace ceiling:
    let widen_chunks = i64::try_from(benten_engine::STREAM_GRANT_CEILING_CHUNK_COUNT)
        .unwrap()
        .saturating_add(1);
    let spec = stream_spec_with_per_handler_config(
        "stream.widen_attempt_chunks",
        Value::Int(1),
        Some(widen_chunks),
        None,
    );
    let handler_id = engine.register_subgraph(spec).unwrap();
    let err = engine
        .call_stream(&handler_id, "main", input_with_up_to(1))
        .expect_err("widening attempt must fire E_INV_STREAM_CONFIG");
    let code = err.error_code();
    assert_eq!(
        code,
        ErrorCode::InvStreamConfig,
        "chunk_count_cap widen-attempt must surface E_INV_STREAM_CONFIG (got {code:?})",
    );

    // Likewise for wallclock_budget_ms:
    let widen_ms = i64::try_from(benten_engine::STREAM_GRANT_CEILING_WALLCLOCK_MS)
        .unwrap()
        .saturating_add(1);
    let spec = stream_spec_with_per_handler_config(
        "stream.widen_attempt_wallclock",
        Value::Int(1),
        None,
        Some(widen_ms),
    );
    let handler_id2 = engine.register_subgraph(spec).unwrap();
    let err2 = engine
        .call_stream(&handler_id2, "main", input_with_up_to(1))
        .expect_err("wallclock_budget_ms widen attempt must fire E_INV_STREAM_CONFIG");
    assert_eq!(
        err2.error_code(),
        ErrorCode::InvStreamConfig,
        "wallclock_budget_ms widen-attempt must surface E_INV_STREAM_CONFIG"
    );
}

/// G19-C2 (stream-r1-3) — per-handler config flows through
/// `build_stream_handle` (production), NOT the dead `stream::execute`
/// arm. The dead arm stays loud-fail with `E_PRIMITIVE_NOT_IMPLEMENTED`
/// per R6FP-G1 r6-stream-3.
///
/// HALF 1: production path (`Engine::call_stream`) honors the per-handler
/// chunk_count_cap end-to-end.
///
/// HALF 2: dead `benten_eval::primitives::stream::execute` arm still
/// fires `E_PRIMITIVE_NOT_IMPLEMENTED` (deceptive-sentinel-removed
/// discipline preserved).
#[test]
fn stream_per_handler_config_consumed_by_build_stream_handle_not_dead_execute_arm() {
    // ---- HALF 1: production path observes per-handler config ----
    let engine = open_engine();
    let spec = stream_spec_with_per_handler_config(
        "stream.config_consumed_by_build_stream_handle",
        Value::Int(10),
        Some(3),
        None,
    );
    let handler_id = engine.register_subgraph(spec).unwrap();
    let mut handle = engine
        .call_stream(&handler_id, "main", input_with_up_to(10))
        .expect("production path must accept the spec");
    let mut received = 0usize;
    loop {
        match handle.next_chunk() {
            Ok(Some(_)) => received += 1,
            _ => break,
        }
        if received > 50 {
            break;
        }
    }
    assert!(
        received <= 3,
        "HALF 1: production path must enforce per-handler chunk_count_cap=3 (got {received})"
    );

    // ---- HALF 2: dead `stream::execute` arm preserves loud-fail ----
    use benten_eval::{EvalError, NullHost, StepResult};

    use benten_core::OperationNode;
    let op = OperationNode::new("dead_arm_test", PrimitiveKind::Stream);
    let host = NullHost;
    let dead_arm: Result<StepResult, EvalError> =
        benten_eval::primitives::stream::execute(&op, &host);
    let err = dead_arm.expect_err(
        "dead arm must remain loud-fail E_PRIMITIVE_NOT_IMPLEMENTED \
         per R6FP-G1 r6-stream-3 deceptive-sentinel-removed discipline; \
         a successful Ok return would prove the dead-arm was resurrected",
    );
    assert!(
        matches!(err, EvalError::PrimitiveNotImplemented(_)),
        "dead arm error must be PrimitiveNotImplemented (got {err:?})"
    );
}

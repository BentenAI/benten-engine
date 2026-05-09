//! G19-C2 wave-7 (§7.1) — SANDBOX execution-metrics propagation tests.
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C2 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C2 must-pass column):
//!
//! - `tests/sandbox_node_metrics_high_water_tracker_round_trip` — §7.1
//! - `tests/describe_sandbox_node_returns_real_fuel_consumed_not_unknown` — §7.1
//! - `tests/sandbox_metrics_propagation_through_cross_process_resume_via_envelope` — stream-r1-8
//!
//! ## What G19-C2 establishes (§7.1)
//!
//! `crates/benten-engine/src/primitive_host.rs::execute_sandbox` records
//! per-handler `fuel_consumed` / `output_consumed` / `last_invocation_ms`
//! observations into `EngineInner::sandbox_metrics`.
//! `engine_sandbox.rs::describe_sandbox_node_for_handler` reads the
//! tracker + returns real metric values.
//!
//! Per stream-r1-8: high-water values are PER-INVOCATION updates against
//! the high-water mark within a single Engine instance; the cross-process
//! WAIT-resume envelope does NOT carry in-flight SANDBOX metrics across
//! the suspend boundary.
//!
//! Pin pattern follows `tests/sandbox_execute_via_engine_dispatch_invokes_executor.rs`
//! (real Engine + `register_module_bytes` + dispatch through `Engine::call`).

#![cfg(not(target_arch = "wasm32"))]
#![cfg(any(test, feature = "test-helpers"))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeMap;

use benten_core::{Cid, Value};
use benten_engine::{Engine, PrimitiveSpec, SubgraphSpec};
use benten_eval::PrimitiveKind;

/// Build a 2-node SANDBOX subgraph (SANDBOX -> RESPOND) the way
/// `sandbox_execute_via_engine_dispatch_invokes_executor` does.
fn sandbox_spec(handler_id: &str, module_cid_str: &str) -> SubgraphSpec {
    let mut sandbox_props: BTreeMap<String, Value> = BTreeMap::new();
    sandbox_props.insert("module".into(), Value::Text(module_cid_str.to_string()));
    sandbox_props.insert(
        "caps".into(),
        Value::List(vec![Value::Text("host:compute:time".to_string())]),
    );

    SubgraphSpec::builder()
        .handler_id(handler_id)
        .primitive_with_props(PrimitiveSpec {
            id: "s0".into(),
            kind: PrimitiveKind::Sandbox,
            properties: sandbox_props,
        })
        .respond()
        .build()
}

fn trivial_run_module_bytes() -> Vec<u8> {
    wat::parse_str("(module (func (export \"run\") (result i32) i32.const 42))")
        .expect("trivial run module compiles")
}

fn cid_for_bytes(bytes: &[u8]) -> Cid {
    let digest = *blake3::hash(bytes).as_bytes();
    Cid::from_blake3_digest(digest)
}

/// G19-C2 (§7.1) — per-handler high-water tracker round-trip.
///
/// First invocation recorded; describe_sandbox_node_for_handler returns
/// real (numeric, non-zero) metrics. Second invocation does not regress
/// the high-water + records a new last_invocation_ms.
#[test]
fn sandbox_node_metrics_high_water_tracker_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    let spec = sandbox_spec("sandbox.metrics_round_trip", &module_cid_str);
    let handler_id = engine.register_subgraph(spec).unwrap();

    // Pre-first-invocation: no metrics record yet.
    assert!(
        engine
            .describe_sandbox_node_for_handler(&handler_id)
            .is_err(),
        "pre-first-invocation describe_sandbox_node_for_handler must \
         return E_SANDBOX_NODE_UNKNOWN until at least one call lands"
    );

    // First invocation.
    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect("first SANDBOX dispatch must succeed");
    assert!(outcome.is_ok_edge(), "first call must route OK edge");

    let metrics_a = engine
        .describe_sandbox_node_for_handler(&handler_id)
        .expect("post-first-invocation describe_sandbox_node_for_handler must return Ok");
    assert!(
        metrics_a.fuel_consumed_high_water.unwrap_or(0) > 0,
        "fuel_consumed_high_water must be a real measured value, not Unknown / zero placeholder"
    );
    let high_water_after_first = metrics_a.fuel_consumed_high_water.unwrap();

    // Second invocation.
    let outcome2 = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect("second SANDBOX dispatch must succeed");
    assert!(outcome2.is_ok_edge());

    let metrics_b = engine
        .describe_sandbox_node_for_handler(&handler_id)
        .unwrap();
    // High-water tracker MUST NOT regress across invocations.
    assert!(
        metrics_b.fuel_consumed_high_water.unwrap() >= high_water_after_first,
        "high-water tracker must not regress (first={}, second={:?})",
        high_water_after_first,
        metrics_b.fuel_consumed_high_water,
    );
    // last_invocation_ms is per-invocation (NOT cumulative). Just
    // assert it's Some after a call returns.
    assert!(
        metrics_b.last_invocation_ms.is_some(),
        "last_invocation_ms must be Some after a successful invocation"
    );
}

/// G19-C2 (§7.1) — describe_sandbox_node returns real fuel_consumed
/// (NOT the legacy placeholder).
///
/// End-to-end pin per pim-2 §3.6b: drives the production
/// `Engine::call` entry point, then asserts the diagnostic accessor
/// returns real numeric metric values rather than the prior
/// "Unknown" sentinel placeholder.
#[test]
fn describe_sandbox_node_returns_real_fuel_consumed_not_unknown() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    let spec = sandbox_spec("sandbox.real_fuel_consumed_not_unknown", &module_cid_str);
    let handler_id = engine.register_subgraph(spec).unwrap();

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect("dispatch must succeed");
    assert!(outcome.is_ok_edge());

    let descriptor = engine
        .describe_sandbox_node_for_handler(&handler_id)
        .expect("describe_sandbox_node_for_handler returns Ok after a call lands");

    // The Phase-2b state returned `fuelConsumedHighWater: "unknown"`
    // sentinel. G19-C2 returns a real numeric value.
    assert!(
        descriptor.fuel_consumed_high_water.is_some(),
        "fuel_consumed_high_water must be Some (real measured value), not the Unknown sentinel"
    );
    assert!(
        descriptor.fuel_consumed_high_water.unwrap() > 0,
        "fuel_consumed_high_water must be > 0 after a real SANDBOX invocation"
    );
    // Resolved-defaults remain at their D24 defaults absent per-node
    // overrides.
    assert_eq!(descriptor.fuel, 1_000_000, "default fuel = 1M per D24");
    assert_eq!(
        descriptor.wallclock_ms, 30_000,
        "default wallclock = 30s per D24"
    );
    assert_eq!(
        descriptor.output_limit_bytes, 1_048_576,
        "default output_limit = 1MB per D15 trap-loudly"
    );
}

/// G19-C2 (stream-r1-8) — SANDBOX metrics are PER-INVOCATION (NOT
/// cross-resume cumulative).
///
/// Documents the chosen semantic: a fresh Engine instance has an empty
/// sandbox_metrics map; in-flight values do NOT cross the suspend
/// boundary. The cross-process WAIT-resume infrastructure is not yet
/// landed in the engine; this test is a SHAPE pin asserting the
/// per-invocation semantic at the single-process level + documents the
/// cross-process semantic as deliberately empty.
#[test]
fn sandbox_metrics_propagation_through_cross_process_resume_via_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();

    // First "process" — populate metrics.
    let handler_id_str;
    {
        let engine_a = Engine::open(dir.path().join("benten.redb")).unwrap();
        engine_a
            .register_module_bytes(&module_cid, &module_bytes)
            .unwrap();
        let spec = sandbox_spec("sandbox.cross_process_resume_pin", &module_cid_str);
        handler_id_str = engine_a.register_subgraph(spec).unwrap();
        let outcome = engine_a
            .call(
                &handler_id_str,
                "run",
                benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
            )
            .expect("engine_a dispatch ok");
        assert!(outcome.is_ok_edge());
        let metrics_a = engine_a
            .describe_sandbox_node_for_handler(&handler_id_str)
            .unwrap();
        assert!(metrics_a.fuel_consumed_high_water.unwrap_or(0) > 0);
        // engine_a drops at end of scope.
    }

    // Second "process" — fresh Engine sharing the same redb file. The
    // sandbox_metrics map is in-RAM ONLY (not persisted to redb), so a
    // freshly-opened engine MUST start with an empty metrics record
    // for this handler — confirming the per-invocation semantic
    // (stream-r1-8) at the cross-process boundary.
    let engine_b = Engine::open(dir.path().join("benten.redb")).unwrap();
    let pre_resume_lookup = engine_b.describe_sandbox_node_for_handler(&handler_id_str);
    assert!(
        pre_resume_lookup.is_err(),
        "second-process Engine MUST have empty sandbox_metrics (stream-r1-8: \
         high-water values are PER-INVOCATION, NOT cumulative across resumes); \
         a non-empty record would prove the metrics map persisted across \
         the suspend boundary, which is a bug"
    );
    // The shape-level pin: the documented semantic is implemented.
}

/// R6 fp Wave C2 (closes obs-r6r1-1 MAJOR — 25th p/c drift instance):
/// would-FAIL-if-no-op'd end-to-end pin per pim-2 §3.6b. Asserts the
/// Phase-3 §7.1 trio (fuel + output + wallclock) all reach the
/// `SandboxNodeDescription` consumer surface after a real SANDBOX
/// invocation through the production-runtime arm. Pre-Wave-C2 the
/// `output_consumed_high_water` field was recorded at
/// `engine.rs::record_sandbox_metric` but DROPPED at
/// `describe_sandbox_node_for_handler` — only fuel + wallclock reached
/// the napi/TS surface. This test would fail (sentinel `None` instead
/// of `Some(>0)`) if the threading regression was reintroduced.
#[test]
fn describe_sandbox_node_returns_real_output_consumed_high_water_phase_3_7_1_trio() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let module_bytes = trivial_run_module_bytes();
    let module_cid = cid_for_bytes(&module_bytes);
    let module_cid_str = module_cid.to_base32();
    engine
        .register_module_bytes(&module_cid, &module_bytes)
        .unwrap();

    let spec = sandbox_spec("sandbox.output_high_water_phase_3_trio", &module_cid_str);
    let handler_id = engine.register_subgraph(spec).unwrap();

    let outcome = engine
        .call(
            &handler_id,
            "run",
            benten_core::Node::new(vec!["test_input".to_string()], Default::default()),
        )
        .expect("dispatch must succeed");
    assert!(outcome.is_ok_edge());

    let descriptor = engine
        .describe_sandbox_node_for_handler(&handler_id)
        .expect("describe_sandbox_node_for_handler returns Ok after a call lands");

    // (1/3) fuel high-water reaches consumer surface.
    assert!(
        descriptor.fuel_consumed_high_water.is_some(),
        "fuel_consumed_high_water must be Some — pre-Wave-C2 covered case (sub-§7.1)"
    );
    // (2/3) output high-water reaches consumer surface — this is the
    // Wave-C2 closure point. Pre-Wave-C2 this would be `None` even
    // though the metric is recorded upstream.
    assert!(
        descriptor.output_consumed_high_water.is_some(),
        "output_consumed_high_water MUST be Some — Phase-3 §7.1 trio closure \
         (R6 fp Wave C2 / obs-r6r1-1). The trivial wat module returns i32 = 42 \
         (4 bytes); the high-water tracker MUST observe >= 0 for any successful \
         invocation. None here means the field is being dropped at \
         describe_sandbox_node_for_handler — the 25th p/c drift instance \
         regression."
    );
    // (3/3) last_invocation_ms reaches consumer surface.
    assert!(
        descriptor.last_invocation_ms.is_some(),
        "last_invocation_ms must be Some after a successful invocation"
    );
}

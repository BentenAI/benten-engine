//! R3-E RED-PHASE pins for G19-C2 SANDBOX execution-metrics propagation
//! (wave 7 parallel; §7.1).
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
//! `crates/benten-engine/src/engine_sandbox.rs` threads `fuel_consumed`,
//! `output_consumed`, and `last_invocation_ms` through engine wrapper into
//! a per-node high-water tracker. `bindings/napi/src/sandbox.rs::describeSandboxNode`
//! returns real metric values rather than the placeholder "Unknown" sentinel.
//!
//! Per stream-r1-8: high-water metrics are PER-INVOCATION, NOT
//! cross-resume cumulative; the suspension envelope does NOT carry
//! in-flight SANDBOX metrics across the suspend/resume boundary.
//!
//! ## RED-PHASE discipline
//!
//! Metrics propagation does not yet exist. R5 implementer wires it.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 threads fuel/output/last_invocation_ms into per-node high-water tracker"]
fn sandbox_node_metrics_high_water_tracker_round_trip() {
    // §7.1 pin. G19-C2 implementer wires this:
    //
    //   let engine = Engine::open_in_memory().unwrap();
    //   let module_cid = engine.install_module(/* sandbox manifest */).unwrap();
    //   let sg = engine.register_subgraph_with_sandbox_handler(module_cid).unwrap();
    //
    //   // First invocation:
    //   engine.call(sg, "main", json!({"input": 1})).unwrap();
    //
    //   // describe_sandbox_node returns real metrics (NOT "Unknown"):
    //   let metrics = engine.describe_sandbox_node(sg).unwrap();
    //   assert!(metrics.fuel_consumed > 0,
    //       "fuel_consumed must be the real measured value, not Unknown");
    //   assert!(metrics.last_invocation_ms > 0,
    //       "last_invocation_ms must reflect the wall-clock of the last call");
    //
    //   let high_water_after_first = metrics.fuel_consumed;
    //
    //   // Second invocation with bigger input:
    //   engine.call(sg, "main", json!({"input": 1000})).unwrap();
    //
    //   let metrics = engine.describe_sandbox_node(sg).unwrap();
    //   // High-water tracker holds the MAX over invocations:
    //   assert!(metrics.fuel_consumed >= high_water_after_first,
    //       "high-water tracker must NOT regress");
    //
    // OBSERVABLE consequence: operator-dashboard consumers see real
    // metrics. Defends against the placeholder-Unknown failure mode
    // where dashboards display dead values.
    unimplemented!("G19-C2 wires fuel/output/last_invocation_ms high-water tracker round-trip");
}

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — describe_sandbox_node returns real fuel_consumed (not Unknown)"]
fn describe_sandbox_node_returns_real_fuel_consumed_not_unknown() {
    // §7.1 closure pin. G19-C2 implementer wires the napi side:
    //
    //   // This test pins the napi binding side specifically:
    //   //   bindings/napi/src/sandbox.rs::describeSandboxNode
    //   //
    //   // The Phase-2b state returns a dict containing
    //   // `{ fuel_consumed: "Unknown", ... }` as a placeholder.
    //   // G19-C2 replaces with real numeric values.
    //
    //   let engine = Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph_with_sandbox_handler_for_test().unwrap();
    //   engine.call(sg, "main", json!({})).unwrap();
    //
    //   let descriptor = engine.describe_sandbox_node(sg).unwrap();
    //   let serialized = serde_json::to_value(&descriptor).unwrap();
    //
    //   // Real numeric value, NOT the legacy placeholder string:
    //   assert!(serialized["fuel_consumed"].is_u64(),
    //       "fuel_consumed must be a u64, not the 'Unknown' placeholder");
    //
    // OBSERVABLE consequence: TS callers receive a real number through
    // `engine.describeSandboxNode(sg).fuelConsumed` instead of an
    // opaque sentinel. End-to-end pin per pim-2 §3.6b.
    unimplemented!("G19-C2 wires describe_sandbox_node to return real fuel_consumed values");
}

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — SANDBOX metrics propagation across cross-process WAIT-resume (stream-r1-8)"]
fn sandbox_metrics_propagation_through_cross_process_resume_via_envelope() {
    // stream-r1-8 pin. G19-C2 implementer wires this with the explicit
    // semantic: high-water metrics are PER-INVOCATION, NOT cumulative
    // across resume boundaries. The test asserts the documented shape.
    //
    //   // Build a SANDBOX-bearing handler inside a WAIT-bearing subgraph:
    //   let engine_a = Engine::open_path(tmpdir.path()).unwrap();
    //   let sg = engine_a.register_subgraph_sandbox_then_wait(/* ... */).unwrap();
    //
    //   // First-process invocation: SANDBOX runs, then WAIT suspends.
    //   let suspended = engine_a.call_with_suspension(sg, "main", json!({})).unwrap();
    //
    //   // Capture the metrics post-SANDBOX-pre-WAIT:
    //   let metrics_a = engine_a.describe_sandbox_node(sg).unwrap();
    //   assert!(metrics_a.fuel_consumed > 0);
    //   drop(engine_a);
    //
    //   // Second process: open + resume:
    //   let engine_b = Engine::open_path(tmpdir.path()).unwrap();
    //   engine_b.resume_with_meta(suspended.envelope, "go").unwrap();
    //
    //   // Per stream-r1-8: NEW invocation's high-water is per-invocation,
    //   // NOT cumulative. The describe_sandbox_node post-resume returns
    //   // the SECOND invocation's high-water if SANDBOX ran again, OR
    //   // the persisted-PER-INVOCATION value (semantic decision RECOMMEND
    //   // per stream-r1-8: per-invocation, NOT cross-resume cumulative).
    //   let metrics_b = engine_b.describe_sandbox_node(sg).unwrap();
    //
    //   // The pinned semantic: metrics_b reflects the most-recent
    //   // invocation's measurement, NOT a sum across resumes:
    //   // (This pin documents the chosen shape per stream-r1-8 RECOMMEND;
    //   //  if R1 picks cross-resume cumulative instead, this test gets
    //   //  rewritten with the symmetric assertion.)
    //   assert!(metrics_b.fuel_consumed > 0);
    //
    // OBSERVABLE consequence: high-water tracker semantic is documented
    // + tested. Defends against silent-data-loss where dashboards see
    // resets across cross-process resume without warning.
    unimplemented!(
        "G19-C2 wires sandbox metrics per-invocation semantic across cross-process resume"
    );
}

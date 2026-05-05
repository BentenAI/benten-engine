//! R3-E RED-PHASE pin for G19-C2 napi describeSandboxNode real metrics
//! (wave-7 parallel; §7.1).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C2):
//!
//! - `tests/describe_sandbox_node_returns_real_fuel_consumed_not_unknown` — §7.1
//!
//! Companion to `crates/benten-engine/tests/sandbox_metrics.rs`; this file
//! pins the napi-binding side specifically (the cross-language boundary
//! where the metric values are exposed to TS callers as real `u64` /
//! `number` values rather than placeholder strings).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 wires bindings/napi/src/sandbox.rs::describe_sandbox_node real metrics"]
fn describe_sandbox_node_napi_returns_real_metric_values_not_unknown_placeholder() {
    // §7.1 napi-side closure pin. G19-C2 implementer wires this:
    //
    //   // Drive a SANDBOX call through the napi entry point + assert
    //   // the returned descriptor carries real numeric values:
    //   let engine = benten_napi::testing::open_in_memory_engine().unwrap();
    //   let sg = benten_napi::testing::register_subgraph_with_sandbox_handler(&engine).unwrap();
    //   benten_napi::testing::call(&engine, &sg, "main", json!({})).unwrap();
    //
    //   let descriptor: serde_json::Value =
    //       benten_napi::testing::describe_sandbox_node_as_json(&engine, &sg).unwrap();
    //
    //   assert!(descriptor["fuelConsumed"].is_u64() || descriptor["fuelConsumed"].is_number(),
    //       "fuelConsumed must be numeric at napi boundary, not 'Unknown'");
    //   assert!(descriptor["lastInvocationMs"].is_u64() || descriptor["lastInvocationMs"].is_number(),
    //       "lastInvocationMs must be numeric");
    //   assert!(descriptor["outputConsumed"].is_u64() || descriptor["outputConsumed"].is_number(),
    //       "outputConsumed must be numeric");
    //
    //   // Negative pin: NO field equals the legacy placeholder string:
    //   for key in ["fuelConsumed", "lastInvocationMs", "outputConsumed"] {
    //       let v = &descriptor[key];
    //       assert!(!v.is_string() || v.as_str() != Some("Unknown"),
    //           "{} must not be 'Unknown' post-G19-C2", key);
    //   }
    //
    // OBSERVABLE consequence: TS callers receive real numbers; existing
    // dashboards that read `engine.describeSandboxNode(sg).fuelConsumed`
    // begin returning live values rather than dead sentinels. End-to-end
    // pin per pim-2 §3.6b.
    unimplemented!("G19-C2 wires napi describe_sandbox_node returning real metric values");
}

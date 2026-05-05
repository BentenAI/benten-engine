//! R3-E RED-PHASE pins for G19-C2 STREAM ESC defenses per-handler config
//! (wave 7 parallel; §7.1.5 + stream-r1-3 + stream-r1-9).
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
//! Per-handler `chunk_count_cap` + `wallclock_budget_ms` overrides land
//! in `crates/benten-eval/src/primitives/stream.rs::StreamPrimitiveSpec`
//! AND get consumed by `crates/benten-engine/src/engine_stream.rs::build_stream_handle`
//! (the production runtime path; NOT the deprecated eval-side
//! stream::execute arm which remains loud-fail per R6FP-G1 r6-stream-3).
//!
//! Per stream-r1-9: per-handler config NARROWS but cannot WIDEN the
//! workspace default grant ceiling — fail-loud on widen-attempt with
//! `E_INV_STREAM_CONFIG`.
//!
//! Per stream-r1-3: tests pin the production-grade entry point
//! (`engine.call_stream` / `openStream`) NOT the dead `stream::execute`
//! arm — defends against the AST-cache (G19-E) wrapper accidentally
//! resurrecting the deceptive-sentinel pattern.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — per-handler chunk_count_cap override"]
fn stream_chunk_count_per_handler_override() {
    // §7.1.5 pin. G19-C2 implementer wires this:
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph_with_stream_handler_chunk_cap(5).unwrap();
    //
    //   // The handler emits 10 chunks but config caps at 5 — surplus chunks
    //   // are dropped OR fire E_STREAM_CHUNK_CAP_EXCEEDED depending on the
    //   // chosen semantic. Either way the cap is enforced.
    //   let handle = engine.call_stream(sg, "main-emits-10", json!({})).unwrap();
    //   let chunks: Vec<_> = handle.collect_for_test();
    //   assert!(chunks.len() <= 5,
    //       "per-handler chunk_count_cap=5 must enforce the cap (got {})",
    //       chunks.len());
    //
    // OBSERVABLE consequence: per-handler config narrows the workspace
    // default. Pim-2 §3.6b end-to-end pin.
    unimplemented!("G19-C2 wires per-handler chunk_count_cap override");
}

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — per-handler wallclock_budget_ms override"]
fn stream_wallclock_budget_per_handler_override() {
    // §7.1.5 pin. G19-C2 implementer wires this:
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph_with_stream_handler_wallclock_ms(100).unwrap();
    //
    //   // Handler exceeds 100ms budget; the per-handler override fires
    //   // before the workspace default would have:
    //   let handle = engine.call_stream(sg, "main-slow-handler", json!({})).unwrap();
    //   let result = handle.collect_for_test_with_budget_observation();
    //   assert!(result.budget_exceeded_at_ms <= 150,
    //       "per-handler wallclock budget must be enforced near 100ms");
    //
    // OBSERVABLE consequence: per-handler config tightens the budget
    // below workspace default.
    unimplemented!("G19-C2 wires per-handler wallclock_budget_ms override");
}

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — per-handler ESC defense fires via build_stream_handle (stream-r1-3)"]
fn stream_per_handler_esc_defense_fires_observable_consequence_via_eval_side_stream_execute() {
    // stream-r1-3 LOAD-BEARING pin. G19-C2 implementer wires the
    // per-handler ESC defense in `build_stream_handle` (production
    // runtime path), NOT in the dead `stream::execute` arm.
    //
    //   // The production entry point: engine.call_stream
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph_with_stream_handler_misbehaving().unwrap();
    //
    //   let result = engine.call_stream(sg, "main-fuel-bomb", json!({}));
    //   assert!(result.is_err(),
    //       "per-handler ESC defense must fire on misbehaving handler");
    //   match result.err().unwrap().error_code() {
    //       benten_errors::ErrorCode::InvStreamConfig
    //       | benten_errors::ErrorCode::SandboxFuelExhausted
    //       | benten_errors::ErrorCode::SandboxOutputExceeded
    //           => {},
    //       other => panic!("unexpected ESC defense fire code {:?}", other),
    //   }
    //
    // OBSERVABLE consequence: ESC defense fires at the production-grade
    // entry point. Defends against the stream-r1-3 failure mode where
    // per-handler config logic lands in the dead-execute arm and is
    // never reached by the production code path.
    unimplemented!("G19-C2 wires per-handler ESC defense in build_stream_handle production path");
}

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — per-handler config exceeds grant ceiling fires E_INV_STREAM_CONFIG (stream-r1-9)"]
fn stream_per_handler_config_exceeds_grant_ceiling_fires_e_inv_stream_config() {
    // stream-r1-9 pin per RECOMMEND: per-handler config NARROWS but
    // cannot WIDEN the workspace default grant ceiling.
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //
    //   // Workspace default chunk_count_cap = 100 (assumption — exact
    //   // value per workspace config). Try to register a handler with
    //   // chunk_count_cap = 1000 (widen attempt):
    //   let result = engine.register_subgraph_with_stream_handler_widening_attempt(
    //       /* chunk_count_cap */ 1000,
    //       /* workspace_default */ 100,
    //   );
    //
    //   assert!(result.is_err(),
    //       "per-handler config widening workspace default must fail-loud");
    //   assert_eq!(result.err().unwrap().error_code(),
    //       benten_errors::ErrorCode::InvStreamConfig,
    //       "widen attempt must fire E_INV_STREAM_CONFIG (stream-r1-9)");
    //
    // OBSERVABLE consequence: extension-vs-replace policy is "narrow
    // only"; widen attempts at registration time are rejected. Defends
    // against the over-permissive-escape failure mode.
    unimplemented!("G19-C2 wires E_INV_STREAM_CONFIG for per-handler grant-ceiling widen attempts");
}

#[test]
#[ignore = "RED-PHASE: G19-C2 wave-7 — config consumed by build_stream_handle, not dead-execute arm (stream-r1-3)"]
fn stream_per_handler_config_consumed_by_build_stream_handle_not_dead_execute_arm() {
    // stream-r1-3 architectural pin. G19-C2 implementer must place
    // per-handler config logic in `build_stream_handle` (production
    // path), NOT in `stream::execute` (deprecated loud-fail arm).
    //
    //   // The dead-execute arm must STAY loud-fail with
    //   // E_PRIMITIVE_NOT_IMPLEMENTED:
    //   let result = benten_eval::testing::call_stream_execute_directly_for_test(
    //       /* fake handler */
    //   );
    //   assert!(result.is_err());
    //   assert_eq!(result.err().unwrap().error_code(),
    //       benten_errors::ErrorCode::PrimitiveNotImplemented,
    //       "stream::execute MUST stay loud-fail per R6FP-G1 r6-stream-3 \
    //        deceptive-sentinel-removed discipline");
    //
    //   // Meanwhile the production path (call_stream → build_stream_handle)
    //   // observes per-handler config:
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph_with_stream_handler_chunk_cap(3).unwrap();
    //   let handle = engine.call_stream(sg, "main-emits-10", json!({})).unwrap();
    //   let chunks: Vec<_> = handle.collect_for_test();
    //   assert!(chunks.len() <= 3, "per-handler config must fire at production path");
    //
    // OBSERVABLE consequence: discipline preserved — dead-execute arm
    // stays dead; production path consumes config. Defends against
    // G19-E AST-cache wrapper resurrecting the silent-no-op pattern.
    unimplemented!("G19-C2 wires per-handler config in build_stream_handle, NOT stream::execute");
}

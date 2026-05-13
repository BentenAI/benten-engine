//! R5 G23-B RED-PHASE pin — companion to the G23-A canary pin at
//! `tests/schema_compiler_emitted_subgraph_carries_cap_scope_annotations_routable_to_cap_policy.rs`.
//!
//! ## What this pin establishes
//!
//! The G23-A canary proves the schema-emitted SubgraphSpec carries
//! non-empty schema-derived cap-scope annotations and that the engine's
//! CapabilityPolicy is injection-routable through
//! `EngineBuilder::capability_policy` + `Engine::register_subgraph`.
//! What G23-A canary does NOT exercise is the full reactive walk: when a
//! user later invokes the handler (via `Engine::call_as(principal, ...)`)
//! or the materializer pipeline reacts to a change, the engine evaluator
//! walks the emitted subgraph + fires the cap-policy at every primitive
//! boundary it crosses.
//!
//! G23-B (materializer + Renderer trait wave) lands the dispatch surface
//! that exercises this walk end-to-end. This pin un-ignores at G23-B
//! wave completion.
//!
//! ## §3.6b end-to-end shape (per pim-2)
//!
//! - PRODUCTION RUNTIME ARM: register the schema-emitted SubgraphSpec
//!   (G23-A) + invoke the handler through the materializer pipeline
//!   (G23-B) with a recording CapabilityPolicy installed.
//! - OBSERVABLE CONSEQUENCE: the recording policy observes
//!   `cap_check_count >= primitive_count` after the walk — every
//!   primitive boundary the evaluator crosses fires a check_read or
//!   check_write against the policy.
//! - WOULD-FAIL-IF-NO-OP: if the materializer dispatched primitives
//!   without invoking the cap-policy at each boundary, the recording
//!   counter would stay below `primitive_count` → assertion fails.
//!
//! Pin source: r2-test-landscape §2.4 row 5 (full-walk arm split from
//! G23-A canary scope per mini-review finding g23a-mr-3).
//!
//! Closes G23-A R5 mini-review MAJOR finding g23a-mr-3 (substantive-arm
//! gap closure via SHAPE-not-SUBSTANCE rename + companion G23-B-tagged
//! pin).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R5 G23-B wave un-ignores) — materializer pipeline + full reactive walk lands at G23-B (materializer + Renderer trait). G23-A canary already covers cap-scope annotation presence + injection seam; this companion arm covers the full-walk cap-policy firing per primitive boundary. Per pim-12 §3.6e: ignore-rationale cites the wave (G23-B) that actually un-ignores. Pin source: r2 §2.4 row 5 (full-walk arm split from G23-A canary scope per g23a-mr-3)."]
fn materializer_pipeline_walks_emitted_subgraph_and_fires_cap_policy_at_each_primitive_boundary() {
    // G23-B implementer wires this. Substantive shape:
    //
    //   use benten_engine::EngineBuilder;
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use std::sync::Arc;
    //   use std::sync::atomic::{AtomicUsize, Ordering};
    //
    //   // Recording cap-policy that counts every check_read + check_write
    //   // invocation.
    //   #[derive(Default)]
    //   struct RecordingCapPolicy { writes: Arc<AtomicUsize>, reads: Arc<AtomicUsize> }
    //   // ... CapabilityPolicy impl that increments + returns Ok(()).
    //
    //   let spec = compile(canonical_note_type_schema_bytes()).unwrap();
    //   let primitive_count = spec.as_subgraph().primitive_count();
    //   let recorder = RecordingCapPolicy::default();
    //   let writes = recorder.writes.clone();
    //   let reads = recorder.reads.clone();
    //
    //   let engine = EngineBuilder::new()
    //       .path(":memory:")
    //       .capability_policy(Box::new(recorder))
    //       .build()
    //       .expect("engine build");
    //   let handler_id = engine.register_subgraph(spec.into_subgraph()).unwrap();
    //
    //   // PRODUCTION RUNTIME ARM: invoke the handler with a real principal.
    //   // The materializer dispatches the walk through the evaluator; the
    //   // evaluator's primitive host fires the cap-policy at every
    //   // primitive boundary it crosses (per Phase-3 G16-B-F shape).
    //   let principal = test_principal_did();
    //   let input = canonical_note_input_payload();
    //   engine.call_as(principal, &handler_id, input).expect("handler dispatch");
    //
    //   // OBSERVABLE CONSEQUENCE: every primitive boundary the walk crossed
    //   // fired a cap-check. WOULD-FAIL-IF-NO-OP: a dispatch that skipped
    //   // cap-checks would leave the counter below primitive_count.
    //   let observed = writes.load(Ordering::SeqCst) + reads.load(Ordering::SeqCst);
    //   assert!(
    //       observed >= primitive_count,
    //       "materializer walk must fire cap-policy at each primitive boundary; \
    //        primitive_count={primitive_count}, observed={observed}"
    //   );
    panic!("RED-PHASE: G23-B wave wires this — materializer pipeline + full walk");
}

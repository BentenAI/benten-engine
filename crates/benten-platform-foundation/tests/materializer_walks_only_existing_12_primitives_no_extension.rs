//! R3 Family E RED-PHASE pin: materializer walks ONLY existing 12 primitives;
//! no new PrimitiveKind variant added (LOAD-BEARING; CLAUDE.md baked-in #1).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 10.
//! - CLAUDE.md baked-in #1 (12 operation primitives — irreducible).
//! - exit-criterion 7 (no new primitive variants in Phase 4-Foundation).
//!
//! ## Pair: grep-assert + runtime-trace (SHAPE-not-SUBSTANCE per §3.6f)
//!
//! - **Grep arm:** materializer.rs source MUST NOT introduce any new
//!   `PrimitiveKind::` variant; ALL dispatches use the existing 12.
//! - **Runtime arm:** drive a real materializer walk + assert the trace
//!   logs ONLY existing 12 PrimitiveKind variants.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    materializer.rs doesn't exist at HEAD; G23-B wave-5 wires materializer over existing \
    12 PrimitiveKind only. Closes r2-test-landscape §2.5 row 10 + CLAUDE.md #1 + \
    exit-criterion 7."]
fn materializer_walks_only_existing_12_primitives_no_extension() {
    // G23-B implementer wires this:
    //
    //   use benten_core::PrimitiveKind;
    //
    //   // GREP-ASSERT arm.
    //   let src = std::fs::read_to_string(
    //       "../../crates/benten-platform-foundation/src/materializer.rs"
    //   ).expect("materializer.rs source readable");
    //
    //   // Existing 12 variant names (CLAUDE.md baked-in #1):
    //   let allowed: &[&str] = &[
    //       "Read", "Write", "Transform", "Branch", "Iterate", "Wait",
    //       "Call", "Respond", "Emit", "Sandbox", "Subscribe", "Stream",
    //   ];
    //
    //   // Find PrimitiveKind:: references; assert each names an allowed variant.
    //   for (idx, _m) in src.match_indices("PrimitiveKind::") {
    //       let tail = &src[idx + "PrimitiveKind::".len()..];
    //       let variant: String = tail
    //           .chars()
    //           .take_while(|c| c.is_alphanumeric() || *c == '_')
    //           .collect();
    //       assert!(
    //           allowed.contains(&variant.as_str()),
    //           "materializer.rs references PrimitiveKind::{variant} — not in 12-primitive set"
    //       );
    //   }
    //
    //   // RUNTIME-TRACE arm (SUBSTANCE not SHAPE).
    //   //
    //   // Drive a real materializer walk under a trace subscriber that records
    //   // every PrimitiveKind the evaluator dispatches on. Assert every dispatch
    //   // names an existing 12-primitive variant.
    //
    //   let trace_subscriber = TracingPrimitiveKindRecorder::new();
    //   let _g = tracing::subscriber::set_default(trace_subscriber.subscriber());
    //
    //   let spec = schema_compiler::compile(
    //       schema_fixtures::canonical_note_type_schema_bytes()
    //   ).unwrap();
    //   let mat = HtmlJsonMaterializer::default();
    //   let _ = mat.materialize_with_gate(/* spec */ ..).unwrap();
    //
    //   let dispatched: std::collections::HashSet<PrimitiveKind> =
    //       trace_subscriber.dispatched_kinds();
    //   assert!(!dispatched.is_empty(), "walk dispatched at least one primitive");
    //   for k in dispatched {
    //       // All variants are within the existing PrimitiveKind enum (any
    //       // new variant would fail to compile in PrimitiveKind anyway —
    //       // but this assertion serves as the production-runtime check).
    //       let _ = k;
    //   }
    //
    //   // The PrimitiveKind discriminant count itself remains 12.
    //   // (cite-drift-detector + exit-criterion 7 surface this at a separate pin.)
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    let _ = materializer_fixtures::sample_note_content_bytes();
    unimplemented!(
        "G23-B wave-5 lands materializer.rs; this pin combines grep-assert (SHAPE) + \
         runtime-trace (SUBSTANCE) per §3.6f"
    );
}

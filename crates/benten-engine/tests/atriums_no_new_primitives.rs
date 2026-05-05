//! R3-E RED-PHASE pin for G20-B atriums-compose-via-existing-primitives
//! (wave-8b; cag-3 + cag-4 + plan §4 seed).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/atriums_compose_via_existing_primitives_no_new_primitive_kind_variants` — plan §4 seed
//! - `tests/atrium_examples_handlers_compose_entirely_from_existing_12_primitives_no_engine_call_outside_subgraph` — cag-4
//! - `tests/exit_criterion_13_no_new_structural_invariants_companion_to_no_new_primitive_kind` — cag-3
//!
//! ## What G20-B establishes
//!
//! Per CLAUDE.md baked-in #1 (12 primitives irreducible) + #4 (SANDBOX as
//! escape hatch): Atrium-related primitives MUST compose entirely from
//! the existing 12 primitives. No new `PrimitiveKind` enum variants.
//!
//! Companion exit-criterion 13: no new structural invariants — Phase 3
//! does NOT extend the 14-invariant set documented in `INVARIANT-COVERAGE.md`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — Atriums compose via existing primitives, no new PrimitiveKind variants"]
fn atriums_compose_via_existing_primitives_no_new_primitive_kind_variants() {
    // CLAUDE.md baked-in #1 architectural pin. G20-B implementer wires this:
    //
    //   // Walk benten-core primitive_kind enum at canonical source:
    //   let core_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("benten-core").join("src");
    //   // Find the PrimitiveKind enum definition + extract variants.
    //   let primitive_kinds = extract_primitive_kind_variants(&core_path);
    //
    //   let canonical_12 = [
    //       "Read", "Write", "Transform", "Branch", "Iterate", "Wait",
    //       "Call", "Respond", "Emit", "Sandbox", "Subscribe", "Stream",
    //   ];
    //
    //   assert_eq!(primitive_kinds.len(), 12,
    //       "PrimitiveKind enum must have exactly 12 variants per \
    //        CLAUDE.md baked-in #1 (got {})", primitive_kinds.len());
    //
    //   for v in &canonical_12 {
    //       assert!(primitive_kinds.contains(&v.to_string()),
    //           "PrimitiveKind missing canonical variant {}", v);
    //   }
    //
    // OBSERVABLE consequence: the irreducible-12-primitives architectural
    // commitment holds at Phase-3 close. Defends against scope-creep
    // from Atrium feature-pressure.
    unimplemented!("G20-B wires PrimitiveKind 12-variant pin");
}

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — Atrium example handlers compose from 12 primitives only (cag-4)"]
fn atrium_examples_handlers_compose_entirely_from_existing_12_primitives_no_engine_call_outside_subgraph()
 {
    // cag-4 architectural pin. G20-B implementer wires this:
    //
    //   // Walk packages/engine/examples/atrium-*/handler.ts (or .rs).
    //   // Inspect each example's compiled OperationNode list; every
    //   // node.kind MUST be one of the canonical 12 primitives.
    //   //
    //   // Additionally: no engine.call outside a subgraph context (the
    //   // "engine.call MUST be invoked from a subgraph handler, not at
    //   // the top level of an example" companion clause).
    //
    //   let examples_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine").join("examples");
    //
    //   for entry in std::fs::read_dir(&examples_dir).unwrap() {
    //       let entry = entry.unwrap();
    //       if !entry.file_name().to_string_lossy().starts_with("atrium-") { continue; }
    //       let handler_src = std::fs::read_to_string(entry.path().join("handler.ts")).unwrap();
    //       // Verify handler does NOT call engine.call directly outside a subgraph context:
    //       assert!(!handler_src.contains("\nengine.call("),
    //           "Atrium example {} calls engine.call at top level", entry.path().display());
    //   }
    //
    // OBSERVABLE consequence: examples are composition-pure.
    unimplemented!("G20-B wires Atrium-examples 12-primitive composition pin");
}

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — exit-criterion 13: no new structural invariants (cag-3)"]
fn exit_criterion_13_no_new_structural_invariants_companion_to_no_new_primitive_kind() {
    // cag-3 architectural pin. G20-B implementer wires this:
    //
    //   let inv_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs").join("INVARIANT-COVERAGE.md");
    //   let inv_doc = std::fs::read_to_string(&inv_path).unwrap();
    //
    //   // Count invariant-table rows. Phase-1 baseline: 14 invariants.
    //   let count = count_invariant_rows(&inv_doc);
    //   assert_eq!(count, 14,
    //       "INVARIANT-COVERAGE.md must list exactly 14 invariants at \
    //        Phase-3 close per exit-criterion 13 (got {})", count);
    //
    // OBSERVABLE consequence: Phase 3 closes without inventing new
    // structural invariants — Atriums compose from existing ones.
    unimplemented!("G20-B wires INVARIANT-COVERAGE.md 14-invariant pin");
}

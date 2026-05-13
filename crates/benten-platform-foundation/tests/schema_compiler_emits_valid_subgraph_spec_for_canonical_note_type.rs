//! R3 Family D RED-PHASE pin for G23-A schema_compiler canonical-note-type
//! round-trip (exit-criterion 1; LOAD-BEARING substantive).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.4
//! G23-A entry + `.addl/phase-4-foundation/00-implementation-plan.md` §3
//! G23-A must-pass-tests.
//!
//! ## What G23-A wave-4 establishes
//!
//! `benten_platform_foundation::schema_compiler::compile(bytes)` parses the
//! canonical Note schema (text body + HLC timestamp + optional FieldRef
//! author) and emits a valid `SubgraphSpec` whose CID round-trips through
//! `benten_core::canonical_subgraph_bytes`. The emitted SubgraphSpec carries
//! at least READ + WRITE + TRANSFORM primitives wiring the schema's field
//! access path.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D canary; G23-A wave-4 un-ignores) — \
    benten_platform_foundation::schema_compiler::compile does not exist at HEAD; G23-A \
    wave-4 wires `compile(bytes) -> Result<SubgraphSpec, SchemaCompileError>` + round-trip \
    pin. Closes r2-test-landscape §2.4 row 1 + plan §3 G23-A primary."]
fn schema_compiler_emits_valid_subgraph_spec_for_canonical_note_type() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_core::canonical_subgraph_bytes;
    //
    //   let bytes = schema_fixtures::canonical_note_type_schema_bytes();
    //   let spec = compile(bytes).expect("canonical Note schema must compile");
    //
    //   // Emitted SubgraphSpec must wire READ + WRITE + TRANSFORM at minimum
    //   // (the canonical Note CRUD path).
    //   let kinds: std::collections::HashSet<_> =
    //       spec.primitives().iter().map(|p| p.kind()).collect();
    //   assert!(kinds.contains(&benten_core::PrimitiveKind::Read),
    //       "Note schema must emit at least one READ primitive (field access path)");
    //   assert!(kinds.contains(&benten_core::PrimitiveKind::Write),
    //       "Note schema must emit at least one WRITE primitive (mutation path)");
    //
    //   // Canonical-bytes round-trip: re-compile must yield identical CID.
    //   let spec2 = compile(bytes).expect("re-compile");
    //   assert_eq!(
    //       canonical_subgraph_bytes(spec.as_subgraph()),
    //       canonical_subgraph_bytes(spec2.as_subgraph()),
    //       "G23-A round-trip: canonical-bytes must be stable across compiles"
    //   );
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    unimplemented!("G23-A wave-4 wires benten_platform_foundation::schema_compiler::compile");
}

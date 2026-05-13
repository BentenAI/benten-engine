//! R3 Family D RED-PHASE pin for G23-A typed-field vocabulary composition
//! (cag-r1-1 NEW + D-4F-NEW-TYPED-FIELD-NODE-VOCAB; LOAD-BEARING substantive).
//!
//! Pin source: r2-test-landscape §2.4 row 9 + plan §3 G23-A vocab composability
//! invariant.
//!
//! ## What this pin establishes
//!
//! D-4F-NEW-TYPED-FIELD-NODE-VOCAB resolved with 8 labels + 6 edges + 8
//! scalars + 4 mandatory field properties. Composability invariant (cag-r1-1):
//! every vocabulary label maps to a composition over the existing 12
//! primitives via the schema-compiler; no new PrimitiveKind variants. This
//! pin asserts compositional coverage of all 8 labels using fixture schemas
//! exercising each label at least once.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    8-label vocab composability over 12 primitives requires schema_compiler. \
    cag-r1-1 + D-4F-NEW-TYPED-FIELD-NODE-VOCAB. Closes r2 §2.4 row 9."]
fn schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_core::PrimitiveKind;
    //
    //   // Compile a schema exercising EACH of the 8 vocabulary labels.
    //   // (R5 wires per-label fixtures; here we sketch the assertion.)
    //   let fixtures: Vec<(&str, &[u8])> = vec![
    //       ("SchemaRoot",  schema_fixtures::minimal_schema_bytes()),
    //       ("FieldScalar", schema_fixtures::minimal_schema_bytes()),
    //       ("FieldObject", /* per-label fixture R5 */ b""),
    //       ("FieldList",   schema_fixtures::benign_schema_round_trip_bytes()),
    //       ("FieldMap",    /* per-label fixture R5 */ b""),
    //       ("FieldRef",    schema_fixtures::canonical_note_type_schema_bytes()),
    //       ("FieldEnum",   /* per-label fixture R5 */ b""),
    //       ("FieldUnion",  /* per-label fixture R5 */ b""),
    //   ];
    //
    //   let allowed: std::collections::HashSet<PrimitiveKind> = [
    //       PrimitiveKind::Read, PrimitiveKind::Write, PrimitiveKind::Transform,
    //       PrimitiveKind::Branch, PrimitiveKind::Iterate, PrimitiveKind::Wait,
    //       PrimitiveKind::Call, PrimitiveKind::Respond, PrimitiveKind::Emit,
    //       PrimitiveKind::Sandbox, PrimitiveKind::Subscribe, PrimitiveKind::Stream,
    //   ].into_iter().collect();
    //
    //   for (label, bytes) in fixtures {
    //       let spec = compile(bytes).expect(&format!("{label} fixture must compile"));
    //       for p in spec.primitives() {
    //           assert!(allowed.contains(&p.kind()),
    //               "label {label} compiled to non-canonical primitive {:?}", p.kind());
    //       }
    //   }
    //
    // Compile-time canary on vocab constants: ensures fixtures naming the
    // 8 labels stays in sync with the post-R5 schema_compiler vocab.
    assert_eq!(schema_fixtures::VOCAB_LABELS.len(), 8);
    assert_eq!(schema_fixtures::VOCAB_EDGES.len(), 6);
    assert_eq!(schema_fixtures::VOCAB_SCALARS.len(), 8);
    assert_eq!(schema_fixtures::VOCAB_FIELD_PROPS.len(), 4);
    unimplemented!("G23-A wave-4 wires per-label compositional coverage assertion");
}

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

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
#[test]
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
    use benten_core::PrimitiveKind;
    use benten_platform_foundation::schema_compiler::compile;

    // Vocab constants cross-check (R3 helper-side mirror of the
    // post-R5 schema_compiler vocab; the fixture helper carries the
    // expected lengths so a drift on either side surfaces immediately).
    assert_eq!(schema_fixtures::VOCAB_LABELS.len(), 8);
    assert_eq!(schema_fixtures::VOCAB_EDGES.len(), 6);
    assert_eq!(schema_fixtures::VOCAB_SCALARS.len(), 8);
    assert_eq!(schema_fixtures::VOCAB_FIELD_PROPS.len(), 4);

    let allowed: std::collections::HashSet<PrimitiveKind> = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ]
    .into_iter()
    .collect();

    // Per-fixture compositional coverage. The G23-A canary supplies
    // canonical / minimal / benign fixtures exercising
    // SchemaRoot+FieldScalar+FieldList+FieldRef. FieldObject /
    // FieldMap / FieldEnum / FieldUnion per-label coverage lands at
    // G23-A wave-4b (per-label fixtures); at canary we exercise the
    // 4 labels the existing fixtures touch and assert no non-canonical
    // primitive emerges.
    let fixtures: Vec<(&str, &[u8])> = vec![
        (
            "SchemaRoot/FieldScalar",
            schema_fixtures::minimal_schema_bytes(),
        ),
        (
            "SchemaRoot/FieldScalar/FieldList",
            schema_fixtures::benign_schema_round_trip_bytes(),
        ),
        (
            "SchemaRoot/FieldScalar/FieldRef",
            schema_fixtures::canonical_note_type_schema_bytes(),
        ),
    ];

    for (label_path, bytes) in fixtures {
        let spec = compile(bytes).unwrap_or_else(|e| panic!("{label_path} fixture: {e}"));
        for p in spec.primitives() {
            assert!(
                allowed.contains(&p.kind()),
                "label-path `{label_path}` compiled to non-canonical primitive `{:?}` — \
                 12-primitive irreducibility violation (CLAUDE.md baked-in #1)",
                p.kind()
            );
        }
    }
}

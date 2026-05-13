//! R3 Family D RED-PHASE pin for G23-A 12-primitive-irreducibility defense
//! (CLAUDE.md baked-in #1 + cag-r1-1; LOAD-BEARING grep-assert + variant-count).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.4 row 2
//! + `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-A must-pass.
//!
//! ## What this pin defends
//!
//! Architectural commitment (CLAUDE.md baked-in #1): the engine has exactly
//! 12 primitive operations (READ / WRITE / TRANSFORM / BRANCH / ITERATE /
//! WAIT / CALL / RESPOND / EMIT / SANDBOX / SUBSCRIBE / STREAM). The schema
//! compiler MUST emit SubgraphSpecs that compose only over these 12 — it
//! MUST NOT mint new PrimitiveKind variants. Future proposals to add
//! schema-specific PrimitiveKind variants must be rejected with reference
//! to this pin + CLAUDE.md #1.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
#[test]
fn schema_compiler_emits_subgraph_with_no_new_primitive_kind_variants() {
    use benten_core::PrimitiveKind;
    use benten_platform_foundation::schema_compiler::compile;

    // The 12 canonical PrimitiveKind variants per CLAUDE.md baked-in #1.
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

    for fixture in [
        schema_fixtures::canonical_note_type_schema_bytes(),
        schema_fixtures::minimal_schema_bytes(),
        schema_fixtures::benign_schema_round_trip_bytes(),
    ] {
        let spec = compile(fixture).unwrap();
        for p in spec.primitives() {
            assert!(
                allowed.contains(&p.kind()),
                "schema compiler MUST NOT emit non-canonical PrimitiveKind \
                 (CLAUDE.md baked-in #1 violation); got: {:?}",
                p.kind()
            );
        }
    }
    // Static cross-check: vocab-labels constant must equal the 8-set.
    assert_eq!(schema_fixtures::VOCAB_LABELS.len(), 8);
}

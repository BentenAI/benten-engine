//! R3 Family D RED-PHASE pin for G23-A cap-scope annotations
//! (sec-3.5-r1-4; LOAD-BEARING substantive).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.4 row 3
//! + `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-A.
//!
//! ## What this pin defends
//!
//! sec-3.5-r1-4 closure: every primitive Node in a schema-emitted SubgraphSpec
//! MUST carry a derived cap-scope annotation (NOT user-supplied). The cap
//! policy fires at each primitive's read/write boundary at walk time and
//! checks against the schema-derived scope.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    schema-emitted SubgraphSpec primitives must carry derived cap-scope annotations \
    (NOT user-supplied) per sec-3.5-r1-4. Closes r2 §2.4 row 3."]
fn schema_compiler_emits_subgraph_with_cap_scope_annotations_per_primitive_node() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //
    //   let bytes = schema_fixtures::canonical_note_type_schema_bytes();
    //   let spec = compile(bytes).unwrap();
    //
    //   // Every emitted primitive MUST carry a cap-scope annotation (the
    //   // derived-from-schema scope; e.g. `read:note` for a Note FieldScalar
    //   // walk). Annotation discovery API at R5; pattern is
    //   // `PrimitiveSpec::cap_scope() -> Option<&CapScope>`.
    //   for p in spec.primitives() {
    //       let scope = p.cap_scope();
    //       assert!(scope.is_some(),
    //           "every emitted primitive must carry derived cap-scope \
    //            annotation per sec-3.5-r1-4; primitive {:?} has none", p);
    //   }
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    unimplemented!("G23-A wave-4 wires cap-scope annotation pin per sec-3.5-r1-4");
}

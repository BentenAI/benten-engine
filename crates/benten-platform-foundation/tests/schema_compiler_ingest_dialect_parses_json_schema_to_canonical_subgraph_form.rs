//! R3 Family D RED-PHASE pin for G23-A ingest-dialect (JSON-Schema → canonical)
//! (schema-r1-3 + D-4F-3 closure).
//!
//! Pin source: r2-test-landscape §2.4 ingest-dialect row + plan §3 G23-A.
//!
//! ## What this pin establishes
//!
//! Ingest dialects (JSON-Schema / TS DSL / Python / other) live at
//! `crates/benten-platform-foundation/src/ingest_dialect/`. They translate
//! input dialect bytes → canonical subgraph form, then the schema_compiler
//! walks the canonical bytes. The engine-side parse locus per schema-r1-3:
//! browser may submit either canonical-bytes or dialect-source-bytes; the
//! schema-compiler ALWAYS validates engine-side as authoritative pre-WRITE
//! gate (schema-r1-6).

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    ingest_dialect::json_schema does not exist at HEAD; G23-A wires JSON-Schema dialect \
    → canonical-form translation. schema-r1-3 + D-4F-3 closure. \
    Closes r2 §2.4 ingest-dialect row."]
fn schema_compiler_ingest_dialect_parses_json_schema_to_canonical_subgraph_form() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::ingest_dialect::json_schema::translate;
    //   use benten_platform_foundation::schema_compiler::compile;
    //
    //   // The canonical Note fixture is already JSON-Schema dialect.
    //   let dialect_bytes = schema_fixtures::canonical_note_type_schema_bytes();
    //   let canonical = translate(dialect_bytes)
    //       .expect("JSON-Schema dialect must translate to canonical form");
    //
    //   // Canonical bytes compile through the schema_compiler.
    //   let spec = compile(&canonical).expect("translated canonical form must compile");
    //   assert!(!spec.primitives().is_empty(),
    //       "ingested dialect must produce non-empty SubgraphSpec");
    //
    //   // Double-translation is idempotent (canonical-bytes invariant).
    //   let canonical2 = translate(dialect_bytes).unwrap();
    //   assert_eq!(canonical, canonical2, "translate() must be deterministic");
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    unimplemented!("G23-A wave-4 wires ingest_dialect::json_schema::translate");
}

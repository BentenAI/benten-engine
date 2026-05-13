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

// Un-ignored at G23-A wave-4 (2026-05-12 canary). At canary the ingest
// dialect set is `{Canonical}`; the JSON-Schema-standard translator
// (`IngestDialect::JsonSchemaStandard`) lands at Phase-4-Meta per
// `docs/future/phase-4-backlog.md` §4.6 (strict input-dialect work).
// This pin exercises the canonical (identity) path to lock the
// `IngestDialect::translate_to_canonical` shape.
#[test]
fn schema_compiler_ingest_dialect_parses_json_schema_to_canonical_subgraph_form() {
    use benten_platform_foundation::schema_compiler::compile;
    use benten_platform_foundation::schema_compiler::ingest_dialect::IngestDialect;

    let dialect_bytes = schema_fixtures::canonical_note_type_schema_bytes();
    let dialect = IngestDialect::detect(dialect_bytes);
    let canonical = dialect
        .translate_to_canonical(dialect_bytes)
        .expect("ingest-dialect translate to canonical form");

    // Canonical bytes compile through the schema_compiler.
    let spec = compile(&canonical).expect("translated canonical form must compile");
    assert!(
        !spec.primitives().is_empty(),
        "ingested dialect must produce non-empty Subgraph"
    );

    // Double-translation is idempotent (canonical-bytes invariant).
    let canonical2 = dialect.translate_to_canonical(dialect_bytes).unwrap();
    assert_eq!(canonical, canonical2, "translate() must be deterministic");
}

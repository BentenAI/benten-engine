//! R3 Family D RED-PHASE pin for G23-A FieldRef cycle rejection
//! (`E_SCHEMA_VOCAB_CYCLE_REJECTED`).
//!
//! Pin source: r2-test-landscape §2.4 implicit per 9-ErrorCode mint + plan
//! §3 G23-A vocab errors.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
#[test]
fn schema_compiler_rejects_field_ref_cycle() {
    use benten_errors::ErrorCode;
    use benten_platform_foundation::schema_compiler::compile;

    let bytes = schema_fixtures::hostile_schema_with_cycle_bytes();
    let err = compile(bytes).expect_err("FieldRef cycle must be rejected");
    assert_eq!(
        err.code(),
        ErrorCode::SchemaVocabCycleRejected,
        "must surface E_SCHEMA_VOCAB_CYCLE_REJECTED"
    );
}

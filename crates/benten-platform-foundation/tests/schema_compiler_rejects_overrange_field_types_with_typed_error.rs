//! R3 Family D RED-PHASE pin for G23-A overrange field-type rejection
//! (threat-model T1 schema injection defense).
//!
//! Pin source: r2-test-landscape §2.4 NEW per T1 + admin-ui-v0-threat-model.md.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

// Un-ignored at G23-A wave-4 (2026-05-12 canary). T1 schema-injection defense.
#[test]
fn schema_compiler_rejects_overrange_field_types_with_typed_error() {
    use benten_errors::ErrorCode;
    use benten_platform_foundation::schema_compiler::compile;

    let bytes = schema_fixtures::hostile_schema_unknown_label_bytes();
    let err = compile(bytes).expect_err("schema with unknown vocabulary label must be rejected");
    assert_eq!(
        err.code(),
        ErrorCode::SchemaVocabInvalidLabel,
        "must surface E_SCHEMA_VOCAB_INVALID_LABEL per T1 defense"
    );

    // T1 regression-guard: the benign fixture must continue to compile.
    let benign = schema_fixtures::benign_schema_round_trip_bytes();
    compile(benign).expect("T1 regression-guard: benign schema must still compile");
}

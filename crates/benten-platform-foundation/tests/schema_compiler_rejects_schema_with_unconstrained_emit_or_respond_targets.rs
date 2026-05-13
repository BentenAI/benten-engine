//! R3 Family D RED-PHASE pin for G23-A unconstrained-EMIT/RESPOND rejection
//! (sec-3.5-r1-4 negative pin).
//!
//! Pin source: r2-test-landscape §2.4 row 4.
//!
//! The schema compiler MUST reject schemas that try to specify EMIT or
//! RESPOND targets without scope constraints — these primitives surface
//! cap-policy boundaries that cannot be left unconstrained per
//! sec-3.5-r1-4.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

// Un-ignored at G23-A wave-4 (2026-05-12 canary).
#[test]
fn schema_compiler_rejects_schema_with_unconstrained_emit_or_respond_targets() {
    use benten_errors::ErrorCode;
    use benten_platform_foundation::schema_compiler::compile;

    // Schema with EMIT target carrying no `scope` — rejected.
    let bad_emit = br#"{
        "label": "SchemaRoot",
        "name": "BadEmit",
        "fields": [],
        "emit_targets": [ { "topic": "anything" } ]
    }"#;
    let err = compile(bad_emit).expect_err("unconstrained EMIT must be rejected");
    assert_eq!(
        err.code(),
        ErrorCode::SchemaValidationFailed,
        "unconstrained EMIT must surface E_SCHEMA_VALIDATION_FAILED"
    );

    // Schema with RESPOND target carrying no `scope` — rejected.
    let bad_respond = br#"{
        "label": "SchemaRoot",
        "name": "BadRespond",
        "fields": [],
        "respond_targets": [ { "handler_id": "anything" } ]
    }"#;
    let err = compile(bad_respond).expect_err("unconstrained RESPOND must be rejected");
    assert_eq!(
        err.code(),
        ErrorCode::SchemaValidationFailed,
        "unconstrained RESPOND must surface E_SCHEMA_VALIDATION_FAILED"
    );

    // Reference: hostile-unknown-label fixture is used by a sibling test.
    let _ = schema_fixtures::hostile_schema_unknown_label_bytes();
}

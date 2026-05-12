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

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    unconstrained EMIT/RESPOND target rejection requires schema_compiler. \
    sec-3.5-r1-4 negative pin. Closes r2 §2.4 row 4."]
fn schema_compiler_rejects_schema_with_unconstrained_emit_or_respond_targets() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_errors::ErrorCode;
    //
    //   // A schema that requests EMIT without a `scope` clause must be rejected.
    //   let bad = br#"{
    //     "label": "SchemaRoot", "name": "BadEmit",
    //     "fields": [],
    //     "emit_targets": [ { "topic": "anything", "scope": null } ]
    //   }"#;
    //   let err = compile(bad).expect_err("unconstrained EMIT must be rejected");
    //   assert_eq!(err.code(), ErrorCode::SchemaValidationFailed,
    //       "must surface E_SCHEMA_VALIDATION_FAILED");
    let _ = schema_fixtures::hostile_schema_unknown_label_bytes();
    unimplemented!("G23-A wave-4 wires unconstrained-EMIT/RESPOND rejection");
}

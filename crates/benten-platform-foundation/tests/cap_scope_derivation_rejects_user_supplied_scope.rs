//! R3 Family D RED-PHASE pin for G23-A cap-scope-must-be-derived defense
//! (sec-3.5-r1-4; substantive negative).
//!
//! Pin source: r2-test-landscape §2.4 helper-list line + Family D charter.
//!
//! ## What this pin defends
//!
//! sec-3.5-r1-4: cap-scope on emitted primitives is SCHEMA-DERIVED, not
//! user-supplied. The schema compiler MUST reject any schema input that
//! attempts to inject a `cap_scope_override` or equivalent override on a
//! primitive. Without this defense, an admin-UI plugin could trivially
//! emit a SubgraphSpec that bypasses schema-derived cap-policy.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family D; G23-A wave-4 un-ignores) — \
    schema_compiler must reject user-supplied cap-scope overrides per sec-3.5-r1-4 \
    schema-derived contract. Closes r2 §2.4 cap-scope-rejection."]
fn cap_scope_derivation_rejects_user_supplied_scope() {
    // G23-A implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_errors::ErrorCode;
    //
    //   // Hostile schema attempts to inject a user-supplied cap_scope override
    //   // on a primitive (rather than letting it be schema-derived).
    //   let bytes = br#"{
    //     "label": "SchemaRoot", "name": "OverrideAttempt",
    //     "fields": [
    //       { "label": "FieldScalar", "name": "x", "scalar": "text",
    //         "required": true, "scope": ["read:override-attempt"],
    //         "_user_supplied_primitive_cap_scope_override": ["*:*"] }
    //     ]
    //   }"#;
    //   let err = compile(bytes).expect_err(
    //       "schema_compiler must reject user-supplied cap-scope override"
    //   );
    //   assert_eq!(err.code(), ErrorCode::SchemaValidationFailed,
    //       "must surface E_SCHEMA_VALIDATION_FAILED per sec-3.5-r1-4");
    //
    //   // Positive control: legitimate `scope` field (which IS the user-supplied
    //   // field-level scope — distinct from per-primitive cap-scope) compiles.
    //   let ok = schema_fixtures::canonical_note_type_schema_bytes();
    //   compile(ok).expect("schema with legitimate field-level scope must compile");
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    unimplemented!("G23-A wave-4 wires cap-scope-must-be-derived defense (sec-3.5-r1-4)");
}

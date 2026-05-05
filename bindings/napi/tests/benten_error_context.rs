//! R3-E RED-PHASE pins for G19-B BentenError.context structured-field
//! coverage at the napi boundary (wave 7 parallel).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-B +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-B must-pass column +
//! Phase-3-backlog §7.2):
//!
//! - `tests/benten_error_context_carries_full_structured_field_coverage_at_napi_boundary` — §7.2
//! - r1-napi-7 (LOC sizing for ~20 EngineError + EvalError variants)
//!
//! ## What G19-B establishes
//!
//! Per §7.2 + r1-napi-7: `bindings/napi/src/lib.rs::engine_err` replaces the
//! message-prefix `E_*` carrier with a JSON-shape `{ code, fields }` carrier.
//! Every `EngineError` / `EvalError` variant gets full structured-field
//! coverage at the napi boundary, surfaced through `mapNativeError` on the
//! TS side as a typed `BentenError` subclass with a `.context` accessor
//! returning the structured fields object.
//!
//! ## RED-PHASE discipline
//!
//! G19-B has not yet shipped. The current `engine_err` implementation
//! still uses the message-prefix carrier (verified at HEAD `a92eba2` —
//! `bindings/napi/src/lib.rs` carries the legacy shape). R5 implementer
//! drops the `#[ignore]` and wires the test against the new carrier.
//!
//! Per §3.6b pim-2 end-to-end pin requirement: this test drives a
//! production-grade entry point (a real engine call that triggers a typed
//! error) AND asserts an observable consequence (the error's structured
//! fields are present). Sentinel-presence (just checking the symbol
//! `engine_err` exists) does NOT suffice.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 wires JSON-shape `{code, fields}` carrier at napi boundary"]
fn benten_error_context_carries_full_structured_field_coverage_at_napi_boundary() {
    // §7.2 pin per r1-napi-7 sizing. G19-B implementer wires this:
    //
    //   // Drive a real engine error via the production entry point (e.g. an
    //   // input-validation failure or capability-gated rejection). The
    //   // canonical shape: ask the engine to do something it must reject
    //   // with a known typed error variant.
    //   let result = benten_napi::testing::engine_err_round_trip(
    //       benten_engine::EngineError::SomeKnownVariant {
    //           field_a: "value".into(),
    //           field_b: 42,
    //       },
    //   );
    //
    //   // The carrier's wire shape MUST be JSON `{ code: "E_*", fields:
    //   // { field_a: "value", field_b: 42 } }`, NOT a message prefix.
    //   let payload: serde_json::Value = serde_json::from_str(&result.message()).unwrap();
    //   assert_eq!(payload["code"], "E_*"); // exact code per variant
    //   assert!(payload["fields"].is_object());
    //   assert_eq!(payload["fields"]["field_a"], "value");
    //   assert_eq!(payload["fields"]["field_b"], 42);
    //
    // OBSERVABLE consequence: TS-side `mapNativeError` (G19-B sibling
    // change) recovers the structured fields verbatim and exposes them as
    // `err.context.field_a` / `err.context.field_b`. Defends against the
    // pim-1 doc-coupling failure shape (engine ships variant; napi keeps
    // legacy carrier; structured fields silently dropped).
    unimplemented!("G19-B wires engine_err JSON-shape carrier round-trip");
}

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 — every EngineError variant carries full structured fields"]
fn engine_err_carrier_covers_every_engine_error_variant_with_no_structured_field_loss() {
    // r1-napi-7 sizing pin: G19-B sweeps ALL ~20 EngineError variants +
    // long-tail EvalError variants. The implementer walks the variant
    // list (or a generated list per the codegen-errors.ts tool) and
    // verifies each variant's structured fields round-trip through the
    // napi boundary without loss.
    //
    // Concrete shape:
    //   for variant in benten_engine::EngineError::all_variants_for_test() {
    //       let napi_err = benten_napi::testing::engine_err(variant.clone());
    //       let payload: serde_json::Value =
    //           serde_json::from_str(&napi_err.message()).unwrap();
    //       assert_eq!(payload["code"], variant.error_code().as_str(),
    //           "variant {:?} carrier code mismatch", variant);
    //       // Structured-field round-trip — all named fields appear:
    //       for (field_name, field_value) in variant.named_fields() {
    //           assert_eq!(payload["fields"][field_name], field_value,
    //               "variant {:?} field {} dropped at napi boundary",
    //               variant, field_name);
    //       }
    //   }
    //
    // OBSERVABLE consequence: no variant loses structured-field coverage
    // at the napi boundary. End-to-end pin per pim-2 §3.6b — the test
    // would FAIL if the carrier silently dropped a variant's fields.
    unimplemented!(
        "G19-B wires variant-walk over EngineError + EvalError structured-field surfacing"
    );
}

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 — typed BentenError subclass via CODE_TO_CTOR"]
fn engine_err_payload_resolves_to_typed_subclass_via_code_to_ctor_no_e_unknown() {
    // §7.6 / r1-napi-5 cross-pin: G19-B implementer wires the napi
    // carrier so that the JSON-shape payload resolves through
    // `CODE_TO_CTOR_GENERATED` (errors.generated.ts) to a typed
    // `BentenError` subclass — never falling back to the generic
    // `E_UNKNOWN` constructor for a known catalog code.
    //
    //   let napi_err = benten_napi::testing::engine_err(
    //       benten_engine::EngineError::SomeKnownVariant { ... });
    //   let code = parse_code_from_payload(&napi_err.message());
    //   let ctor = CODE_TO_CTOR_GENERATED.get(&code).expect("known code");
    //   assert_ne!(code, "E_UNKNOWN", "known catalog code must NOT fall \
    //       back to E_UNKNOWN at the napi boundary");
    //   assert!(ctor.is_some(), "every known code must resolve to a \
    //       typed BentenError subclass via CODE_TO_CTOR_GENERATED");
    //
    // Cross-references G19-B sibling test `code_to_ctor_no_e_unknown_fallback_for_known_code`
    // in `crates/benten-engine/tests/code_to_ctor.rs`.
    unimplemented!("G19-B wires payload-to-typed-subclass round-trip via CODE_TO_CTOR_GENERATED");
}

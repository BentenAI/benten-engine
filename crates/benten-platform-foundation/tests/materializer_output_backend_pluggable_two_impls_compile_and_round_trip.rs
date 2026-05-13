//! R3 Family E RED-PHASE pin: Renderer / output-format pluggability validated
//! empirically by 2 impls (arch-r1-10 + D-4F-11 ratified).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 11.
//! - arch-r1-10 (architect-reviewer R1): 1-impl trait can hide accidental
//!   coupling to that impl; ship a 2nd impl to empirically falsify.
//! - D-4F-11 ratified: `PlaintextMaterializer` as 2nd impl.
//! - cag-r1-6 (code-as-graph): output-FORMAT pluggability ↔ renderer-BACKEND
//!   pluggability distinction; THIS pin covers the format axis (Materializer
//!   trait); renderer-backend axis covered by sibling crate
//!   `benten-renderer-tauri` at G24-E.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    Materializer trait + 2 impls do not exist at HEAD; G23-B wave-5 wires HtmlJsonMaterializer \
    + PlaintextMaterializer (2nd impl). Closes r2-test-landscape §2.5 row 11 + arch-r1-10."]
fn materializer_output_backend_pluggable_two_impls_compile_and_round_trip() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::materializer::{
    //       HtmlJsonMaterializer, PlaintextMaterializer, Materializer,
    //   };
    //
    //   // BOTH impls MUST compile against the SAME trait signature.
    //   fn assert_materializer<M: Materializer>(_: &M) {}
    //   let html = HtmlJsonMaterializer::default();
    //   let plain = PlaintextMaterializer::default();
    //   assert_materializer(&html);
    //   assert_materializer(&plain);
    //
    //   // Run BOTH against identical inputs.
    //   let spec = schema_compiler::compile(
    //       schema_fixtures::canonical_note_type_schema_bytes()
    //   ).unwrap();
    //   let inputs = MaterializerWalkInputs::test_fixture(&spec);
    //
    //   let html_out = html.materialize_with_gate(inputs.clone()).unwrap();
    //   let plain_out = plain.materialize_with_gate(inputs).unwrap();
    //
    //   // OUTPUTS DIFFER STRUCTURALLY — proves trait is not accidentally
    //   // HtmlJson-specific. PlaintextMaterializer MUST NOT produce HTML
    //   // angle brackets; HtmlJsonMaterializer MUST.
    //   let html_str = std::str::from_utf8(html_out.primary_bytes()).unwrap();
    //   let plain_str = std::str::from_utf8(plain_out.primary_bytes()).unwrap();
    //   assert!(
    //       html_str.contains('<') && html_str.contains('>'),
    //       "HtmlJsonMaterializer emits HTML tags"
    //   );
    //   assert!(
    //       !plain_str.contains('<') && !plain_str.contains('>'),
    //       "PlaintextMaterializer MUST NOT emit HTML tags (else trait is HTML-coupled)"
    //   );
    //
    //   // ROUND-TRIP: both impls re-produce identical output across runs
    //   // (per mat-r1-3 inherited determinism).
    //   let html_out2 = html.materialize_with_gate(inputs.clone()).unwrap();
    //   let plain_out2 = plain.materialize_with_gate(inputs).unwrap();
    //   assert_eq!(html_out.primary_bytes(), html_out2.primary_bytes());
    //   assert_eq!(plain_out.primary_bytes(), plain_out2.primary_bytes());
    //
    //   // Structural-but-shape: both produce SOME field for "body".
    //   assert!(
    //       html_str.contains("body") && plain_str.contains("body"),
    //       "both impls surface the body field; format differs"
    //   );
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    let _ = materializer_fixtures::note_content_html_expected_skeleton();
    let _ = materializer_fixtures::note_content_plaintext_expected_skeleton();
    unimplemented!(
        "G23-B wave-5 wires Materializer trait + HtmlJsonMaterializer + PlaintextMaterializer \
         (2nd impl) per arch-r1-10 empirical pluggability validation"
    );
}

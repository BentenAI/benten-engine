//! R3 Family E RED-PHASE pin for G23-B materializer end-to-end SubgraphSpec
//! → HTML+JSON output walk (exit-criterion 2; LOAD-BEARING substantive).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 1.
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-B primary
//!   must-pass-test.
//! - Plan §1 deliverable 2 — materializer pipeline lands as IVM-view-shaped
//!   subgraph composition (Ben D-4F-2 ratified).
//!
//! ## What G23-B wave-5 establishes
//!
//! `benten_platform_foundation::materializer::HtmlJsonMaterializer` walks a
//! `SubgraphSpec` (emitted by G23-A `schema_compiler::compile`) under a
//! supplied content snapshot + actor principal and produces deterministic
//! HTML+JSON output bytes. The walk routes through the engine's existing
//! 12-primitive evaluator (NOT a separate pipeline) and threads
//! `Engine::read_node_as(principal, cid)` at every READ fanout per §3.Y
//! dual-gate inheritance commitment.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    benten_platform_foundation::materializer::HtmlJsonMaterializer does not exist at HEAD; \
    G23-B wave-5 wires Materializer trait + HtmlJsonMaterializer default impl + walks \
    composed SubgraphSpec to HTML+JSON output via existing 12 primitives. Closes \
    r2-test-landscape §2.5 row 1 + plan §3 G23-B primary."]
fn materializer_pipeline_walks_composed_subgraph_to_html_output() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::schema_compiler::compile;
    //   use benten_platform_foundation::materializer::{
    //       HtmlJsonMaterializer, Materializer, MaterializerWalkInputs,
    //   };
    //
    //   let schema_bytes = schema_fixtures::canonical_note_type_schema_bytes();
    //   let spec = compile(schema_bytes).expect("Note schema compiles");
    //   let content_bytes = materializer_fixtures::sample_note_content_bytes();
    //   let alice = materializer_fixtures::actor_principal_alice_cid();
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let out = mat.materialize_with_gate(MaterializerWalkInputs {
    //       spec: &spec,
    //       content_bytes,
    //       walk_principal: &alice,
    //       cap_recheck: cap_recheck::allow_all(),
    //   }).expect("walk succeeds for canonical Note");
    //
    //   // Output bytes must contain the canonical HTML skeleton.
    //   let html = std::str::from_utf8(&out.html_bytes()).unwrap();
    //   assert!(
    //       html.contains(materializer_fixtures::note_content_html_expected_skeleton()),
    //       "HTML+JSON materializer produces HTML article wrapping the body field"
    //   );
    //
    //   // JSON side carries cap-scope annotation (per sec-3.5-r1-4 schema-derived).
    //   let json = std::str::from_utf8(&out.json_bytes()).unwrap();
    //   assert!(json.contains("\"scope\":[\"read:note\"]"));
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    let _ = materializer_fixtures::sample_note_content_bytes();
    let _ = materializer_fixtures::actor_principal_alice_cid();
    unimplemented!(
        "G23-B wave-5 wires benten_platform_foundation::materializer::HtmlJsonMaterializer + \
         materialize_with_gate end-to-end against canonical Note schema"
    );
}

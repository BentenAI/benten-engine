//! R3 Family E RED-PHASE pin: canonical-bytes determinism across runs
//! (mat-r1-3).
//!
//! Pin sources:
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.5 G23-B row 8.
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-B.
//! - mat-r1-3 finding (R1 materializer-correctness-reviewer): materializer
//!   output bytes MUST be deterministic across runs so they can be
//!   content-addressed downstream (admin UI cache invalidation; Phase-3
//!   sync of materialised view CIDs).

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;
#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

#[test]
#[ignore = "RED-PHASE (Phase 4-Foundation R3 Family E; G23-B wave-5 un-ignores) — \
    canonical-bytes determinism pin; materializer impl doesn't exist at HEAD. G23-B wave-5 \
    lands DAG-CBOR-stable HTML+JSON output. Closes r2-test-landscape §2.5 row 8 + mat-r1-3."]
fn materializer_canonical_bytes_determinism_across_runs() {
    // G23-B implementer wires this:
    //
    //   use benten_platform_foundation::materializer::{HtmlJsonMaterializer, Materializer};
    //
    //   let mat = HtmlJsonMaterializer::default();
    //   let inputs = /* fixed inputs from schema_fixtures + materializer_fixtures */;
    //
    //   // Run the walk 3 times; output bytes MUST be identical (BLAKE3 stable).
    //   let out1 = mat.materialize_with_gate(inputs.clone()).unwrap();
    //   let out2 = mat.materialize_with_gate(inputs.clone()).unwrap();
    //   let out3 = mat.materialize_with_gate(inputs).unwrap();
    //
    //   assert_eq!(out1.html_bytes(), out2.html_bytes(), "HTML bytes stable across runs");
    //   assert_eq!(out2.html_bytes(), out3.html_bytes(), "HTML bytes stable across 3+ runs");
    //   assert_eq!(out1.json_bytes(), out2.json_bytes(), "JSON bytes stable across runs");
    //
    //   // Content-addressing: a BLAKE3-derived CID over the canonical-bytes
    //   // form is stable. mirrors `benten_core::canonical_subgraph_bytes` shape.
    //   let cid1 = out1.canonical_cid();
    //   let cid2 = out2.canonical_cid();
    //   assert_eq!(cid1, cid2, "canonical CID stable across runs (downstream content-addressing)");
    let _ = schema_fixtures::canonical_note_type_schema_bytes();
    let _ = materializer_fixtures::sample_note_content_bytes();
    unimplemented!(
        "G23-B wave-5 wires HtmlJsonMaterializer canonical-bytes determinism per mat-r1-3"
    );
}

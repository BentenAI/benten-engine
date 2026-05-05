//! R3-E RED-PHASE pin for G20-B paper-prototype revalidation re-run
//! (wave 8b; phase-3-backlog §8 row 8 + scope-real-19).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/paper_prototype_revalidation_phase_3_re_run_doc_present`
//! - `tests/sandbox_rate_under_30_percent_phase_3_corpus`
//!
//! ## What G20-B establishes
//!
//! Per scope-real-19: ~2-4 hours human classification time + ~200-400
//! LOC doc rewrite + ~50 LOC test pin. The revalidation re-runs the
//! paper-prototype exercise on the Atrium-extended primitive vocabulary
//! + asserts SANDBOX rate ≤ 30% gate (exit-criterion 14).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — paper-prototype revalidation Phase-3 re-run doc present"]
fn paper_prototype_revalidation_phase_3_re_run_doc_present() {
    // G20-B doc-presence pin. Implementer wires this:
    //
    //   let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs")
    //       .join("PAPER-PROTOTYPE-REVALIDATION.md");
    //   let content = std::fs::read_to_string(&path).unwrap();
    //
    //   // The doc must contain a Phase-3 re-run section:
    //   assert!(content.contains("Phase 3") || content.contains("Phase-3"),
    //       "PAPER-PROTOTYPE-REVALIDATION.md must include Phase-3 re-run section");
    //
    //   // The re-run must reference Atrium-extended vocabulary:
    //   assert!(content.contains("atrium") || content.contains("Atrium"),
    //       "Phase-3 re-run must include Atrium primitives");
    //
    // OBSERVABLE consequence: the human-driven revalidation step lands
    // before phase-3-close.
    unimplemented!("G20-B wires PAPER-PROTOTYPE-REVALIDATION.md Phase-3 re-run doc pin");
}

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — SANDBOX rate ≤ 30% on Phase-3 corpus (exit-criterion 14)"]
fn sandbox_rate_under_30_percent_phase_3_corpus() {
    // exit-criterion 14 pin. G20-B implementer wires this:
    //
    //   let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs")
    //       .join("PAPER-PROTOTYPE-REVALIDATION.md");
    //   let content = std::fs::read_to_string(&path).unwrap();
    //
    //   // The doc must record the SANDBOX rate measurement on the
    //   // Phase-3 corpus + assert ≤ 30%:
    //   //
    //   //   regex `SANDBOX rate.*[<≤=]\s*([0-9.]+)\s*%`
    //   //   captured value MUST be <= 30.0.
    //   let pct = extract_sandbox_rate_pct(&content);
    //   assert!(pct <= 30.0,
    //       "SANDBOX rate on Phase-3 corpus = {}%; must be ≤ 30% per \
    //        exit-criterion 14", pct);
    //
    // OBSERVABLE consequence: SANDBOX-as-escape-hatch discipline holds
    // — the revalidation confirms <= 30% of Phase-3 surface needs SANDBOX.
    unimplemented!("G20-B wires SANDBOX rate ≤ 30% Phase-3 corpus pin");
}

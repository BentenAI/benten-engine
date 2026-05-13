//! Architectural pin for the Component-Model decision
//! (D-PHASE-3-6 + D-PHASE-3-16).
//!
//! The Component-Model integration was HELD CUT — destination
//! "Phase 4+ Thrum-driven OR wasmtime-Component-Model-GA" per
//! D-PHASE-3-16. The structural sibling pin at
//! `component_model_phase3_decision_lands_per_d_phase_3_6.rs` asserts
//! the decision is reflected in `docs/ARCHITECTURE.md` via Cargo.toml
//! + roadmap walks. The test body in this file is a redundant driver
//! placeholder; the substantive load is carried by the sibling.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — Component-Model decision per D-PHASE-3-6 lands. G20-A3 wave-8a CLOSED Component-Model HELD CUT decision per §7.3.A.8 (RATIONALES REWRITTEN at Phase-3 G20-A3 wave-8a; D-PHASE-3-6 RESOLVED-at-R1; component_model_phase3_decision_lands_per_d_phase_3_6.rs structural pin lives at HEAD); this test body is its own pin sibling that needs driver authoring; un-ignore at Phase-4-Foundation pre-tag sweep per docs/future/phase-4-backlog.md §4.29 (HARD RULE 12 clause-(b))."]
fn component_model_phase3_decision_lands_per_d_phase_3_6() {
    // D-PHASE-3-6 architectural pin. G20-A3 implementer wires this:
    //
    //   // Read the relevant test/source files and verify the decision
    //   // is reflected:
    //   let arch_md_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs").join("ARCHITECTURE.md");
    //   let arch_md = std::fs::read_to_string(&arch_md_path).unwrap();
    //
    //   // The decision must be documented (either as held-cut or reopened):
    //   let held_cut = arch_md.contains("Phase 4+ Thrum-driven")
    //       || arch_md.contains("wasmtime-Component-Model-GA");
    //   let reopened = arch_md.contains("Component-Model integration")
    //       && !arch_md.contains("DEFERRED");
    //
    //   assert!(held_cut || reopened,
    //       "D-PHASE-3-6 Component-Model decision must be reflected in \
    //        ARCHITECTURE.md as either held-cut (Phase 4+ Thrum-driven \
    //        / wasmtime-CM-GA destination) OR reopened");
    //
    //   // The §7.3.A.8 test bodies must align with the chosen branch:
    //   let test_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("tests").join("component_model_decision.rs");
    //   let src = std::fs::read_to_string(&test_path).unwrap();
    //
    //   // Per HARD RULE rule-12: no `#[ignore]` with phantom-destination
    //   // rationale; either OOS-with-named-destination OR fix-now:
    //   for line in src.lines() {
    //       if line.contains("#[ignore") {
    //           assert!(line.contains("Phase 4+") || line.contains("Component-Model GA"),
    //               "remaining #[ignore] must name the D-PHASE-3-16 destination");
    //       }
    //   }
    //
    // OBSERVABLE consequence: the decision is durably reflected; HARD
    // RULE clause-(b) destination-realness is satisfied.
    unimplemented!("G20-A3 wires D-PHASE-3-6 Component-Model decision pin");
}

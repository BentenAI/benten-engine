//! R3-E RED-PHASE pin for G20-B FINAL 10-crate ARCHITECTURE.md transition
//! (wave-8b; C-15 + arch-r1-3 final-state).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/architecture_md_lists_10_crates_with_benten_id_and_benten_sync` (FINAL state)
//!
//! ## Ownership
//!
//! Per r2-test-landscape §13 ambiguous-ownership pre-emption:
//! - R3-A: G14-A1 intermediate (9-crate in-flight callout) — already landed
//! - R3-C: G16-A intermediate (10-crate in-flight callout naming benten-sync)
//! - **R3-E (this file): G20-B FINAL** 10-crate transition narrative
//!
//! Disjoint test-fn: this file owns the FINAL pin only; R3-A's
//! `architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only`
//! lives in the sibling `architecture_md_state.rs` file.
//!
//! ## What G20-B establishes
//!
//! Final 8 → 10 crate transition narrative + benten-id row + benten-sync
//! row + crate-graph DAG update. Replaces G14-A's + G16-A's in-flight
//! callouts with the durable narrative.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G20-B wave-8b — ARCHITECTURE.md FINAL 10-crate transition narrative"]
fn architecture_md_lists_10_crates_with_benten_id_and_benten_sync() {
    // C-15 + arch-r1-3 FINAL pin. G20-B implementer wires this:
    //
    //   let arch_md_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs").join("ARCHITECTURE.md");
    //   let arch_md = std::fs::read_to_string(&arch_md_path).unwrap();
    //
    //   // The doc must enumerate all 10 crates by name:
    //   let required_crates = [
    //       "benten-errors", "benten-core", "benten-graph", "benten-ivm",
    //       "benten-caps", "benten-eval", "benten-engine", "benten-dsl-compiler",
    //       "benten-id", "benten-sync",
    //   ];
    //   for c in &required_crates {
    //       assert!(arch_md.contains(c),
    //           "ARCHITECTURE.md must list crate '{}' at G20-B FINAL", c);
    //   }
    //
    //   // benten-sync must be marked native-only per CLAUDE.md baked-in #17:
    //   // (find the benten-sync section + verify the constraint is named)
    //   assert!(arch_md.contains("native-only") || arch_md.contains("native only"),
    //       "ARCHITECTURE.md must declare benten-sync as native-only \
    //        per CLAUDE.md baked-in #17");
    //
    //   // The intermediate "in-flight" callouts from G14-A1 + G16-A are
    //   // GONE (replaced by the durable 10-crate narrative):
    //   assert!(!arch_md.contains("Phase-3 in flight"),
    //       "G20-B must replace in-flight callouts with durable narrative");
    //
    // OBSERVABLE consequence: the 8→10 crate transition lands as the
    // canonical narrative. Defends against the failure mode where
    // intermediate callouts persist after the phase closes.
    unimplemented!("G20-B wires ARCHITECTURE.md FINAL 10-crate transition narrative pin");
}

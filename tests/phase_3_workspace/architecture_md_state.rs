//! Phase-3 ARCHITECTURE.md narrative state pins.
//!
//! ## Ownership (per r2-test-landscape §13 ambiguous-ownership pre-emption)
//!
//! - **R3-A** (this file's first dispatch landing): authors
//!   `architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only`
//!   asserting the G14-A1 intermediate-state callout (9-crate in-flight
//!   narrative; 10-crate not-yet) is present.
//!
//! - **R3-C** (subsequent dispatch): extends this file with the G16-A
//!   intermediate-state assertion (10-crate in-flight callout naming
//!   `benten-sync` as native-only).
//!
//! - **R3-E** (G20-B closure dispatch): authors `architecture_md_lists_10_crates_with_benten_id_and_benten_sync`
//!   asserting the FINAL 10-crate transition narrative is landed.
//!
//! Disjoint test-fn ownership: each R3 agent's contribution is a
//! separately-named `#[test] fn` so PR conflicts only fire on shared
//! prelude / helper edits.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.2 G14-A1 row + §2.4 G16-A row (intermediate
//!   states).
//! - C-15 (architecture-md-doc-drift cluster).
//! - arch-r1-3 BLOCKER (in-flight callouts present at every wave that
//!   adds a workspace crate).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A1 wave-4a updates ARCHITECTURE.md with the 9-crate in-flight callout"]
fn architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only() {
    // arch-r1-3 BLOCKER pin. G14-A1 implementer adds an HTML comment
    // (or visible line) to `docs/ARCHITECTURE.md` of the form:
    //
    //   <!-- Phase-3 in flight: 9th crate benten-id added at G14-A1;
    //        10th crate benten-sync added at G16-A as native-only;
    //        full 8→10 transition narrative lands at G20-B docs sweep -->
    //
    // (Per plan §3 G14-A1 row.)
    //
    // R3-A test: asserts the comment is present at the time G14-A1
    // PR lands.
    //
    // Concrete shape:
    //   let arch_md = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("docs").join("ARCHITECTURE.md")
    //   ).unwrap();
    //   assert!(arch_md.contains("benten-id added at G14-A1")
    //         || arch_md.contains("Phase-3 in flight"),
    //       "ARCHITECTURE.md must carry the 9-crate in-flight callout at G14-A1 landing");
    //   assert!(arch_md.contains("benten-sync") && arch_md.contains("native-only"),
    //       "ARCHITECTURE.md must name benten-sync as native-only per CLAUDE.md baked-in #17");
    //
    // OBSERVABLE consequence: after G14-A1 lands, ARCHITECTURE.md
    // explicitly tells readers a 9th crate has been added with a
    // forthcoming 10th. Defends against the "code shipped, doc didn't"
    // failure shape that arch-r1-3 named as a BLOCKER.
    unimplemented!(
        "G14-A1 wires ARCHITECTURE.md grep assertion for benten-id + benten-sync in-flight callouts"
    );
}

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
//! ## Sibling-file companion: `architecture_md_g20b_final.rs` (per R3-CPC-5 R4-R2 close)
//!
//! R3-E's FINAL 10-crate transition pin lives in a separate sibling
//! file `tests/phase_3_workspace/architecture_md_g20b_final.rs` (single
//! `#[test] fn architecture_md_lists_10_crates_with_benten_id_and_benten_sync`).
//! R5 implementer of G20-B wave-8b reads BOTH files: this file holds
//! the in-flight-callout pins that become OBSOLETE at G20-B; the
//! sibling file holds the FINAL state pin that lands when G20-B closes.
//!
//! ## Obsolescence at G20-B wave-8b (per R3-CPC-5 R4-R2 close)
//!
//! Both `#[test] fn` declarations below are IN-FLIGHT-CALLOUT pins that
//! become DELETION/RETENSE candidates when the FINAL 10-crate transition
//! pin in the sibling file replaces them. Specifically:
//!
//! - `architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only`
//!   (R3-A G14-A1 9-crate intermediate state)
//! - `architecture_md_in_flight_callout_present_for_benten_sync_native_only_at_g16_a`
//!   (R3-C G16-A 10-crate intermediate state with native-only callout)
//!
//! At G20-B wave-8b, the FINAL 10-crate transition narrative replaces
//! both intermediate-state HTML comments. The G20-B docs sweep should
//! either DELETE these two test fns OR retense them to assert the
//! transitional narrative was REPLACED (not merely extended). Pair with
//! `architecture_md_g20b_final.rs::architecture_md_lists_10_crates_with_benten_id_and_benten_sync`
//! as the surviving pin.
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
fn architecture_md_in_flight_callouts_present_for_benten_id_and_benten_sync_native_only() {
    // arch-r1-3 BLOCKER pin (un-ignored at G14-A1 wave-4a landing).
    // After G14-A1 lands, ARCHITECTURE.md carries an HTML comment of
    // the form:
    //
    //   <!-- Phase-3 in flight: 9th crate benten-id added at G14-A1;
    //        10th crate benten-sync added at G16-A as native-only;
    //        full 8→10 transition narrative lands at G20-B docs sweep -->
    let arch_md = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("ARCHITECTURE.md"),
    )
    .unwrap();
    assert!(
        arch_md.contains("benten-id added at G14-A1") || arch_md.contains("Phase-3 in flight"),
        "ARCHITECTURE.md must carry the 9-crate in-flight callout at G14-A1 landing"
    );
    assert!(
        arch_md.contains("benten-sync") && arch_md.contains("native-only"),
        "ARCHITECTURE.md must name benten-sync as native-only per CLAUDE.md baked-in #17"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 updates ARCHITECTURE.md with the 10-crate in-flight callout naming benten-sync as native-only"]
fn architecture_md_in_flight_callout_present_for_benten_sync_native_only_at_g16_a() {
    // R3-C extension per r2-test-landscape §13 ambiguous-ownership
    // pre-emption ("R3-C extends with G16-A intermediate-state
    // assertion (10-crate in-flight callout)"). G16-A implementer
    // updates `docs/ARCHITECTURE.md` so the in-flight callout
    // narrates the 10-crate transition (post-G14-A1 9-crate state +
    // G16-A landing the 10th crate `benten-sync` as native-only).
    //
    // Concrete shape:
    //   let arch_md = std::fs::read_to_string(
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..").join("..").join("docs").join("ARCHITECTURE.md")
    //   ).unwrap();
    //   assert!(
    //       arch_md.contains("benten-sync added at G16-A")
    //         || arch_md.contains("10th crate benten-sync"),
    //       "ARCHITECTURE.md must carry the 10-crate in-flight callout at G16-A landing per arch-r1-3 BLOCKER"
    //   );
    //   assert!(
    //       arch_md.contains("native-only"),
    //       "ARCHITECTURE.md must explicitly mark benten-sync as native-only per CLAUDE.md baked-in #17"
    //   );
    //
    // OBSERVABLE consequence: after G16-A lands, ARCHITECTURE.md
    // explicitly tells readers the 10th crate (benten-sync) is now
    // present + is native-only (excluded from wasm32 targets).
    // Defends against the "code shipped, doc didn't" failure shape
    // that arch-r1-3 named as a BLOCKER.
    unimplemented!(
        "G16-A wires ARCHITECTURE.md grep assertion for benten-sync 10th-crate native-only in-flight callout"
    );
}

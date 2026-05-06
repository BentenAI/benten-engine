//! Unit pins for [`DurabilityMode`]: enum variants preserved; default
//! posture history (Phase 2a/2b → `Immediate`; Phase 3 G13-E → `Group`).
//!
//! Originally landed at Phase 2a as the arch-r1-1 / Gate-5 descope pin
//! ("default MUST remain `Immediate`"). Updated at Phase-3 G13-E
//! (`crates/benten-graph/tests/durability_default.rs::durability_mode_group_default_for_crud_fast_path`)
//! when the CRUD fast-path default flipped to `Group` to close
//! `docs/SECURITY-POSTURE.md` Compromise #12 (macOS APFS fsync floor).
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.3, arch-r1-1; updated
//! Phase-3 R5 wave-3 G13-E).

#![allow(clippy::unwrap_used)]

use benten_graph::DurabilityMode;

/// SHAPE-PIN: validates the enum shape stays exhaustive over the three
/// known variants. Fails to compile if a future refactor drops `Group`
/// silently (which would re-open Compromise #12 closure regression).
#[test]
fn durability_group_enum_preserved() {
    // Enum variant `Group` compiles and can be constructed.
    let m = DurabilityMode::Group;

    // Pattern-match is exhaustive over the three known variants. This fails
    // to compile if a future refactor drops `Group` silently.
    match m {
        DurabilityMode::Immediate => unreachable!("not Immediate"),
        DurabilityMode::Group => {}
        DurabilityMode::Async => unreachable!("not Async"),
    }
}

/// HISTORY-PIN: documents the Phase-2a→Phase-3 default flip explicitly so
/// future readers see the lineage. The Phase-2a-era assertion
/// (`default() == Immediate`) was deliberately retired at G13-E; the
/// post-flip pin lives at
/// [`crates/benten-graph/tests/durability_default.rs::durability_mode_group_default_for_crud_fast_path`].
#[test]
fn durability_default_lineage_phase_3_flip_to_group() {
    // Phase-3 G13-E: default flipped from `Immediate` (Phase 2a/2b) to
    // `Group` for the CRUD fast-path; closes
    // `docs/SECURITY-POSTURE.md` Compromise #12.
    let default = DurabilityMode::default();
    assert_eq!(
        default,
        DurabilityMode::Group,
        "Phase-3 G13-E: DurabilityMode::default() flipped to Group; \
         closes SECURITY-POSTURE.md Compromise #12 (macOS APFS fsync floor)"
    );
}

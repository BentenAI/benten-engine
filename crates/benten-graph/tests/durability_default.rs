//! R3-A RED-PHASE pin: `DurabilityMode::default()` flips to `Group`
//! (G13-E wave 3; plan §3 G13-E + Compromise #12 closure).
//!
//! Pin source: r2-test-landscape §2.1 G13-E row
//! `durability_mode_group_default_for_crud_fast_path`; plan §3 G13-E;
//! S-3 / C-8 (security-posture Compromise #12 → CLOSED at G13-E).
//!
//! ## What G13-E does
//!
//! Phase-1 + Phase-2 shipped `DurabilityMode::Immediate` as the default
//! (every commit fsyncs); the existing
//! `crates/benten-graph/tests/durability_group_enum_preserved.rs`
//! pinned the `Group` variant existed but did not flip the default.
//!
//! G13-E flips `DurabilityMode::default()` to `Group` for the CRUD
//! fast-path: writes batch-fsync at controlled intervals (per S-3 /
//! C-8 spec) instead of fsyncing per commit. Closes Compromise #12
//! (APFS fsync floor) per `docs/SECURITY-POSTURE.md`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-E wave 3 flips DurabilityMode::default to Group"]
fn durability_mode_group_default_for_crud_fast_path() {
    // G13-E implementer wires this:
    //   let default = benten_graph::DurabilityMode::default();
    //   assert_eq!(default, benten_graph::DurabilityMode::Group,
    //       "G13-E flips DurabilityMode::default() to Group per S-3 / C-8 / \
    //        SECURITY-POSTURE.md Compromise #12 closure");
    //
    // OBSERVABLE consequence: opening a backend without explicitly
    // setting durability gets the Group fast-path. Defends against
    // a G13-E regression that lands the supporting machinery but
    // forgets to flip the actual default value.
    //
    // Companion regression guard: the existing
    // `crates/benten-graph/tests/durability_group_enum_preserved.rs`
    // pins the `Group` variant exists; this pin asserts it is the
    // default after G13-E.
    unimplemented!("G13-E wires DurabilityMode::default() == Group assertion");
}

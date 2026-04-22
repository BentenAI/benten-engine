//! R3 unit tests for G2-A / arch-r1-1 Gate-5 descoped: `DurabilityMode::Group`
//! enum variant is preserved; `DurabilityMode::default()` stays `Immediate`.
//!
//! TDD red-phase: the existing `Group` variant lives today (Phase 1) but
//! Phase-2a must keep the default pinned at `Immediate` after arch-r1-1
//! triage descoped the `Group`-flip.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.3, arch-r1-1).

#![allow(clippy::unwrap_used)]

use benten_graph::DurabilityMode;

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
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

#[test]
fn durability_default_stays_immediate_in_2a() {
    // arch-r1-1: Gate-5 descoped; default MUST remain `Immediate`.
    let default = DurabilityMode::default();
    assert_eq!(
        default,
        DurabilityMode::Immediate,
        "DurabilityMode::default() must stay Immediate in Phase 2a (arch-r1-1 descope)"
    );
}

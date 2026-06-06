//! **F-MS-1** — EXACTLY-3 `MembershipSetKind` structural-compile
//! HALT-AND-SURFACE.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4 (MS-structure + RBAC), family **F-MS-1** (merges A1 + WF-E1/E2 +
//! GNI-1).
//!
//! # What this pins (r2-test-landscape §1 Group 9 / R0 §1.3.A + §3.6.A +
//! §4.2 + §15.c)
//!
//! `MembershipSetKind { Atrium, DeviceMesh, SingleDevice }` is **EXACTLY 3**,
//! ordinal-stable; a 4th arm is a *missing-pattern compile error*, NOT a
//! `#[non_exhaustive]` wildcard. Garden/Grove are NOT Kinds (they are
//! GovernanceConfig tiers — graph Nodes). Codepoint `0x6600` binds to the
//! Kind. The two keying-reserves `AtriumWithRotatingGroupKey` /
//! `EphemeralLobby` **typed-reject** at v1-beta (reserved, not selectable).
//!
//! Red-phase intent (compressed): exhaustive `match` with NO wildcard (the
//! `scope.rs:44` 2-arm HALT-AND-SURFACE shape + `tf3b_no_opaque_selector_arm
//! _structural.rs` clone); `variant_count == 3`; selecting a reserve-Kind →
//! typed-reject.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Each `#[test]` compiles **green at baseline** behind `#[ignore]` against a
//! **self-contained in-file stub-shim** — this wave does NOT depend on a
//! sibling wave's module (parallel-safety: each R3 wave is independent). The
//! R5 canary wave (F-MS-1/3/4 minting) deletes the shim, inserts the real
//! `use benten_membership_set::kind::{MembershipSetKind, …}`, un-ignores, and
//! verifies green. The shim shape below IS the intended frozen surface — the
//! observable consequences asserted here are exactly the ones the real type
//! must satisfy (would-FAIL-if-no-op'd: a `#[non_exhaustive]` 4th-arm Kind,
//! a Garden/Grove Kind, or a silently-accepted reserve all break a pin).

#![allow(dead_code)]

// ── self-contained in-file stub-shim (R5 replaces with the real crate
//    surface) ──────────────────────────────────────────────────────────────

// ── R5: the real crate surface (the in-file stub-shim is deleted) ───────────
//
// The shapes the shim modeled are now the real
// `benten_membership_set::kind::{MembershipSetKind, RequestedReserveKind,
// KindDispatchError, dispatch_reserve}`. `MembershipSetKind::codepoint()` IS
// the exhaustive no-wildcard HALT-AND-SURFACE match; `::VARIANT_COUNT` IS the
// EXACTLY-3 count.
use benten_membership_set::kind::{
    KindDispatchError, MembershipSetKind, RequestedReserveKind, dispatch_reserve,
};

/// The real codepoint constant the canary mints (`0x6600`).
const MEMBERSHIP_SET_ENCRYPTION: u16 = benten_membership_set::codepoints::MEMBERSHIP_SET_ENCRYPTION;

/// Drive the production codepoint dispatch — delegates to the real
/// `Kind::codepoint()` (the exhaustive no-wildcard match / HALT-AND-SURFACE).
fn kind_to_codepoint(k: MembershipSetKind) -> u16 {
    k.codepoint()
}

/// The real variant count the canary's `MembershipSetKind` exposes.
const KIND_VARIANT_COUNT: usize = MembershipSetKind::VARIANT_COUNT;

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
fn ms1_kind_is_exactly_three_ordinal_stable() {
    // Drive the production-shaped codepoint dispatch for every Kind: the
    // exhaustive no-wildcard match is the HALT-AND-SURFACE mechanism. All 3
    // bind to 0x6600 (Kind→codepoint is the keying axis).
    let all = [
        MembershipSetKind::Atrium,
        MembershipSetKind::DeviceMesh,
        MembershipSetKind::SingleDevice,
    ];
    for k in all {
        assert_eq!(
            kind_to_codepoint(k),
            0x6600,
            "every MembershipSetKind binds to the 0x6600 MembershipSet codepoint band (F-MS-1)"
        );
    }
    // Ordinal stability: the discriminants are wire-keying-load-bearing.
    assert_eq!(MembershipSetKind::Atrium as u8, 0);
    assert_eq!(MembershipSetKind::DeviceMesh as u8, 1);
    assert_eq!(MembershipSetKind::SingleDevice as u8, 2);
    // EXACTLY-3 (the canary's VARIANT_COUNT). Would-FAIL if a 4th Kind landed.
    assert_eq!(
        KIND_VARIANT_COUNT, 3,
        "MembershipSetKind is EXACTLY-3; a 4th arm is a §15.c HALT-AND-SURFACE compile error, not a #[non_exhaustive] wildcard"
    );
}

#[test]
fn ms1_reserve_kinds_typed_reject_at_v1_beta() {
    // Selecting either reserve at v1-beta is a typed-reject — NEVER a silent
    // promotion to a 4th real Kind. Would-FAIL if a reserve were dispatchable.
    for r in [
        RequestedReserveKind::AtriumWithRotatingGroupKey,
        RequestedReserveKind::EphemeralLobby,
    ] {
        assert_eq!(
            dispatch_reserve(r),
            Err(KindDispatchError::ReserveTypedReject),
            "reserve-Kind {r:?} must typed-reject at v1-beta (reserved-not-selectable)"
        );
    }
}

#[test]
fn ms1_garden_grove_are_not_kinds() {
    // Garden/Grove are NOT in the Kind enum (they are GovernanceConfig tiers,
    // graph Nodes — F-GOV-1's territory). The structural guarantee is that the
    // EXACTLY-3 Kind set contains NO Garden/Grove arm. We assert it by
    // enumerating the full Kind set and confirming its cardinality is the
    // keying-axis-only 3 — a Garden/Grove Kind would push the count past 3 and
    // also break the no-wildcard `kind_to_codepoint` match.
    let keying_kinds = [
        MembershipSetKind::Atrium,
        MembershipSetKind::DeviceMesh,
        MembershipSetKind::SingleDevice,
    ];
    assert_eq!(
        keying_kinds.len(),
        KIND_VARIANT_COUNT,
        "the Kind enum is the SCALE/keying axis only; Garden/Grove live on the GOVERNANCE axis (F-GOV-1), never as Kinds"
    );
    // No Kind's codepoint dispatch is governance-tier-shaped: all map to the
    // single keying codepoint, proving governance is orthogonal to keying.
    assert!(
        keying_kinds.iter().all(|&k| kind_to_codepoint(k) == 0x6600),
        "the keying axis (Kind→0x6600) is orthogonal to the governance axis (Garden/Grove tiers)"
    );
}

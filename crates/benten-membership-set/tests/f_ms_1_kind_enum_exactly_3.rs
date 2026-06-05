//! **F-MS-1** — EXACTLY-3 `MembershipSetKind` structural-compile
//! HALT-AND-SURFACE.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! family **F-MS-1** (merges A1 + WF-E1/E2 + GNI-1).
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
//! # R5 (un-ignored against the real `benten_membership_set::kind` surface)
//!
//! The W4 in-file stub-shim is deleted; this exercises the real
//! `MembershipSetKind` enum (`kind.codepoint()` is the exhaustive no-wildcard
//! HALT-AND-SURFACE dispatch). Would-FAIL-if-no-op'd: a `#[non_exhaustive]`
//! 4th-arm Kind, a Garden/Grove Kind, or a silently-accepted reserve all break
//! a pin.

use benten_membership_set::kind::{
    KIND_VARIANT_COUNT, KindDispatchError, MEMBERSHIP_SET_ENCRYPTION, MembershipSetKind,
    RequestedReserveKind, dispatch_reserve,
};

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
fn ms1_kind_is_exactly_three_ordinal_stable() {
    // Drive the production codepoint dispatch for every Kind: the exhaustive
    // no-wildcard match is the HALT-AND-SURFACE mechanism. All 3 bind to 0x6600
    // (Kind→codepoint is the keying axis).
    for k in MembershipSetKind::all() {
        assert_eq!(
            k.codepoint(),
            0x6600,
            "every MembershipSetKind binds to the 0x6600 MembershipSet codepoint band (F-MS-1)"
        );
    }
    assert_eq!(MEMBERSHIP_SET_ENCRYPTION, 0x6600);
    // Ordinal stability: the discriminants are wire-keying-load-bearing.
    assert_eq!(MembershipSetKind::Atrium as u8, 0);
    assert_eq!(MembershipSetKind::DeviceMesh as u8, 1);
    assert_eq!(MembershipSetKind::SingleDevice as u8, 2);
    assert_eq!(MembershipSetKind::Atrium.ordinal(), 0);
    // EXACTLY-3 (the canary's VARIANT_COUNT). Would-FAIL if a 4th Kind landed.
    assert_eq!(
        KIND_VARIANT_COUNT, 3,
        "MembershipSetKind is EXACTLY-3; a 4th arm is a §15.c HALT-AND-SURFACE compile error, not a #[non_exhaustive] wildcard"
    );
    assert_eq!(MembershipSetKind::all().len(), KIND_VARIANT_COUNT);
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
    // EXACTLY-3 Kind set contains NO Garden/Grove arm: enumerating the full
    // Kind set confirms its cardinality is the keying-axis-only 3.
    let keying_kinds = MembershipSetKind::all();
    assert_eq!(
        keying_kinds.len(),
        KIND_VARIANT_COUNT,
        "the Kind enum is the SCALE/keying axis only; Garden/Grove live on the GOVERNANCE axis (F-GOV-1), never as Kinds"
    );
    // No Kind's codepoint dispatch is governance-tier-shaped: all map to the
    // single keying codepoint, proving governance is orthogonal to keying.
    assert!(
        keying_kinds.iter().all(|&k| k.codepoint() == 0x6600),
        "the keying axis (Kind→0x6600) is orthogonal to the governance axis (Garden/Grove tiers)"
    );
}

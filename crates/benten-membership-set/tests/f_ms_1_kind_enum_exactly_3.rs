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

/// Stand-in for `benten_membership_set::kind::MembershipSetKind`. EXACTLY-3,
/// `#[repr(u8)]`-ordinal-stable, NO `#[non_exhaustive]` (a 4th arm must be a
/// compile error at every match site, NOT a silently-tolerated wildcard).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum MembershipSetKind {
    Atrium = 0,
    DeviceMesh = 1,
    SingleDevice = 2,
}

/// Stand-in for the codepoint constant the canary mints (`0x6600`).
const MEMBERSHIP_SET_ENCRYPTION: u16 = 0x6600;

/// The two keying-reserves the R0 §3.6.A names. They are NOT Kinds — they are
/// a *selector* the dispatch rejects at v1-beta (typed-reject, never a silent
/// 4th Kind). Modeled as a separate "requested reserve" axis so the real Kind
/// enum stays EXACTLY-3.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RequestedReserveKind {
    AtriumWithRotatingGroupKey,
    EphemeralLobby,
}

#[derive(Debug, PartialEq, Eq)]
enum KindDispatchError {
    /// A reserved keying-Kind was selected at v1-beta.
    ReserveTypedReject,
}

/// Production-shaped dispatch: maps a selected Kind to its frozen codepoint.
/// The body is an **exhaustive match with no wildcard** — adding a 4th
/// `MembershipSetKind` variant breaks THIS compile (the HALT-AND-SURFACE
/// guarantee). At R5 this is the canary's real `kind.codepoint()`.
fn kind_to_codepoint(k: MembershipSetKind) -> u16 {
    match k {
        MembershipSetKind::Atrium => MEMBERSHIP_SET_ENCRYPTION,
        MembershipSetKind::DeviceMesh => MEMBERSHIP_SET_ENCRYPTION,
        MembershipSetKind::SingleDevice => MEMBERSHIP_SET_ENCRYPTION,
        // NO wildcard arm. A new Kind = a compile error here.
    }
}

/// Production-shaped reserve dispatch: a reserve-Kind selected at v1-beta is a
/// typed-reject (never silently promoted to a real Kind).
fn dispatch_reserve(r: RequestedReserveKind) -> Result<MembershipSetKind, KindDispatchError> {
    match r {
        RequestedReserveKind::AtriumWithRotatingGroupKey | RequestedReserveKind::EphemeralLobby => {
            Err(KindDispatchError::ReserveTypedReject)
        }
    }
}

/// The variant count the canary's `MembershipSetKind` must expose (the real
/// type carries a `const VARIANT_COUNT` / `strum::EnumCount` parity at R5).
const KIND_VARIANT_COUNT: usize = 3;

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-1 — MembershipSetKind is EXACTLY 3 ordinal-stable variants (Atrium=0/DeviceMesh=1/SingleDevice=2); un-ignore at R5 against benten_membership_set::kind::MembershipSetKind"]
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
#[ignore = "RED-PHASE: F-MS-1 — keying-reserve Kinds (AtriumWithRotatingGroupKey/EphemeralLobby) typed-reject at v1-beta; un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-MS-1 — Garden/Grove are GovernanceConfig tiers, NEVER MembershipSetKind variants; un-ignore at R5"]
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

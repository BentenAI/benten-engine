//! **F-MS-4** + **F-MS-5** — RoleId 5-value ordinal golden-vector +
//! Invitee derives ZERO content.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, families **F-MS-4** (merges C1 + T-F1 + WF-E4 + GNI-5, M-13) and
//! **F-MS-5** (merges C2 + T-F2 + GNI-5, M-11 hard floor).
//!
//! # What F-MS-4 pins (R0 §2.5 BC-9 / §3.6.B / M-13 / §4.2 / Inv-20 clause-j)
//!
//! `RoleId { Invitee = 0, Viewer = 1, Member = 2, Moderator = 3, Admin = 4 }`
//! — the ordinal is **keying-AAD-bound** (`role_assignments_generation` ∈
//! AAD), so the exact byte values are golden-vector-pinned at canary. This
//! **supersedes M-CONS-FINAL** (which had Viewer=0/Invitee=1; M-13 swaps to
//! Invitee=0 the zero-content floor / Viewer=1). **All 5 active** (Inv-20
//! clause-j corrected from the stale "3 active 2 reserved").
//!
//! # What F-MS-5 pins (R0 M-11 / §3.6.B / Inv-20 clause-j)
//!
//! **Invitee (RoleId=0) derives ZERO content** — no `K(N)`, no read cap;
//! pre-acceptance handshake state only. The most security-load-bearing RBAC
//! constraint: admitting an Invitee must NOT yield any key-derivation or read
//! capability; the Invitee UCAN ability-template is NONE.
//!
//! # Per-role UCAN ability cardinality (F-MS-7 authoritative; ruling 2)
//!
//! The full per-role UCAN ability-templates are golden-vector-pinned in
//! `f_ms_6_7_moderator_subset_ucan_templates.rs` (family F-MS-7). This file
//! only needs the *cardinalities* (for the Invitee=0 floor + a non-vacuous
//! positive control), and they MUST agree with that authoritative golden:
//! `Invitee = 0`, `Viewer = 1`, `Member = 3`, `Moderator = 4`,
//! **`Admin = 9`** = Moderator's `{read, write, share, moderate_content}` (4)
//! ∪ the five admin-exclusive governance abilities
//! `{admit-member, kick-member, rotate-keys, assign-roles,
//! edit-governance-config}` (5) per Ben 2026-06-02 ruling 2 (restores
//! `assign-roles`). The prior `Admin => 8` was the pre-ruling-2 count (before
//! `assign-roles` was restored) and is corrected here.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::role::RoleId` +
//! `…::keying::derive_member_key` and un-ignores. Would-FAIL-if-no-op'd: a
//! Viewer=0 ordinal (the M-CONS-FINAL value), a missing role, an Invitee
//! that derives a non-empty `K(N)` / non-None UCAN template, or an Admin
//! cardinality that drifts from the F-MS-7 nine all break a pin.

#![allow(dead_code)]

// ── R5: the real crate surface (the in-file stub-shim is deleted) ───────────
//
// `RoleId` + the key-derivation gate (`derive_member_content_key`) + the
// read-cap gate (`RoleId::grants_read_cap`) + the per-role UCAN ability
// cardinality (`ability_template(role).len()`) are now the real
// `benten_membership_set::role` surface. `derive_member_content_key` routes
// `K(N)` through the BLAKE3 KDF (#5 ONLY-call-site).
use benten_membership_set::role::{RoleId, ability_template, derive_member_content_key};

/// The 5 active roles in ordinal order (Inv-20 clause-j: ship-all-5-active).
const ALL_ROLES: [RoleId; 5] = RoleId::ALL;

/// Drive the REAL read-capability gate keyed off role (Invitee gets none).
fn grants_read_cap(role: RoleId) -> bool {
    role.grants_read_cap()
}

/// The REAL per-role UCAN ability-template cardinality — `ability_template`'s
/// `.len()` (the single source-of-truth shared with F-MS-7).
fn ucan_ability_count(role: RoleId) -> usize {
    ability_template(role).len()
}

// ── F-MS-4 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms4_roleid_ordinal_golden_vector() {
    // The exact keying-AAD-bound ordinal values. M-13 supersedes M-CONS-FINAL:
    // Invitee=0 (NOT Viewer=0). Would-FAIL if any role's ordinal drifted.
    assert_eq!(
        RoleId::Invitee as u8,
        0,
        "Invitee=0 is the zero-content floor (M-13 supersedes M-CONS-FINAL Viewer=0/Invitee=1)"
    );
    assert_eq!(RoleId::Viewer as u8, 1);
    assert_eq!(RoleId::Member as u8, 2);
    assert_eq!(RoleId::Moderator as u8, 3);
    assert_eq!(RoleId::Admin as u8, 4);
}

#[test]
fn ms4_roleid_serialized_ordinal_hex_pin() {
    // The role ordinal is what travels in `role_assignments_generation`-bound
    // AAD. Pin the exact little-bytes (single byte; no endianness ambiguity)
    // so two engines AAD-bind identically. Clone of the
    // tf2_no_hardcoded_sizes_ml_dsa65_vector.rs golden-vector shape.
    let golden: [(RoleId, u8); 5] = [
        (RoleId::Invitee, 0x00),
        (RoleId::Viewer, 0x01),
        (RoleId::Member, 0x02),
        (RoleId::Moderator, 0x03),
        (RoleId::Admin, 0x04),
    ];
    for (role, byte) in golden {
        let serialized = role as u8;
        assert_eq!(
            serialized, byte,
            "RoleId {role:?} serializes to the AAD-keying-bound ordinal byte 0x{byte:02x}"
        );
    }
}

#[test]
fn ms4_all_five_roles_active() {
    // Inv-20 clause-j corrected from "3 active 2 reserved" → all-5-active.
    // Constructing each role succeeds AND yields a distinct ordinal.
    let mut seen = std::collections::BTreeSet::new();
    for role in ALL_ROLES {
        seen.insert(role as u8);
    }
    assert_eq!(
        seen.len(),
        5,
        "all 5 RoleId variants are active at v1-beta with distinct ordinals (Inv-20 clause-j)"
    );
}

// ── F-MS-5 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms5_invitee_derives_no_content_key() {
    let node_cid = b"bafyNODE";
    // Invitee: NO K(N). The most security-load-bearing RBAC constraint (M-11).
    assert!(
        derive_member_content_key(RoleId::Invitee, node_cid).is_none(),
        "Invitee derives ZERO content — no K(N) (M-11 hard floor)"
    );
    // Paired positive control: every content-bearing role DOES derive a key,
    // so the negative isn't trivially vacuous.
    for role in [
        RoleId::Viewer,
        RoleId::Member,
        RoleId::Moderator,
        RoleId::Admin,
    ] {
        assert!(
            derive_member_content_key(role, node_cid).is_some(),
            "{role:?} derives a content key (so the Invitee=None result is load-bearing)"
        );
    }
}

#[test]
fn ms5_invitee_no_read_cap_no_ucan_abilities() {
    // No read capability.
    assert!(
        !grants_read_cap(RoleId::Invitee),
        "Invitee holds NO read cap (pre-acceptance handshake only)"
    );
    // Empty UCAN ability-template (NONE).
    assert_eq!(
        ucan_ability_count(RoleId::Invitee),
        0,
        "Invitee's UCAN ability-template is NONE (M-11 zero content)"
    );
    // Paired positive control: Viewer (the next ordinal up) DOES get a read cap
    // + a non-empty template — so the Invitee floor is a real boundary.
    assert!(grants_read_cap(RoleId::Viewer));
    assert!(ucan_ability_count(RoleId::Viewer) >= 1);
}

// ── F-MS-7 cross-file cardinality agreement (F4-RBAC-1) ──────────────────────

#[test]
fn ms_ucan_ability_cardinality_agrees_with_f_ms_7() {
    // F-MS-7 (`f_ms_6_7_moderator_subset_ucan_templates.rs`) is the
    // authoritative per-role UCAN ability-template golden. The cardinalities
    // this file uses for its Invitee floor + positive controls MUST match
    // `ability_template(role).len()` there, or the two stubs disagree about a
    // frozen RBAC surface (the F4-006-class "two stubs, one type, divergent
    // semantics" the fix-round exists to close).
    //
    // Authoritative cardinalities (ruling 2; Admin restores assign-roles → 9):
    let expected: [(RoleId, usize); 5] = [
        (RoleId::Invitee, 0),
        (RoleId::Viewer, 1),
        (RoleId::Member, 3),
        (RoleId::Moderator, 4),
        // Admin = 4 Moderator ∪ 5 admin-exclusive governance abilities
        // {admit-member, kick-member, rotate-keys, assign-roles,
        // edit-governance-config} (ruling 2). Would-FAIL on the stale 8 (the
        // pre-ruling-2 count before assign-roles was restored).
        (RoleId::Admin, 9),
    ];
    for (role, count) in expected {
        assert_eq!(
            ucan_ability_count(role),
            count,
            "{role:?} UCAN ability cardinality must agree with F-MS-7's authoritative golden (ability_template(role).len())"
        );
    }

    // Structural cross-check of the ruling-2 decomposition that pins Admin=9:
    // Moderator(4) ∪ admin-exclusive(5) with empty intersection = 9.
    let moderator = ucan_ability_count(RoleId::Moderator);
    let admin_exclusive_governance = 5; // {admit,kick,rotate,assign-roles,edit-governance-config}
    assert_eq!(
        ucan_ability_count(RoleId::Admin),
        moderator + admin_exclusive_governance,
        "Admin = Moderator (4) ∪ the 5 admin-exclusive governance abilities (ruling 2) = 9"
    );
}

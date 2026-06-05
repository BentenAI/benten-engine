//! **F-MS-6** + **F-MS-7** — Moderator ⊊ Admin strict-subset + per-role
//! UCAN ability-template golden-vectors.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! families **F-MS-6** (merges C3 + T-F3 + GNI-5, M-11; #52/#60) and
//! **F-MS-7** (merges C4 + T-F4 + GNI-19).
//!
//! # What F-MS-6 pins (R0 M-11 / §3.6.B / §9.1-5)
//!
//! Moderator's ability-set (read / write / share / moderate-content) is a
//! **STRICT subset** of Admin's: `admin ⊇ moderator` AND `moderator ⊉ admin`.
//! Moderator has **NO** admit / kick / rotate / assign-roles / governance-config.
//!
//! # What F-MS-7 pins (R0 §3.6.B / R1-Q-9 / §9.1-5)
//!
//! Each role's UCAN ability-template is golden-vector-pinned; the per-role
//! *semantics* compose from UCAN + the signed `GovernanceConfig`, **NOT** a
//! frozen permission-flags bitfield (the rejected shape — grep-defense).
//!
//! # Ratified Admin ability set (Ben 2026-06-02 — ruling 2; F4-020/F4-032)
//!
//! `Admin = Moderator's {read, write, share, moderate_content}
//!        ∪ {admit-member, kick-member, rotate-keys, assign-roles,
//!           edit-governance-config}`. Restores `assign-roles` (the role-change
//! ability) — the Admin set INCLUDES role-change.
//!
//! # R5 (un-ignored against `benten_membership_set::role::ability_template`)
//!
//! Drives the real `ability_template(role)`. The canonical frozen literals
//! ([`EXPECTED_MODERATOR_ABILITIES`] / [`ADMIN_EXCLUSIVE`]) are local to this
//! test (the golden-vector reference); the real `ability_template` must
//! reproduce them. Would-FAIL: a Moderator holding any admin-exclusive ability,
//! an Admin that doesn't superset Moderator OR drops `assign-roles`, OR a
//! permission-flags-bitfield representation all break a pin.

use std::collections::BTreeSet;

use benten_membership_set::role::{RoleId, ability_template};

// ── canonical frozen ability literals (the golden-vector reference) ───────────

const EXPECTED_MODERATOR_ABILITIES: [&str; 4] = ["moderate_content", "read", "share", "write"];
const EXPECTED_MEMBER_ABILITIES: [&str; 3] = ["read", "share_within_policy", "write_own"];
const EXPECTED_VIEWER_ABILITIES: [&str; 1] = ["read"];
const ADMIN_EXCLUSIVE: [&str; 5] = [
    "admit-member",
    "assign-roles",
    "edit-governance-config",
    "kick-member",
    "rotate-keys",
];

fn expected_admin_abilities() -> BTreeSet<&'static str> {
    EXPECTED_MODERATOR_ABILITIES
        .iter()
        .chain(ADMIN_EXCLUSIVE.iter())
        .copied()
        .collect()
}

// ── F-MS-6 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms6_moderator_strict_subset_of_admin() {
    let admin = ability_template(RoleId::Admin);
    let moderator = ability_template(RoleId::Moderator);
    assert!(
        moderator.is_subset(&admin),
        "Moderator's abilities are contained in Admin's (M-11)"
    );
    assert!(
        !admin.is_subset(&moderator),
        "Admin is NOT a subset of Moderator — the containment is STRICT (Moderator ⊊ Admin)"
    );
    let diff: BTreeSet<&str> = admin.difference(&moderator).copied().collect();
    let exclusive: BTreeSet<&str> = ADMIN_EXCLUSIVE.iter().copied().collect();
    assert_eq!(
        diff, exclusive,
        "Admin = Moderator ∪ {{admit-member, kick-member, rotate-keys, assign-roles, edit-governance-config}} (ruling 2)"
    );
}

#[test]
fn ms6_moderator_denied_admin_governance() {
    let moderator = ability_template(RoleId::Moderator);
    for ability in ADMIN_EXCLUSIVE {
        assert!(
            !moderator.contains(ability),
            "Moderator must NOT hold the admin-exclusive '{ability}' ability (M-11)"
        );
    }
    let admin = ability_template(RoleId::Admin);
    for ability in ADMIN_EXCLUSIVE {
        assert!(admin.contains(ability), "Admin holds '{ability}'");
    }
    // The Admin set INCLUDES role-change (F4-020/F4-032; ruling 2).
    assert!(
        admin.contains("assign-roles"),
        "Admin holds 'assign-roles' (the role-change ability restored per ruling 2)"
    );
}

// ── F-MS-7 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms7_per_role_ucan_ability_golden_vector() {
    let golden: [(RoleId, BTreeSet<&'static str>); 5] = [
        (RoleId::Invitee, BTreeSet::new()),
        (
            RoleId::Viewer,
            EXPECTED_VIEWER_ABILITIES.iter().copied().collect(),
        ),
        (
            RoleId::Member,
            EXPECTED_MEMBER_ABILITIES.iter().copied().collect(),
        ),
        (
            RoleId::Moderator,
            EXPECTED_MODERATOR_ABILITIES.iter().copied().collect(),
        ),
        (RoleId::Admin, expected_admin_abilities()),
    ];
    for (role, expected_set) in golden {
        let actual = ability_template(role);
        assert_eq!(
            actual, expected_set,
            "role {role:?} UCAN ability-template golden-vector (canary-pinned; drift fails)"
        );
    }
}

#[test]
fn ms7_no_permission_flags_bitfield() {
    // (1) Production representation IS the canonical named-token set.
    let admin = ability_template(RoleId::Admin);
    assert_eq!(
        admin,
        expected_admin_abilities(),
        "Admin's materialized abilities equal the frozen named-token set (NOT a bitfield)"
    );
    let moderator = ability_template(RoleId::Moderator);
    let expected_moderator: BTreeSet<&str> = EXPECTED_MODERATOR_ABILITIES.iter().copied().collect();
    assert_eq!(
        moderator, expected_moderator,
        "Moderator's materialized abilities equal the frozen named-token set (NOT a bitfield)"
    );

    // (2) Negative control: a packed-bitfield decode yields bit-INDEX strings,
    // never the named UCAN tokens — a bitfield genuinely loses the token
    // identities the set-shaped representation preserves.
    let n = admin.len();
    let packed: u16 = if n >= 16 { u16::MAX } else { (1u16 << n) - 1 };
    let bitfield_decoded: BTreeSet<String> = (0..16)
        .filter(|i| packed & (1u16 << i) != 0)
        .map(|i| format!("bit{i}"))
        .collect();
    for token in &admin {
        assert!(
            !bitfield_decoded.contains(*token),
            "a packed-bitfield decode yields bit-index positions, never the named UCAN token '{token}' (R1-Q-9)"
        );
    }
    assert!(
        admin
            .iter()
            .all(|a| !a.starts_with("bit") && a.chars().any(|c| c.is_alphabetic())),
        "abilities are named UCAN tokens, not numeric bitfield positions"
    );
}

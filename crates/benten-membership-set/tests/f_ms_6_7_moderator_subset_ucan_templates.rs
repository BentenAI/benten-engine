//! **F-MS-6** + **F-MS-7** — Moderator ⊊ Admin strict-subset + per-role
//! UCAN ability-template golden-vectors.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, families **F-MS-6** (merges C3 + T-F3 + GNI-5, M-11; #52/#60) and
//! **F-MS-7** (merges C4 + T-F4 + GNI-19).
//!
//! # What F-MS-6 pins (R0 M-11 / §3.6.B / §9.1-5)
//!
//! Moderator's ability-set (read / write / share / moderate-content) is a
//! **STRICT subset** of Admin's: `admin ⊇ moderator` AND `moderator ⊉ admin`
//! (proper containment). Moderator has **NO** admit / kick / rotate-on-fork /
//! governance-config. (#52 role-downgrade-subsume context.)
//!
//! # What F-MS-7 pins (R0 §3.6.B / R1-Q-9 / §9.1-5)
//!
//! Each role's UCAN ability-template is pinned at canary and stable
//! (golden-vector per role); the per-role *semantics* compose from UCAN + the
//! signed `GovernanceConfig`, **NOT** a frozen permission-flags bitfield (a
//! bitfield is the rejected shape — grep-defense).
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::role::{RoleId, ability_template}` (which
//! composes against benten-caps UCAN) and un-ignores. Would-FAIL-if-no-op'd: a
//! Moderator that contains `admit`/`kick`/`rotate`, an Admin that doesn't
//! superset Moderator, or a frozen permission-flags-bitfield representation all
//! break a pin.

#![allow(dead_code)]

use std::collections::BTreeSet;

// ── self-contained in-file stub-shim ────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RoleId {
    Invitee,
    Viewer,
    Member,
    Moderator,
    Admin,
}

/// The UCAN abilities a role's template grants. Modeled as a string-set
/// (semantics compose from UCAN, NOT a packed bitfield — F-MS-7 grep-defense).
/// At R5 these come from `ability_template(role)` resolving real UCAN
/// ability-tokens.
fn ability_template(role: RoleId) -> BTreeSet<&'static str> {
    let v: &[&str] = match role {
        RoleId::Invitee => &[], // NONE — zero content (F-MS-5)
        RoleId::Viewer => &["read"],
        RoleId::Member => &["read", "write_own", "share_within_policy"],
        RoleId::Moderator => &["read", "write", "share", "moderate_content"],
        RoleId::Admin => &[
            "read",
            "write",
            "share",
            "moderate_content",
            "admit",
            "kick",
            "rotate_on_fork",
            "governance_config",
        ],
    };
    v.iter().copied().collect()
}

/// The set of admin-exclusive governance abilities a Moderator must NEVER hold.
const ADMIN_EXCLUSIVE: [&str; 4] = ["admit", "kick", "rotate_on_fork", "governance_config"];

// ── F-MS-6 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-6 — Moderator ⊊ Admin strict subset (admin contains mod; mod NOT contains admin); un-ignore at R5"]
fn ms6_moderator_strict_subset_of_admin() {
    let admin = ability_template(RoleId::Admin);
    let moderator = ability_template(RoleId::Moderator);
    // admin ⊇ moderator (containment).
    assert!(
        moderator.is_subset(&admin),
        "Moderator's abilities are contained in Admin's (M-11)"
    );
    // moderator ⊉ admin (PROPER subset — strict). Would-FAIL if Moderator
    // somehow held everything Admin does.
    assert!(
        !admin.is_subset(&moderator),
        "Admin is NOT a subset of Moderator — the containment is STRICT (Moderator ⊊ Admin)"
    );
    // There is at least one ability Admin has that Moderator lacks.
    assert!(
        admin.difference(&moderator).count() >= 1,
        "Admin strictly exceeds Moderator"
    );
}

#[test]
#[ignore = "RED-PHASE: F-MS-6 — Moderator has NO admit/kick/rotate/governance abilities; un-ignore at R5"]
fn ms6_moderator_denied_admin_governance() {
    let moderator = ability_template(RoleId::Moderator);
    // Moderator must NOT hold any admin-exclusive governance ability.
    for ability in ADMIN_EXCLUSIVE {
        assert!(
            !moderator.contains(ability),
            "Moderator must NOT hold the admin-exclusive '{ability}' ability (M-11)"
        );
    }
    // Paired positive control: Admin DOES hold all of them — so the negatives
    // above are load-bearing, not vacuous.
    let admin = ability_template(RoleId::Admin);
    for ability in ADMIN_EXCLUSIVE {
        assert!(admin.contains(ability), "Admin holds '{ability}'");
    }
}

// ── F-MS-7 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-7 — per-role UCAN ability-template golden-vector (stable across canary); un-ignore at R5"]
fn ms7_per_role_ucan_ability_golden_vector() {
    // Golden-vector per role. Drift fails the pin (the template is canary-pinned
    // because the abilities AAD-bind via role_assignments_generation).
    let golden: [(RoleId, &[&str]); 5] = [
        (RoleId::Invitee, &[]),
        (RoleId::Viewer, &["read"]),
        (
            RoleId::Member,
            &["read", "share_within_policy", "write_own"],
        ),
        (
            RoleId::Moderator,
            &["moderate_content", "read", "share", "write"],
        ),
        (
            RoleId::Admin,
            &[
                "admit",
                "governance_config",
                "kick",
                "moderate_content",
                "read",
                "rotate_on_fork",
                "share",
                "write",
            ],
        ),
    ];
    for (role, expected) in golden {
        let actual = ability_template(role);
        let expected_set: BTreeSet<&str> = expected.iter().copied().collect();
        assert_eq!(
            actual, expected_set,
            "role {role:?} UCAN ability-template golden-vector (canary-pinned; drift fails)"
        );
    }
}

#[test]
#[ignore = "RED-PHASE: F-MS-7 — semantics compose from UCAN, NOT a frozen permission-flags bitfield (grep-defense); un-ignore at R5"]
fn ms7_no_permission_flags_bitfield() {
    // Grep-defense: the ability-template is a SET of UCAN ability tokens (which
    // compose with the signed GovernanceConfig), not a packed integer bitfield.
    // We assert the representation is set-shaped by checking abilities are
    // independently addressable string tokens — a bitfield would expose a
    // single u-integer, not named composable abilities.
    let admin = ability_template(RoleId::Admin);
    // Each ability is an independently-present named token (composable).
    assert!(admin.contains("read") && admin.contains("write"));
    // Removing one ability yields a strictly-smaller set (bitfield-free: no
    // implicit flag-coupling forces multiple bits to move together).
    let mut narrowed = admin.clone();
    narrowed.remove("governance_config");
    assert_eq!(
        narrowed.len(),
        admin.len() - 1,
        "abilities are independently addressable UCAN tokens (NOT a frozen permission-flags bitfield — R1-Q-9)"
    );
    // The token names are human-readable UCAN abilities, never bit indices.
    assert!(
        admin.iter().all(|a| a.chars().any(|c| c.is_alphabetic())),
        "abilities are named UCAN tokens, not numeric bitfield positions"
    );
}

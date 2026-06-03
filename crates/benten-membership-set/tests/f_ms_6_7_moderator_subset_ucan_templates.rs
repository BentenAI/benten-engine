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
//! (proper containment). Moderator has **NO** admit / kick / rotate / assign-
//! roles / governance-config. (#52 role-downgrade-subsume context.)
//!
//! # What F-MS-7 pins (R0 §3.6.B / R1-Q-9 / §9.1-5)
//!
//! Each role's UCAN ability-template is pinned at canary and stable
//! (golden-vector per role); the per-role *semantics* compose from UCAN + the
//! signed `GovernanceConfig`, **NOT** a frozen permission-flags bitfield (a
//! bitfield is the rejected shape — grep-defense, R0 §3.6.B).
//!
//! # Ratified Admin ability set (Ben 2026-06-02 — ruling 2; R0.5 §3.6.B table)
//!
//! `Admin = Moderator's {read, write, share, moderate_content}
//!        ∪ {admit-member, kick-member, rotate-keys, assign-roles,
//!           edit-governance-config}`.
//! This satisfies M-11 (Moderator ⊊ Admin) AND restores `assign-roles`
//! (the role-change ability the prior R3 draft dropped). The 5 admin-exclusive
//! governance abilities are `{admit-member, kick-member, rotate-keys,
//! assign-roles, edit-governance-config}`.
//!
//! # One canonical source-of-truth (anti-drift)
//!
//! [`EXPECTED_MODERATOR_ABILITIES`] and [`ADMIN_EXCLUSIVE`] are the single
//! frozen literals for the per-role ability tokens. `EXPECTED_ADMIN_ABILITIES`
//! is derived as `EXPECTED_MODERATOR_ABILITIES ∪ ADMIN_EXCLUSIVE`. **Both**
//! the golden-vector arm ([`ms7_per_role_ucan_ability_golden_vector`]) **and**
//! the anti-bitfield falsifiability arm ([`ms7_no_permission_flags_bitfield`])
//! consume these same literals, so the two arms can never freeze divergent
//! ability sets for the same role (closes that drift class).
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::role::{RoleId, ability_template}` (which
//! composes against benten-caps UCAN) and un-ignores. Would-FAIL-if-no-op'd: a
//! Moderator that contains `admit-member`/`kick-member`/`rotate-keys`/
//! `assign-roles`, an Admin that doesn't superset Moderator OR that drops
//! `assign-roles`, OR a frozen permission-flags-bitfield representation (whose
//! bit-indexed decode cannot reproduce the named UCAN tokens) all break a pin.

#![allow(dead_code)]

use std::collections::BTreeSet;

// ── canonical frozen ability literals (single source-of-truth) ───────────────

/// Moderator's frozen ability-token set (R0.5 §3.6.B). Sorted (BTreeSet order)
/// so that callers can compare exactly. The golden-vector arm AND the
/// anti-bitfield arm both derive from this — they cannot drift apart.
const EXPECTED_MODERATOR_ABILITIES: [&str; 4] =
    ["moderate_content", "read", "share", "write"];

/// Member's frozen ability-token set (sorted).
const EXPECTED_MEMBER_ABILITIES: [&str; 3] =
    ["read", "share_within_policy", "write_own"];

/// Viewer's frozen ability-token set (sorted).
const EXPECTED_VIEWER_ABILITIES: [&str; 1] = ["read"];

/// The five admin-exclusive governance abilities a Moderator must NEVER hold
/// (the ratified ruling-2 set). Restores `assign-roles` (F4-020). Sorted.
const ADMIN_EXCLUSIVE: [&str; 5] = [
    "admit-member",
    "assign-roles",
    "edit-governance-config",
    "kick-member",
    "rotate-keys",
];

/// Admin's frozen ability-token set, *derived* as
/// `EXPECTED_MODERATOR_ABILITIES ∪ ADMIN_EXCLUSIVE`. Computed (not re-typed) so
/// it can never drift from its two component literals.
fn expected_admin_abilities() -> BTreeSet<&'static str> {
    EXPECTED_MODERATOR_ABILITIES
        .iter()
        .chain(ADMIN_EXCLUSIVE.iter())
        .copied()
        .collect()
}

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
///
/// Admin = Moderator ∪ {admit-member, kick-member, rotate-keys, assign-roles,
/// edit-governance-config} per the ratified ruling-2 set — produced from the
/// canonical literals so the shim itself can't drift from the golden vectors.
fn ability_template(role: RoleId) -> BTreeSet<&'static str> {
    match role {
        RoleId::Invitee => BTreeSet::new(), // NONE — zero content (F-MS-5)
        RoleId::Viewer => EXPECTED_VIEWER_ABILITIES.iter().copied().collect(),
        RoleId::Member => EXPECTED_MEMBER_ABILITIES.iter().copied().collect(),
        RoleId::Moderator => {
            EXPECTED_MODERATOR_ABILITIES.iter().copied().collect()
        }
        RoleId::Admin => expected_admin_abilities(),
    }
}

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
    // Admin strictly exceeds Moderator by exactly the 5 admin-exclusive
    // governance abilities (ruling 2). Would-FAIL if Admin dropped one (e.g.
    // assign-roles) or gained a non-governance ability beyond the union.
    let diff: BTreeSet<&str> = admin.difference(&moderator).copied().collect();
    let exclusive: BTreeSet<&str> = ADMIN_EXCLUSIVE.iter().copied().collect();
    assert_eq!(
        diff, exclusive,
        "Admin = Moderator ∪ {{admit-member, kick-member, rotate-keys, assign-roles, edit-governance-config}} (ruling 2)"
    );
}

#[test]
#[ignore = "RED-PHASE: F-MS-6 — Moderator has NO admit/kick/rotate/assign-roles/governance abilities; un-ignore at R5"]
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
    // above are load-bearing, not vacuous. (Includes assign-roles, restored
    // per ruling 2.)
    let admin = ability_template(RoleId::Admin);
    for ability in ADMIN_EXCLUSIVE {
        assert!(admin.contains(ability), "Admin holds '{ability}'");
    }
    // Explicitly assert the restored role-change ability lives on Admin (the
    // F4-020 finding: the prior draft dropped it).
    assert!(
        admin.contains("assign-roles"),
        "Admin holds 'assign-roles' (the role-change ability restored per ruling 2)"
    );
}

// ── F-MS-7 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-7 — per-role UCAN ability-template golden-vector (stable across canary); un-ignore at R5"]
fn ms7_per_role_ucan_ability_golden_vector() {
    // Golden-vector per role, sourced from the canonical frozen literals so the
    // golden arm and the anti-bitfield arm cannot diverge. Drift fails the pin
    // (the template is canary-pinned because the abilities AAD-bind via
    // role_assignments_generation).
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
#[ignore = "RED-PHASE: F-MS-7 — semantics compose from UCAN, NOT a frozen permission-flags bitfield (grep-defense); un-ignore at R5"]
fn ms7_no_permission_flags_bitfield() {
    // Grep-defense (R0 §3.6.B): the ability-template is a SET of UCAN ability
    // *tokens* (which compose with the signed GovernanceConfig), NOT a packed
    // permission-flags bitfield. This arm is FALSIFIABLE: a bitfield encoding
    // would not round-trip to the canonical named-token set without an
    // out-of-band bit→name table, so we assert (1) the materialized set equals
    // the frozen named-token golden literal, AND (2) a representative packed
    // bitfield decode of the SAME role cannot reproduce those tokens.

    // (1) Production representation IS the canonical named-token set — same
    // source-of-truth literal as the golden-vector arm (cannot drift).
    let admin = ability_template(RoleId::Admin);
    let expected_admin = expected_admin_abilities();
    assert_eq!(
        admin, expected_admin,
        "Admin's materialized abilities equal the frozen named-token set (NOT a bitfield) — shared golden literal"
    );
    let moderator = ability_template(RoleId::Moderator);
    let expected_moderator: BTreeSet<&str> =
        EXPECTED_MODERATOR_ABILITIES.iter().copied().collect();
    assert_eq!(
        moderator, expected_moderator,
        "Moderator's materialized abilities equal the frozen named-token set (NOT a bitfield) — shared golden literal"
    );

    // (2) Negative control: model the REJECTED packed-bitfield shape — one bit
    // per ability, decoded to bit *indices*. A bitfield representation exposes
    // ordinal positions, not UCAN ability strings. Decode the bits back and
    // confirm they are NOT the named tokens — i.e. a bitfield genuinely loses
    // the token identities that the set-shaped representation preserves. If a
    // future change made the production representation a bitfield, the round-
    // trip in (1) above would fail; this control documents *why* (the decode
    // yields positions, never names).
    let n = admin.len();
    // A would-be packed bitfield: all n abilities present => low n bits set.
    let packed: u16 = if n >= 16 { u16::MAX } else { (1u16 << n) - 1 };
    // The bitfield decode produces bit-INDEX strings ("bit0", "bit1", …) — it
    // has no inherent knowledge of UCAN token names.
    let bitfield_decoded: BTreeSet<String> = (0..16)
        .filter(|i| packed & (1u16 << i) != 0)
        .map(|i| format!("bit{i}"))
        .collect();
    // The named-token set and the bitfield decode share NO element: the
    // bitfield cannot reproduce a single named UCAN ability. This is what makes
    // "abilities are tokens not a bitfield" a falsifiable property rather than
    // a tautology over an arbitrary BTreeSet.
    for token in &admin {
        assert!(
            !bitfield_decoded.contains(*token),
            "a packed-bitfield decode yields bit-index positions, never the named UCAN token '{token}' — abilities are token-shaped, not a permission-flags bitfield (R1-Q-9)"
        );
    }
    // And the named tokens are human-readable UCAN abilities, never the bare
    // bit-index forms a bitfield would expose.
    assert!(
        admin.iter().all(|a| !a.starts_with("bit")
            && a.chars().any(|c| c.is_alphabetic())),
        "abilities are named UCAN tokens, not numeric bitfield positions"
    );
}

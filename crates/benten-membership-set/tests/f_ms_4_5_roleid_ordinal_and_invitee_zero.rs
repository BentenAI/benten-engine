//! **F-MS-4** + **F-MS-5** — RoleId 5-value ordinal golden-vector +
//! Invitee derives ZERO content.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! families **F-MS-4** (merges C1 + T-F1 + WF-E4 + GNI-5, M-13) and
//! **F-MS-5** (merges C2 + T-F2 + GNI-5, M-11 hard floor).
//!
//! # What F-MS-4 pins (R0 §2.5 BC-9 / §3.6.B / M-13 / §4.2 / Inv-20 clause-j)
//!
//! `RoleId { Invitee = 0, Viewer = 1, Member = 2, Moderator = 3, Admin = 4 }`
//! — the ordinal is **keying-AAD-bound** (golden-vector-pinned). Supersedes
//! M-CONS-FINAL (Viewer=0/Invitee=1) per M-13. **All 5 active** (Inv-20
//! clause-j).
//!
//! # What F-MS-5 pins (R0 M-11 / §3.6.B / Inv-20 clause-j)
//!
//! **Invitee (RoleId=0) derives ZERO content** — no `K(N)`, no read cap; the
//! Invitee UCAN ability-template is NONE.
//!
//! # Per-role UCAN ability cardinality (F-MS-7 authoritative; ruling 2)
//!
//! `Invitee = 0`, `Viewer = 1`, `Member = 3`, `Moderator = 4`, **`Admin = 9`**
//! (ruling 2; `assign-roles` restored).
//!
//! # R5 (un-ignored against `benten_membership_set::{role, keying}`)
//!
//! Drives the real `RoleId` ordinal + `ability_count` + `derive_member_content_key`
//! + `grants_read_cap`. Would-FAIL: a Viewer=0 ordinal, a missing role, an
//! Invitee that derives a non-empty `K(N)` / non-None UCAN template, or an Admin
//! cardinality that drifts from the F-MS-7 nine all break a pin.

use benten_membership_set::keying::{derive_member_content_key, grants_read_cap};
use benten_membership_set::role::{ROLE_VARIANT_COUNT, RoleId, ability_count};

// ── F-MS-4 pins ──────────────────────────────────────────────────────────────

#[test]
fn ms4_roleid_ordinal_golden_vector() {
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
    // AAD. Pin the exact single byte so two engines AAD-bind identically.
    let golden: [(RoleId, u8); 5] = [
        (RoleId::Invitee, 0x00),
        (RoleId::Viewer, 0x01),
        (RoleId::Member, 0x02),
        (RoleId::Moderator, 0x03),
        (RoleId::Admin, 0x04),
    ];
    for (role, byte) in golden {
        let serialized = role.ordinal();
        assert_eq!(
            serialized, byte,
            "RoleId {role:?} serializes to the AAD-keying-bound ordinal byte 0x{byte:02x}"
        );
    }
}

#[test]
fn ms4_all_five_roles_active() {
    // Inv-20 clause-j: all-5-active. Each role yields a distinct ordinal.
    let mut seen = std::collections::BTreeSet::new();
    for role in RoleId::all() {
        seen.insert(role as u8);
    }
    assert_eq!(
        seen.len(),
        ROLE_VARIANT_COUNT,
        "all 5 RoleId variants are active at v1-beta with distinct ordinals (Inv-20 clause-j)"
    );
    assert_eq!(ROLE_VARIANT_COUNT, 5);
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
    // Paired positive control: every content-bearing role DOES derive a key.
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
    assert!(
        !grants_read_cap(RoleId::Invitee),
        "Invitee holds NO read cap (pre-acceptance handshake only)"
    );
    assert_eq!(
        ability_count(RoleId::Invitee),
        0,
        "Invitee's UCAN ability-template is NONE (M-11 zero content)"
    );
    // Paired positive control: Viewer DOES get a read cap + a non-empty template.
    assert!(grants_read_cap(RoleId::Viewer));
    assert!(ability_count(RoleId::Viewer) >= 1);
}

// ── F-MS-7 cross-file cardinality agreement (F4-RBAC-1) ──────────────────────

#[test]
fn ms_ucan_ability_cardinality_agrees_with_f_ms_7() {
    // The cardinalities here MUST match F-MS-7's authoritative
    // `ability_template(role).len()` (the SAME `ability_count` source).
    let expected: [(RoleId, usize); 5] = [
        (RoleId::Invitee, 0),
        (RoleId::Viewer, 1),
        (RoleId::Member, 3),
        (RoleId::Moderator, 4),
        (RoleId::Admin, 9),
    ];
    for (role, count) in expected {
        assert_eq!(
            ability_count(role),
            count,
            "{role:?} UCAN ability cardinality must agree with F-MS-7's authoritative golden"
        );
    }

    // Structural cross-check of the ruling-2 decomposition that pins Admin=9.
    let moderator = ability_count(RoleId::Moderator);
    let admin_exclusive_governance = 5;
    assert_eq!(
        ability_count(RoleId::Admin),
        moderator + admin_exclusive_governance,
        "Admin = Moderator (4) ∪ the 5 admin-exclusive governance abilities (ruling 2) = 9"
    );
}

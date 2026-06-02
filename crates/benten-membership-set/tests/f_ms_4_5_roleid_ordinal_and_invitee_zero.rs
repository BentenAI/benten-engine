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
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green behind `#[ignore]`. R5
//! swaps in `benten_membership_set::role::RoleId` +
//! `…::keying::derive_member_key` and un-ignores. Would-FAIL-if-no-op'd: a
//! Viewer=0 ordinal (the M-CONS-FINAL value), a missing role, or an Invitee
//! that derives a non-empty `K(N)` / non-None UCAN template all break a pin.

#![allow(dead_code)]

// ── self-contained in-file stub-shim ────────────────────────────────────────

/// Stand-in for `benten_membership_set::role::RoleId`. Ordinals are
/// keying-AAD-bound → golden-vector-pinned. M-13: Invitee=0 (zero-content
/// floor), Viewer=1 (supersedes M-CONS-FINAL).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
enum RoleId {
    Invitee = 0,
    Viewer = 1,
    Member = 2,
    Moderator = 3,
    Admin = 4,
}

/// The 5 active roles in ordinal order (Inv-20 clause-j: ship-all-5-active).
const ALL_ROLES: [RoleId; 5] = [
    RoleId::Invitee,
    RoleId::Viewer,
    RoleId::Member,
    RoleId::Moderator,
    RoleId::Admin,
];

/// Production-shaped key-derivation gate keyed off role. At R5 this is the real
/// `members_table`-driven keying glue (delegates K(N) to benten-crypto-suite).
/// **Invitee derives NOTHING** (M-11 zero-content floor).
fn derive_member_content_key(role: RoleId, node_cid: &[u8]) -> Option<Vec<u8>> {
    match role {
        // Invitee = pre-acceptance handshake only: NO content key.
        RoleId::Invitee => None,
        // Every content-bearing role derives a (stubbed) per-Node key.
        RoleId::Viewer | RoleId::Member | RoleId::Moderator | RoleId::Admin => {
            let mut k = b"K(N):".to_vec();
            k.extend_from_slice(node_cid);
            Some(k)
        }
    }
}

/// Production-shaped read-capability gate keyed off role. Invitee gets none.
fn grants_read_cap(role: RoleId) -> bool {
    !matches!(role, RoleId::Invitee)
}

/// Production-shaped UCAN ability-template count per role (F-MS-7 pins the full
/// templates; here we only need that Invitee's template is empty / NONE).
fn ucan_ability_count(role: RoleId) -> usize {
    match role {
        RoleId::Invitee => 0,   // NONE — zero content
        RoleId::Viewer => 1,    // read only
        RoleId::Member => 3,    // read / write(own) / share-within-policy
        RoleId::Moderator => 4, // read / write / share / moderate
        RoleId::Admin => 8,     // full superset
    }
}

// ── F-MS-4 pins ──────────────────────────────────────────────────────────────

#[test]
#[ignore = "RED-PHASE: F-MS-4 — RoleId ordinals Invitee=0/Viewer=1/Member=2/Moderator=3/Admin=4 (supersedes M-CONS-FINAL Viewer=0); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-MS-4 — serialized RoleId ordinal hex-pin (the AAD-bound byte); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-MS-4 — all 5 RoleId variants construct (ship-all-5-active, Inv-20 clause-j); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-MS-5 — Invitee (RoleId=0) derives ZERO content: no K(N); un-ignore at R5"]
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
#[ignore = "RED-PHASE: F-MS-5 — Invitee gets NO read cap and an EMPTY UCAN ability-template; un-ignore at R5"]
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

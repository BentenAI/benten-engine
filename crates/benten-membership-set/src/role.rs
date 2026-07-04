//! The RBAC governance axis: [`RoleId`] (5-value ordinal) + the per-role
//! UCAN [`ability_template`] golden + the role-keyed key-derivation gate.
//!
//! `RoleId { Invitee = 0, Viewer = 1, Member = 2, Moderator = 3, Admin = 4 }`
//! — the ordinal is **keying-AAD-bound** (`role_assignments_generation` ∈
//! AAD), so the exact byte values are golden-vector-pinned (M-13 supersedes
//! M-CONS-FINAL's Viewer=0/Invitee=1). **All 5 active** (Inv-20 clause-j).
//!
//! `Invitee (RoleId=0)` derives **ZERO content** — no `K(N)`, no read cap,
//! empty UCAN ability-template (M-11 hard floor). The per-role UCAN
//! ability-templates are golden-vector-pinned; semantics compose from UCAN +
//! the signed `GovernanceConfig`, NOT a frozen permission-flags bitfield
//! (R0 §3.6.B grep-defense).
//!
//! Ratified Admin set (Ben 2026-06-02 ruling 2): `Admin = Moderator's
//! {read, write, share, moderate_content} ∪ {admit-member, kick-member,
//! rotate-keys, assign-roles, edit-governance-config}` (Moderator ⊊ Admin,
//! strict; `assign-roles` restored — F4-020).

use serde::Serialize;
use std::collections::BTreeSet;

/// The RBAC role ordinal. Ordinals are keying-AAD-bound → golden-vector-pinned.
///
/// Serialized as its `u8` ordinal (the AAD-keying-bound canonical wire form;
/// determinism + AAD compactness), symmetric with the `MemberRef` int-tag
/// (R0.5 §3.5 F4-007 ruling).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Serialize)]
#[serde(into = "u8")]
#[repr(u8)]
pub enum RoleId {
    /// Pre-acceptance handshake state only — derives ZERO content (M-11).
    Invitee = 0,
    /// Read-only.
    Viewer = 1,
    /// Read / write(own) / share-within-policy.
    Member = 2,
    /// Read / write / share / moderate-content (Moderator ⊊ Admin, strict).
    Moderator = 3,
    /// Moderator ∪ the 5 admin-exclusive governance abilities (ruling 2).
    Admin = 4,
}

impl From<RoleId> for u8 {
    fn from(r: RoleId) -> u8 {
        r as u8
    }
}

impl RoleId {
    /// All 5 active roles in ordinal order (Inv-20 clause-j: ship-all-5-active).
    pub const ALL: [RoleId; 5] = [
        RoleId::Invitee,
        RoleId::Viewer,
        RoleId::Member,
        RoleId::Moderator,
        RoleId::Admin,
    ];

    /// The stable ordinal byte (the AAD-keying-bound discriminant).
    #[must_use]
    pub fn ordinal(self) -> u8 {
        self as u8
    }

    /// Whether this role grants any read capability. Invitee gets none
    /// (pre-acceptance handshake only; M-11 zero-content floor).
    #[must_use]
    pub fn grants_read_cap(self) -> bool {
        !matches!(self, RoleId::Invitee)
    }
}

// ── per-role UCAN ability-templates (the canonical frozen literals) ──────────

/// Viewer's frozen ability-token set.
const VIEWER_ABILITIES: [&str; 1] = ["read"];

/// Member's frozen ability-token set.
const MEMBER_ABILITIES: [&str; 3] = ["read", "share_within_policy", "write_own"];

/// Moderator's frozen ability-token set.
const MODERATOR_ABILITIES: [&str; 4] = ["moderate_content", "read", "share", "write"];

/// The five admin-exclusive governance abilities a Moderator must NEVER hold
/// (the ratified ruling-2 set; restores `assign-roles` per F4-020).
const ADMIN_EXCLUSIVE_ABILITIES: [&str; 5] = [
    "admit-member",
    "assign-roles",
    "edit-governance-config",
    "kick-member",
    "rotate-keys",
];

/// The per-role UCAN ability-template (golden-vector-pinned at canary; the
/// template AAD-binds via `role_assignments_generation`).
///
/// Modeled as a SET of UCAN ability *tokens* (which compose with the signed
/// `GovernanceConfig`), NOT a packed permission-flags bitfield (R0 §3.6.B
/// grep-defense). `Admin` is *derived* as `Moderator ∪ admin-exclusive` so it
/// can never drift from its two component literals.
///
/// At v1-beta this composes against the `benten-caps` UCAN ability tokens; the
/// frozen golden literals here are the canonical per-role templates.
#[must_use]
pub fn ability_template(role: RoleId) -> BTreeSet<&'static str> {
    match role {
        // Invitee = NONE: zero content (M-11 hard floor).
        RoleId::Invitee => BTreeSet::new(),
        RoleId::Viewer => VIEWER_ABILITIES.iter().copied().collect(),
        RoleId::Member => MEMBER_ABILITIES.iter().copied().collect(),
        RoleId::Moderator => MODERATOR_ABILITIES.iter().copied().collect(),
        RoleId::Admin => MODERATOR_ABILITIES
            .iter()
            .chain(ADMIN_EXCLUSIVE_ABILITIES.iter())
            .copied()
            .collect(),
    }
}

/// The five admin-exclusive governance abilities (the ruling-2 set). Exposed
/// so callers can assert the `Admin = Moderator ∪ exclusive` decomposition.
#[must_use]
pub fn admin_exclusive_abilities() -> BTreeSet<&'static str> {
    ADMIN_EXCLUSIVE_ABILITIES.iter().copied().collect()
}

/// The role-keyed content key-derivation gate. **Invitee derives NOTHING**
/// (M-11 zero-content floor); every content-bearing role derives a per-Node
/// membership key.
///
/// **What this actually computes (v1-beta — honest scope).** For a
/// content-bearing role this returns the membership `K(N)` computed by the
/// IN-CRATE BLAKE3 KDF [`crate::keying::derive_member_key`] —
/// `blake3::derive_key("benten-membership-set:K(V):v1", node_cid)` over the
/// **PUBLIC** `node_cid`. It is NOT a delegation to `benten-crypto-suite`, and
/// it is NOT the secret-keyed at-rest confidentiality key: the derivation is
/// keyed only by a public domain-context string + the public CID, so the
/// output is a **role-eligibility STAND-IN** — a deterministic per-Node handle
/// that keeps the membership-keying shape stable, gated by role. The
/// **confidentiality half** of the Principal primitive (secret-keyed
/// per-principal encryption) is DEFERRED per CLAUDE.md baked-in #18 — the LIVE
/// protection is the AUTHORITY half (role gate + capability/namespace
/// isolation), which binds a cooperating engine only. This gate encodes just
/// the role-eligibility rule (which roles may derive at all): `None` for
/// `Invitee`, `Some(key_material)` for any content-bearing role.
#[must_use]
pub fn derive_member_content_key(role: RoleId, node_cid: &[u8]) -> Option<Vec<u8>> {
    if matches!(role, RoleId::Invitee) {
        // Invitee = pre-acceptance handshake only: NO content key.
        return None;
    }
    Some(crate::keying::derive_member_key(node_cid))
}

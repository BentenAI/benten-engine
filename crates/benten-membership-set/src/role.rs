//! `RoleId` — the 5-value ordinal RBAC role + per-role UCAN ability-templates.
//!
//! # Ordinal golden vector (M-13 supersedes M-CONS-FINAL)
//!
//! `RoleId { Invitee = 0, Viewer = 1, Member = 2, Moderator = 3, Admin = 4 }`
//! — the ordinal is **keying-AAD-bound** (`role_assignments_generation` ∈ the
//! group AAD), so the exact byte values are golden-vector-pinned. M-13 swaps
//! the M-CONS-FINAL `Viewer=0/Invitee=1` to `Invitee=0` (the zero-content
//! floor) / `Viewer=1`. **All 5 active** at v1-beta (Inv-20 clause-j).
//!
//! # Canonical serialization (R0.5 §3.5 / F4-007)
//!
//! `RoleId` serializes as its `u8` ordinal in canonical-CBOR / AAD (via
//! `#[serde(into = "u8")]`) — an INTEGER discriminant, NOT a text string
//! (symmetric with `MemberRef`; determinism + AAD compactness).
//!
//! # RBAC (M-11 hard floor)
//!
//! - **Invitee (RoleId=0) derives ZERO content** — no `K(N)`, no read cap, and
//!   an EMPTY UCAN ability-template (pre-acceptance handshake state only).
//! - **Moderator ⊊ Admin** (strict subset): Admin = Moderator's
//!   `{read, write, share, moderate_content}` ∪ the 5 admin-exclusive
//!   governance abilities `{admit-member, kick-member, rotate-keys,
//!   assign-roles, edit-governance-config}` (Ben 2026-06-02 ruling 2).
//! - Semantics compose from **UCAN ability-tokens + the signed
//!   `GovernanceConfig`**, NOT a packed permission-flags bitfield (R0 §3.6.B).

use serde::Serialize;

extern crate alloc;
use alloc::collections::BTreeSet;

/// The RBAC role of a member — a 5-value ordinal (all active at v1-beta).
///
/// The discriminant is **keying-AAD-bound** (golden-vector-pinned). M-13:
/// `Invitee=0` is the zero-content floor (supersedes M-CONS-FINAL `Viewer=0`).
/// Serializes as its `u8` ordinal (R0.5 §3.5 / F4-007 int-tag ruling).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord, Serialize)]
#[serde(into = "u8")]
#[repr(u8)]
pub enum RoleId {
    /// Pre-acceptance handshake state — derives ZERO content (M-11 floor).
    Invitee = 0,
    /// Read-only.
    Viewer = 1,
    /// Read / write-own / share-within-policy.
    Member = 2,
    /// Read / write / share / moderate-content.
    Moderator = 3,
    /// Moderator's abilities ∪ the 5 admin-exclusive governance abilities.
    Admin = 4,
}

impl From<RoleId> for u8 {
    fn from(r: RoleId) -> u8 {
        r as u8
    }
}

/// The number of [`RoleId`] variants — all 5 active (Inv-20 clause-j).
pub const ROLE_VARIANT_COUNT: usize = 5;

impl RoleId {
    /// The stable wire ordinal (`#[repr(u8)]` discriminant) — AAD-keying-bound.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self as u8
    }

    /// All 5 roles, in ordinal order (ship-all-5-active).
    #[must_use]
    pub const fn all() -> [RoleId; ROLE_VARIANT_COUNT] {
        [
            RoleId::Invitee,
            RoleId::Viewer,
            RoleId::Member,
            RoleId::Moderator,
            RoleId::Admin,
        ]
    }

    /// Whether this role grants a read capability. **Invitee gets none**
    /// (pre-acceptance handshake only).
    #[must_use]
    pub const fn grants_read_cap(self) -> bool {
        !matches!(self, RoleId::Invitee)
    }

    /// Whether this role derives content keys (any per-Node `K(N)`).
    /// **Invitee derives ZERO content** (M-11 hard floor).
    #[must_use]
    pub const fn derives_content(self) -> bool {
        !matches!(self, RoleId::Invitee)
    }
}

// ── per-role UCAN ability-templates (the single source-of-truth) ──────────────

/// Viewer's frozen ability-token set (sorted).
pub const VIEWER_ABILITIES: [&str; 1] = ["read"];

/// Member's frozen ability-token set (sorted).
pub const MEMBER_ABILITIES: [&str; 3] = ["read", "share_within_policy", "write_own"];

/// Moderator's frozen ability-token set (sorted).
pub const MODERATOR_ABILITIES: [&str; 4] = ["moderate_content", "read", "share", "write"];

/// The five admin-exclusive governance abilities (ruling 2; restores
/// `assign-roles`). A Moderator must NEVER hold any of these. Sorted.
pub const ADMIN_EXCLUSIVE_ABILITIES: [&str; 5] = [
    "admit-member",
    "assign-roles",
    "edit-governance-config",
    "kick-member",
    "rotate-keys",
];

/// The UCAN ability-template for a role — a SET of named UCAN ability tokens
/// (which compose against the signed `GovernanceConfig`), NOT a packed
/// permission-flags bitfield (R0 §3.6.B grep-defense).
///
/// Admin is **derived** as `MODERATOR_ABILITIES ∪ ADMIN_EXCLUSIVE_ABILITIES`
/// (computed, never re-typed) so it can never drift from its two component
/// literals. Cardinalities: Invitee=0 / Viewer=1 / Member=3 / Moderator=4 /
/// Admin=9 (ruling 2).
#[must_use]
pub fn ability_template(role: RoleId) -> BTreeSet<&'static str> {
    match role {
        RoleId::Invitee => BTreeSet::new(), // NONE — zero content (M-11)
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

/// The cardinality of a role's UCAN ability-template (Invitee=0 / Viewer=1 /
/// Member=3 / Moderator=4 / Admin=9).
#[must_use]
pub fn ability_count(role: RoleId) -> usize {
    ability_template(role).len()
}

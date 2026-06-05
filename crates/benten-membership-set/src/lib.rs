//! `benten-membership-set` — the 15th workspace crate (the MembershipSet
//! keying primitive).
//!
//! A **thin keying-glue Rust engine plugin** (mechanism-half per GN-2 /
//! R0 §6.1). Its entire governance / audit / members / economics / federation
//! surface is **graph Nodes** (data-half); the Rust-mechanism that lives here
//! is the keying-frozen minimum.
//!
//! # What this crate owns (frozen Rust-mechanism)
//!
//! - The EXACTLY-3 [`kind::MembershipSetKind`] enum + its codepoint family
//!   (`0x6600` / `0x6610` / `0x6620`).
//! - The multi-stanza-HPKE keying glue ([`keying`]) — delegates ALL crypto
//!   primitives to `benten-crypto-suite` (**NEVER forks #5**; the ONLY-call-site
//!   discipline).
//! - The `members_table` snapshot canonical-CBOR serialization
//!   ([`aad::canonical_members_table_bytes`]; the AAD-bound keying minimum,
//!   NQ-W4) + the `0x6610` group **BLINDED 11-field** AAD assembly
//!   ([`aad::assemble_group_aad`]; R0.7 §3.10/§4.1) → **OPAQUE bytes** handed to
//!   the crypto-suite (m-15 GNC-5; no reverse dep).
//! - Per-Kind constructors + cardinality validation ([`set`]).
//! - The 5-value [`role::RoleId`] ordinal + per-role UCAN ability-templates
//!   (RBAC; Invitee derives ZERO content; Moderator ⊊ Admin).
//! - Stanza role-staleness verify ([`verify`]) + ephemeral-grant survival
//!   ([`ucan`]).
//!
//! # What lives as graph (data-half — NEVER inside the sealed Policy)
//!
//! `GovernanceConfig` / `RoleId` permission-semantics / Garden/Grove
//! sub-config / the members-table relation + derived nature (IVM views) / the
//! audit log / federation `SubsetRef` links / Compute + economics.
//!
//! # Crate boundary (F-CRATE-2)
//!
//! B-1 dep set `{crypto-suite, core, caps, id, graph, sync}` — every one is
//! UPSTREAM; NONE depend on `benten-membership-set` (no reverse edge). The
//! membership crate makes NO direct crypto-primitive construction (it never
//! names a hash / AEAD / KEM / signature primitive crate directly) — all
//! crypto delegates through `benten-crypto-suite` (the #5 ONLY-call-site
//! grep-defense; the `f_crate_1_2` pin enforces the forbidden symbol list).

#![forbid(unsafe_code)]

pub mod aad;
pub mod keying;
pub mod kind;
pub mod member;
pub mod role;
pub mod roster;
pub mod set;
pub mod ucan;
pub mod verify;

// ── flat re-exports of the frozen keying-mechanism surface ───────────────────

pub use crate::aad::{
    AAD_VERSION, GroupAadInputs, assemble_group_aad, audience_set_commitment, body_cid_bytes,
    canonical_members_table_bytes, membership_set_id_commitment, self_describing_cid_bytes,
};
pub use crate::keying::{
    KV_DERIVE_CONTEXT, MEMBER_CONTENT_DERIVE_CONTEXT, derive_member_content_key,
    derive_member_step_key, derive_version_node_key, grants_read_cap, role_derives_content,
};
pub use crate::kind::{
    KIND_VARIANT_COUNT, KindDispatchError, MEMBERSHIP_SET_BAND_HI, MEMBERSHIP_SET_BAND_LO,
    MEMBERSHIP_SET_BAND_RESERVE, MEMBERSHIP_SET_ENCRYPTION, MEMBERSHIP_SET_GROUP_MULTI_STANZA,
    MembershipSetKind, RequestedReserveKind, dispatch_reserve,
};
pub use crate::member::{MemberEntry, MemberRef, MembersTable, MembersTableError, SigPubKey};
pub use crate::role::{
    ADMIN_EXCLUSIVE_ABILITIES, MEMBER_ABILITIES, MODERATOR_ABILITIES, ROLE_VARIANT_COUNT, RoleId,
    VIEWER_ABILITIES, ability_count, ability_template,
};
pub use crate::set::{ConstructError, validate_construction, wire_cost_ceiling};
pub use crate::ucan::{DEFAULT_EXP_BOUND_SECS, EphemeralGrant};
pub use crate::verify::{RoleStaleError, Stanza, verify_stanza};

/// Crate scaffold marker — proves the 15th crate exists and names its codepoint
/// band (the F-CRATE-2 boundary pin floor).
pub mod scaffold {
    /// The crate's own name, asserted by the F-CRATE-2 boundary pin.
    pub const CRATE_NAME: &str = "benten-membership-set";

    /// The MembershipSet codepoint band lower bound (`0x6600`).
    pub const MEMBERSHIP_SET_BAND_LO: u16 = 0x6600;

    /// The MembershipSet codepoint band upper bound (`0x66FF`).
    pub const MEMBERSHIP_SET_BAND_HI: u16 = 0x66FF;
}

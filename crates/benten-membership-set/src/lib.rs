//! `benten-membership-set` — the 15th workspace crate (the MembershipSet
//! keying primitive).
//!
//! A **thin keying-glue Rust engine plugin** (mechanism-half per GN-2 / R0
//! §6.1). Its entire governance / audit / members / economics / federation
//! surface is **graph Nodes** (data-half); the Rust-mechanism that lives here
//! is the keying-frozen minimum.
//!
//! # What this crate owns (frozen Rust-mechanism)
//!
//! - The EXACTLY-3 [`kind::MembershipSetKind`] enum + its codepoint family
//!   (`0x6600` / `0x6610` / `0x6620` — [`codepoints`]).
//! - The multi-stanza-HPKE keying glue ([`keying`]) — delegates every crypto
//!   primitive to `benten-crypto-suite` (the #5 ONLY-call-site; **NEVER forks
//!   #5**).
//! - The fused per-DID [`member::MemberEntry`] record + [`member::MembersTable`]
//!   (one-DID-one-record; ZERO nature field — Inv-22).
//! - The `members_table` snapshot canonical-CBOR serialization
//!   ([`aad::canonical_members_table_bytes`]; the AAD-bound keying minimum,
//!   NQ-W4).
//! - The per-Kind cardinality constructors ([`set::MembershipSet`]).
//! - The `0x6610` group AAD = the BLINDED 11-field set assembly
//!   ([`aad::assemble_group_aad`]; R0.7 §3.10/§4.1) → **OPAQUE bytes** handed
//!   to the crypto-suite (m-15 GNC-5; no reverse dep).
//! - The RBAC [`role::RoleId`] (5-value ordinal, all-active) + per-role UCAN
//!   ability-templates ([`role::ability_template`]).
//! - The role-staleness verify gate ([`verify::verify_stanza`] →
//!   `E_ROLE_STALE_AT_VERIFY`) + ephemeral UCAN grants ([`ucan`]).
//!
//! # What lives as graph (data-half — NEVER inside the sealed Policy)
//!
//! `GovernanceConfig` / `RoleId` permission-*semantics* (UCAN templates) /
//! Garden/Grove sub-config / the members-table relation + derived nature (IVM
//! views) / the audit log / federation `SubsetRef` links / Compute + economics.
//!
//! # Dependency direction (F-CRATE-2)
//!
//! The B-1 dep set is `{crypto-suite, core, caps, id, graph, sync}` — every one
//! UPSTREAM; **none of them depend on this crate** (the no-reverse-edge
//! compile-fence). The AAD assembler hands OPAQUE bytes across the crypto-suite
//! seam (the crypto-suite never sees this crate's types).

#![forbid(unsafe_code)]

// NATIVE-ONLY per CLAUDE.md baked-in #17. The crate's B-1 dep set includes
// `benten-sync` (iroh transport + Loro CRDT — native-only), so the keying
// primitive is itself a full-peer native mechanism. Browser tabs / thin-client
// surfaces do NOT carry the MembershipSet keying glue (the membership-set
// snapshot reaches them via the thin-client protocol, not the in-bundle crate).
// This guard fires before the transitive `benten-sync` guard for a clearer
// diagnostic.
#[cfg(target_arch = "wasm32")]
compile_error!(
    "benten-membership-set is native-only per CLAUDE.md baked-in #17 (its B-1 \
     dep set includes the native-only benten-sync). Browser/thin-client surfaces \
     receive the materialized membership-set snapshot via the D-PHASE-3-30 \
     thin-client protocol, NOT the in-bundle keying crate."
);

pub mod aad;
pub mod audit;
pub mod codepoints;
pub mod error;
pub mod federation;
pub mod governance;
pub mod keying;
pub mod keying_kv;
pub mod kind;
pub mod member;
pub mod privacy;
pub mod role;
pub mod set;
pub mod ucan;
pub mod verify;

pub use crate::error::{E_ROLE_STALE_AT_VERIFY, MembershipSetError};
pub use crate::kind::{
    KindDispatchError, MembershipSetKind, RequestedReserveKind, dispatch_reserve,
};
pub use crate::member::{
    Did, Hlc, MemberEntry, MemberNature, MemberRef, MembersTable, RoleId, SigPubKey,
    derive_member_nature, is_ai_operated,
};
pub use crate::set::{MembershipSet, wire_cost_ceiling};

/// Crate scaffold marker — proves the 15th crate exists and is in the
/// workspace graph (the F-CRATE-2 "crate exists" pin floor). Retained from the
/// R3-W4 scaffold; the authoritative codepoint constants now live in
/// [`codepoints`].
pub mod scaffold {
    /// The crate's own name, asserted by the F-CRATE-2 boundary pin so the
    /// 15th-crate skeleton is observable.
    pub const CRATE_NAME: &str = "benten-membership-set";

    /// The MembershipSet codepoint band lower bound (`0x6600`).
    pub const MEMBERSHIP_SET_BAND_LO: u16 = crate::codepoints::MEMBERSHIP_SET_BAND_LO;

    /// The MembershipSet codepoint band upper bound (`0x66FF`).
    pub const MEMBERSHIP_SET_BAND_HI: u16 = crate::codepoints::MEMBERSHIP_SET_BAND_HI;
}

//! The fused `members_table` + the canonical per-DID `MemberEntry` record.
//!
//! `members_table: BTreeMap<Did, MemberEntry>` **fuses** members + authorities
//! + role-assignments by construction: one DID → exactly one record (the
//! `BTreeMap` key uniqueness IS the cross-table invariant — Inv-20 clause-i).
//! Coupling rule: `is_authority ⟹ sig_pubkey.is_some()`. There is **ZERO**
//! `member_type` / nature field on [`MemberEntry`] — nature is DERIVED
//! (Inv-22), never stored. The snapshot is the CURRENT-materialization of the
//! membership event version-chain.
//!
//! ## Canonical R0.5 §3.5 shapes (byte-frozen — F4-006/F4-007)
//!
//! - [`MemberEntry`] is the EXACTLY-5-field
//!   `{ role, is_authority, sig_pubkey, admitted_at_hlc, member_ref }`.
//! - [`Hlc`] is the 3-field `{ physical_ms, logical, node_id }` member-property
//!   clock (NOT the Inv-21 `created_at_hlc`).
//! - [`RoleId`] and [`MemberRef`] BOTH serialize as INTEGER discriminants in
//!   canonical-CBOR/AAD (R4.2 F4-007 ruling) — `RoleId` as its `u8` ordinal,
//!   `MemberRef` as a `u8`-tagged variant (NOT a text string). The integer
//!   discriminant is the AAD-keying-bound canonical wire form (determinism +
//!   AAD compactness).
//!
//! The canonical serialization order is NOT the struct's field-DECLARATION
//! order: `MemberEntry` serializes as a CBOR MAP, and the canonical DAG-CBOR
//! encoder ([`crate::aad::canonical_members_table_bytes`]) sorts map keys by
//! the canonical rule (length-first, then bytewise) — so the frozen on-wire
//! key order is `role`, `member_ref`, `sig_pubkey`, `is_authority`,
//! `admitted_at_hlc`. That KEY-SORTED byte order (not the declaration order)
//! is what the `f_aad_1` byte-pin freezes; the declaration order below is
//! free to differ.

use crate::error::MembershipSetError;
use serde::Serialize;
use std::collections::BTreeMap;

pub use crate::role::RoleId;

/// A content-addressed DID, ordered for `BTreeMap` keying. At v1-beta this is
/// the `benten_id::did::Did` string form; the canonical byte shape (a CBOR
/// text string) is what the `members_table` byte-pin freezes.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Did(pub String);

impl Did {
    /// Construct a `Did` from any string-like.
    pub fn new(s: impl Into<String>) -> Self {
        Did(s.into())
    }

    /// The DID string bytes (used by the AAD blinding commitments).
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// The `admitted_at_hlc` member-property clock — the canonical R0.5 §3.5
/// 3-field shape `{ physical_ms, logical, node_id }`. Every integer is
/// authored big-endian on the wire (M-19). This is a member-property clock,
/// NOT the Inv-21 `created_at_hlc`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Hlc {
    /// The physical-clock component (ms).
    pub physical_ms: u64,
    /// The logical (Lamport) component.
    pub logical: u32,
    /// The originating node id.
    pub node_id: u64,
}

impl Hlc {
    /// Construct an `Hlc` with `logical = 0` and `node_id = 0` (the common
    /// member-admit shape).
    #[must_use]
    pub fn at(physical_ms: u64) -> Self {
        Hlc {
            physical_ms,
            logical: 0,
            node_id: 0,
        }
    }
}

/// A signing public key. `Option<SigPubKey>` presence is the authority
/// discriminant (`is_authority ⟹ sig_pubkey.is_some()`). Serialized as a CBOR
/// byte string (`serde_bytes`) so the presence-encoding is the canonical
/// length-injective shape the byte-pin freezes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SigPubKey(#[serde(with = "serde_bytes")] pub Vec<u8>);

/// The KEYING / FEDERATION axis member reference — **Kind-determined**
/// (homogeneous-per-Kind), NOT a per-member nature discriminator (m-15 GNC-7
/// / Inv-22 boundary).
///
/// Serialized as a `u8`-tagged INTEGER variant (R0.5 §3.5 F4-007 ruling),
/// symmetric with [`RoleId`] — NOT a CBOR text string. Tags: `UserDid=0`,
/// `DeviceDid=1`, `LocalDevice=2`; tag `3` is reserved for a future
/// `SubsetRef` (federation). The integer discriminant is the AAD-keying-bound
/// canonical wire form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(into = "u8")]
#[repr(u8)]
pub enum MemberRef {
    /// Atrium members are user-DIDs.
    UserDid = 0,
    /// DeviceMesh members are device-DIDs (carry a DeviceAttestation at R5).
    DeviceDid = 1,
    /// SingleDevice's sole member is the local device.
    LocalDevice = 2,
    // tag 3 reserved for a future `SubsetRef` (R0.5 §3.5 federation).
}

impl From<MemberRef> for u8 {
    fn from(m: MemberRef) -> u8 {
        m as u8
    }
}

impl MemberRef {
    /// The Kind a given `MemberRef` is *only* valid inside (Kind-determined;
    /// m-15 GNC-7). The constructor enforces this coupling.
    #[must_use]
    pub fn required_kind(&self) -> crate::kind::MembershipSetKind {
        use crate::kind::MembershipSetKind;
        match self {
            MemberRef::UserDid => MembershipSetKind::Atrium,
            MemberRef::DeviceDid => MembershipSetKind::DeviceMesh,
            MemberRef::LocalDevice => MembershipSetKind::SingleDevice,
        }
    }
}

/// The per-DID fused membership record — the canonical R0.5 §3.5 **5-field**
/// shape.
///
/// **ZERO nature field by construction** — `is_ai` / `is_plugin` /
/// `member_type` are DERIVED (Inv-22), never stored. The fields are DECLARED
/// `role`, `is_authority`, `sig_pubkey`, `admitted_at_hlc`, `member_ref`, but
/// this struct serializes as a CBOR MAP whose keys the canonical DAG-CBOR
/// encoder sorts (length-first, then bytewise). The frozen on-wire KEY-SORTED
/// order the `f_aad_1` byte-pin freezes is therefore `role`, `member_ref`,
/// `sig_pubkey`, `is_authority`, `admitted_at_hlc` — NOT the declaration
/// order; reordering the declaration below is byte-neutral.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MemberEntry {
    /// The governance-axis role (F-MS-4 owns the ordinal golden vector).
    pub role: RoleId,
    /// Fused authorities-table membership (coupling: `⟹ sig_pubkey.is_some()`).
    pub is_authority: bool,
    /// Present IFF `is_authority` (the fused authorities coupling rule).
    pub sig_pubkey: Option<SigPubKey>,
    /// The member-property admit clock (NOT the Inv-21 `created_at_hlc`).
    pub admitted_at_hlc: Hlc,
    /// The KEYING axis (Kind-determined; NOT nature).
    pub member_ref: MemberRef,
    // NO `member_type` / `nature` / `is_ai` field — nature is DERIVED (Inv-22).
}

impl MemberEntry {
    /// Validate the fused-record coupling rule (`is_authority ⟹
    /// sig_pubkey.is_some()`).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipSetError::AuthorityMissingPubkey`] if
    /// `is_authority` is set but `sig_pubkey` is absent.
    pub fn validate(&self) -> Result<(), MembershipSetError> {
        if self.is_authority && self.sig_pubkey.is_none() {
            return Err(MembershipSetError::AuthorityMissingPubkey);
        }
        Ok(())
    }

    /// Inv-22 struct-fence introspection: does [`MemberEntry`] carry ANY
    /// stored nature discriminator (`member_type` / `MemberKind` / `is_ai` /
    /// `member_nature`)?
    ///
    /// The answer is **always `false` by construction** — the canonical
    /// 5-field shape `{ role, is_authority, sig_pubkey, admitted_at_hlc,
    /// member_ref }` carries ZERO nature field. Nature is DERIVED
    /// ([`derive_member_nature`]), never stored (Inv-22).
    ///
    /// **F-12 (R12) attribution:** this is a NON-load-bearing NAMING helper —
    /// a `const` literal `false`. It does NOT itself detect a reintroduced
    /// nature field (a literal cannot). The **load-bearing Inv-22 enforcement**
    /// is the EXHAUSTIVE DESTRUCTURE
    /// `let MemberEntry { role, is_authority, sig_pubkey, admitted_at_hlc,
    /// member_ref } = &e;` (NO `..` rest) in
    /// `crates/benten-membership-set/tests/f_ms_3_members_table_fusion.rs` —
    /// adding a 6th (nature) field breaks that pattern at compile time. This
    /// const exists only so callers/tests can NAME the "zero stored nature
    /// field" property; the destructure is what would-FAIL on a reintroduced
    /// discriminator.
    #[must_use]
    pub const fn has_any_nature_field() -> bool {
        false
    }
}

/// The DERIVED member nature — an IVM-materialized view, NEVER an
/// authoritative stored field (Inv-22).
///
/// Every flag is recomputed from the (DID-method, install-manifest-presence)
/// inputs by [`derive_member_nature`]; none of them is read from a
/// `MemberEntry` field (there is none — see [`MemberEntry::has_any_nature_field`]).
/// The struct exists only as the return shape of the derivation; it is never
/// serialized into the canonical `members_table` (the wire shape stays the
/// frozen 5-field record).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemberNature {
    /// Whether the member is AI-operated — DERIVED purely from
    /// `did.method() == "agent"` (an optional allowlist alias, not a stored
    /// discriminator; see [`is_ai_operated`]).
    pub is_ai_operated: bool,
    /// Whether the member is a plugin — DERIVED from install-manifest
    /// presence (Inv-14 / manifest), not a stored flag.
    pub is_plugin: bool,
    /// Whether the member is an autonomous AI — DERIVED from
    /// (AI-operated AND plugin-backed), composing the two derivations rather
    /// than reading a stored field.
    pub is_autonomous_ai: bool,
}

/// Derive whether a member is AI-operated PURELY from the DID method string.
///
/// `is_ai_operated(method) = (method == "agent")` — the `did:agent:` method is
/// an optional allowlist ALIAS, NOT a stored discriminator. A `did:key:`
/// member parses to method `"key"` → `false`. The derivation reads NOTHING
/// from any stored member field (Inv-22): the method-parse of the member's DID
/// IS the derivation.
///
/// The caller passes the already-parsed method segment of the DID
/// (`did:<method>:<id>`), which they obtain from
/// `benten_id::did::Did::as_str().split(':').nth(1)` or the engine's DID
/// accessor.
#[must_use]
pub fn is_ai_operated(did_method: &str) -> bool {
    did_method == "agent"
}

/// Recompute the full [`MemberNature`] IVM view from its derivation inputs.
///
/// The nature is a MATERIALIZED VIEW (Inv-22): it is recomputed from
/// `(did_method, has_install_manifest)` and is never read from an
/// authoritative stored field. Identical inputs yield an identical
/// `MemberNature` (deterministic derivation), which is exactly what makes it a
/// view rather than mutable state.
///
/// - `is_ai_operated` = [`is_ai_operated`]`(did_method)` (method == "agent").
/// - `is_plugin` = `has_install_manifest` (Inv-14 / manifest presence).
/// - `is_autonomous_ai` = `is_ai_operated && is_plugin` (an AI-operated
///   member backed by an install manifest is autonomous).
#[must_use]
pub fn derive_member_nature(did_method: &str, has_install_manifest: bool) -> MemberNature {
    let ai = is_ai_operated(did_method);
    MemberNature {
        is_ai_operated: ai,
        is_plugin: has_install_manifest,
        is_autonomous_ai: ai && has_install_manifest,
    }
}

/// The fused `members_table`. The `BTreeMap<Did, MemberEntry>` key uniqueness
/// IS the one-DID-one-record invariant — there is NO parallel
/// authorities/role-assignments table to drift against (Inv-20 clause-i).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MembersTable {
    table: BTreeMap<Did, MemberEntry>,
}

impl MembersTable {
    /// An empty members table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Admit (or re-admit) a member: validates the coupling rule, then
    /// upserts. Re-admitting an existing DID OVERWRITES the one record — it
    /// never creates a second record (one-DID-one-record).
    ///
    /// # Errors
    ///
    /// Returns [`MembershipSetError::AuthorityMissingPubkey`] if the entry
    /// violates the authority/pubkey coupling rule.
    pub fn admit(&mut self, did: Did, entry: MemberEntry) -> Result<(), MembershipSetError> {
        entry.validate()?;
        self.table.insert(did, entry);
        Ok(())
    }

    /// The number of distinct member DIDs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// The record for a DID, if present.
    #[must_use]
    pub fn get(&self, did: &Did) -> Option<&MemberEntry> {
        self.table.get(did)
    }

    /// The underlying `BTreeMap` (for canonical-CBOR serialization).
    #[must_use]
    pub fn as_map(&self) -> &BTreeMap<Did, MemberEntry> {
        &self.table
    }
}

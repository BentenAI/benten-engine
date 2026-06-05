//! `MemberRef` / `MemberEntry` / `MembersTable` — the fused per-DID membership
//! snapshot (the AAD-keying-bound minimum; NQ-W4).
//!
//! # The fusion invariant (Inv-20 clause-i)
//!
//! `members_table: BTreeMap<Did, MemberEntry>` **fuses** members + authorities
//! + role-assignments by construction: one DID → exactly one record (the
//! `BTreeMap` key uniqueness IS the cross-table invariant — there is no
//! parallel authorities/role-assignments table to drift against).
//!
//! # Coupling + nature (m-15 GNC-7 / Inv-22)
//!
//! - `is_authority ⟹ sig_pubkey.is_some()` (the fused-authorities coupling).
//! - There is **ZERO** `member_type` / nature / `is_ai` field on `MemberEntry`
//!   — nature is DERIVED (Inv-22), never stored.
//! - `MemberRef` is **Kind-determined** (`UserDid` ↔ Atrium / `DeviceDid` ↔
//!   DeviceMesh / `LocalDevice` ↔ SingleDevice) — the KEYING / FEDERATION axis,
//!   NOT a per-member nature discriminator.
//!
//! # Canonical serialization (R0.5 §3.5 / F4-006 / F4-007)
//!
//! The canonical R0.5 §3.5 5-field `MemberEntry` shape
//! `{ role, is_authority, sig_pubkey, admitted_at_hlc, member_ref }` with the
//! 3-field HLC, serialized to canonical DAG-CBOR (sorted map keys, integer
//! discriminants for `RoleId` + `MemberRef`). Field ORDER is load-bearing —
//! the canonical serialization order R5 must preserve (the F-AAD-1 golden
//! freezes the exact bytes). `member_ref` is an INTEGER discriminant
//! (UserDid=0 / DeviceDid=1 / LocalDevice=2; tag 3 reserved for a future
//! `SubsetRef`), symmetric with the `role` ordinal.

use benten_core::hlc::BentenHlc;
use benten_id::did::Did;
use serde::Serialize;

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::kind::MembershipSetKind;
use crate::role::RoleId;

/// A signing public key — present iff the member `is_authority`. The
/// `Option<SigPubKey>` presence-encoding is one of the three NQ-W4 drift risks
/// (serialized via `serde_bytes` as a CBOR byte-string).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SigPubKey(#[serde(with = "serde_bytes")] pub Vec<u8>);

/// The KEYING / FEDERATION axis of a member — **Kind-determined**,
/// homogeneous-per-Kind, NOT a nature discriminator (m-15 GNC-7 / Inv-22).
///
/// Serialized as a `u8`-tagged integer variant (R0.5 §3.5 / F4-007), symmetric
/// with [`RoleId`] — NOT a CBOR text string. Tags: `UserDid=0`, `DeviceDid=1`,
/// `LocalDevice=2`; tag `3` is reserved for a future `SubsetRef` (federation).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(into = "u8")]
#[repr(u8)]
pub enum MemberRef {
    /// Atrium members are user-DIDs.
    UserDid = 0,
    /// DeviceMesh members are device-DIDs (carry a DeviceAttestation at the
    /// admission path).
    DeviceDid = 1,
    /// SingleDevice's sole member is the local device.
    LocalDevice = 2,
    // tag 3 reserved for a future `SubsetRef` (federation; R0.5 §3.5).
}

impl From<MemberRef> for u8 {
    fn from(m: MemberRef) -> u8 {
        m as u8
    }
}

impl MemberRef {
    /// The [`MembershipSetKind`] a given `MemberRef` is *only* valid inside
    /// (the Kind↔MemberRef coupling — m-15 GNC-7).
    #[must_use]
    pub const fn required_kind(self) -> MembershipSetKind {
        match self {
            MemberRef::UserDid => MembershipSetKind::Atrium,
            MemberRef::DeviceDid => MembershipSetKind::DeviceMesh,
            MemberRef::LocalDevice => MembershipSetKind::SingleDevice,
        }
    }

    /// The stable wire ordinal (`#[repr(u8)]` discriminant) — AAD-keying-bound.
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        self as u8
    }
}

/// Serializable mirror of [`BentenHlc`]'s `(physical_ms, logical, node_id)`
/// triple. `BentenHlc` itself does not derive `Serialize` (it is the engine's
/// runtime HLC value type); the canonical `admitted_at_hlc` member-property
/// clock is serialized through this 3-field mirror so the byte shape matches
/// the frozen F-AAD-1 golden (DAG-CBOR sorts the keys to
/// `logical`/`node_id`/`physical_ms`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
struct HlcWire {
    physical_ms: u64,
    logical: u32,
    node_id: u64,
}

impl From<&BentenHlc> for HlcWire {
    fn from(h: &BentenHlc) -> Self {
        HlcWire {
            physical_ms: h.physical_ms(),
            logical: h.logical(),
            node_id: h.node_id(),
        }
    }
}

fn serialize_hlc<S>(hlc: &BentenHlc, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    HlcWire::from(hlc).serialize(s)
}

/// The per-DID fused record — the canonical R0.5 §3.5 5-field shape.
///
/// **ZERO nature field by construction** — `is_ai` / `is_plugin` / member_type
/// are DERIVED (Inv-22), never stored. Field ORDER is the load-bearing
/// canonical serialization order (the F-AAD-1 golden freezes these bytes).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MemberEntry {
    /// The member's RBAC role (governance axis; serializes as its `u8`
    /// ordinal).
    pub role: RoleId,
    /// Fused authorities-table membership. `true ⟹ sig_pubkey.is_some()`.
    pub is_authority: bool,
    /// Present IFF `is_authority` (the coupling rule).
    pub sig_pubkey: Option<SigPubKey>,
    /// The member-property admission clock (NOT the Inv-21 `created_at_hlc`).
    /// Serialized through the [`HlcWire`] mirror.
    #[serde(serialize_with = "serialize_hlc")]
    pub admitted_at_hlc: BentenHlc,
    /// The KEYING axis (NOT nature) — serializes as its `u8` ordinal.
    pub member_ref: MemberRef,
    // NO `member_type` / `nature` / `is_ai` field — nature is DERIVED (Inv-22).
}

impl MemberEntry {
    /// Validate the fused-record coupling rule (`is_authority ⟹ sig_pubkey`).
    ///
    /// # Errors
    ///
    /// Returns [`MembersTableError::AuthorityMissingPubkey`] when
    /// `is_authority` is set but `sig_pubkey` is absent.
    pub fn validate(&self) -> Result<(), MembersTableError> {
        if self.is_authority && self.sig_pubkey.is_none() {
            return Err(MembersTableError::AuthorityMissingPubkey);
        }
        Ok(())
    }
}

/// An error admitting a record into a [`MembersTable`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MembersTableError {
    /// `is_authority=true` but `sig_pubkey` absent (the coupling rule).
    AuthorityMissingPubkey,
}

/// The fused `members_table`. The `BTreeMap<Did, MemberEntry>` key uniqueness
/// IS the one-DID-one-record invariant (Inv-20 clause-i) — there is no parallel
/// authorities/role-assignments table to drift against. The `BTreeMap`
/// additionally fixes iteration order at the type level, so the canonical-CBOR
/// bytes are independent of insertion order (the cross-engine convergence
/// guarantee).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct MembersTable {
    table: BTreeMap<Did, MemberEntry>,
}

impl MembersTable {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self {
            table: BTreeMap::new(),
        }
    }

    /// Admit (upsert) a record. Validates the coupling rule first, then
    /// inserts. Re-admitting an existing DID **OVERWRITES** (one record) — it
    /// never creates a second record (the CURRENT-materialization semantics).
    ///
    /// # Errors
    ///
    /// Returns [`MembersTableError::AuthorityMissingPubkey`] if the entry sets
    /// `is_authority` without a `sig_pubkey` (the rejected entry never lands).
    pub fn admit(&mut self, did: Did, entry: MemberEntry) -> Result<(), MembersTableError> {
        entry.validate()?;
        self.table.insert(did, entry);
        Ok(())
    }

    /// Remove a member by DID (returns the removed record if present).
    pub fn remove(&mut self, did: &Did) -> Option<MemberEntry> {
        self.table.remove(did)
    }

    /// The number of distinct members (one DID → one record).
    #[must_use]
    pub fn len(&self) -> usize {
        self.table.len()
    }

    /// Whether the table is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }

    /// Look up a member's record by DID.
    #[must_use]
    pub fn get(&self, did: &Did) -> Option<&MemberEntry> {
        self.table.get(did)
    }

    /// Borrow the underlying fused map (for canonical-bytes assembly /
    /// IVM-view materialization).
    #[must_use]
    pub fn as_map(&self) -> &BTreeMap<Did, MemberEntry> {
        &self.table
    }
}

//! Per-Kind [`MembershipSet`] constructors with cardinality validation.
//!
//! Cardinality rules (R0 §3.6.A / §3.7 / Inv-22 / Compromise #46):
//!
//! - **Atrium**: ≥1 admin; members are `UserDid`; ≤32 members (#46 ceiling).
//! - **DeviceMesh**: exactly-1 user-DID admin; members are `DeviceDid`; ≤5
//!   members (#46 ceiling).
//! - **SingleDevice**: exactly-1 self-admin; exactly-1 `LocalDevice` member.
//!
//! `MemberRef` is **Kind-determined** (`UserDid` ↔ Atrium / `DeviceDid` ↔
//! DeviceMesh / `LocalDevice` ↔ SingleDevice), NOT a per-member nature
//! discriminator (m-15 GNC-7 / Inv-22). The per-Kind wire-cost ceiling
//! (Compromise #46) is the O(N) multi-stanza wire-cost cap.

use crate::error::MembershipSetError;
use crate::kind::MembershipSetKind;
use crate::member::MemberRef;

/// The per-Kind member-count ceiling — Compromise #46 (Atrium 32 /
/// DeviceMesh 5 / SingleDevice 1). Wire-cost is O(N) in members
/// (multi-stanza-HPKE emits one stanza per member).
#[must_use]
pub fn wire_cost_ceiling(kind: MembershipSetKind) -> usize {
    match kind {
        MembershipSetKind::Atrium => 32,
        MembershipSetKind::DeviceMesh => 5,
        MembershipSetKind::SingleDevice => 1,
    }
}

/// A constructed MembershipSet of a given Kind. The full members-table /
/// keying state hangs off this at v1-beta; the canary surface is the validated
/// Kind + the cardinality contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MembershipSet {
    kind: MembershipSetKind,
    admin_count: usize,
    member_refs: Vec<MemberRef>,
}

impl MembershipSet {
    /// Construct a MembershipSet of `kind` with `admin_count` admins and the
    /// given member references, validating per-Kind cardinality + the
    /// Kind↔MemberRef coupling + the #46 wire-cost ceiling.
    ///
    /// # Errors
    ///
    /// - [`MembershipSetError::MemberRefKindMismatch`] if any member's
    ///   `MemberRef` is not valid for `kind`.
    /// - [`MembershipSetError::AdminCardinality`] if the per-Kind admin count
    ///   is violated (Atrium ≥1; DeviceMesh/SingleDevice exactly-1).
    /// - [`MembershipSetError::MemberCardinality`] if a SingleDevice has ≠1
    ///   member.
    /// - [`MembershipSetError::WireCostCeilingExceeded`] if the member count
    ///   exceeds the per-Kind #46 ceiling.
    pub fn construct(
        kind: MembershipSetKind,
        admin_count: usize,
        member_refs: &[MemberRef],
    ) -> Result<Self, MembershipSetError> {
        // Kind↔MemberRef coupling: every member's ref must be valid for `kind`.
        if member_refs.iter().any(|m| m.required_kind() != kind) {
            return Err(MembershipSetError::MemberRefKindMismatch);
        }
        // Per-Kind admin + member cardinality (the exact-count rules take
        // precedence over the generic wire-cost ceiling).
        match kind {
            MembershipSetKind::Atrium => {
                if admin_count < 1 {
                    return Err(MembershipSetError::AdminCardinality);
                }
            }
            MembershipSetKind::DeviceMesh => {
                if admin_count != 1 {
                    return Err(MembershipSetError::AdminCardinality);
                }
            }
            MembershipSetKind::SingleDevice => {
                if admin_count != 1 {
                    return Err(MembershipSetError::AdminCardinality);
                }
                // SingleDevice is exactly-1 member — checked as a cardinality
                // constraint BEFORE the generic ceiling (both reject a
                // 2-member SingleDevice, but the semantic error is
                // MemberCardinality).
                if member_refs.len() != 1 {
                    return Err(MembershipSetError::MemberCardinality);
                }
            }
        }
        // Per-Kind wire-cost ceiling (#46) — the upper bound for the
        // variable-cardinality Kinds (Atrium 32 / DeviceMesh 5).
        if member_refs.len() > wire_cost_ceiling(kind) {
            return Err(MembershipSetError::WireCostCeilingExceeded);
        }
        Ok(MembershipSet {
            kind,
            admin_count,
            member_refs: member_refs.to_vec(),
        })
    }

    /// Construct an Atrium (≥1 admin; user-DID members).
    ///
    /// # Errors
    ///
    /// See [`MembershipSet::construct`].
    pub fn new_atrium(
        admin_count: usize,
        member_refs: &[MemberRef],
    ) -> Result<Self, MembershipSetError> {
        Self::construct(MembershipSetKind::Atrium, admin_count, member_refs)
    }

    /// Construct a DeviceMesh (exactly-1 user-DID admin; device-DID members).
    ///
    /// # Errors
    ///
    /// See [`MembershipSet::construct`].
    pub fn new_device_mesh(
        admin_count: usize,
        member_refs: &[MemberRef],
    ) -> Result<Self, MembershipSetError> {
        Self::construct(MembershipSetKind::DeviceMesh, admin_count, member_refs)
    }

    /// Construct a SingleDevice (exactly-1 self-admin; exactly-1 local-device
    /// member).
    ///
    /// # Errors
    ///
    /// See [`MembershipSet::construct`].
    pub fn new_single_device(
        admin_count: usize,
        member_refs: &[MemberRef],
    ) -> Result<Self, MembershipSetError> {
        Self::construct(MembershipSetKind::SingleDevice, admin_count, member_refs)
    }

    /// The Kind of this set.
    #[must_use]
    pub fn kind(&self) -> MembershipSetKind {
        self.kind
    }

    /// The admin count.
    #[must_use]
    pub fn admin_count(&self) -> usize {
        self.admin_count
    }

    /// The member references.
    #[must_use]
    pub fn member_refs(&self) -> &[MemberRef] {
        &self.member_refs
    }
}

/// The MembershipSet CRDT convergence rules (M-7 / M-8 / M-10 / Inv-21).
///
/// Two object classes co-exist in one merge round, in DELIBERATELY OPPOSITE
/// directions (the load-bearing M-7 asymmetry):
///
/// - **member PROPERTY** (e.g. `admitted_at_hlc` / role) resolves by
///   **larger-HLC-wins** (last-writer-wins LWW;
///   [`admitted_at_hlc_lww_keeps_a`](crdt::admitted_at_hlc_lww_keeps_a)).
/// - **set-IDENTITY fork** (Inv-21) resolves by **smaller-`created_at_hlc`-wins**
///   = oldest-anchor-wins, made TOTAL via the forking-event Version-Node CID
///   ([`fork_total_order_key`](crdt::fork_total_order_key) /
///   [`fork_a_wins`](crdt::fork_a_wins)).
///
/// The HLC discriminant is the real [`benten_core::hlc::BentenHlc`] (lexicographic
/// `(physical_ms, logical, node_id)`); the Version-Node CID is the immutable
/// content-addressed fork-event identifier (the `MembershipSetId` can NEVER
/// disambiguate concurrent same-anchor forks — they share it).
pub mod crdt {
    use benten_core::hlc::BentenHlc;

    /// Member-PROPERTY LWW: keep `a`'s value iff its HLC is `>=` `b`'s
    /// (larger-HLC-wins; ties broken by the total `(physical_ms, logical,
    /// node_id)` lex order, so the result is deterministic + commutative).
    /// Returns `true` if `a` wins. This is the `admitted_at_hlc` property rule
    /// (the in-tree `crdt.rs` LWW applied to the membership property clock).
    #[must_use]
    pub fn admitted_at_hlc_lww_keeps_a(a: BentenHlc, b: BentenHlc) -> bool {
        a >= b
    }

    /// The Inv-21 fork TOTAL-ordering key (M-8): `(created_at_hlc ASC, then
    /// fork_event_version_node_cid ASC)`. NOT keyed on `MembershipSetId`
    /// (concurrent same-anchor forks share it). The CID is the terminal
    /// discriminator that makes the order total.
    #[must_use]
    pub fn fork_total_order_key(
        created_at_hlc: BentenHlc,
        fork_event_version_node_cid: &[u8],
    ) -> (BentenHlc, Vec<u8>) {
        (created_at_hlc, fork_event_version_node_cid.to_vec())
    }

    /// The Inv-21 fork tie-break: fork `a` wins iff its total-order key is
    /// `<=` fork `b`'s — **SMALLER key wins** (oldest-anchor; the DELIBERATE
    /// opposite of property LWW). Order-independent (antisymmetric): the same
    /// fork wins regardless of argument order, because distinct fork events have
    /// distinct CIDs so the keys are never equal. An adversarial larger-HLC
    /// re-fork can NEVER displace the original.
    #[must_use]
    pub fn fork_a_wins(
        a_created_at_hlc: BentenHlc,
        a_fork_event_version_node_cid: &[u8],
        b_created_at_hlc: BentenHlc,
        b_fork_event_version_node_cid: &[u8],
    ) -> bool {
        fork_total_order_key(a_created_at_hlc, a_fork_event_version_node_cid)
            <= fork_total_order_key(b_created_at_hlc, b_fork_event_version_node_cid)
    }
}

//! `MembershipSet` construction — per-Kind cardinality + wire-cost ceiling +
//! Kind↔MemberRef coupling (F-MS-2 / Compromise #46 / Inv-22).
//!
//! Per-Kind cardinality at construction:
//! - **Atrium** ≥1 admin; member ceiling 32 (Compromise #46).
//! - **DeviceMesh** exactly-1 user-DID admin; member ceiling 5.
//! - **SingleDevice** exactly-1 self-admin; exactly-1 member (ceiling 1).
//!
//! `MemberRef` is **Kind-determined** (every member's ref must be valid for the
//! set's Kind), NOT a per-member nature discriminator (m-15 GNC-7).

use crate::kind::MembershipSetKind;
use crate::member::MemberRef;

/// A construction error (per-Kind cardinality / coupling / wire-cost).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConstructError {
    /// Atrium needs ≥1 admin; DeviceMesh/SingleDevice need exactly-1.
    AdminCardinality,
    /// SingleDevice is exactly-1 member.
    MemberCardinality,
    /// A `MemberRef` variant not valid for this Kind (the Kind↔MemberRef
    /// coupling — e.g. `DeviceDid` in an Atrium).
    MemberRefKindMismatch,
    /// Per-Kind member ceiling (Compromise #46) exceeded.
    WireCostCeilingExceeded,
}

/// The per-Kind member-count ceiling — Compromise #46 (Atrium 32 / DeviceMesh 5
/// / SingleDevice 1). Wire-cost is O(N) in members (multi-stanza HPKE).
#[must_use]
pub const fn wire_cost_ceiling(kind: MembershipSetKind) -> usize {
    match kind {
        MembershipSetKind::Atrium => 32,
        MembershipSetKind::DeviceMesh => 5,
        MembershipSetKind::SingleDevice => 1,
    }
}

/// Validate per-Kind construction cardinality + the Kind↔MemberRef coupling +
/// the wire-cost ceiling.
///
/// # Errors
///
/// - [`ConstructError::MemberRefKindMismatch`] if any member's `MemberRef` is
///   not valid for `kind`.
/// - [`ConstructError::AdminCardinality`] if the admin count violates the
///   per-Kind rule (Atrium ≥1, DeviceMesh/SingleDevice exactly-1).
/// - [`ConstructError::MemberCardinality`] if a SingleDevice has ≠1 member.
/// - [`ConstructError::WireCostCeilingExceeded`] if member count exceeds the
///   per-Kind #46 ceiling.
pub fn validate_construction(
    kind: MembershipSetKind,
    admin_count: usize,
    member_refs: &[MemberRef],
) -> Result<(), ConstructError> {
    // Kind↔MemberRef coupling: every member's ref must be valid for this Kind.
    if member_refs.iter().any(|m| m.required_kind() != kind) {
        return Err(ConstructError::MemberRefKindMismatch);
    }
    // Per-Kind admin + member cardinality (the exact-count rules take
    // precedence over the generic wire-cost ceiling).
    match kind {
        MembershipSetKind::Atrium => {
            if admin_count < 1 {
                return Err(ConstructError::AdminCardinality);
            }
        }
        MembershipSetKind::DeviceMesh => {
            if admin_count != 1 {
                return Err(ConstructError::AdminCardinality);
            }
        }
        MembershipSetKind::SingleDevice => {
            if admin_count != 1 {
                return Err(ConstructError::AdminCardinality);
            }
            // SingleDevice is exactly-1 member — checked as a cardinality
            // constraint BEFORE the generic ceiling.
            if member_refs.len() != 1 {
                return Err(ConstructError::MemberCardinality);
            }
        }
    }
    // Per-Kind wire-cost ceiling (#46).
    if member_refs.len() > wire_cost_ceiling(kind) {
        return Err(ConstructError::WireCostCeilingExceeded);
    }
    Ok(())
}

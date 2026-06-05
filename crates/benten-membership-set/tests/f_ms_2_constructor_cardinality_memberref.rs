//! **F-MS-2** — Per-Kind constructor cardinality + Kind-determined
//! `MemberRef`.
//!
//! ADDL R3 (TDD RED-phase) test-writer — Phase-4-Meta-Core F-full Wave
//! R3-W4, family **F-MS-2** (merges A2 + WF-E3 + GNI-1).
//!
//! # What this pins (r2-test-landscape §1 Group 9 / R0 §3.6.A + §3.7 +
//! Inv-22 + Compromise #46)
//!
//! - Per-Kind cardinality at construction: **Atrium ≥1 admin**; **DeviceMesh
//!   exactly-1 user-DID admin**; **SingleDevice exactly-1 self-admin**.
//! - `MemberRef` is **Kind-determined** (`UserDid` ↔ Atrium / `DeviceDid` ↔
//!   DeviceMesh / `LocalDevice` ↔ SingleDevice), NOT a per-member nature
//!   discriminator (m-15 GNC-7 / Inv-22 boundary).
//! - Per-Kind wire-cost ceiling (Compromise #46: Atrium 32 / DeviceMesh 5 /
//!   SingleDevice 1).
//!
//! Red-phase intent: 0-admin Atrium / 2-admin DeviceMesh / 2-member
//! SingleDevice all `Err`; `MemberRef::DeviceDid` only valid inside a
//! DeviceMesh.
//!
//! # RED-PHASE status (pim-12 §3.6e)
//!
//! Self-contained in-file stub-shim; compiles green at baseline behind
//! `#[ignore]`. R5 replaces the shim with
//! `benten_membership_set::{kind::MembershipSetKind, member::MemberRef,
//! set::MembershipSet}` and un-ignores. The cardinality rules + the
//! Kind↔MemberRef coupling are the frozen surface (would-FAIL-if-no-op'd: a
//! constructor that admits 0 Atrium admins, 2 DeviceMesh admins, or a
//! `DeviceDid` in a SingleDevice set breaks a pin).

#![allow(dead_code)]

// ── R5: the real crate surface (the in-file stub-shim is deleted) ───────────
//
// `MembershipSetKind` / `MemberRef` / `wire_cost_ceiling` / the per-Kind
// constructor (`MembershipSet::construct`) are now the real
// `benten_membership_set` surface. The construction error maps 1:1 to a local
// `ConstructError` so the pin bodies that compare error variants stay intact —
// but each `construct()` call drives the REAL `MembershipSet::construct`
// production entry point (would-FAIL-if-reverted).
use benten_membership_set::MembershipSetError;
use benten_membership_set::kind::MembershipSetKind;
use benten_membership_set::member::MemberRef;
use benten_membership_set::set::{MembershipSet, wire_cost_ceiling};

/// 1:1 mirror of the construction-time `MembershipSetError` variants this file
/// exercises — kept local so the pin bodies compare named error variants while
/// the wrapper below drives the real constructor.
#[derive(Debug, PartialEq, Eq)]
enum ConstructError {
    /// Atrium needs ≥1 admin; DeviceMesh/SingleDevice need exactly-1.
    AdminCardinality,
    /// SingleDevice is exactly-1 member.
    MemberCardinality,
    /// A `MemberRef` variant not valid for this Kind (e.g. DeviceDid in Atrium).
    MemberRefKindMismatch,
    /// Per-Kind member ceiling (Compromise #46) exceeded.
    WireCostCeilingExceeded,
}

impl From<MembershipSetError> for ConstructError {
    fn from(e: MembershipSetError) -> Self {
        match e {
            MembershipSetError::AdminCardinality => ConstructError::AdminCardinality,
            MembershipSetError::MemberCardinality => ConstructError::MemberCardinality,
            MembershipSetError::MemberRefKindMismatch => ConstructError::MemberRefKindMismatch,
            MembershipSetError::WireCostCeilingExceeded => ConstructError::WireCostCeilingExceeded,
            other => panic!("unexpected construction error in F-MS-2: {other:?}"),
        }
    }
}

/// Drive the REAL per-Kind constructor (`MembershipSet::construct`), mapping
/// the production `MembershipSetError` to the local `ConstructError`. A
/// constructor that admitted 0 Atrium admins / 2 DeviceMesh admins / a
/// `DeviceDid` in a SingleDevice set would FAIL the pins.
fn construct(
    kind: MembershipSetKind,
    admin_count: usize,
    member_refs: &[MemberRef],
) -> Result<(), ConstructError> {
    MembershipSet::construct(kind, admin_count, member_refs)
        .map(|_| ())
        .map_err(ConstructError::from)
}

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
fn ms2_admin_cardinality_per_kind() {
    // Positive: 1-admin Atrium succeeds.
    assert!(construct(MembershipSetKind::Atrium, 1, &[MemberRef::UserDid]).is_ok());
    // Negative: 0-admin Atrium rejects (would-FAIL if constructor skipped the check).
    assert_eq!(
        construct(MembershipSetKind::Atrium, 0, &[MemberRef::UserDid]),
        Err(ConstructError::AdminCardinality),
        "Atrium requires ≥1 admin"
    );
    // Negative: 2-admin DeviceMesh rejects (exactly-1 user-DID admin).
    assert_eq!(
        construct(
            MembershipSetKind::DeviceMesh,
            2,
            &[MemberRef::DeviceDid, MemberRef::DeviceDid]
        ),
        Err(ConstructError::AdminCardinality),
        "DeviceMesh requires exactly-1 admin"
    );
    // Positive: 1-admin DeviceMesh succeeds.
    assert!(construct(MembershipSetKind::DeviceMesh, 1, &[MemberRef::DeviceDid]).is_ok());
}

#[test]
fn ms2_single_device_exactly_one_member() {
    // Positive: exactly-1 LocalDevice member, 1 admin.
    assert!(
        construct(
            MembershipSetKind::SingleDevice,
            1,
            &[MemberRef::LocalDevice]
        )
        .is_ok()
    );
    // Negative: 2-member SingleDevice rejects.
    assert_eq!(
        construct(
            MembershipSetKind::SingleDevice,
            1,
            &[MemberRef::LocalDevice, MemberRef::LocalDevice]
        ),
        Err(ConstructError::MemberCardinality),
        "SingleDevice is exactly-1 member"
    );
}

#[test]
fn ms2_memberref_is_kind_determined() {
    // `DeviceDid` is valid ONLY in a DeviceMesh — a DeviceDid in an Atrium
    // rejects (the Kind↔MemberRef coupling, m-15 GNC-7). Would-FAIL if the
    // constructor treated MemberRef as a free per-member nature field.
    assert_eq!(
        construct(MembershipSetKind::Atrium, 1, &[MemberRef::DeviceDid]),
        Err(ConstructError::MemberRefKindMismatch),
        "DeviceDid MemberRef is only valid inside a DeviceMesh"
    );
    // `UserDid` is valid ONLY in an Atrium.
    assert_eq!(
        construct(MembershipSetKind::DeviceMesh, 1, &[MemberRef::UserDid]),
        Err(ConstructError::MemberRefKindMismatch),
        "UserDid MemberRef is only valid inside an Atrium"
    );
    // Coupling is total: each MemberRef declares exactly one required Kind.
    assert_eq!(
        MemberRef::UserDid.required_kind(),
        MembershipSetKind::Atrium
    );
    assert_eq!(
        MemberRef::DeviceDid.required_kind(),
        MembershipSetKind::DeviceMesh
    );
    assert_eq!(
        MemberRef::LocalDevice.required_kind(),
        MembershipSetKind::SingleDevice
    );
}

#[test]
fn ms2_per_kind_wire_cost_ceiling() {
    // The #46 ceilings are the frozen per-Kind member caps.
    assert_eq!(wire_cost_ceiling(MembershipSetKind::Atrium), 32);
    assert_eq!(wire_cost_ceiling(MembershipSetKind::DeviceMesh), 5);
    assert_eq!(wire_cost_ceiling(MembershipSetKind::SingleDevice), 1);
    // A DeviceMesh with 6 members exceeds the #46 ceiling of 5.
    let six_devices = vec![MemberRef::DeviceDid; 6];
    assert_eq!(
        construct(MembershipSetKind::DeviceMesh, 1, &six_devices),
        Err(ConstructError::WireCostCeilingExceeded),
        "DeviceMesh members are capped at 5 (Compromise #46 O(N) multi-stanza wire cost)"
    );
    // A 5-member DeviceMesh is at the ceiling and admits.
    let five_devices = vec![MemberRef::DeviceDid; 5];
    assert!(construct(MembershipSetKind::DeviceMesh, 1, &five_devices).is_ok());
}

//! **F-MS-2** — Per-Kind constructor cardinality + Kind-determined
//! `MemberRef`.
//!
//! ADDL R5 (impl-to-green) — Phase-4-Meta-Core F-full Wave w-ms-canary,
//! family **F-MS-2** (merges A2 + WF-E3 + GNI-1).
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
//! # R5 (un-ignored against `benten_membership_set::{kind, member, set}`)
//!
//! Drives the real `validate_construction` + `MemberRef` coupling. Would-FAIL:
//! a constructor that admits 0 Atrium admins, 2 DeviceMesh admins, or a
//! `DeviceDid` in a SingleDevice set breaks a pin.

use benten_membership_set::kind::MembershipSetKind;
use benten_membership_set::member::MemberRef;
use benten_membership_set::set::{ConstructError, validate_construction, wire_cost_ceiling};

// ── pins ────────────────────────────────────────────────────────────────────

#[test]
fn ms2_admin_cardinality_per_kind() {
    // Positive: 1-admin Atrium succeeds.
    assert!(validate_construction(MembershipSetKind::Atrium, 1, &[MemberRef::UserDid]).is_ok());
    // Negative: 0-admin Atrium rejects (would-FAIL if constructor skipped the check).
    assert_eq!(
        validate_construction(MembershipSetKind::Atrium, 0, &[MemberRef::UserDid]),
        Err(ConstructError::AdminCardinality),
        "Atrium requires ≥1 admin"
    );
    // Negative: 2-admin DeviceMesh rejects (exactly-1 user-DID admin).
    assert_eq!(
        validate_construction(
            MembershipSetKind::DeviceMesh,
            2,
            &[MemberRef::DeviceDid, MemberRef::DeviceDid]
        ),
        Err(ConstructError::AdminCardinality),
        "DeviceMesh requires exactly-1 admin"
    );
    // Positive: 1-admin DeviceMesh succeeds.
    assert!(
        validate_construction(MembershipSetKind::DeviceMesh, 1, &[MemberRef::DeviceDid]).is_ok()
    );
}

#[test]
fn ms2_single_device_exactly_one_member() {
    // Positive: exactly-1 LocalDevice member, 1 admin.
    assert!(
        validate_construction(
            MembershipSetKind::SingleDevice,
            1,
            &[MemberRef::LocalDevice]
        )
        .is_ok()
    );
    // Negative: 2-member SingleDevice rejects.
    assert_eq!(
        validate_construction(
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
        validate_construction(MembershipSetKind::Atrium, 1, &[MemberRef::DeviceDid]),
        Err(ConstructError::MemberRefKindMismatch),
        "DeviceDid MemberRef is only valid inside a DeviceMesh"
    );
    // `UserDid` is valid ONLY in an Atrium.
    assert_eq!(
        validate_construction(MembershipSetKind::DeviceMesh, 1, &[MemberRef::UserDid]),
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
        validate_construction(MembershipSetKind::DeviceMesh, 1, &six_devices),
        Err(ConstructError::WireCostCeilingExceeded),
        "DeviceMesh members are capped at 5 (Compromise #46 O(N) multi-stanza wire cost)"
    );
    // A 5-member DeviceMesh is at the ceiling and admits.
    let five_devices = vec![MemberRef::DeviceDid; 5];
    assert!(validate_construction(MembershipSetKind::DeviceMesh, 1, &five_devices).is_ok());
}

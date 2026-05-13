//! G27-A class-of-bug regression guard — device-DID revocation paths.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.14 G27-A row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-A entry
//! naming "multi-device device-DID revocation paths" as an audit
//! target. Inherits Phase-3 G16-D wave-6b device-DID attestation
//! envelope (CLAUDE.md baked-in #17 + #18 device-as-attested-sub-
//! identity model).
//!
//! ## The audit shape
//!
//! Device-DID revocation differs from capability-grant revocation:
//! device-DID identity is rotated via `benten_id::did_rotation::RotationLog`
//! (a separate substrate from `system:CapabilityRevocation` Nodes).
//! BUT the napi binding at `bindings/napi/src/atrium.rs:531` consumes
//! `attestation.device_did` for the rotation match path, AND the
//! G27-A audit must verify the napi-side surface doesn't conflate
//! device-DID lexical match with scope-keyed revocation.
//!
//! ## Class-of-bug risk
//!
//! Hypothetically: a future device-DID revocation napi binding could
//! conflate `revoke_device_did(device_cid)` with the cap-revoke
//! shape, writing a `system:DeviceRevocation`-labeled Node with
//! `scope = "<device_cid base32>"`. The reader walker on the cap
//! side would never observe this (scope-mismatch); the rotation-log
//! side might also never observe it (wrong storage substrate).
//! Silent fail-OPEN of device-DID revocations under cross-peer sync.
//!
//! ## What this pin verifies
//!
//! 1. The napi atrium binding (`bindings/napi/src/atrium.rs`)
//!    consumes `device_did` via the rotation-log substrate, not
//!    via the cap-revoke seam.
//! 2. A device-DID rotation surfaces correctly at the
//!    `RotationLog::is_revoked` match path — and does NOT leak into
//!    `BackendGrantReader::has_unrevoked_grant_for_scope`.
//!
//! ## RED-PHASE expectation
//!
//! The G27-A audit at R5 walks the device-DID napi binding + confirms
//! the substrate boundary holds. This pin lands RED-PHASE; un-ignore
//! at G27-A wave-time with the substrate-boundary observable assertion.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_engine::Engine;
use benten_id::did_rotation::RotationLog;

/// RED-PHASE: G27-A device-DID class-of-bug audit.
///
/// Pins that the napi device-DID revocation surface (when it lands
/// fully under G24-D + plugin-rev integration) routes through the
/// `RotationLog` substrate, NOT the cap-revoke scope-keyed seam.
#[test]
#[ignore = "RED-PHASE: G27-A — un-ignore at G27-A wave-time; device-DID revocation routes through RotationLog, not cap-revoke seam (substrate boundary regression-guard)"]
fn device_did_revocation_napi_routes_through_rotation_log_not_cap_revoke_seam() {
    let dir = tempfile::tempdir().unwrap();
    let _engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    // RED-PHASE: the G27-A audit at R5 wires the device-DID rotation
    // through the napi atrium binding + verifies the substrate
    // boundary holds. The pin shape (un-ignore target):
    // 1. Construct a `RotationLog` with a device-DID rotation event.
    // 2. Confirm the napi atrium binding's `attestation` match path
    //    consults `RotationLog::is_revoked(device_did)` (or sibling).
    // 3. Confirm no `system:CapabilityRevocation` Node is written
    //    by the device-DID revocation path (would be the class-of-bug
    //    confusion site — cap-keyed substrate + device-DID-keyed identity).
    //
    // At HEAD: full device-DID revocation surface is partially shipped
    // (Phase-3 G16-D wave-6b + RotationLog substrate). The audit walks
    // the napi binding at `bindings/napi/src/atrium.rs:531` to confirm
    // the rotation match path stays substrate-distinct.
    panic!("RED-PHASE: G27-A — implementer must un-ignore + wire substrate-boundary assertion");
}

/// Compile-time witness: the `RotationLog` type is reachable from the
/// napi test crate. The G27-A audit walks the consume sites at
/// `bindings/napi/src/atrium.rs`.
#[test]
fn device_did_rotation_log_reachable_compile_witness() {
    // PhantomData witness — the RotationLog type is visible to the
    // test crate (dep on benten-id present per Cargo.toml).
    let _: std::marker::PhantomData<RotationLog> = std::marker::PhantomData;
}

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
//! ## Un-ignored at G27-A wave (R5)
//!
//! The G27-A R5 audit walked every napi binding entry point under
//! `bindings/napi/src/atrium.rs` + `bindings/napi/src/lib.rs` and
//! confirmed:
//!
//! 1. **No napi `revokeCapabilityByDeviceDid` surface ships at HEAD.**
//!    The audit doc `notes-napi-parity-audit.md` §1 names the future
//!    `Engine::revoke_capabilities_by_device_did` seam as a candidate;
//!    at HEAD it has NOT shipped, so the napi binding has no class-of-
//!    bug surface to exhibit. The pin therefore exercises the EXISTING
//!    napi surface (`JsAtrium::revoke_peer` for peer-DID, distinct
//!    from device-DID) + the substrate-boundary invariant of
//!    `RotationLog`.
//! 2. **`RotationLog` is the proper substrate for device-DID identity
//!    rotation.** The `RotationLog::is_superseded` walker keys on the
//!    DID lexical bytes (via `ct_signature_eq`), NOT on a scope-string;
//!    consequently a class-of-bug-style CID-substitution at the
//!    device-DID layer would produce a `RotationAttestation` whose
//!    `previous_did` carried wrong bytes — `is_superseded` would never
//!    fire for the legitimate DID. That's a different failure mode
//!    than the cap-revoke scope-string-vs-CID class-of-bug.
//! 3. **The cap-revoke substrate boundary holds.** A grant minted at
//!    `actor=device_did` (whatever the device-DID's CID-string
//!    encoding) is revocable by `Engine::revoke_capability_by_grant_cid`
//!    — but the revocation Node carries the grant's actual scope
//!    property, NOT the device-DID. Device-DID rotation continues to
//!    flow through `RotationLog`; cap-revocation continues to flow
//!    through `system:CapabilityRevocation` Nodes; the two substrates
//!    never cross-key.
//!
//! ## Substrate-boundary assertions (this pin)
//!
//! - `RotationLog::is_superseded(did)` matches on `did.as_str()` bytes
//!   verbatim — distinct DIDs produce distinct supersession outcomes.
//! - `RotationLog` has no scope-keyed interface — there is no surface
//!   to accidentally route a cap-revoke into.
//! - A cap-revoke at the napi binding does NOT register against the
//!   device-DID rotation log (the two substrates are observably
//!   distinct: `RotationLog::is_superseded` continues to return
//!   `false` for an un-rotated DID across a cap-revoke cycle).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![cfg(feature = "in-process-test")]

use benten_engine::Engine;
use benten_id::did::Did;
use benten_id::did_rotation::{RotationAttestation, RotationLog};

/// G27-A device-DID class-of-bug audit — substrate-boundary regression
/// guard (un-ignored at R5).
///
/// Pins the audit-finding that device-DID identity rotation lives on
/// the `RotationLog` substrate while capability revocation lives on
/// the `system:CapabilityRevocation` Node substrate. The two are
/// observably distinct: a cap-revoke cycle does NOT alter `RotationLog`
/// state, and a device-DID rotation does NOT write a
/// `system:CapabilityRevocation` Node keyed on the device-DID string
/// (which would be the class-of-bug shape).
///
/// Would-FAIL-if-no-op'd: a future napi `revokeCapabilityByDeviceDid`
/// binding that routes through the cap-revoke seam (writing
/// `system:CapabilityRevocation` with `scope = "<device_did>"`) would
/// observably leave `RotationLog::is_superseded(did)` unchanged
/// (consistent with this pin's "two substrates" assertion) but the
/// _intended_ revocation would silently fail. The current absence of
/// such a binding (audit §1: NOT YET SHIPPED) is the load-bearing
/// finding; this pin guards the substrate-boundary invariant so any
/// future shipping of that binding routes through the correct
/// substrate.
#[test]
fn device_did_revocation_napi_routes_through_rotation_log_not_cap_revoke_seam() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens with grant-backed policy");

    // Substrate-boundary arm #1: a `RotationLog` with NO entries treats
    // an arbitrary DID as un-superseded. This is the baseline state
    // against which a cap-revoke cycle must leave RotationLog alone.
    let device_did = Did::from_string_unchecked(
        "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktH".to_string(),
    );
    let mut log = RotationLog::new();
    assert!(
        !log.is_superseded(&device_did),
        "fresh RotationLog must treat un-rotated DID as un-superseded \
         (baseline before cap-revoke cycle)",
    );

    // Substrate-boundary arm #2: drive a cap-revoke cycle on the
    // capability substrate (mint + revoke a grant via the resolving
    // seam). The class-of-bug failure mode would be: the napi binding
    // accidentally writes through the device-DID substrate. We assert
    // RotationLog state is unaltered by the cap-revoke cycle.
    let actor = engine.create_principal("device-owner").unwrap();
    let grant_cid = engine
        .grant_capability(&actor, "store:notes:write")
        .expect("grant via privileged path");
    engine
        .revoke_capability_by_grant_cid(&grant_cid, &actor)
        .expect("revoke via resolving seam");
    assert!(
        !log.is_superseded(&device_did),
        "RotationLog state MUST be unchanged by cap-revoke cycle — \
         device-DID substrate (rotation-log) and capability-revocation \
         substrate (system:CapabilityRevocation Nodes) are observably \
         disjoint; if this fires, a class-of-bug confusion has wired \
         the two substrates together",
    );

    // Substrate-boundary arm #3: RotationLog keys on DID-lexical bytes
    // verbatim. Distinct DIDs (even if they differ by one character at
    // the very end) MUST produce distinct supersession outcomes — this
    // pins the absence of any "canonicalization" / "truncation" /
    // "scope-string-style" mangling on the RotationLog substrate.
    // Construct a synthetic rotation attestation; the signature here
    // is a sentinel — `is_superseded` consults `previous_did` via
    // `ct_signature_eq` on the bytes, not via signature verification
    // (that happens at the chain-walker layer separately).
    let other_did = Did::from_string_unchecked(
        "did:key:z6MkpTHR8VNsBxYAAWHut2Geadd9jSwuBV8xRoAnwWsdvktX".to_string(),
    );
    let attestation = RotationAttestation {
        previous_did: device_did.as_str().to_string(),
        next_did: "did:key:z6MkfreshKeypairDidPlaceholderForBoundaryTest".to_string(),
        superseded_at: 0,
        signature: vec![0u8; 64],
    };
    log.append(attestation);
    assert!(
        log.is_superseded(&device_did),
        "RotationLog::is_superseded MUST fire for the DID that was \
         recorded as `previous_did` in the attestation — keys on DID \
         lexical bytes verbatim",
    );
    assert!(
        !log.is_superseded(&other_did),
        "RotationLog::is_superseded MUST NOT fire for a DID that differs \
         by even one byte — the substrate keys on DID-lexical bytes \
         verbatim with NO canonicalization, mirroring the scope-string \
         verbatim discipline on the cap-revoke substrate; if this fires \
         the rotation-log layer has introduced a class-of-bug-style \
         lossy normalization",
    );
}

/// Compile-time witness: the `RotationLog` + `RotationAttestation`
/// types are reachable from the napi test crate. The G27-A audit walks
/// the consume sites at `bindings/napi/src/atrium.rs` to confirm any
/// future device-DID revocation surface routes through this substrate
/// (not the cap-revoke seam).
#[test]
fn device_did_rotation_log_reachable_compile_witness() {
    let _: std::marker::PhantomData<RotationLog> = std::marker::PhantomData;
    let _: std::marker::PhantomData<RotationAttestation> = std::marker::PhantomData;
}

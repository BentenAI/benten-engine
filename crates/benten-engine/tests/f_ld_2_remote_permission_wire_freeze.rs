//! F-LD-2 — Remote-permission wire freeze `PermissionRequest`/`PermissionGrant`
//! (RED-PHASE; byte-pinning — V2 + BE + canonical-CBOR from first commit per M-20).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-2: "e2r §6.4 structs freeze; operation enum
//!     `Decrypt|SignUcanDelegation|RemoteUnlock|ExecuteWorkflow`; `0x6320..0x632F`.
//!     golden CBOR per struct; sig round-trip; unknown-operation typed-reject."
//!   - R0.3 plan §3.4 (`...f-full-r0-plan.md:504-508`): the `PermissionRequest`
//!     / `PermissionGrant` wire shape; "signed by B" / "signed by A's
//!     user-DID-signing-key".
//!   - §4.1 codepoint table (`...:820-821`): `0x6310..0x631F` DeviceLink band,
//!     `0x6320..0x632F` RemotePermission band — FREEZE.
//!
//! ## RED-PHASE + byte-pinning (pim-12 §3.6e + M-20)
//!
//! SELF-CONTAINED stub-shim models the e2r §6.4 wire structs with **big-endian**
//! integer encoding + canonical DAG-CBOR + V2 framing from the first commit.
//! There is NO LE / V1 golden vector here. At R5 the shim is deleted and the
//! real `benten_engine` Layer-D remote-permission types are `use`d.

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

use benten_id::keypair::Keypair;

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — e2r §6.4 wire structs (V2 + BE + canonical-CBOR).
// ---------------------------------------------------------------------------
mod shim {
    /// Wire-format version. Wave-0 (M-20): V2 from the first commit. NEVER V1.
    pub const REMOTE_PERMISSION_WIRE_VERSION: u8 = 2;

    /// RemotePermission band codepoint base (§4.1 `0x6320..0x632F` FREEZE).
    pub const REMOTE_PERMISSION_BAND_BASE: u16 = 0x6320;
    pub const REMOTE_PERMISSION_BAND_END: u16 = 0x632F;

    /// The operation enum (e2r §6.4). The `ExecuteWorkflow` variant slot is
    /// frozen at v1-beta (F-LD-3 owns its AAD-binding pins); runtime
    /// enforcement is post-v1-beta.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PermissionOperation {
        Decrypt { node_cid: [u8; 32] },
        SignUcanDelegation { audience: Vec<u8>, expires_at: u64 },
        RemoteUnlock,
        ExecuteWorkflow { workflow_cid: [u8; 32] },
    }

    impl PermissionOperation {
        /// BE-encoded discriminant tag + body. Wave-0 M-20: every integer is
        /// big-endian on the wire. would-FAIL if any field switches to LE.
        pub fn to_wire_be(&self) -> Vec<u8> {
            let mut out = Vec::new();
            match self {
                Self::Decrypt { node_cid } => {
                    out.push(0x00);
                    out.extend_from_slice(node_cid);
                }
                Self::SignUcanDelegation {
                    audience,
                    expires_at,
                } => {
                    out.push(0x01);
                    // length-prefix BE, then BE u64 expiry.
                    out.extend_from_slice(&(audience.len() as u32).to_be_bytes());
                    out.extend_from_slice(audience);
                    out.extend_from_slice(&expires_at.to_be_bytes());
                }
                Self::RemoteUnlock => out.push(0x02),
                Self::ExecuteWorkflow { workflow_cid } => {
                    out.push(0x03);
                    out.extend_from_slice(workflow_cid);
                }
            }
            out
        }
    }

    /// e2r §6.4 — signed by requesting device B.
    pub struct PermissionRequest {
        pub version: u8,
        pub request_id: [u8; 16],
        pub requesting_device_pubkey: [u8; 32],
        pub operation: PermissionOperation,
        pub timestamp_bucket: u64, // coarse 1-hr bucket (F-LD-8 owns granularity)
        pub nonce: [u8; 32],
        pub signature: Vec<u8>,
    }

    /// e2r §6.4 — signed by approving device A's user-DID signing key.
    pub struct PermissionGrant {
        pub version: u8,
        pub request_id: [u8; 16],
        pub granted_at_bucket: u64,
        pub valid_until: u64,
        pub operation_result: Vec<u8>,
        /// MUST be present + resolvable (F-LD-6 pass-class 6 audit-binding).
        pub audit_node_cid: [u8; 32],
        pub signature: Vec<u8>,
    }

    impl PermissionRequest {
        /// Canonical signing bytes (BE integers, V2 framing, deterministic
        /// field order — the freeze contract).
        pub fn signing_bytes(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.push(self.version);
            b.extend_from_slice(&self.request_id);
            b.extend_from_slice(&self.requesting_device_pubkey);
            b.extend_from_slice(&self.operation.to_wire_be());
            b.extend_from_slice(&self.timestamp_bucket.to_be_bytes());
            b.extend_from_slice(&self.nonce);
            b
        }
    }

    impl PermissionGrant {
        pub fn signing_bytes(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.push(self.version);
            b.extend_from_slice(&self.request_id);
            b.extend_from_slice(&self.granted_at_bucket.to_be_bytes());
            b.extend_from_slice(&self.valid_until.to_be_bytes());
            b.extend_from_slice(&self.audit_node_cid);
            b.extend_from_slice(&self.operation_result);
            b
        }
    }

    /// Dispatch a raw RemotePermission codepoint. Out-of-band integers
    /// typed-reject (no silent fallback; CLAUDE.md #5).
    #[derive(Debug)]
    pub struct UnsupportedOperationCodepoint(pub u16);

    pub fn dispatch_remote_permission_codepoint(
        cp: u16,
    ) -> Result<(), UnsupportedOperationCodepoint> {
        if (REMOTE_PERMISSION_BAND_BASE..=REMOTE_PERMISSION_BAND_END).contains(&cp) {
            Ok(())
        } else {
            Err(UnsupportedOperationCodepoint(cp))
        }
    }
}

use shim::{
    dispatch_remote_permission_codepoint, PermissionGrant, PermissionOperation, PermissionRequest,
    REMOTE_PERMISSION_BAND_BASE, REMOTE_PERMISSION_WIRE_VERSION,
};

/// F-LD-2 PermissionRequest sig round-trip: B signs the canonical (BE/V2)
/// bytes; the pubkey-on-wire verifies. would-FAIL-if-no-op'd: tampering ANY
/// field (operation, nonce, timestamp) changes `signing_bytes` and the
/// signature no longer verifies.
#[test]
#[ignore = "RED-PHASE: F-LD-2 — PermissionRequest BE/V2 canonical sig round-trip; un-ignore at R5"]
fn f_ld_2_permission_request_signature_round_trips_over_canonical_be_bytes() {
    let device_b = Keypair::generate();
    let mut req = PermissionRequest {
        version: REMOTE_PERMISSION_WIRE_VERSION,
        request_id: [7u8; 16],
        requesting_device_pubkey: device_b.public_key().to_bytes(),
        operation: PermissionOperation::Decrypt { node_cid: [9u8; 32] },
        timestamp_bucket: 1_900_000_800, // bucket-aligned (F-LD-8)
        nonce: [3u8; 32],
        signature: Vec::new(),
    };
    let sig = device_b.sign(&req.signing_bytes());
    req.signature = sig.to_bytes().to_vec();

    // V2 framing pin (M-20 — never V1).
    assert_eq!(req.version, 2, "remote-permission wire is V2 from first commit");

    // Verify the on-wire pubkey validates the signature over the canonical bytes.
    let verify_pk = device_b.public_key();
    verify_pk
        .verify(&req.signing_bytes(), &sig)
        .expect("PermissionRequest signature MUST verify over canonical BE/V2 bytes");

    // would-FAIL-if-no-op'd: mutate the operation → signature breaks.
    let tampered = PermissionRequest {
        operation: PermissionOperation::RemoteUnlock,
        ..PermissionRequest {
            version: req.version,
            request_id: req.request_id,
            requesting_device_pubkey: req.requesting_device_pubkey,
            operation: req.operation.clone(),
            timestamp_bucket: req.timestamp_bucket,
            nonce: req.nonce,
            signature: req.signature.clone(),
        }
    };
    assert!(
        verify_pk.verify(&tampered.signing_bytes(), &sig).is_err(),
        "mutating the operation MUST break the signature (field is bound)"
    );
}

/// F-LD-2 PermissionGrant sig round-trip: A's user-DID key signs the grant;
/// `audit_node_cid` is part of the signed bytes (so a "grant without audit"
/// can't be forged by stripping the field — couples F-LD-6 class 6).
#[test]
#[ignore = "RED-PHASE: F-LD-2 — PermissionGrant BE/V2 sig round-trip binds audit_node_cid; un-ignore at R5"]
fn f_ld_2_permission_grant_signature_binds_audit_node_cid() {
    let device_a_user_did = Keypair::generate();
    let mut grant = PermissionGrant {
        version: REMOTE_PERMISSION_WIRE_VERSION,
        request_id: [7u8; 16],
        granted_at_bucket: 1_900_000_800,
        valid_until: 1_900_000_860,
        operation_result: b"hpke-wrapped-key-material".to_vec(),
        audit_node_cid: [0xAB; 32],
        signature: Vec::new(),
    };
    let sig = device_a_user_did.sign(&grant.signing_bytes());
    grant.signature = sig.to_bytes().to_vec();

    device_a_user_did
        .public_key()
        .verify(&grant.signing_bytes(), &sig)
        .expect("PermissionGrant signature MUST verify");

    // would-FAIL-if-no-op'd: zero the audit_node_cid (simulate a stripped
    // audit trail) → signing bytes change → signature breaks.
    let mut stripped = grant;
    stripped.audit_node_cid = [0u8; 32];
    assert!(
        device_a_user_did
            .public_key()
            .verify(&stripped.signing_bytes(), &sig)
            .is_err(),
        "stripping audit_node_cid MUST break the grant signature"
    );
}

/// F-LD-2 golden BE byte-pin: the operation wire encoding is big-endian +
/// the discriminant tags are frozen. A future LE regression OR a tag
/// reordering flips these hex pins. (M-20 / M-19 BE conformance gate.)
#[test]
#[ignore = "RED-PHASE: F-LD-2 — operation BE wire-encoding golden byte-pin; un-ignore at R5"]
fn f_ld_2_operation_wire_encoding_is_big_endian_golden_pin() {
    // SignUcanDelegation with expires_at = 0x0102_0304_0506_0708 → MUST appear
    // big-endian (01 02 03 04 05 06 07 08), NOT little-endian.
    let op = PermissionOperation::SignUcanDelegation {
        audience: vec![0xAA, 0xBB],
        expires_at: 0x0102_0304_0506_0708,
    };
    let wire = op.to_wire_be();
    // tag(0x01) + len_be(00 00 00 02) + audience(AA BB) + expiry_be(01..08)
    let expected: Vec<u8> = vec![
        0x01, 0x00, 0x00, 0x00, 0x02, 0xAA, 0xBB, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ];
    assert_eq!(
        wire, expected,
        "operation wire-encoding MUST be big-endian (M-19/M-20); a LE expiry would read 08 07 06 05 04 03 02 01"
    );
}

/// F-LD-2 unknown-operation codepoint typed-rejects: an integer outside the
/// `0x6320..0x632F` RemotePermission band → typed `UnsupportedOperationCodepoint`,
/// NEVER silent acceptance (CLAUDE.md #5).
#[test]
#[ignore = "RED-PHASE: F-LD-2 — out-of-band remote-permission codepoint typed-rejects; un-ignore at R5"]
fn f_ld_2_out_of_band_codepoint_typed_rejects() {
    // In-band base accepts.
    dispatch_remote_permission_codepoint(REMOTE_PERMISSION_BAND_BASE)
        .expect("in-band remote-permission codepoint MUST dispatch");
    // DeviceLink band (0x6310) is a DIFFERENT band — out of RemotePermission range.
    let err = dispatch_remote_permission_codepoint(0x6310)
        .expect_err("DeviceLink-band codepoint MUST NOT dispatch as remote-permission");
    assert_eq!(err.0, 0x6310);
    // A wholly-unknown codepoint also rejects.
    assert!(dispatch_remote_permission_codepoint(0x0001).is_err());
}

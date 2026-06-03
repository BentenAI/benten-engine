//! F-LD-2 — Remote-permission wire freeze `PermissionRequest`/`PermissionGrant`
//! (RED-PHASE; byte-pinning — V2 + BE + canonical-TLV from first commit per M-20).
//!
//! R3 wave **W3-layer-d**. Pin sources:
//!   - `db2d7d6d:.addl/phase-4-meta/f-full-r2-test-landscape.md` §1 Group 8
//!     F-LD-2: "e2r §6.4 structs freeze; operation enum
//!     `Decrypt|SignUcanDelegation|RemoteUnlock|ExecuteWorkflow`; `0x6320..0x632F`.
//!     golden CBOR per struct; sig round-trip; unknown-operation typed-reject."
//!   - R0.3 plan §3.4 (`...f-full-r0-plan.md:504-508`): the FULL `PermissionRequest`
//!     / `PermissionGrant` wire shape:
//!     `PermissionRequest { request_id, requesting_device_did,
//!     requesting_device_pubkey, operation: { Decrypt(node_cid)
//!     | SignUcanDelegation(scope, audience, expires_at) | RemoteUnlock
//!     | ExecuteWorkflow(…) }, reason, timestamp, ephemeral_signing_key, nonce }`
//!     (signed by B) → `PermissionGrant { request_id, granted_at, valid_until,
//!     operation_result, audit_node_cid }` (signed by A's user-DID-signing-key).
//!   - §4.1 codepoint table (`...:820-821`): `0x6310..0x631F` DeviceLink band,
//!     `0x6320..0x632F` RemotePermission band — FREEZE.
//!
//! ## ENCODING CONTRACT (F4-015 reconciliation — canonical-TLV, NOT DAG-CBOR)
//!
//! R0.3 §4.1 freezes the AAD/wire path as **`aad_version: u8` prefix +
//! canonical-TLV length-injective** (U1/U3/U14) — NOT DAG-CBOR. (DAG-CBOR is
//! the Layer-A `vault.cbor` surface ONLY.) Earlier RED-PHASE header prose said
//! "canonical DAG-CBOR" while the encoder was already length-prefixed concat;
//! that contradiction is RESOLVED here in favour of the §4.1 canonical-TLV
//! contract. Every integer is **big-endian** (M-19/M-20); the V2 framing byte
//! and a per-struct domain-separation prefix lead the canonical bytes. There is
//! NO LE / V1 golden vector here.
//!
//! ## F4-015 — DROPPED FIELDS RESTORED
//!
//! The prior R3 stub had dropped `requesting_device_did`, `reason`,
//! `ephemeral_signing_key` from `PermissionRequest` and `scope` from
//! `SignUcanDelegation`. All four are RESTORED so the wire/AAD freeze is the
//! FULL e2r §6.4 shape — a future struct-field drop now flips the frozen golden
//! byte-vector below.
//!
//! ## RED-PHASE + byte-pinning (pim-12 §3.6e + M-20)
//!
//! SELF-CONTAINED stub-shim models the e2r §6.4 wire structs. The golden arms
//! freeze ABSOLUTE byte vectors (`const *_HEX` literals computed from the
//! canonical-TLV encoder) — not relative structure — so any field-order /
//! endianness / dropped-field / encoding drift flips the pin. At R5 the shim is
//! deleted, the real `benten_engine` Layer-D remote-permission types are `use`d,
//! and these frozen literals are confirmed-or-deliberately-updated against the
//! real encoder (M-20). Because the real types do not exist at this SHA, the
//! R5 un-ignore step is what produces the RED state (the `use` fails to compile
//! until the surface lands).

#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(dead_code)]
#![cfg(not(target_arch = "wasm32"))]

use benten_id::keypair::Keypair;

// ---------------------------------------------------------------------------
// SELF-CONTAINED STUB-SHIM — e2r §6.4 wire structs (V2 + BE + canonical-TLV).
// At R5: DELETE this module and replace with
//     use benten_engine::layer_d::remote_permission::{
//         PermissionRequest, PermissionGrant, PermissionOperation,
//         dispatch_remote_permission_codepoint,
//         REMOTE_PERMISSION_BAND_BASE, REMOTE_PERMISSION_WIRE_VERSION,
//     };
// ---------------------------------------------------------------------------
mod shim {
    /// Wire-format version. Wave-0 (M-20): V2 from the first commit. NEVER V1.
    pub const REMOTE_PERMISSION_WIRE_VERSION: u8 = 2;

    /// RemotePermission band codepoint base (§4.1 `0x6320..0x632F` FREEZE).
    pub const REMOTE_PERMISSION_BAND_BASE: u16 = 0x6320;
    pub const REMOTE_PERMISSION_BAND_END: u16 = 0x632F;

    /// Canonical domain-separation prefixes (part of the frozen signing bytes;
    /// a request signature can never be replayed as a grant signature).
    pub const REQUEST_DOMAIN: &[u8] = b"benten-remote-permission-request-v2:";
    pub const GRANT_DOMAIN: &[u8] = b"benten-remote-permission-grant-v2:";

    fn be_u32_len(n: usize) -> [u8; 4] {
        (n as u32).to_be_bytes()
    }

    /// The operation enum (e2r §6.4). `SignUcanDelegation` carries the RESTORED
    /// `scope` field (F4-015). The `ExecuteWorkflow` variant slot is frozen at
    /// v1-beta (F-LD-3 owns its AAD-binding pins); runtime enforcement is
    /// post-v1-beta.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum PermissionOperation {
        Decrypt {
            node_cid: [u8; 32],
        },
        SignUcanDelegation {
            scope: Vec<u8>, // RESTORED (F4-015): R0.3 §3.4 SignUcanDelegation(scope, audience, expires_at)
            audience: Vec<u8>,
            expires_at: u64,
        },
        RemoteUnlock,
        ExecuteWorkflow {
            workflow_cid: [u8; 32],
        },
    }

    impl PermissionOperation {
        /// Canonical-TLV BE wire encoding: discriminant tag + length-prefixed
        /// (BE u32) variable fields + BE integers. would-FAIL if any field
        /// switches to LE OR if a variable field drops its length prefix.
        pub fn to_wire_be(&self) -> Vec<u8> {
            let mut out = Vec::new();
            match self {
                Self::Decrypt { node_cid } => {
                    out.push(0x00);
                    out.extend_from_slice(node_cid);
                }
                Self::SignUcanDelegation {
                    scope,
                    audience,
                    expires_at,
                } => {
                    out.push(0x01);
                    // RESTORED scope: length-prefixed (BE) ahead of audience.
                    out.extend_from_slice(&be_u32_len(scope.len()));
                    out.extend_from_slice(scope);
                    out.extend_from_slice(&be_u32_len(audience.len()));
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

    /// e2r §6.4 — signed by requesting device B. FULL field set (F4-015).
    #[derive(Clone)]
    pub struct PermissionRequest {
        pub version: u8,
        pub request_id: [u8; 16],
        /// RESTORED (F4-015): the requesting device's DID (string bytes).
        pub requesting_device_did: Vec<u8>,
        pub requesting_device_pubkey: [u8; 32],
        pub operation: PermissionOperation,
        /// RESTORED (F4-015): human-readable reason shown in the approval UX.
        pub reason: Vec<u8>,
        pub timestamp_bucket: u64, // coarse 1-hr bucket (F-LD-8 owns granularity)
        /// RESTORED (F4-015): the ephemeral signing key B mints per request
        /// (Signal-Provisioning shape; bound so a request can't be re-keyed).
        pub ephemeral_signing_key: [u8; 32],
        pub nonce: [u8; 32],
        pub signature: Vec<u8>,
    }

    /// e2r §6.4 — signed by approving device A's user-DID signing key.
    #[derive(Clone)]
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
        /// Canonical signing bytes (canonical-TLV, BE integers, V2 framing,
        /// domain-separation prefix, deterministic field order — the freeze
        /// contract). Field order MIRRORS e2r §6.4 / R0.3 §3.4. Variable-length
        /// fields are BE-u32-length-prefixed so the concat is length-injective
        /// (U3 — no two distinct field tuples share a byte string).
        pub fn signing_bytes(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.extend_from_slice(REQUEST_DOMAIN);
            b.push(self.version);
            b.extend_from_slice(&self.request_id);
            b.extend_from_slice(&be_u32_len(self.requesting_device_did.len()));
            b.extend_from_slice(&self.requesting_device_did);
            b.extend_from_slice(&self.requesting_device_pubkey);
            b.extend_from_slice(&self.operation.to_wire_be());
            b.extend_from_slice(&be_u32_len(self.reason.len()));
            b.extend_from_slice(&self.reason);
            b.extend_from_slice(&self.timestamp_bucket.to_be_bytes());
            b.extend_from_slice(&self.ephemeral_signing_key);
            b.extend_from_slice(&self.nonce);
            b
        }
    }

    impl PermissionGrant {
        pub fn signing_bytes(&self) -> Vec<u8> {
            let mut b = Vec::new();
            b.extend_from_slice(GRANT_DOMAIN);
            b.push(self.version);
            b.extend_from_slice(&self.request_id);
            b.extend_from_slice(&self.granted_at_bucket.to_be_bytes());
            b.extend_from_slice(&self.valid_until.to_be_bytes());
            b.extend_from_slice(&self.audit_node_cid);
            b.extend_from_slice(&be_u32_len(self.operation_result.len()));
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

// ---------------------------------------------------------------------------
// FROZEN GOLDEN HEX (computed ONCE from the canonical-TLV encoder; embedded as
// literals so any encoding/field-order/endianness/dropped-field drift flips the
// pin). See `/tmp/r4fix/compute_f_ld_2.py` for the generator.
// R5 confirms-or-deliberately-updates these frozen literals against the real
// encoder (M-20).
// ---------------------------------------------------------------------------

/// `SignUcanDelegation { scope: b"atrium:read", audience: [0xAA,0xBB],
/// expires_at: 0x0102_0304_0506_0708 }.to_wire_be()`.
const OP_SIGN_UCAN_WIRE_HEX: &str =
    "010000000b61747269756d3a7265616400000002aabb0102030405060708";

/// `PermissionRequest::signing_bytes()` over the all-fixed fixture in
/// `request_fixture()` (no random keypair — this arm pins the ENCODER).
const REQUEST_SIGNING_BYTES_HEX: &str = "62656e74656e2d72656d6f74652d7065726d697373696f6e2d726571756573742d76323a0207070707070707070707070707070707000000106469643a6b65793a7a44657669636542111111111111111111111111111111111111111111111111111111111111111100090909090909090909090909090909090909090909090909090909090909090900000016756e6c6f636b207661756c74206f6e206c6170746f7000000000713fb62022222222222222222222222222222222222222222222222222222222222222220303030303030303030303030303030303030303030303030303030303030303";

/// `PermissionGrant::signing_bytes()` over the all-fixed fixture in
/// `grant_fixture()`.
const GRANT_SIGNING_BYTES_HEX: &str = "62656e74656e2d72656d6f74652d7065726d697373696f6e2d6772616e742d76323a020707070707070707070707070707070700000000713fb62000000000713fb65cabababababababababababababababababababababababababababababababab0000001968706b652d777261707065642d6b65792d6d6174657269616c";

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Deterministic all-fixed `PermissionRequest` fixture (signature empty; the
/// golden arm pins `signing_bytes`, not a signature).
fn request_fixture() -> PermissionRequest {
    PermissionRequest {
        version: 2,
        request_id: [7u8; 16],
        requesting_device_did: b"did:key:zDeviceB".to_vec(),
        requesting_device_pubkey: [0x11u8; 32],
        operation: PermissionOperation::Decrypt {
            node_cid: [0x09u8; 32],
        },
        reason: b"unlock vault on laptop".to_vec(),
        timestamp_bucket: 1_900_000_800,
        ephemeral_signing_key: [0x22u8; 32],
        nonce: [0x03u8; 32],
        signature: Vec::new(),
    }
}

fn grant_fixture() -> PermissionGrant {
    PermissionGrant {
        version: 2,
        request_id: [7u8; 16],
        granted_at_bucket: 1_900_000_800,
        valid_until: 1_900_000_860,
        operation_result: b"hpke-wrapped-key-material".to_vec(),
        audit_node_cid: [0xABu8; 32],
        signature: Vec::new(),
    }
}

/// F-LD-2 PermissionRequest sig round-trip: B signs the canonical (TLV/BE/V2)
/// bytes; the pubkey-on-wire verifies. would-FAIL-if-no-op'd: tampering ANY
/// field (operation, nonce, timestamp, did, reason, ephemeral key) changes
/// `signing_bytes` and the signature no longer verifies.
#[test]
#[ignore = "RED-PHASE: F-LD-2 — PermissionRequest TLV/BE/V2 canonical sig round-trip (full e2r §6.4 fields); un-ignore at R5"]
fn f_ld_2_permission_request_signature_round_trips_over_canonical_be_bytes() {
    let device_b = Keypair::generate();
    let mut req = request_fixture();
    req.requesting_device_pubkey = device_b.public_key().to_bytes();
    let sig = device_b.sign(&req.signing_bytes());
    req.signature = sig.to_bytes().to_vec();

    // V2 framing pin (M-20 — never V1).
    assert_eq!(req.version, 2, "remote-permission wire is V2 from first commit");
    assert_eq!(req.version, REMOTE_PERMISSION_WIRE_VERSION);

    // Verify the on-wire pubkey validates the signature over the canonical bytes.
    let verify_pk = device_b.public_key();
    verify_pk
        .verify(&req.signing_bytes(), &sig)
        .expect("PermissionRequest signature MUST verify over canonical TLV/BE/V2 bytes");

    // would-FAIL-if-no-op'd: mutate the operation → signature breaks.
    let mut tampered = req.clone();
    tampered.operation = PermissionOperation::RemoteUnlock;
    assert!(
        verify_pk.verify(&tampered.signing_bytes(), &sig).is_err(),
        "mutating the operation MUST break the signature (field is bound)"
    );

    // would-FAIL-if-no-op'd: mutate the RESTORED reason field → signature breaks.
    let mut tampered_reason = req.clone();
    tampered_reason.reason = b"attacker-substituted-reason".to_vec();
    assert!(
        verify_pk
            .verify(&tampered_reason.signing_bytes(), &sig)
            .is_err(),
        "mutating `reason` MUST break the signature (RESTORED field is bound — F4-015)"
    );

    // would-FAIL-if-no-op'd: substitute the RESTORED ephemeral_signing_key →
    // signature breaks (the request can't be re-keyed post-signature).
    let mut tampered_ephemeral = req.clone();
    tampered_ephemeral.ephemeral_signing_key = [0xEEu8; 32];
    assert!(
        verify_pk
            .verify(&tampered_ephemeral.signing_bytes(), &sig)
            .is_err(),
        "substituting `ephemeral_signing_key` MUST break the signature (RESTORED field is bound — F4-015)"
    );

    // would-FAIL-if-no-op'd: substitute the RESTORED requesting_device_did →
    // signature breaks (a different device can't claim the request).
    let mut tampered_did = req.clone();
    tampered_did.requesting_device_did = b"did:key:zAttacker".to_vec();
    assert!(
        verify_pk.verify(&tampered_did.signing_bytes(), &sig).is_err(),
        "mutating `requesting_device_did` MUST break the signature (RESTORED field is bound — F4-015)"
    );
}

/// F-LD-2 PermissionGrant sig round-trip: A's user-DID key signs the grant;
/// `audit_node_cid` is part of the signed bytes (so a "grant without audit"
/// can't be forged by stripping the field — couples F-LD-6 class 6).
#[test]
#[ignore = "RED-PHASE: F-LD-2 — PermissionGrant TLV/BE/V2 sig round-trip binds audit_node_cid; un-ignore at R5"]
fn f_ld_2_permission_grant_signature_binds_audit_node_cid() {
    let device_a_user_did = Keypair::generate();
    let mut grant = grant_fixture();
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

/// F-LD-2 golden BE byte-pin: the `SignUcanDelegation` operation wire encoding
/// (incl. the RESTORED `scope` field) is canonical-TLV + big-endian + the
/// discriminant tags are frozen. A future LE regression, a tag reordering, a
/// dropped `scope` length-prefix, OR a dropped `scope` field flips this FROZEN
/// hex literal. (M-20 / M-19 BE conformance gate; F4-015 scope-restoration.)
#[test]
#[ignore = "RED-PHASE: F-LD-2 — SignUcanDelegation(scope,…) BE canonical-TLV golden byte-pin; un-ignore at R5"]
fn f_ld_2_operation_wire_encoding_is_big_endian_golden_pin() {
    // scope = b"atrium:read" (RESTORED — F4-015), audience = [0xAA,0xBB],
    // expires_at = 0x0102_0304_0506_0708 → MUST appear big-endian.
    let op = PermissionOperation::SignUcanDelegation {
        scope: b"atrium:read".to_vec(),
        audience: vec![0xAA, 0xBB],
        expires_at: 0x0102_0304_0506_0708,
    };
    let wire = op.to_wire_be();
    assert_eq!(
        to_hex(&wire),
        OP_SIGN_UCAN_WIRE_HEX,
        "SignUcanDelegation wire-encoding MUST match the FROZEN canonical-TLV/BE \
         golden literal — a LE expiry, a reordered tag, OR a dropped `scope` \
         field/length-prefix flips this pin (F4-015 / M-19 / M-20)"
    );

    // Explicit BE-vs-LE differentiator: the frozen hex ends in the BE expiry
    // `0102030405060708`; the LE reading `0807060504030201` MUST NOT appear.
    assert!(
        OP_SIGN_UCAN_WIRE_HEX.ends_with("0102030405060708"),
        "expiry MUST be encoded big-endian"
    );
    assert!(
        !OP_SIGN_UCAN_WIRE_HEX.contains("0807060504030201"),
        "a little-endian expiry encoding MUST NOT appear (M-19 BE freeze)"
    );
}

/// F-LD-2 full-struct golden byte-pin (F4-015): the COMPLETE `signing_bytes`
/// for both wire structs is frozen as an absolute byte vector. Restoring the
/// dropped fields means the pin freezes the FULL e2r §6.4 shape — dropping ANY
/// field (did/reason/ephemeral_signing_key/scope) again, reordering fields, or
/// switching any integer to LE flips one of these literals.
#[test]
#[ignore = "RED-PHASE: F-LD-2 — full PermissionRequest/PermissionGrant signing_bytes golden byte-pin (all fields); un-ignore at R5"]
fn f_ld_2_full_struct_signing_bytes_golden_pin() {
    assert_eq!(
        to_hex(&request_fixture().signing_bytes()),
        REQUEST_SIGNING_BYTES_HEX,
        "PermissionRequest::signing_bytes MUST match the FROZEN golden literal \
         (full e2r §6.4 field set incl. requesting_device_did / reason / \
         ephemeral_signing_key — F4-015)"
    );
    assert_eq!(
        to_hex(&grant_fixture().signing_bytes()),
        GRANT_SIGNING_BYTES_HEX,
        "PermissionGrant::signing_bytes MUST match the FROZEN golden literal"
    );

    // Cross-struct domain separation: the two byte strings can never collide
    // even with an equal request_id (distinct domain-separation prefixes).
    assert_ne!(
        REQUEST_SIGNING_BYTES_HEX, GRANT_SIGNING_BYTES_HEX,
        "request/grant signing bytes MUST be domain-separated"
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

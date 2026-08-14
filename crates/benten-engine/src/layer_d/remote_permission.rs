//! Layer-D remote-permission-call wire protocol (e2r §6.4; R0.7 §3.4).
//!
//! Two flows under one wire protocol (codepoint band `0x6320..0x632F`):
//!   - **R1 Remote-unlock** — device A unlocks device B without B's local
//!     password (`PermissionOperation::RemoteUnlock`).
//!   - **R2 Remote permission-grant** — device A issues an ephemeral UCAN
//!     delegation or HPKE-wrapped key material to B
//!     (`Decrypt` / `SignUcanDelegation` / `ExecuteWorkflow`).
//!
//! # Wire encoding contract (R0.7 §4.1 — canonical-TLV, NOT DAG-CBOR)
//!
//! The AAD/wire/signing path is `aad_version: u8` prefix + canonical-TLV
//! length-injective (U1/U3/U14) — NOT DAG-CBOR (DAG-CBOR is the Layer-A
//! `vault.cbor` surface only). Every integer is **big-endian** (M-19/M-20);
//! the V2 framing byte + a per-struct domain-separation prefix lead the
//! canonical bytes. There is NO LE / V1 golden vector.
//!
//! The `aad_version: u8` prefix (R0.7 §4.1) is a DISTINCT axis from
//! [`REMOTE_PERMISSION_WIRE_VERSION`] (F4-004/005-LD2): a future
//! envelope-format bump moves the wire version, NOT the AAD-prefix byte; an
//! AAD-binding-contract bump moves the AAD-prefix byte. Conflating them
//! re-versions the AAD on an envelope bump and breaks cross-version replay
//! semantics.
//!
//! # ExecuteWorkflow (M-3; CODEPOINT-RESERVE)
//!
//! The `ExecuteWorkflow` variant slot + the AAD-binding of
//! `(executor_did, max_decrypt_count, result_recipient_pubkey)` are FROZEN at
//! v1-beta; runtime enforcement (no-egress / bounded-decrypt) is post-v1-beta
//! (NQ-T3 — the frozen 3-field AAD scope is SUFFICIENT to express the
//! constraint even though enforcement defers).

#![allow(clippy::doc_markdown)]

use benten_id::keypair::{PublicKey, Signature};
use zeroize::Zeroize as _;

/// AAD-prefix / cross-version-replay-defense version byte (R0.7 §4.1:
/// `aad_version: u8` prefix — DISTINCT from the wire/format version). Mirrors
/// the MembershipSet `AAD_VERSION = 0x01` convention. A future envelope-format
/// bump moves [`REMOTE_PERMISSION_WIRE_VERSION`], NOT this byte; an
/// AAD-binding-contract bump moves THIS byte.
pub const AAD_VERSION: u8 = 0x01;

/// Wire-format / struct framing version. Wave-0 (M-20): V2 from the first
/// commit. NEVER V1. Orthogonal to [`AAD_VERSION`] (F4-004/005-LD2).
pub const REMOTE_PERMISSION_WIRE_VERSION: u8 = 2;

/// RemotePermission band codepoint base (R0.7 §4.1 `0x6320..0x632F` FREEZE).
pub const REMOTE_PERMISSION_BAND_BASE: u16 = 0x6320;

/// RemotePermission band codepoint end (inclusive).
pub const REMOTE_PERMISSION_BAND_END: u16 = 0x632F;

/// Canonical request domain-separation prefix (part of the frozen signing
/// bytes; a request signature can never be replayed as a grant signature).
pub const REQUEST_DOMAIN: &[u8] = b"benten-remote-permission-request-v2:";

/// Canonical grant domain-separation prefix.
pub const GRANT_DOMAIN: &[u8] = b"benten-remote-permission-grant-v2:";

/// Intra-variant domain-separation prefix for the [`ExecuteWorkflow`]
/// constraint AAD (R0.7 §3.4; NQ-T3). At the envelope layer this binding folds
/// into the enclosing [`PermissionRequest`] AAD (which carries the §4.1
/// `aad_version=0x01` byte-0 prefix); the string here is the intra-variant
/// domain-separation tag (F-LD-3-AADVER-COHERENCE).
pub const EXEC_WORKFLOW_AAD_DOMAIN: &[u8] = b"benten-exec-workflow-v1:";

#[inline]
fn be_u32_len(n: usize) -> [u8; 4] {
    // Wire lengths are u32 BE; usize is narrowed deliberately (lengths are
    // bounded far below u32::MAX on every wire path).
    #[allow(clippy::cast_possible_truncation)]
    {
        (n as u32).to_be_bytes()
    }
}

/// The remote-permission operation enum (e2r §6.4 / R0.7 §3.4). The blast-radius
/// ladder (O-6): `Decrypt`=1 Node < `SignUcanDelegation`=attenuated-exp-bounded
/// < `ExecuteWorkflow`=bounded-decrypt-count <
/// `RemoteUnlock`/device-link=full `K_principal`-permanent.
///
/// `#[non_exhaustive]` (§11 SemVer-readiness): a future remote-permission
/// operation lands ADDITIVELY without a breaking SemVer bump. The wire-tag
/// space (`to_wire_be` discriminant bytes `0x00..`) is NOT a frozen-cardinality
/// roster (no `ALL` const / non-wildcard roster-index guard exists, unlike
/// `GrantRejection`); the same-crate `to_wire_be` match stays exhaustive so a
/// new variant still forces its wire-tag here at compile time, while
/// cross-crate consumers get additive forward-compat.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PermissionOperation {
    /// Decrypt a single Node (`node_cid`).
    Decrypt {
        /// The Node CID to decrypt.
        node_cid: [u8; 32],
    },
    /// Sign an ephemeral UCAN delegation. Carries the `scope` (RESTORED —
    /// F4-015), the `audience`, and the `expires_at` (tight-`exp` default per
    /// m-6 / Compromise #60 + #52).
    SignUcanDelegation {
        /// The capability scope bytes (e.g. `b"atrium:read"`).
        scope: Vec<u8>,
        /// The delegation audience (DID bytes).
        audience: Vec<u8>,
        /// Unix-seconds expiry (BE on the wire). SHORT default.
        expires_at: u64,
    },
    /// Remote-unlock device B (full `K_principal` permission — top of the
    /// blast-radius ladder).
    RemoteUnlock,
    /// Execute a workflow on rented compute (M-3 CODEPOINT-RESERVE). The slot
    /// + AAD-binding are frozen at v1-beta; runtime enforcement is
    /// post-v1-beta (NQ-T3).
    ExecuteWorkflow {
        /// The workflow subgraph CID.
        workflow_cid: [u8; 32],
    },
}

impl PermissionOperation {
    /// Canonical-TLV BE wire encoding: discriminant tag + length-prefixed
    /// (BE u32) variable fields + BE integers. Length-injective (U3). The
    /// operation wire carries no `aad_version` — that prefix leads the
    /// enclosing struct's signing bytes, not each operation.
    #[must_use]
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

/// e2r §6.4 — `PermissionRequest`, signed by requesting device B. FULL field
/// set (F4-015: `requesting_device_did`, `reason`, `ephemeral_signing_key`
/// RESTORED).
///
/// Secret-hygiene (D-74/75/76): `ephemeral_signing_key` is a per-request
/// PRIVATE signing key. `Debug` is a MANUAL impl that renders it as
/// `<redacted>` (the `#[derive(Debug)]` was replaced — it dumped the raw
/// key bytes via `{:?}`), and the key is zeroized on drop. Field types are
/// unchanged, so there is NO wire / serialization / public-API-shape change —
/// redacted-Debug + zeroize are non-wire additions only.
#[derive(Clone)]
pub struct PermissionRequest {
    /// AAD-prefix version (F4-004/005-LD2). MUST equal [`AAD_VERSION`].
    pub aad_version: u8,
    /// Wire/struct framing version. MUST equal [`REMOTE_PERMISSION_WIRE_VERSION`].
    pub version: u8,
    /// 16-byte request identifier.
    pub request_id: [u8; 16],
    /// The requesting device's DID (string bytes) — RESTORED (F4-015).
    pub requesting_device_did: Vec<u8>,
    /// The requesting device's public key.
    pub requesting_device_pubkey: [u8; 32],
    /// The operation being requested.
    pub operation: PermissionOperation,
    /// Human-readable reason shown in the approval UX — RESTORED (F4-015).
    pub reason: Vec<u8>,
    /// Coarse 1-hour timestamp bucket (Layer-D-only; M-14).
    pub timestamp_bucket: u64,
    /// The ephemeral signing key B mints per request (Signal-Provisioning
    /// shape; bound so a request can't be re-keyed) — RESTORED (F4-015).
    pub ephemeral_signing_key: [u8; 32],
    /// Per-request nonce (replay defense rides the `jti`-keyed nonce-cache).
    pub nonce: [u8; 32],
    /// Detached signature over [`PermissionRequest::signing_bytes`].
    pub signature: Vec<u8>,
}

impl PermissionRequest {
    /// Canonical signing bytes (canonical-TLV, BE integers, V2 framing,
    /// domain-separation prefix, deterministic field order — the freeze
    /// contract). The `aad_version: u8` prefix (R0.7 §4.1) leads, then the
    /// wire/framing `version`, mirroring the MembershipSet AAD layout. Field
    /// order MIRRORS e2r §6.4 / R0.7 §3.4. Variable-length fields are
    /// BE-u32-length-prefixed so the concat is length-injective (U3).
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(REQUEST_DOMAIN);
        b.push(self.aad_version); // §4.1 aad_version prefix (DISTINCT axis)
        b.push(self.version); // wire/struct framing version
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

/// Debug-redacting: the per-request PRIVATE `ephemeral_signing_key` MUST NOT
/// leak into logs / panics / tracing. All other (non-secret) fields render
/// normally; only the ephemeral signing key is a `<redacted>` marker. Replaces
/// the former `#[derive(Debug)]`, which dumped the raw key bytes.
impl core::fmt::Debug for PermissionRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PermissionRequest")
            .field("aad_version", &self.aad_version)
            .field("version", &self.version)
            .field("request_id", &self.request_id)
            .field("requesting_device_did", &self.requesting_device_did)
            .field("requesting_device_pubkey", &self.requesting_device_pubkey)
            .field("operation", &self.operation)
            .field("reason", &self.reason)
            .field("timestamp_bucket", &self.timestamp_bucket)
            .field("ephemeral_signing_key", &"<redacted>")
            .field("nonce", &self.nonce)
            .field("signature", &self.signature)
            .finish()
    }
}

/// Zeroize-on-drop: wipe the per-request PRIVATE ephemeral signing key so it
/// does not linger in freed heap / coredump. Non-secret fields are left to
/// their normal drop. Field types are unchanged (no wire impact).
impl Drop for PermissionRequest {
    fn drop(&mut self) {
        self.ephemeral_signing_key.zeroize();
    }
}

/// e2r §6.4 — `PermissionGrant`, signed by approving device A's user-DID
/// signing key. The `audit_node_cid` is part of the signed bytes so a "grant
/// without audit" can't be forged by stripping the field (couples F-LD-6
/// pass-class 6).
#[derive(Clone, Debug)]
pub struct PermissionGrant {
    /// AAD-prefix version (F4-004/005-LD2). MUST equal [`AAD_VERSION`].
    pub aad_version: u8,
    /// Wire/struct framing version. MUST equal [`REMOTE_PERMISSION_WIRE_VERSION`].
    pub version: u8,
    /// 16-byte request identifier (matches the request).
    pub request_id: [u8; 16],
    /// Coarse 1-hour bucket of grant time (Layer-D-only metadata; M-14).
    pub granted_at_bucket: u64,
    /// Full-granularity `valid_until` enforcement clock (NQ-T2: orthogonal to
    /// the coarse bucket; enforced strictly with NO grace/skew).
    pub valid_until: u64,
    /// The operation result (e.g. HPKE-wrapped key material).
    pub operation_result: Vec<u8>,
    /// MUST be present + resolvable (F-LD-6 pass-class 6 audit-binding).
    pub audit_node_cid: [u8; 32],
    /// Detached signature over [`PermissionGrant::signing_bytes`].
    pub signature: Vec<u8>,
}

impl PermissionGrant {
    /// Canonical signing bytes (canonical-TLV, BE integers, V2 framing,
    /// grant domain-separation prefix).
    #[must_use]
    pub fn signing_bytes(&self) -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(GRANT_DOMAIN);
        b.push(self.aad_version);
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

/// The `ExecuteWorkflow` rented-compute constraint binding (M-3; NQ-T3).
///
/// The variant-slot + the AAD-binding of `(executor_did, max_decrypt_count,
/// result_recipient_pubkey)` are FROZEN at v1-beta; runtime enforcement is
/// post-v1-beta. The frozen 3-field AAD is SUFFICIENT to express the
/// no-egress / bounded-decrypt constraint (the wire freeze IS the AAD scope).
/// The 3-field constraint AAD binds at **result-seal time** (NQ-T3-ratified
/// envelope-fold) via [`exec_workflow_seal`] / [`exec_workflow_open`] over the
/// result envelope — it is NOT folded into [`PermissionRequest::signing_bytes`]
/// at v1-beta.
#[derive(Clone, Debug)]
pub struct ExecuteWorkflow {
    /// The workflow subgraph CID.
    pub workflow_cid: [u8; 32],
    /// Input Node CIDs the workflow reads.
    pub input_node_cids: Vec<[u8; 32]>,
    /// The maximum number of decrypts the rented executor is granted (BE on
    /// the wire). Bound in the AAD so the executor can't raise its own budget.
    pub max_decrypt_count: u32,
    /// The pubkey the result MUST be encrypted to (the no-egress channel
    /// binding). Bound in the AAD so the executor can't redirect the result.
    pub result_recipient_pubkey: [u8; 32],
    /// The rented executor's DID (string bytes). Bound in the AAD so the
    /// executor can't swap itself for another.
    pub executor_did: Vec<u8>,
}

impl ExecuteWorkflow {
    /// The FROZEN constraint binding: binds exactly the three constraint
    /// fields `(executor_did, max_decrypt_count, result_recipient_pubkey)` —
    /// the "sufficient to express no-egress" scope (NQ-T3, RATIFIED). BE
    /// integers.
    ///
    /// The [`EXEC_WORKFLOW_AAD_DOMAIN`] tag is an INTRA-VARIANT
    /// domain-separation prefix, NOT the §4.1 `aad_version:u8` wire prefix —
    /// that prefix is supplied by the enclosing [`PermissionRequest`] envelope
    /// at the envelope layer (F-LD-3-AADVER-COHERENCE).
    #[must_use]
    pub fn constraint_aad(&self) -> Vec<u8> {
        let mut aad = Vec::new();
        aad.extend_from_slice(EXEC_WORKFLOW_AAD_DOMAIN);
        aad.extend_from_slice(&be_u32_len(self.executor_did.len()));
        aad.extend_from_slice(&self.executor_did);
        aad.extend_from_slice(&self.max_decrypt_count.to_be_bytes());
        aad.extend_from_slice(&self.result_recipient_pubkey);
        aad
    }
}

/// Seal a plaintext under `key` with the [`ExecuteWorkflow`] constraint AAD,
/// via the real ChaCha20-Poly1305 AEAD (the production
/// ChaCha20-Poly1305-under-HPKE stand-in for the Layer-D rented-compute path).
/// The three constraint fields ride the AAD, so mutating any of them makes the
/// open fail (NQ-T3 — the constraint is bound, not advisory).
///
/// # Errors
///
/// Returns the AEAD error on a seal failure.
pub fn exec_workflow_seal(
    ew: &ExecuteWorkflow,
    key: &[u8; 32],
    plaintext: &[u8],
) -> Result<benten_crypto_suite::aead::AeadEnvelope, benten_crypto_suite::aead::AeadError> {
    use benten_crypto_suite::aead::{AeadKeyMaterial, wrap};
    use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
    let km = AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, key);
    wrap(plaintext, &km, &ew.constraint_aad())
}

/// Open an [`exec_workflow_seal`] envelope under `key` with the PRESENTED
/// [`ExecuteWorkflow`] constraint AAD. If any constraint field was mutated
/// post-seal (executor_did / max_decrypt_count / result_recipient_pubkey), the
/// recomputed AAD differs and the AEAD tag check fails closed.
///
/// # Errors
///
/// Returns the AEAD error when the constraint AAD does not match (a bound
/// constraint was tampered) or the ciphertext is corrupt.
pub fn exec_workflow_open(
    ew: &ExecuteWorkflow,
    key: &[u8; 32],
    envelope: &benten_crypto_suite::aead::AeadEnvelope,
) -> Result<Vec<u8>, benten_crypto_suite::aead::AeadError> {
    use benten_crypto_suite::aead::{AeadKeyMaterial, unwrap};
    use benten_crypto_suite::codepoint::CipherSuiteCodepoint;
    let km = AeadKeyMaterial::from_raw_bytes(CipherSuiteCodepoint::HYBRID_X25519_MLKEM768, key);
    unwrap(envelope, &km, &ew.constraint_aad())
}

/// Verify a detached signature over arbitrary canonical bytes with a
/// [`PublicKey`]. Re-exported convenience over [`PublicKey::verify`].
///
/// # Errors
///
/// Returns the verify error if the signature does not validate.
pub fn verify_canonical(
    pubkey: &PublicKey,
    bytes: &[u8],
    sig: &Signature,
) -> Result<(), benten_crypto_suite::primitives::ed25519_dalek::SignatureError> {
    pubkey.verify(bytes, sig)
}

/// An out-of-band RemotePermission codepoint — typed-reject (no silent
/// fallback; CLAUDE.md #5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedOperationCodepoint(pub u16);

/// Dispatch a raw RemotePermission codepoint. Integers outside the
/// `0x6320..=0x632F` band typed-reject (fail-closed; CLAUDE.md #5 — never a
/// silent fallback). The DeviceLink band (`0x6310`) is a DIFFERENT band and
/// is rejected here.
///
/// # Errors
///
/// Returns [`UnsupportedOperationCodepoint`] for any out-of-band integer.
pub fn dispatch_remote_permission_codepoint(cp: u16) -> Result<(), UnsupportedOperationCodepoint> {
    if (REMOTE_PERMISSION_BAND_BASE..=REMOTE_PERMISSION_BAND_END).contains(&cp) {
        Ok(())
    } else {
        Err(UnsupportedOperationCodepoint(cp))
    }
}

#[cfg(test)]
mod domain_registry_mirror {
    /// C-01/C-02 drift defense: the Layer-D remote-permission domain tags are
    /// mirrored in the central [`benten_crypto_suite::domain_registry`] corpus
    /// table over which the prefix-free collision check runs. Pin byte-equality
    /// so the mirror can never silently diverge from these home definitions.
    #[test]
    fn remote_permission_domains_match_central_registry() {
        use benten_crypto_suite::domain_registry as reg;
        assert_eq!(
            super::REQUEST_DOMAIN,
            reg::REQUEST_DOMAIN,
            "REQUEST_DOMAIN drifted from the central domain_registry mirror"
        );
        assert_eq!(
            super::GRANT_DOMAIN,
            reg::GRANT_DOMAIN,
            "GRANT_DOMAIN drifted from the central domain_registry mirror"
        );
        assert_eq!(
            super::EXEC_WORKFLOW_AAD_DOMAIN,
            reg::EXEC_WORKFLOW_AAD_DOMAIN,
            "EXEC_WORKFLOW_AAD_DOMAIN drifted from the central domain_registry mirror"
        );
    }
}

/// E-05/E-06 — the `PermissionOperation` wire-tag freeze (the compile-gate half).
///
/// The R6 falsification sweep mutated `RemoteUnlock` `0x02 -> 0x00` (colliding with
/// `Decrypt`) and `ExecuteWorkflow` `0x03 -> 0x02` (colliding with `RemoteUnlock`) and
/// ALL 61 targets stayed GREEN. Three things conspired: the only `to_wire_be` golden
/// covered `SignUcanDelegation`; `Decrypt` was pinned only transitively through
/// `PermissionRequest::signing_bytes`; and there is no `from_wire_be` decoder to
/// disagree with the encoder. `to_wire_be` is ONE-WAY — signer and verifier both call
/// it, so they agree bilaterally on ANY tag assignment. That is the same shape as the
/// plugin install-record preimage: only an ABSOLUTE per-arm golden, or a structural
/// cross-arm relation, can catch it.
///
/// This module carries the two checks that need NO golden (so they are live from the
/// first run, before capture): the tag-equals-roster-index pin and the pairwise
/// privilege-confusion pin. The ABSOLUTE per-arm hex goldens live in
/// `tests/f_ld_2_remote_permission_wire_freeze.rs`, which is already listed in
/// `.github/frozen-bytes-corpus.txt` and therefore runs on the REQUIRED
/// `frozen-bytes` lane; this `--lib` module runs on that lane too since the R6
/// round-#1 widening of `frozen-bytes.yml` step 5 to the full corpus package set.
///
/// The match in `roster_index` is deliberately wildcard-free. `PermissionOperation` is
/// `#[non_exhaustive]`, which suppresses exhaustiveness checking only ACROSS crates —
/// inside the defining crate a fifth variant FAILS TO COMPILE here, forcing its author
/// to declare a wire tag, a fixture, and (via `EXPECTED_VARIANT_COUNT`) a golden row.
#[cfg(test)]
mod operation_wire_tag_freeze {
    use super::PermissionOperation;

    /// The frozen variant cardinality. This is the ARRAY LENGTH of [`roster`], so it
    /// is the gate, not a runtime assert: a fifth variant fails to compile in
    /// [`roster_index`] first, and adding its fixture then fails to compile here
    /// until this const is bumped.
    const EXPECTED_VARIANT_COUNT: usize = 4;

    /// ONE 32-byte payload shared by the `Decrypt` and `ExecuteWorkflow` fixtures, so
    /// those two wires differ ONLY in their discriminant tag — the exact shape a tag
    /// collision erases (E-06).
    const SHARED_CID: [u8; 32] = [0x5A; 32];

    /// EXHAUSTIVE, wildcard-free — this match IS the compile-time roster gate.
    fn roster_index(op: &PermissionOperation) -> usize {
        match op {
            PermissionOperation::Decrypt { .. } => 0,
            PermissionOperation::SignUcanDelegation { .. } => 1,
            PermissionOperation::RemoteUnlock => 2,
            PermissionOperation::ExecuteWorkflow { .. } => 3,
        }
    }

    /// One deterministic fixture per variant, in frozen wire-tag order. Field values
    /// are IDENTICAL to the `f_ld_2` integration fixtures so the two homes freeze the
    /// same bytes.
    fn roster() -> [(&'static str, PermissionOperation); EXPECTED_VARIANT_COUNT] {
        [
            (
                "Decrypt",
                PermissionOperation::Decrypt {
                    node_cid: SHARED_CID,
                },
            ),
            (
                "SignUcanDelegation",
                PermissionOperation::SignUcanDelegation {
                    scope: b"atrium:read".to_vec(),
                    audience: vec![0xAA, 0xBB],
                    expires_at: 0x0102_0304_0506_0708,
                },
            ),
            ("RemoteUnlock", PermissionOperation::RemoteUnlock),
            (
                "ExecuteWorkflow",
                PermissionOperation::ExecuteWorkflow {
                    workflow_cid: SHARED_CID,
                },
            ),
        ]
    }

    /// E-05 — the leading discriminant byte of every arm equals its frozen roster
    /// index. `roster_index` is a SECOND, independent home for the tag ordering, so
    /// this catches a tag edit without needing any captured hex.
    ///
    /// would-FAIL-on-mutation (the exact line the sweep flipped, in `to_wire_be`):
    ///   `Self::RemoteUnlock => out.push(0x02),`  ->  `Self::RemoteUnlock => out.push(0x00),`
    /// (`to_wire_be()[0]` becomes 0 while `roster_index` still says 2).
    ///
    /// The tags are frozen DENSE from 0x00 in roster order. A future variant that
    /// wants a non-dense tag must change this pin deliberately — that is the point:
    /// the wire-tag space is frozen at v1-beta, so a sparse assignment is a decision,
    /// not a drive-by.
    #[test]
    fn e_05_operation_wire_tag_equals_frozen_roster_index() {
        // No `roster.len() == EXPECTED_VARIANT_COUNT` assert here on purpose: the
        // array TYPE already carries that, so the assert would be a tautology (and
        // this partition's whole point is that tautological pins are worthless).
        let roster = roster();
        for (i, (name, op)) in roster.iter().enumerate() {
            assert_eq!(
                roster_index(op),
                i,
                "E-05: fixture `{name}` is out of frozen wire-tag order (roster slot {i})"
            );
            let wire = op.to_wire_be();
            assert!(
                !wire.is_empty(),
                "E-05: `{name}` MUST emit at least its discriminant tag byte"
            );
            assert_eq!(
                usize::from(wire[0]),
                i,
                "E-05: `{name}` MUST carry the FROZEN discriminant tag 0x{i:02x}. The \
                 wire tags are frozen dense from 0x00 in roster order (Decrypt=0x00, \
                 SignUcanDelegation=0x01, RemoteUnlock=0x02, ExecuteWorkflow=0x03) and \
                 `to_wire_be` has NO decoder to disagree with it."
            );
        }
    }

    /// E-06 — no two operations encode to the same wire bytes, and no two share a
    /// discriminant tag.
    ///
    /// `Decrypt` and `ExecuteWorkflow` deliberately carry the SAME 32-byte payload
    /// ([`SHARED_CID`]), so their wires are distinguishable ONLY by the tag byte.
    ///
    /// would-FAIL-on-mutation (one line, in `to_wire_be`):
    ///   `Self::ExecuteWorkflow { workflow_cid } => { out.push(0x03);`
    ///     ->  `... => { out.push(0x00);`
    /// The two wires then become BYTE-IDENTICAL — a signed grant reading
    /// "execute this workflow on rented compute" and one reading "decrypt this one
    /// Node" are the same bytes. The module doc puts `RemoteUnlock` at the
    /// TOP of the blast-radius ladder (full permanent `K_principal` authority), which
    /// is what makes a tag collision a privilege confusion rather than a cosmetic
    /// wire break.
    ///
    /// Needs no golden, so it is enforcing from the first run.
    #[test]
    fn e_06_operation_wires_pairwise_distinct_privilege_confusion_guard() {
        let roster = roster();
        for (i, (name_i, op_i)) in roster.iter().enumerate() {
            let wire_i = op_i.to_wire_be();
            for (name_j, op_j) in roster.iter().skip(i + 1) {
                let wire_j = op_j.to_wire_be();
                assert_ne!(
                    wire_i[0], wire_j[0],
                    "E-06: `{name_i}` and `{name_j}` MUST NOT share a discriminant tag \
                     — a colliding tag is a privilege confusion across the O-6 \
                     blast-radius ladder (Decrypt = 1 Node; RemoteUnlock = full \
                     permanent K_principal)"
                );
                assert_ne!(
                    wire_i, wire_j,
                    "E-06: `{name_i}` and `{name_j}` MUST NOT encode to IDENTICAL wire \
                     bytes. These fixtures share a payload on purpose, so an equal \
                     encoding means the tag byte collided and the two authorities are \
                     indistinguishable in the signed bytes."
                );
            }
        }
    }
}

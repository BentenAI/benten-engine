//! Device-DID capability-attestation surface (G14-A2 wave-4a').
//!
//! ## D-PHASE-3-25 heterogeneity contract
//!
//! Each device under a shared logical identity declares its
//! capability envelope via a signed device-DID attestation:
//!
//! - `runs_sandbox: bool` — does this device execute SANDBOX modules?
//! - `holds_zones: ZoneScope` — full / cache-only / specific-list
//! - `online_uptime: UptimePolicy` — always-on / session-bounded
//! - `runs_atrium_peer: bool` — full peer or thin client?
//!
//! Per CLAUDE.md baked-in #17, the thin compute surface (browser tab,
//! Phase-9+ edge worker) declares minimum-capability envelopes
//! (`runs_sandbox=false`, `holds_zones=CacheOnly`,
//! `online_uptime=SessionBounded`, `runs_atrium_peer=false`). The
//! attestation is consumed at UCAN delegation chain-walk so per-
//! device cap policy can enforce envelope-derived limits.
//!
//! ## Replay-resistance (defect-class) — COLLAPSE P3
//!
//! Per the device-DID-attestation-replay defect-class +
//! `pim-r1-pim-induction-7`, attestations carry a 32-byte nonce +
//! `issued_at` epoch second. Under COLLAPSE (DECISION-RECORD §4
//! RATIFIED) the device-attestation *acceptance* pipe (the former
//! `Acceptor` — expected-parent gate / nonce-store / revocation-list)
//! is DELETED: the device envelope is no longer a distinct
//! trust-root. Stale-frame replay is now bounded by a plain freshness
//! window on the consuming Atrium handle
//! (`benten_engine::engine_sync::DeviceAttestationEnvelope::verify`);
//! durable revocation collapses to user-root UCAN revocation
//! (`benten_caps::revoke`). This module retains only the pure
//! primitives: the `CapabilityEnvelope` ceiling type, the signed
//! `DeviceAttestation` struct + `issue*` / `verify_signature_with` /
//! `canonical_bytes`, `envelope_widens`, and `generate_fresh_nonce`.
//!
//! ## Br-r4-r1-4 / br-r4-r2-3 MAJOR — trust-graph-forgery defense
//!
//! [`DeviceAttestation::issue_with_runtime_check`] rejects at
//! CONSTRUCTION TIME (not invocation time) when a browser-target
//! context attempts to claim `runs_sandbox=true`. The typed error
//! carries the catalog code
//! `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME` per
//! `crates/benten-id/tests/device_attestation.rs::browser_target_with_runs_sandbox_true_claim_rejected_at_attestation_construction_time`.

use ed25519_dalek::{Signature, Verifier};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::did::Did;
use crate::errors::DeviceAttestationError;
use crate::keypair::{Keypair, PublicKey};

/// Zone scope — what storage zones a device participates in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ZoneScope {
    /// Full peer holds every zone the user owns.
    Full,
    /// Thin client carries cache-only views of zones.
    CacheOnly,
    /// Device holds an explicit list of zone names.
    Specific(Vec<String>),
}

impl Default for ZoneScope {
    fn default() -> Self {
        Self::CacheOnly
    }
}

/// Online-uptime policy — when the device is reachable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UptimePolicy {
    /// Long-lived peer (always-on; full-peer phone-OS-app / desktop).
    AlwaysOn,
    /// Session-bounded (browser tab; closes on tab close).
    SessionBounded,
}

impl Default for UptimePolicy {
    fn default() -> Self {
        Self::SessionBounded
    }
}

/// Capability envelope declared by a device-DID.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityEnvelope {
    /// Whether this device executes SANDBOX modules.
    pub runs_sandbox: bool,
    /// Zone-holding shape.
    pub holds_zones: ZoneScope,
    /// Uptime policy.
    pub online_uptime: UptimePolicy,
    /// Whether this device runs as a full Atrium peer.
    pub runs_atrium_peer: bool,
}

impl Default for CapabilityEnvelope {
    fn default() -> Self {
        Self {
            runs_sandbox: false,
            holds_zones: ZoneScope::default(),
            online_uptime: UptimePolicy::default(),
            runs_atrium_peer: false,
        }
    }
}

/// Runtime-target tag used by
/// [`DeviceAttestation::issue_with_runtime_check`] to detect
/// envelope/runtime mismatches at construction time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeTarget {
    /// Native full-peer (host has wasmtime, redb, full disk).
    Native,
    /// Browser thin-client (`wasm32-unknown-unknown`; no wasmtime).
    Browser,
}

// COLLAPSE (P3): `RevocationReason` DELETED with `DeviceRevocation`
// (J3 — the device-revocation parallel pipe). Revocation collapses to
// user-root UCAN revocation (`benten_caps::revoke(ucan_cid)`).

/// Signed device-DID capability attestation.
///
/// Construct via [`DeviceAttestation::issue`] /
/// [`DeviceAttestation::issue_at`] /
/// [`DeviceAttestation::issue_for_browser_target`] /
/// [`DeviceAttestation::issue_with_runtime_check`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceAttestation {
    /// Device-DID (the keypair this device holds).
    pub device_did: String,
    /// Parent-DID (the user-identity DID issuing the attestation).
    pub parent_did: String,
    /// Capability envelope declared by the parent for this device.
    pub envelope: CapabilityEnvelope,
    /// 32-byte nonce — replay defense.
    pub nonce: [u8; 32],
    /// Issuance epoch seconds.
    pub issued_at: u64,
    /// 64-byte Ed25519 signature by the parent keypair.
    ///
    /// **Visibility is load-bearing.** The
    /// `acceptor_rejects_attestation_with_forged_signature` integration
    /// test at `crates/benten-id/tests/device_attestation.rs` mutates
    /// this field directly (`attestation.signature[0] ^= 0x01`) to
    /// drive the negative-pin path. Canonical-bytes round-trip support
    /// (serde-derived serialize/deserialize) also relies on direct
    /// field access. Narrowing this field to `pub(crate)` or routing
    /// it through a setter would silently break the test + the
    /// canonical-bytes contract — update both in lockstep with any
    /// such change.
    pub signature: Vec<u8>,
}

impl DeviceAttestation {
    /// Borrow the device-DID.
    pub fn device_did(&self) -> Did {
        Did::from_string_unchecked(self.device_did.clone())
    }

    /// Borrow the parent-DID.
    pub fn parent_did(&self) -> Did {
        Did::from_string_unchecked(self.parent_did.clone())
    }

    /// Borrow the envelope.
    pub fn envelope(&self) -> &CapabilityEnvelope {
        &self.envelope
    }

    /// Issue an attestation. Generates a fresh OS-CSPRNG nonce; uses
    /// `issued_at = 0` (caller controls the timestamp via
    /// [`DeviceAttestation::issue_at`] for tests).
    pub fn issue(
        parent_kp: &Keypair,
        device_did: Did,
        envelope: CapabilityEnvelope,
    ) -> Result<Self, DeviceAttestationError> {
        Self::issue_at(parent_kp, device_did, envelope, 0)
    }

    /// Issue at a specific epoch second (used by replay-resistance
    /// tests). Generates a fresh OS-CSPRNG nonce.
    pub fn issue_at(
        parent_kp: &Keypair,
        device_did: Did,
        envelope: CapabilityEnvelope,
        issued_at: u64,
    ) -> Result<Self, DeviceAttestationError> {
        // Generate fresh nonce from OS CSPRNG. The buffer is zero-init
        // ONLY as scratch space immediately overwritten by `OsRng::fill_bytes`;
        // CodeQL pattern-match on `[0u8; 32]` is a false-positive that
        // doesn't see the next-line randomization. Per crypto-major-2.
        let nonce = generate_fresh_nonce();
        Self::issue_with_nonce(parent_kp, device_did, envelope, issued_at, nonce)
    }

    /// Issue at a specific epoch second + nonce (used by tests that
    /// pin specific nonce shapes; production callers go through
    /// [`Self::issue_at`]).
    pub fn issue_with_nonce(
        parent_kp: &Keypair,
        device_did: Did,
        envelope: CapabilityEnvelope,
        issued_at: u64,
        nonce: [u8; 32],
    ) -> Result<Self, DeviceAttestationError> {
        let mut attestation = Self {
            device_did: device_did.as_str().to_string(),
            parent_did: parent_kp.public_key().to_did().as_str().to_string(),
            envelope,
            nonce,
            issued_at,
            signature: Vec::new(),
        };
        let bytes = canonical_bytes(&attestation);
        let sig = parent_kp.sign(&bytes);
        attestation.signature = sig.to_bytes().to_vec();
        Ok(attestation)
    }

    /// Issue an attestation pre-populated with the browser-target
    /// minimum-capability envelope per CLAUDE.md baked-in #17 +
    /// D-PHASE-3-25.
    ///
    /// Auto-asserts `runs_sandbox=false`, `holds_zones=CacheOnly`,
    /// `online_uptime=SessionBounded`, `runs_atrium_peer=false`.
    pub fn issue_for_browser_target(
        parent_kp: &Keypair,
        device_did: Did,
    ) -> Result<Self, DeviceAttestationError> {
        let envelope = CapabilityEnvelope {
            runs_sandbox: false,
            holds_zones: ZoneScope::CacheOnly,
            online_uptime: UptimePolicy::SessionBounded,
            runs_atrium_peer: false,
        };
        Self::issue_at(parent_kp, device_did, envelope, 0)
    }

    /// Issue with a runtime-target check. Per `br-r4-r1-4` /
    /// `br-r4-r2-3` MAJOR, a `Browser` runtime + `runs_sandbox=true`
    /// envelope rejects at CONSTRUCTION TIME with the typed
    /// catalog code `E_DEVICE_ATTESTATION_INCOMPATIBLE_WITH_RUNTIME`.
    pub fn issue_with_runtime_check(
        parent_kp: &Keypair,
        device_did: Did,
        envelope: CapabilityEnvelope,
        target: RuntimeTarget,
    ) -> Result<Self, DeviceAttestationError> {
        if target == RuntimeTarget::Browser && envelope.runs_sandbox {
            return Err(DeviceAttestationError::IncompatibleWithRuntime {
                detail: "browser-target runtime cannot honor runs_sandbox=true \
                         (wasmtime unavailable on wasm32-unknown-unknown per Phase-2b \
                         E_SANDBOX_UNAVAILABLE_ON_WASM)",
            });
        }
        if target == RuntimeTarget::Browser && envelope.runs_atrium_peer {
            return Err(DeviceAttestationError::IncompatibleWithRuntime {
                detail: "browser-target runtime cannot honor runs_atrium_peer=true \
                         per CLAUDE.md baked-in #17",
            });
        }
        Self::issue(parent_kp, device_did, envelope)
    }

    /// Issue subject to a parent authority envelope (cap-r4-7 closure).
    /// Rejects if the issuance envelope claims wider authority than
    /// `parent_authority`.
    pub fn issue_with_authority(
        parent_kp: &Keypair,
        device_did: Did,
        envelope: CapabilityEnvelope,
        parent_authority: &CapabilityEnvelope,
    ) -> Result<Self, DeviceAttestationError> {
        if envelope_widens(&envelope, parent_authority) {
            return Err(DeviceAttestationError::EnvelopeWidening {
                detail: "device envelope claims wider authority than parent (cap-r4-7)",
            });
        }
        Self::issue(parent_kp, device_did, envelope)
    }

    /// Verify the signature against the supplied parent public key.
    pub fn verify_signature_with(
        &self,
        parent_pk: &PublicKey,
    ) -> Result<(), DeviceAttestationError> {
        let bytes = canonical_bytes(self);
        let sig_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| DeviceAttestationError::BadSignature)?;
        let sig = Signature::from_bytes(&sig_bytes);
        parent_pk
            .as_verifying_key()
            .verify(&bytes, &sig)
            .map_err(|_| DeviceAttestationError::BadSignature)
    }

    /// Encode to canonical bytes (DAG-CBOR).
    pub fn canonical_bytes(&self) -> Vec<u8> {
        serde_ipld_dagcbor::to_vec(self)
            .expect("DAG-CBOR encoding of fixed-shape DeviceAttestation cannot fail")
    }

    /// Decode from canonical bytes.
    ///
    /// **Hyg-1 #329 — DISAGREE-WITH-EXPLANATION (HARD RULE 12 (c)),
    /// production-zero / test caller exists.** This is the decode half
    /// of the canonical-bytes round-trip contract whose encode half
    /// (`canonical_bytes`) is load-bearing in `issue_with_nonce` /
    /// `verify_signature_with`. The `device_attestation.rs`
    /// integration suite drives it (round-trip pin at
    /// `tests/device_attestation.rs`). Deleting it would break
    /// canonical-bytes symmetry (a wire-format-adjacent surface —
    /// out of scope to mutate per the lane's wire-format rule).
    /// T-3v-D wording sharpen: "production-zero", NOT "zero callers".
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, DeviceAttestationError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(|_| DeviceAttestationError::DecodeFailed)
    }
}

/// Returns `true` if `device` claims wider authority than `parent`
/// in any envelope dimension.
///
/// **g14-a2-mr-6 fix-pass:** zone-widening matrix made exhaustive over
/// all 9 `(parent, device)` combinations. The prior version had two
/// edge cases — `(Specific(empty), Full)` fell through to false and
/// `(Specific(_), CacheOnly)` was implicit. The matrix now enumerates
/// every case explicitly so future contributors don't need to derive
/// semantics from missing match arms.
fn envelope_widens(device: &CapabilityEnvelope, parent: &CapabilityEnvelope) -> bool {
    if device.runs_sandbox && !parent.runs_sandbox {
        return true;
    }
    if device.runs_atrium_peer && !parent.runs_atrium_peer {
        return true;
    }
    // Zone widening: exhaustive 3 × 3 matrix.
    match (&parent.holds_zones, &device.holds_zones) {
        // Parent::Full grants everything; nothing widens.
        (ZoneScope::Full, _) => false,
        // Parent::CacheOnly is the narrowest — anything else widens.
        (ZoneScope::CacheOnly, ZoneScope::CacheOnly) => false,
        (ZoneScope::CacheOnly, ZoneScope::Full) => true,
        (ZoneScope::CacheOnly, ZoneScope::Specific(_)) => true,
        // Parent::Specific is a (possibly-empty) subset.
        // Specific(_) → CacheOnly is a narrowing (read-only); not widening.
        (ZoneScope::Specific(_), ZoneScope::CacheOnly) => false,
        // Specific(_) → Full widens (Full is everything; subset → all is wider).
        // Both empty-and-non-empty parent cases widen since Full > any subset.
        (ZoneScope::Specific(_), ZoneScope::Full) => true,
        // Specific(p) → Specific(c) widens iff any child zone is outside parent.
        // Empty parent + non-empty child → child widens (parent grants nothing).
        // Empty parent + empty child → no widening.
        (ZoneScope::Specific(parent_zones), ZoneScope::Specific(child_zones)) => {
            child_zones.iter().any(|c| !parent_zones.contains(c))
        }
    }
}

/// Canonical-bytes encoding of the signature input. Excludes the
/// `signature` field (signature self-reference hygiene).
fn canonical_bytes(attestation: &DeviceAttestation) -> Vec<u8> {
    #[derive(Serialize)]
    struct SigInput<'a> {
        device_did: &'a str,
        parent_did: &'a str,
        envelope: &'a CapabilityEnvelope,
        nonce: &'a [u8; 32],
        issued_at: u64,
    }
    serde_ipld_dagcbor::to_vec(&SigInput {
        device_did: &attestation.device_did,
        parent_did: &attestation.parent_did,
        envelope: &attestation.envelope,
        nonce: &attestation.nonce,
        issued_at: attestation.issued_at,
    })
    .expect("DAG-CBOR encoding of fixed-shape SigInput cannot fail")
}

// COLLAPSE (P3): `FreshnessPolicy` + `DeviceRevocation` +
// `revocation_canonical_bytes` + `Acceptor` (the device-attestation
// *acceptance* pipe — expected-parent gate / nonce-store replay /
// revocation-list) DELETED per DECISION-RECORD §4 RATIFIED. The
// device envelope is no longer a distinct trust-root. Revocation
// collapses to user-root UCAN revocation (`benten_caps::revoke`);
// the J8 envelope-ceiling is ANDed once at the engine's single
// inbound-sync recheck seam; the receiver-side freshness window is
// a plain `u64` on the Atrium handle (no `Acceptor` / no
// `FreshnessPolicy` type). `envelope_widens` + the kept
// `DeviceAttestation` struct + `generate_fresh_nonce` survive as
// pure primitives.

/// Generate a fresh 32-byte nonce from the OS CSPRNG (per crypto-major-2).
///
/// Composes 4 `u64` reads from `OsRng` and concatenates their byte
/// representations. Avoids the `[0u8; 32]` zero-init literal pattern
/// that CodeQL pattern-matches as "hardcoded nonce" even when the
/// buffer is immediately overwritten. Each `next_u64` call pulls
/// fresh entropy from the OS CSPRNG.
fn generate_fresh_nonce() -> [u8; 32] {
    let r0 = OsRng.next_u64().to_le_bytes();
    let r1 = OsRng.next_u64().to_le_bytes();
    let r2 = OsRng.next_u64().to_le_bytes();
    let r3 = OsRng.next_u64().to_le_bytes();
    [
        r0[0], r0[1], r0[2], r0[3], r0[4], r0[5], r0[6], r0[7], r1[0], r1[1], r1[2], r1[3], r1[4],
        r1[5], r1[6], r1[7], r2[0], r2[1], r2[2], r2[3], r2[4], r2[5], r2[6], r2[7], r3[0], r3[1],
        r3[2], r3[3], r3[4], r3[5], r3[6], r3[7],
    ]
}

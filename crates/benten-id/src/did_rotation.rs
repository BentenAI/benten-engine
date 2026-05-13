//! DID-rotation primitive (G14-A2 wave-4a').
//!
//! ## Crypto-major-3 contract
//!
//! When a private key is lost or compromised, the user rotates to a
//! new keypair. Per `crates/benten-id/tests/did_rotation.rs`,
//! [`rotate_keypair`] emits a signed [`RotationAttestation`] linking
//! the OLD DID to the NEW DID; the attestation is signed by the OLD
//! keypair (proving the rotation was authorized by whoever held the
//! old secret).
//!
//! Cross-wave integration: the durable UCAN backend (G14-B) consumes
//! [`RotationAttestation`] events to revoke pre-rotation UCANs. The
//! G14-A2 surface lands the in-memory primitive + the
//! [`RotationLog`] in-RAM helper that the chain-walker
//! [`is_did_superseded`] consults; G14-B replaces the log with a
//! durable backing store.
//!
//! ## Logical-DID stability under rotation
//!
//! Per `crates/benten-id/tests/did_rotation.rs::did_rotate_keypair_preserves_did_under_canonical_bytes`,
//! the LOGICAL DID's canonical-bytes encoding is stable across
//! rotation events. The canonical-bytes function here just borrows
//! the underlying string bytes (the `did:key:z<...>` string is
//! itself the canonical form per W3C did-method-key spec). On
//! rotation the user's POSSESSION of the secret behind that DID
//! changes; the DID string identity does not.
//!
//! For Phase-3 we treat the OLD did:key as the "logical DID" — it
//! is what UCAN audience fields bind to. The rotation attestation
//! lets verifiers walk forward from the OLD DID to discover the
//! NEW keypair without breaking long-lived audience references.
//! G14-B's durable backend replaces this in-memory walk with a
//! persistent rotation log.

use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};

use crate::did::Did;
use crate::errors::DidRotationError;
use crate::keypair::{Keypair, PublicKey};

/// Signed attestation recording a `did:key` rotation event.
///
/// Construct via [`rotate_keypair`]. The attestation carries:
///
/// - `previous_did`: the OLD DID (signed-by the OLD keypair)
/// - `next_did`: the NEW DID (the rotation target)
/// - `superseded_at`: epoch seconds when the rotation was authorized
/// - `signature`: 64-byte Ed25519 signature by the OLD keypair over
///   the canonical-bytes encoding of the (previous_did, next_did,
///   superseded_at) tuple
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RotationAttestation {
    /// OLD `did:key` DID — signs the attestation.
    pub previous_did: String,
    /// NEW `did:key` DID — the rotation target.
    pub next_did: String,
    /// Epoch seconds at which the OLD keypair authorized the rotation.
    pub superseded_at: u64,
    /// 64-byte Ed25519 signature by the OLD keypair.
    pub signature: Vec<u8>,
}

/// Attestation kind tag — pinned by the must-pass test
/// `did_rotate_keypair_emits_superseded_by_attestation_chain`.
///
/// Phase-3 lands a single kind (`SupersededBy`); the enum keeps a
/// stable shape across post-Phase-3 extensions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttestationKind {
    /// "Old DID is superseded by new DID" — the only kind in Phase 3.
    SupersededBy,
}

impl RotationAttestation {
    /// Convenience accessor returning the kind tag.
    pub fn kind(&self) -> AttestationKind {
        AttestationKind::SupersededBy
    }

    /// Borrow the previous keypair's `did:key`.
    pub fn previous_keypair_did(&self) -> Did {
        Did::from_string_unchecked(self.previous_did.clone())
    }

    /// Borrow the next keypair's `did:key`.
    pub fn next_keypair_did(&self) -> Did {
        Did::from_string_unchecked(self.next_did.clone())
    }

    /// Verify the attestation's signature against the supplied OLD
    /// public key. Returns [`DidRotationError::BadSignature`] on
    /// mismatch.
    pub fn verify_signature_with(&self, old_pk: &PublicKey) -> Result<(), DidRotationError> {
        let bytes = canonical_bytes(self);
        let sig_bytes: [u8; 64] = self
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| DidRotationError::BadSignature)?;
        let sig = Signature::from_bytes(&sig_bytes);
        old_pk
            .as_verifying_key()
            .verify(&bytes, &sig)
            .map_err(|_| DidRotationError::BadSignature)
    }
}

/// Canonical-bytes encoding of the rotation attestation's signature
/// input — `(previous_did, next_did, superseded_at)`. The signature
/// field is intentionally excluded.
fn canonical_bytes(attestation: &RotationAttestation) -> Vec<u8> {
    #[derive(Serialize)]
    struct SigInput<'a> {
        previous_did: &'a str,
        next_did: &'a str,
        superseded_at: u64,
    }
    serde_ipld_dagcbor::to_vec(&SigInput {
        previous_did: &attestation.previous_did,
        next_did: &attestation.next_did,
        superseded_at: attestation.superseded_at,
    })
    .expect("DAG-CBOR encoding of fixed-shape SigInput cannot fail")
}

/// Rotate a keypair, producing a [`RotationAttestation`] signed by
/// the OLD keypair.
///
/// `did` must equal the OLD keypair's `did:key` (rejected with
/// [`DidRotationError::PreviousDidMismatch`] otherwise — defends
/// against caller bugs that pass mis-matched did/keypair pairs).
pub fn rotate_keypair(
    did: &Did,
    old_kp: &Keypair,
    new_kp: &Keypair,
    superseded_at: u64,
) -> Result<RotationAttestation, DidRotationError> {
    let old_did = old_kp.public_key().to_did();
    if did.as_str() != old_did.as_str() {
        return Err(DidRotationError::PreviousDidMismatch {
            claimed: did.as_str().to_string(),
            actual: old_did.as_str().to_string(),
        });
    }
    let new_did = new_kp.public_key().to_did();
    let mut attestation = RotationAttestation {
        previous_did: old_did.as_str().to_string(),
        next_did: new_did.as_str().to_string(),
        superseded_at,
        signature: Vec::new(),
    };
    let bytes = canonical_bytes(&attestation);
    let sig = old_kp.sign(&bytes);
    attestation.signature = sig.to_bytes().to_vec();
    Ok(attestation)
}

/// In-RAM rotation log for chain-walk consultation.
///
/// G14-B replaces this with a durable backing store. The shape here
/// (a flat list of [`RotationAttestation`] entries) is sufficient
/// for the must-pass tests — chain-walkers consult [`Self::is_superseded`]
/// to determine whether a UCAN issuer DID has been rotated.
#[derive(Default, Debug, Clone)]
pub struct RotationLog {
    entries: Vec<RotationAttestation>,
}

impl RotationLog {
    /// Construct an empty log.
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct from a list of attestations. The list MUST have
    /// pre-verified signatures — this constructor does not re-verify.
    pub fn from_entries(entries: Vec<RotationAttestation>) -> Self {
        Self { entries }
    }

    /// Append an attestation.
    pub fn append(&mut self, attestation: RotationAttestation) {
        self.entries.push(attestation);
    }

    /// G24-D-FP-2 (per phase-4-backlog §4.10) — accept a rotation event
    /// with HLC-monotonic-strict ordering + nonce-binding replay defense.
    ///
    /// Defenses (composed):
    /// 1. **Verbatim replay**: identical `(previous_did, next_did,
    ///    superseded_at, signature)` as an existing entry is rejected
    ///    with [`DidRotationError::VerbatimReplay`].
    /// 2. **HLC-monotonic-strict**: an incoming event for `previous_did`
    ///    whose `superseded_at` is NOT strictly greater than the latest
    ///    accepted `superseded_at` for the same `previous_did` is
    ///    rejected with [`DidRotationError::HlcNotStrictlyMonotonic`].
    ///    This is the defense against nonce-swap attacks: even if the
    ///    attacker mutates the nonce / signature, the HLC of the
    ///    replay event is the same as the original, so the strict-
    ///    monotonic check rejects it.
    ///
    /// # Errors
    ///
    /// Returns [`DidRotationError::VerbatimReplay`] on byte-identical
    /// replay; [`DidRotationError::HlcNotStrictlyMonotonic`] on
    /// at-or-before-HLC replay.
    pub fn accept_rotation_event(
        &mut self,
        attestation: &RotationAttestation,
    ) -> Result<(), crate::errors::DidRotationError> {
        // Verbatim replay defense — DID + signature compares routed
        // through ct_signature_eq per crypto-major-4 UNIFORMITY.
        if self.entries.iter().any(|e| {
            crate::ucan::ct_signature_eq(
                e.previous_did.as_bytes(),
                attestation.previous_did.as_bytes(),
            ) && crate::ucan::ct_signature_eq(
                e.next_did.as_bytes(),
                attestation.next_did.as_bytes(),
            ) && e.superseded_at == attestation.superseded_at
                && crate::ucan::ct_signature_eq(&e.signature, &attestation.signature)
        }) {
            return Err(crate::errors::DidRotationError::VerbatimReplay {
                prev_did: attestation.previous_did.clone(),
                hlc: attestation.superseded_at,
            });
        }
        // HLC-monotonic-strict defense: latest superseded_at for the
        // same previous_did MUST be strictly less than incoming.
        let latest = self
            .entries
            .iter()
            .filter(|e| {
                crate::ucan::ct_signature_eq(
                    e.previous_did.as_bytes(),
                    attestation.previous_did.as_bytes(),
                )
            })
            .map(|e| e.superseded_at)
            .max();
        if let Some(latest_hlc) = latest
            && attestation.superseded_at <= latest_hlc
        {
            return Err(crate::errors::DidRotationError::HlcNotStrictlyMonotonic {
                prev_did: attestation.previous_did.clone(),
                incoming_hlc: attestation.superseded_at,
                latest_hlc,
            });
        }
        self.entries.push(attestation.clone());
        Ok(())
    }

    /// Return `true` if `did` has been superseded by any attestation
    /// in the log. Used by the chain-walker to reject UCANs signed
    /// by a superseded keypair.
    pub fn is_superseded(&self, did: &Did) -> bool {
        // ct-eq per crypto-major-4 UNIFORMITY (g14-a2-mr-2 fix-pass).
        self.entries.iter().any(|a| {
            crate::ucan::ct_signature_eq(a.previous_did.as_bytes(), did.as_str().as_bytes())
        })
    }

    /// Borrow the entries.
    pub fn entries(&self) -> &[RotationAttestation] {
        &self.entries
    }
}

/// Convenience: returns `true` if `did` has been superseded by any
/// attestation in `log`. Equivalent to [`RotationLog::is_superseded`].
pub fn is_did_superseded(did: &Did, log: &RotationLog) -> bool {
    log.is_superseded(did)
}

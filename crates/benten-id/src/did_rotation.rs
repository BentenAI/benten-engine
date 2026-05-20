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
//! Cross-wave integration: the durable UCAN backend consumes
//! [`RotationAttestation`] events to revoke pre-rotation UCANs. This
//! surface lands the in-memory primitive + the [`RotationLog`] in-RAM
//! helper that the chain-walker consults via
//! [`RotationLog::is_superseded`]. Durable rehydration of the log at
//! engine-open (replacing the in-RAM helper with a persistent backing
//! store) is named for Phase-4-Meta at
//! `docs/future/phase-4-backlog.md §4.26` (RotationLog rehydration at
//! engine open).
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
//! Durable rehydration of this in-memory walk is named for
//! Phase-4-Meta at `docs/future/phase-4-backlog.md §4.26`.

use benten_crypto_suite::primitives::ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};

use crate::CanonicalBytes;
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
///
/// **Qual-1 #722 — DISAGREE-WITH-EXPLANATION (HARD RULE 12 (c)).**
/// The "1-variant enum + `kind()` returning a constant" is not
/// premature flexibility to collapse: `AttestationKind` is a
/// `Serialize`/`Deserialize` discriminator that a must-pass test pins
/// (`did_rotation.rs:29`). Collapsing it to a unit would (a) break
/// that pin and (b) force a wire-shape change when a second kind
/// lands post-Phase-3 — the deliberate forward-stable shape is the
/// point. Wire-format-adjacent; out of scope to mutate per the lane
/// rule regardless.
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

    /// Borrow the previous keypair's `did:key` as a typed [`Did`].
    ///
    /// **Qual-1 #711 — DISAGREE-WITH-EXPLANATION, production-zero /
    /// test caller exists.** These two accessors wrap the pub `String`
    /// fields into the typed [`Did`] newtype; the `did_rotation.rs`
    /// integration suite (`:31`, `:35`) relies on them. They are not
    /// redundant re-wraps to delete (CLAUDE.md #5): the typed-`Did`
    /// return is the safe handle callers should use rather than
    /// touching the raw `String`; deleting them would push every
    /// consumer to re-implement `Did::from_string_for_test_fixture` inline.
    pub fn previous_keypair_did(&self) -> Did {
        Did::from_string_for_test_fixture(self.previous_did.clone())
    }

    /// Borrow the next keypair's `did:key` as a typed [`Did`]. See
    /// [`RotationAttestation::previous_keypair_did`] for the Qual-1
    /// #711 disposition.
    pub fn next_keypair_did(&self) -> Did {
        Did::from_string_for_test_fixture(self.next_did.clone())
    }

    /// Verify the attestation's signature against the supplied OLD
    /// public key. Returns [`DidRotationError::BadSignature`] on
    /// mismatch.
    pub fn verify_signature_with(&self, old_pk: &PublicKey) -> Result<(), DidRotationError> {
        let bytes = self.to_canonical_bytes();
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
/// field is intentionally excluded (signature self-reference hygiene
/// per the [`CanonicalBytes`] contract).
///
/// Qual-2 #759: byte-identical reproduction of the prior free-fn
/// `to_canonical_bytes(&RotationAttestation)` body, lifted onto the
/// shared trait. The `SigInput` projection + DAG-CBOR encoding are
/// unchanged (v1-wire-adjacent — §3.5m P-III; covered by the
/// byte-equality pin in `tests/canonical_bytes_trait.rs`).
impl crate::CanonicalBytes for RotationAttestation {
    fn to_canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct SigInput<'a> {
            previous_did: &'a str,
            next_did: &'a str,
            superseded_at: u64,
        }
        serde_ipld_dagcbor::to_vec(&SigInput {
            previous_did: &self.previous_did,
            next_did: &self.next_did,
            superseded_at: self.superseded_at,
        })
        .expect("DAG-CBOR encoding of fixed-shape SigInput cannot fail")
    }
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
    // ct-eq per crypto-major-4 UNIFORMITY (#599). DIDs are public so
    // the leak surface is essentially zero, but the project commits to
    // ct-eq at EVERY security-decision compare; this is the
    // caller-DID-vs-derived-DID rejection arm.
    if !crate::ucan::ct_signature_eq(did.as_str().as_bytes(), old_did.as_str().as_bytes()) {
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
    let bytes = attestation.to_canonical_bytes();
    let sig = old_kp.sign(&bytes);
    attestation.signature = sig.to_bytes().to_vec();
    Ok(attestation)
}

/// In-RAM rotation log for chain-walk consultation.
///
/// Durable rehydration of this log at engine-open is named for
/// Phase-4-Meta at `docs/future/phase-4-backlog.md §4.26`. The shape
/// here (a flat list of [`RotationAttestation`] entries) is sufficient
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
    /// 3. **Authenticity (signature-verify)**: the attestation MUST
    ///    carry a valid Ed25519 signature by the OLD keypair behind
    ///    `previous_did`. The `previous_did` is a self-resolving
    ///    `did:key` string (the public key is encoded in the DID per
    ///    W3C did-method-key), so the verifying key is resolved from
    ///    `previous_did` itself — no caller-supplied key parameter is
    ///    needed (and no breaking API change). This is the
    ///    **authenticity axis** the verbatim-replay + HLC-strict
    ///    defenses presuppose: those gate the *ordering* of replay but
    ///    assume the underlying attestation is genuine. Without this
    ///    gate (Safe-1 #509 / F-FWD-2-01 #1051), a peer that
    ///    synthesizes a `RotationAttestation` byte-blob with ANY
    ///    64-byte signature could be `accept`-ed and silently revoke a
    ///    recipient's view of an issuer-DID (fail-OPEN in the worst
    ///    direction). This is the same authenticity-gate step-4
    ///    pattern the (COLLAPSE-deleted) device-attestation acceptance
    ///    pipe enforced before its envelope-ceiling recheck.
    ///
    /// # Errors
    ///
    /// Returns [`DidRotationError::BadSignature`] when `previous_did`
    /// is unresolvable OR the signature does not verify against the
    /// resolved OLD public key; [`DidRotationError::VerbatimReplay`]
    /// on byte-identical replay; [`DidRotationError::HlcNotStrictlyMonotonic`]
    /// on at-or-before-HLC replay.
    pub fn accept_rotation_event(
        &mut self,
        attestation: &RotationAttestation,
    ) -> Result<(), crate::errors::DidRotationError> {
        // AUTHENTICITY GATE (Safe-1 #509 / F-FWD-2-01 #1051): verify
        // the attestation is genuinely signed by the OLD keypair
        // BEFORE any ordering check. `previous_did` is a self-resolving
        // did:key; resolve it to the OLD public key and verify the
        // 64-byte Ed25519 signature. Resolution failure OR signature
        // mismatch both map to BadSignature (matches Acceptor step-4).
        let prev_did = Did::from_string_for_test_fixture(attestation.previous_did.clone());
        let old_pk = prev_did
            .resolve()
            .map_err(|_| crate::errors::DidRotationError::BadSignature)?;
        attestation.verify_signature_with(&old_pk)?;
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

// Hyg-1 #301: the `is_did_superseded(did, log)` free-fn alias is
// removed — it was a no-op wrapper over `RotationLog::is_superseded`
// with zero callers (only a module-doc backreference, fixed in #428).
// CLAUDE.md #5 (no deprecated aliases / no-op shims). Callers use
// `RotationLog::is_superseded` (or `is_did_superseded`'s prior
// behavior via `log.is_superseded(did)`) directly.

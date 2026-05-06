//! W3C Verifiable Credential v1.1-INSPIRED surface (G14-A2 wave-4a').
//!
//! ## Wire-format compatibility (per g14-a2-mr-3 docstring sharpen)
//!
//! This surface ships W3C VC v1.1-INSPIRED **field shape** over
//! DAG-CBOR + Ed25519. It is **NOT wire-format-compatible** with
//! external W3C JSON-LD VC consumers:
//!
//! - dates are emitted as `u64` epoch seconds, NOT ISO 8601 strings
//! - the encoding is DAG-CBOR, NOT JSON-LD
//! - `proof: Vec<u8>` is a flat 64-byte Ed25519 signature, NOT the
//!   W3C Linked-Data-Proofs envelope (`type: Ed25519Signature2020` +
//!   `verificationMethod` + `proofPurpose` + `created` + `proofValue`)
//!
//! External W3C VC consumers cannot verify these credentials without a
//! translation layer. That translation layer (full JSON-LD / LDP
//! interop via `ssi`) lives at G14-B per
//! `docs/future/phase-3-backlog.md §2.1-followup`. Internal
//! Phase-3 consumers (Atrium replicas, capability backend) verify
//! these credentials directly via this hand-rolled surface.
//!
//!
//! ## Crypto-minor-1 contract
//!
//! Issuance: [`Credential::builder`] → [`CredentialBuilder::sign`].
//! Verification: [`verify`] / [`verify_at`] / [`verify_with_registry`] /
//! [`verify_in_trust_domain`] / [`verify_bytes_in_trust_domain`].
//!
//! Each rejection mode fires a DISTINCT [`crate::errors::VcError`]
//! variant:
//!
//! - `Expired` — `expirationDate <= now`
//! - `Revoked` — `credentialStatus.id` listed in
//!   [`RevocationRegistry`]
//! - `IssuerNotTrusted` — issuer DID not in the [`TrustDomain`]
//!   allow-list
//! - `BadSignature` — signature does not verify against the issuer
//!   DID's resolved public key
//! - `DecodeFailed` / `MissingField` — malformed canonical bytes
//!
//! ## On `ssi` (re-introduction-deferred follow-up)
//!
//! The dispatch brief authorized re-introducing the `ssi` crate for
//! W3C VC v1.1 spec-completeness; the workspace does NOT currently
//! list `ssi` as a workspace dep, and pulling it in transitively
//! brings JSON-LD / `serde_json` heavy machinery + a non-trivial
//! transitive graph that materially shifts the wasm32-target
//! posture of the napi binding's full-peer side.
//!
//! Per Q3 IFF-clause + HARD RULE rule-12 (only 3 valid non-FIX-NOW
//! dispositions), this is `(c) DISAGREE-WITH-EXPLANATION` against
//! the brief's optional-but-recommended `ssi` re-intro: we land
//! G14-A2 with hand-rolled W3C VC v1.1-INSPIRED field shape over
//! the existing DAG-CBOR + Ed25519 surface that `keypair.rs` already
//! carries (NOT wire-format-compatible — see the Wire-format
//! compatibility section above). The follow-up entry stays at
//! `docs/future/phase-3-backlog.md §2.1-followup` (ssi-for-VC-spec-
//! completeness — the JSON-LD / Linked-Data-Proofs interop layer
//! that lets external systems consume our VCs) — that is the
//! genuine value of `ssi`, and it is naturally G14-B's external-
//! interop scope.
//!
//! The crypto-minor-1 contract (issuance + verification + expiration +
//! revocation + trust-domain + no-panic-on-malformed-input) is
//! independent of the JSON-LD layer and is fully covered by the
//! hand-rolled surface here.
//!
//! ## Cag-r4-2 (graph-encoding) — RED-PHASE-deferred
//!
//! Per `cag-r4-2` MAJOR, VC receipts MUST persist as graph Nodes with
//! label `id:vc-receipt` + structured properties. That pin requires
//! `benten_core::Node` / `benten_core::Edge` reach, which is a
//! cross-crate seam — `benten-core` is upstream of `benten-id` per
//! `arch-r1-10`'s dependency-edge contract (`benten-id` cannot depend
//! on `benten-graph`; `benten-core` carries `Node` / `Edge` types).
//! The graph-encoding test pin
//! (`benten_id_vc_issuance_receipt_persisted_as_graph_node`) stays
//! `#[ignore]`'d at G14-A2 with rationale routing it to G14-B (where
//! the durable backend lands the graph-side persistence layer). Per
//! HARD RULE rule-12 disposition (b), the BELONGS-ELSEWHERE entry
//! lives at `crates/benten-id/tests/graph_encoded.rs` (named
//! destination) with the rationale string anchored on the test
//! body's `#[ignore = "..."]` attribute.

use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Mutex;

use crate::did::Did;
use crate::errors::VcError;
use crate::keypair::{Keypair, PublicKey};

/// W3C VC v1.1 `@context` literal — the primary VC context URL.
pub const VC_CONTEXT_V1: &str = "https://www.w3.org/2018/credentials/v1";

/// W3C VC v1.1 `type` literal — every VC carries this base type.
pub const VC_TYPE_BASE: &str = "VerifiableCredential";

/// Verifiable Credential — the W3C v1.1 core fields.
///
/// Built via [`Credential::builder`] / [`CredentialBuilder::sign`].
/// The `proof` field carries the Ed25519 signature over the
/// canonical-bytes encoding of the [`CredentialClaims`] payload (the
/// signature input excludes `proof` itself per signature-self-reference
/// hygiene).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Credential {
    /// Inner claim payload — what is signed.
    pub claims: CredentialClaims,
    /// 64-byte Ed25519 signature.
    pub proof: Vec<u8>,
}

/// W3C VC v1.1 claim payload (signature input).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialClaims {
    /// `@context` URLs.
    pub context: Vec<String>,
    /// Credential identifier.
    pub id: Option<String>,
    /// Credential types (always carries `VerifiableCredential` first).
    pub r#type: Vec<String>,
    /// Issuer DID.
    pub issuer: String,
    /// Issuance epoch seconds (W3C `issuanceDate` field, normalized
    /// to epoch seconds for canonical-bytes stability).
    pub issuance_date: u64,
    /// Optional expiration epoch seconds (W3C `expirationDate`).
    pub expiration_date: Option<u64>,
    /// Subject DID + claim key/value pairs.
    pub credential_subject: CredentialSubject,
    /// Optional credentialStatus URL identifier (revocation registry
    /// lookup key).
    pub credential_status: Option<CredentialStatus>,
}

/// `credentialSubject` entry. The W3C spec allows arbitrary
/// JSON-LD here; for G14-A2 we carry a subject-DID + a flat
/// `(key, value)` claim pair (sufficient for the must-pass test
/// fleet).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialSubject {
    /// Subject DID.
    pub id: String,
    /// Single claim — name → value.
    pub claim_name: String,
    /// Single claim — value.
    pub claim_value: String,
}

/// `credentialStatus` reference — points at a revocation registry
/// entry.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CredentialStatus {
    /// Status entry identifier (e.g. `https://example/status/1#42`).
    pub id: String,
    /// W3C VC v1.1 status type (informational; the registry consults
    /// `id` regardless of type tag).
    pub r#type: String,
}

impl Credential {
    /// Begin a builder.
    pub fn builder() -> CredentialBuilder {
        CredentialBuilder::default()
    }

    /// Compute canonical-bytes for the claims payload (signature input).
    fn canonical_bytes(&self) -> Vec<u8> {
        canonical_bytes(&self.claims)
    }

    /// Borrow the issuer DID string.
    pub fn issuer(&self) -> &str {
        &self.claims.issuer
    }

    /// Borrow the subject DID string.
    pub fn subject(&self) -> &str {
        &self.claims.credential_subject.id
    }

    /// Borrow the single (claim_name, claim_value) pair.
    pub fn claim(&self) -> (&str, &str) {
        (
            &self.claims.credential_subject.claim_name,
            &self.claims.credential_subject.claim_value,
        )
    }
}

fn canonical_bytes(claims: &CredentialClaims) -> Vec<u8> {
    serde_ipld_dagcbor::to_vec(claims)
        .expect("DAG-CBOR encoding of fixed-shape CredentialClaims cannot fail")
}

/// Builder for issuing [`Credential`] values.
#[derive(Default)]
pub struct CredentialBuilder {
    id: Option<String>,
    types: Vec<String>,
    issuer: Option<String>,
    issuance_date: Option<u64>,
    expiration_date: Option<u64>,
    subject_id: Option<String>,
    claim: Option<(String, String)>,
    credential_status: Option<CredentialStatus>,
}

impl CredentialBuilder {
    /// Set the `id` field (optional).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Add a credential type (the base `VerifiableCredential` type is
    /// auto-prepended at sign time).
    #[must_use]
    pub fn extra_type(mut self, t: impl Into<String>) -> Self {
        self.types.push(t.into());
        self
    }

    /// Set the issuer DID.
    #[must_use]
    pub fn issuer(mut self, did: &Did) -> Self {
        self.issuer = Some(did.as_str().to_string());
        self
    }

    /// Set the issuance epoch seconds.
    #[must_use]
    pub fn issued_at(mut self, secs: u64) -> Self {
        self.issuance_date = Some(secs);
        self
    }

    /// Set the expiration epoch seconds.
    #[must_use]
    pub fn expires_at(mut self, secs: u64) -> Self {
        self.expiration_date = Some(secs);
        self
    }

    /// Set the subject DID.
    #[must_use]
    pub fn subject(mut self, did: &Did) -> Self {
        self.subject_id = Some(did.as_str().to_string());
        self
    }

    /// Set the single claim (name + value).
    #[must_use]
    pub fn claim(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.claim = Some((name.into(), value.into()));
        self
    }

    /// Set the `credentialStatus` reference (revocation registry
    /// lookup key).
    #[must_use]
    pub fn credential_status_id(mut self, id: impl Into<String>) -> Self {
        self.credential_status = Some(CredentialStatus {
            id: id.into(),
            r#type: "RevocationList2020Status".to_string(),
        });
        self
    }

    /// Sign the assembled claims with `issuer_kp`. Returns
    /// `MissingField` if `issuer` / `issuance_date` / `subject` /
    /// `claim` were not set.
    pub fn sign(self, issuer_kp: &Keypair) -> Result<Credential, VcError> {
        let issuer = self
            .issuer
            .ok_or(VcError::MissingField { field: "issuer" })?;
        let issuance_date = self.issuance_date.ok_or(VcError::MissingField {
            field: "issuanceDate",
        })?;
        let subject_id = self.subject_id.ok_or(VcError::MissingField {
            field: "credentialSubject.id",
        })?;
        let (claim_name, claim_value) = self.claim.ok_or(VcError::MissingField {
            field: "credentialSubject.claim",
        })?;

        let mut types = Vec::with_capacity(self.types.len() + 1);
        types.push(VC_TYPE_BASE.to_string());
        types.extend(self.types);

        let claims = CredentialClaims {
            context: vec![VC_CONTEXT_V1.to_string()],
            id: self.id,
            r#type: types,
            issuer,
            issuance_date,
            expiration_date: self.expiration_date,
            credential_subject: CredentialSubject {
                id: subject_id,
                claim_name,
                claim_value,
            },
            credential_status: self.credential_status,
        };

        let bytes = canonical_bytes(&claims);
        let sig = issuer_kp.sign(&bytes);
        Ok(Credential {
            claims,
            proof: sig.to_bytes().to_vec(),
        })
    }
}

/// Trust-domain allow-list of issuer DIDs.
///
/// Construct via [`TrustDomain::new`] / [`TrustDomain::empty`]. Used
/// by [`verify_in_trust_domain`] / [`verify_bytes_in_trust_domain`]
/// to gate VC acceptance on issuer identity.
#[derive(Clone, Debug, Default)]
pub struct TrustDomain {
    issuers: HashSet<String>,
}

impl TrustDomain {
    /// Construct a trust-domain from a list of issuer DIDs.
    pub fn new(issuers: Vec<Did>) -> Self {
        Self {
            issuers: issuers
                .into_iter()
                .map(|d| d.as_str().to_string())
                .collect(),
        }
    }

    /// Construct an empty trust-domain (rejects all VCs).
    pub fn empty() -> Self {
        Self::default()
    }

    /// Add an issuer DID to the allow-list.
    pub fn allow(&mut self, did: &Did) {
        self.issuers.insert(did.as_str().to_string());
    }

    /// Check whether `issuer` is allow-listed.
    pub fn contains(&self, issuer: &str) -> bool {
        self.issuers.contains(issuer)
    }
}

/// In-RAM revocation registry. Phase 3 G14-B replaces this with a
/// durable backend; the trait surface here is intentionally minimal
/// (the must-pass tests drive `revoke` + `is_revoked`).
#[derive(Default)]
pub struct RevocationRegistry {
    revoked: Mutex<HashSet<String>>,
}

impl RevocationRegistry {
    /// Construct an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark a `credentialStatus.id` as revoked.
    pub fn revoke(&self, status_id: impl Into<String>) {
        self.revoked
            .lock()
            .expect("registry mutex poisoned")
            .insert(status_id.into());
    }

    /// Check whether a `credentialStatus.id` is revoked.
    pub fn is_revoked(&self, status_id: &str) -> bool {
        self.revoked
            .lock()
            .expect("registry mutex poisoned")
            .contains(status_id)
    }
}

/// Verify a Verifiable Credential against the issuer DID's resolved
/// public key. Skips `expirationDate` (use [`verify_at`] for the
/// timed gate). Returns [`VcError::BadSignature`] on signature
/// mismatch.
pub fn verify(vc: &Credential, expected_issuer: &Did) -> Result<(), VcError> {
    if vc.claims.issuer != expected_issuer.as_str() {
        return Err(VcError::BadSignature);
    }
    let pk: PublicKey = expected_issuer
        .resolve()
        .map_err(|_| VcError::BadSignature)?;
    let sig_bytes: [u8; 64] = vc
        .proof
        .as_slice()
        .try_into()
        .map_err(|_| VcError::BadSignature)?;
    let sig = Signature::from_bytes(&sig_bytes);
    let bytes = vc.canonical_bytes();
    pk.as_verifying_key()
        .verify(&bytes, &sig)
        .map_err(|_| VcError::BadSignature)?;
    Ok(())
}

/// Verify a VC at a given `now` epoch second, enforcing the
/// `expirationDate` gate per `crypto-minor-1`. Composes with
/// [`verify`]; both signature verification and expiration checking
/// must pass.
pub fn verify_at(vc: &Credential, expected_issuer: &Did, now: u64) -> Result<(), VcError> {
    if vc.claims.issuance_date > now {
        return Err(VcError::NotYetValid {
            issued_at: vc.claims.issuance_date,
            now,
        });
    }
    if let Some(exp) = vc.claims.expiration_date
        && now >= exp
    {
        return Err(VcError::Expired { exp, now });
    }
    verify(vc, expected_issuer)
}

/// Verify a VC against the issuer DID, consulting `registry` to
/// reject revoked credentials. Returns [`VcError::Revoked`] if the
/// `credentialStatus.id` is listed in the registry.
pub fn verify_with_registry(
    vc: &Credential,
    expected_issuer: &Did,
    registry: &RevocationRegistry,
) -> Result<(), VcError> {
    if let Some(status) = &vc.claims.credential_status
        && registry.is_revoked(&status.id)
    {
        return Err(VcError::Revoked {
            status_id: status.id.clone(),
        });
    }
    verify(vc, expected_issuer)
}

/// Verify a VC under a [`TrustDomain`] allow-list. Rejects with
/// [`VcError::IssuerNotTrusted`] if the issuer is not on the list,
/// independent of signature validity.
pub fn verify_in_trust_domain(vc: &Credential, trust_domain: &TrustDomain) -> Result<(), VcError> {
    if !trust_domain.contains(&vc.claims.issuer) {
        return Err(VcError::IssuerNotTrusted {
            issuer: vc.claims.issuer.clone(),
        });
    }
    let issuer = Did::from_string_unchecked(vc.claims.issuer.clone());
    verify(vc, &issuer)
}

/// Verify raw canonical bytes (untrusted-input path) under a
/// [`TrustDomain`]. The proptest fleet at
/// `crates/benten-id/tests/prop_vc_arbitrary.rs` drives 10 000
/// arbitrary byte sequences through this entry point and asserts NO
/// panic — the function returns a typed [`VcError`] for every
/// rejection, including malformed CBOR.
pub fn verify_bytes_in_trust_domain(
    bytes: &[u8],
    trust_domain: &TrustDomain,
) -> Result<(), VcError> {
    let vc: Credential =
        serde_ipld_dagcbor::from_slice(bytes).map_err(|_| VcError::DecodeFailed)?;
    verify_in_trust_domain(&vc, trust_domain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_field_round_trip() {
        // Builder rejects empty input with typed MissingField.
        let kp = Keypair::generate();
        let err = Credential::builder().sign(&kp).unwrap_err();
        assert!(matches!(err, VcError::MissingField { .. }));
    }
}

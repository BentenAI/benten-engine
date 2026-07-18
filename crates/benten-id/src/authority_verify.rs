//! GAP-KDB Shape-B / Fork-A — the ONE codepoint-dispatched authority
//! signature verify.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A + Worry-#1 ("verifying
//! only the Ed25519 half of a composite `did:benten` issuer is a **silent
//! PQ-downgrade on the authority path**; the walk MUST become ONE
//! codepoint-dispatched hybrid verify, not a bolt-on"). Before Fork-A the
//! authority path (UCAN chain-walk, rotation-verify, device-attestation,
//! VC-verify) each carried its OWN inline Ed25519-only `[u8; 64]` signature
//! extraction — four candidate silent-PQ-strip sites. This module is the
//! single shared verify all four route through.
//!
//! # Dispatch is on the RESOLVED SIGNING-KEY SHAPE, never the wire
//! The one load-bearing property: the arm is selected by the issuer's
//! resolved [`sig::PublicKey`] shape (recovered zero-I/O from the issuer
//! DID via [`crate::did::Did::resolve_signing`]), NOT by the signature
//! bytes.
//!
//! - A composite issuer (`did:benten`, or a hybrid `did:key`) →
//!   `is_hybrid() == true` → the FULL LAMPS composite is verified via
//!   [`SignatureSuite::verify`] (already strip-resistant: both the ML-DSA
//!   and Ed25519 halves must verify against the SAME representative). A
//!   token carrying only the classical half fails the composite wire-length
//!   decode; a token with a forged/zeroed ML-DSA half fails the ML-DSA
//!   verify. **NEVER an Ed25519-only accept for a composite issuer** — this
//!   is the closure of FLAGSHIP-2.
//! - A classical issuer (`did:key`, `pq = None`) → the single shared
//!   Ed25519-only arm (the ONE `[u8; 64]` extraction in the whole
//!   authority surface — AUTH-7 completeness net).
//!
//! Because `resolve_signing` fails closed on a `did:benten` string
//! (returning the composite key, never a classical fallback), a caller
//! cannot resolve a composite identity down to a classical key and then
//! silent-strip it — the shape drives the arm.

use benten_crypto_suite::SigCodepoint;
use benten_crypto_suite::sig::{self, HybridSignature, SignatureSuite, SuiteConfig};

/// Failure of the consolidated authority-signature verify. Callers map
/// both variants onto their own typed `BadSignature` (the authority-path
/// audit trail lives in the caller's error).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityVerifyError {
    /// The signature wire is the wrong length / shape for the resolved
    /// signing key (e.g. a 64-byte classical wire presented for a composite
    /// issuer — the silent-PQ-strip shape).
    Malformed, // drift-detect: internal-only
    /// The signature did not cryptographically verify against the resolved
    /// signing key (a forged half, a tampered message, or a wrong key).
    VerifyFailed, // drift-detect: internal-only
}

/// A resolved public-key handle that can be projected to the crypto-suite
/// [`sig::PublicKey`] the authority verify dispatches on.
///
/// Implemented for both issuer-key shapes so the ONE
/// [`crate::device_attestation::DeviceAttestation::verify_signature_with`] /
/// [`crate::did_rotation::RotationAttestation::verify_signature_with`]
/// entry accepts BOTH a composite `sig::PublicKey` (from
/// [`crate::did::Did::resolve_signing`], a `did:benten` / hybrid `did:key`
/// issuer) AND the classical [`crate::keypair::PublicKey`] legacy callers
/// already hold — without a second verify path.
pub trait ToSigningKey {
    /// Project this handle to the crypto-suite signing key.
    fn to_signing_key(&self) -> sig::PublicKey;
}

impl ToSigningKey for sig::PublicKey {
    fn to_signing_key(&self) -> sig::PublicKey {
        self.clone()
    }
}

impl ToSigningKey for crate::keypair::PublicKey {
    fn to_signing_key(&self) -> sig::PublicKey {
        // A `benten_id::keypair::PublicKey` is ALWAYS a classical Ed25519
        // verifying key (it is only reachable via the classical `did:key`
        // resolve path; a `did:benten` string never resolves to it). So the
        // projection is unconditionally the `pq = None` classical handle —
        // it can never mis-type a composite identity as classical.
        sig::PublicKey::from_classical_ed25519_bytes(&self.to_bytes()).expect(
            "a benten_id::keypair::PublicKey is a valid Ed25519 curve point by construction",
        )
    }
}

/// The single Fork-A codepoint-dispatched authority-signature verify.
///
/// `signing_pk` is the issuer's resolved signing key (its SHAPE selects the
/// arm); `msg` is the canonical-bytes signature input; `sig_wire` is the
/// raw signature bytes off the wire.
///
/// # Errors
///
/// [`AuthorityVerifyError::Malformed`] on a wire shape that does not match
/// the resolved key; [`AuthorityVerifyError::VerifyFailed`] on a
/// cryptographic verify failure. Fail-closed — never a silent accept.
pub fn verify_authority_signature(
    signing_pk: &sig::PublicKey,
    msg: &[u8],
    sig_wire: &[u8],
) -> Result<(), AuthorityVerifyError> {
    if signing_pk.is_hybrid() {
        // Composite arm — the wire MUST be a full LAMPS composite
        // (`mldsaSig ‖ tradSig`). A bare / PQ-stripped 64-byte wire fails
        // this length-exact decode (the silent-PQ-strip shape), and a
        // full-length forged/zeroed ML-DSA half fails the crypto verify
        // below. `SignatureSuite::verify` requires BOTH halves to verify
        // against the SAME LAMPS representative — never a single-half accept.
        let sig = HybridSignature::from_lamps_composite_wire(sig_wire)
            .map_err(|_| AuthorityVerifyError::Malformed)?;
        SignatureSuite::v1_default()
            .verify(signing_pk.clone(), msg, &sig)
            .map_err(|_| AuthorityVerifyError::VerifyFailed)
    } else {
        // Classical arm — the SINGLE Ed25519-only `[u8; 64]` signature
        // extraction in the entire authority-verify surface (AUTH-7). A
        // re-introduced inline extraction elsewhere is a candidate
        // silent-PQ-strip site and fails the AUTH-7 grep-audit net.
        let sig_bytes: [u8; 64] = sig_wire
            .try_into()
            .map_err(|_| AuthorityVerifyError::Malformed)?;
        let sig = HybridSignature::from_parts_internal(
            SigCodepoint::CLASSICAL_ED25519,
            sig_bytes.to_vec(),
            Vec::new(),
        );
        SignatureSuite::from_config(SuiteConfig::classical_only())
            .verify(signing_pk.clone(), msg, &sig)
            .map_err(|_| AuthorityVerifyError::VerifyFailed)
    }
}

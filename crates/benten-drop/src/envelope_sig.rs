//! Envelope-level Ed25519 signature surface for `DropBundle`.
//!
//! Per Spike G's defense-in-depth contract, the bundle envelope is
//! signed BEFORE serialization with the issuer's Ed25519 keypair. The
//! signature covers the bundle header (everything except the
//! `envelope_sig` field itself + the `issuer_verifying_key` carrier);
//! per-Node integrity is provided by the AEAD authentication tags
//! inside each `EncryptedContent`.
//!
//! The signing primitive routes through `benten_id::keypair::Keypair`
//! (which in turn routes through `benten_crypto_suite::primitives::
//! ed25519_dalek` — the ONLY crypto-primitive call site per
//! `crypto-agility-contract:6` / CLAUDE.md baked-in #5).
//!
//! # Domain separation
//!
//! The signed message is prefixed with a per-surface domain-separation
//! tag so a signature over a Drop bundle envelope cannot be
//! reinterpreted as a signature over any other workspace surface. The
//! tag value (held as the module-private `ENVELOPE_SIG_DOMAIN`
//! constant) is `b"benten/g-core-3f/drop-bundle-envelope/v1"`.

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use benten_crypto_suite::primitives::ed25519_dalek::{Signature, Verifier, VerifyingKey};
use benten_id::keypair::Keypair;
use thiserror::Error;

/// Domain-separation tag for the envelope-sig message construction
/// per CLAUDE.md baked-in #5 (per-surface domain tag to prevent
/// cross-surface signature reuse).
const ENVELOPE_SIG_DOMAIN: &[u8] = b"benten/g-core-3f/drop-bundle-envelope/v1";

/// Typed envelope-sig failure.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EnvelopeSigError {
    /// The verifying-key bytes do not parse as an Ed25519 32-byte
    /// public key (covers truncation, wrong length, invalid Edwards
    /// point).
    #[error("envelope-sig verifying-key malformed: {0}")]
    VerifyingKeyMalformed(String),

    /// The signature bytes do not parse as an Ed25519 64-byte
    /// signature (covers truncation, wrong length).
    #[error("envelope-sig signature bytes malformed: {0}")]
    SignatureMalformed(String),

    /// The Ed25519 cryptographic verification failed — header was
    /// tampered post-sign, OR the wrong verifying key is paired
    /// with the signature.
    #[error("envelope-sig Ed25519 verify failed against re-built envelope message: {0}")]
    VerifyFailed(String),
}

/// Build the canonical envelope-sig message — the byte string the
/// issuer signs and the validator re-builds at verify time.
///
/// **Defense-in-depth layering (Spike G):** the envelope-sig binds
/// the bundle **HEADER ONLY** (version + mode + spec_cid + audience +
/// auth_grant + restricted_spec + per_node_attestation) — **NOT the
/// raw content bytes**. A byte-flip in any `content[i]` ciphertext
/// does NOT change the signed payload; the envelope-sig still
/// verifies. That tamper is caught at the **inner** AEAD-tag layer
/// inside each `EncryptedNode` (the per-Node integrity layer that
/// `decrypt` consults).
///
/// Per `tf3f_per_node_ciphertext_tamper_detected_envelope_sig_still_valid`,
/// the envelope-sig MUST still verify after a content-only tamper —
/// that IS the defense-in-depth property (two independent integrity
/// layers; tamper at either layer is caught at the appropriate
/// boundary).
///
/// Implementation: clones the bundle, zeroes the `envelope_sig`,
/// `issuer_verifying_key`, AND `content` fields, then encodes the
/// remaining header to canonical DAG-CBOR + prepends the domain-
/// separation tag.
#[must_use]
pub fn build_envelope_message(bundle: &crate::bundle::DropBundle) -> Vec<u8> {
    // Clone the bundle + zero the fields excluded from the signed
    // payload:
    //   - envelope_sig + issuer_verifying_key: communicated
    //     alongside the sig outside the signed payload
    //   - content: the AEAD-tag layer provides per-Node integrity;
    //     the envelope-sig is intentionally INDEPENDENT of content
    //     bytes per the defense-in-depth pin
    let mut for_sig = bundle.clone();
    for_sig.envelope_sig = Vec::new();
    for_sig.issuer_verifying_key = Vec::new();
    for_sig.content = Vec::new();

    // Encode the zeroed-sig + content-stripped bundle (= the header).
    let header_bytes = serde_ipld_dagcbor::to_vec(&for_sig)
        .expect("DropBundle re-encodes for envelope-sig message construction");

    let mut msg = Vec::with_capacity(ENVELOPE_SIG_DOMAIN.len() + header_bytes.len());
    msg.extend_from_slice(ENVELOPE_SIG_DOMAIN);
    msg.extend_from_slice(&header_bytes);
    msg
}

/// Sign the envelope-message bytes with the issuer's `Keypair`. Returns
/// the 64-byte Ed25519 signature as a `Vec<u8>`.
#[must_use]
pub fn sign_envelope(issuer_kp: &Keypair, msg: &[u8]) -> Vec<u8> {
    let sig = issuer_kp.sign(msg);
    sig.to_bytes().to_vec()
}

/// Verify an envelope signature against the supplied verifying-key
/// bytes + message bytes.
///
/// # Errors
///
/// - [`EnvelopeSigError::VerifyingKeyMalformed`] if `vk_bytes` is not
///   a valid 32-byte Ed25519 public key.
/// - [`EnvelopeSigError::SignatureMalformed`] if `sig_bytes` is not a
///   64-byte Ed25519 signature.
/// - [`EnvelopeSigError::VerifyFailed`] if the cryptographic verify
///   step fails.
pub fn verify_envelope(
    vk_bytes: &[u8],
    sig_bytes: &[u8],
    msg: &[u8],
) -> Result<(), EnvelopeSigError> {
    let vk_arr: [u8; 32] = vk_bytes.try_into().map_err(|_| {
        EnvelopeSigError::VerifyingKeyMalformed(format!(
            "expected 32-byte Ed25519 key, got {} bytes",
            vk_bytes.len()
        ))
    })?;
    let vk = VerifyingKey::from_bytes(&vk_arr).map_err(|e| {
        EnvelopeSigError::VerifyingKeyMalformed(format!("not a valid Edwards point: {e}"))
    })?;

    let sig_arr: [u8; 64] = sig_bytes.try_into().map_err(|_| {
        EnvelopeSigError::SignatureMalformed(format!(
            "expected 64-byte Ed25519 sig, got {} bytes",
            sig_bytes.len()
        ))
    })?;
    let sig = Signature::from_bytes(&sig_arr);

    // Classical Ed25519 by design: envelope_sig is INTEGRITY-only (the
    // outer defense-in-depth layer). Authority comes from the issuer-
    // anchored `auth_grant` (verified in `consume_offline` Layer 2 +
    // anchored to `auth_grant.issuer_verifying_key` in Layer 2b / F-INJ-2),
    // NOT from envelope_sig — on its own this is "a signature-by-nobody".
    // So there is no authority PQ-strip to close here; it correctly stays
    // classical and does NOT route through the Fork-A hybrid chokepoint.
    vk.verify(msg, &sig)
        .map_err(|e| EnvelopeSigError::VerifyFailed(format!("{e}")))
}

#[cfg(test)]
mod domain_registry_mirror {
    /// C-01/C-02 drift defense: this module-private `ENVELOPE_SIG_DOMAIN` is
    /// mirrored in the central
    /// [`benten_crypto_suite::domain_registry`] corpus table (the prefix-free
    /// collision check runs over the mirror). If the two ever diverge the
    /// collision check would silently run over the wrong bytes — pin equality.
    #[test]
    fn envelope_sig_domain_matches_central_registry() {
        assert_eq!(
            super::ENVELOPE_SIG_DOMAIN,
            benten_crypto_suite::domain_registry::ENVELOPE_SIG_DOMAIN,
            "ENVELOPE_SIG_DOMAIN drifted from the central domain_registry mirror"
        );
    }
}

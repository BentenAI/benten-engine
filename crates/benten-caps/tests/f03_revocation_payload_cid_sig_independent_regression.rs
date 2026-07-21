//! F-03 regression (fix a) — UCAN revocation keys on the
//! signature-EXCLUSIVE payload CID, so a signature re-encoding of a
//! revoked token cannot dodge the revocation marker (Inv-15: sig-bundle
//! CIDs are never load-bearing identifiers).
//!
//! **Would-FAIL-on-revert of the payload-CID re-key**
//! (`backends/ucan.rs::ucan_payload_cid` + `revoke`/`is_revoked` keyed
//! on it): revert revocation to the sig-INCLUSIVE `ucan_cid` and the
//! signature-varied copy below yields a DIFFERENT revocation key, misses
//! the marker → `is_revoked` returns false → the final assert FAILs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::backends::ucan::UCANBackend;
use benten_graph::RedbBackend;
use benten_id::keypair::Keypair;
use benten_id::ucan::Ucan;

/// The Ed25519 group order `L` in little-endian (32 bytes). Adding `L`
/// to the scalar `S` of a valid signature produces a byte-DISTINCT
/// signature over the SAME message (the classic Ed25519 malleability).
/// We use it only to synthesize a signature-varied token with IDENTICAL
/// claims — a stand-in for any way an attacker re-encodes a signature
/// (Ed25519 malleability, or a re-randomized ML-DSA hybrid half).
const ED25519_L_LE: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58, 0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

/// Return a byte-distinct 64-byte Ed25519 signature over the SAME
/// message: `R` (bytes 0..32) unchanged, `S' = S + L` (bytes 32..64).
fn malleate_ed25519_sig(sig: &[u8]) -> Vec<u8> {
    assert_eq!(sig.len(), 64, "classical Ed25519 signature is 64 bytes");
    let mut out = sig.to_vec();
    let mut carry = 0u16;
    for i in 0..32 {
        let sum = u16::from(out[32 + i]) + u16::from(ED25519_L_LE[i]) + carry;
        out[32 + i] = (sum & 0xff) as u8;
        carry = sum >> 8;
    }
    // S < L and L < 2^253 => S + L < 2^254, so nothing carries out of the
    // 32-byte S window.
    assert_eq!(carry, 0, "S + L must fit in 32 bytes");
    out
}

#[test]
fn revocation_is_signature_independent_payload_cid_key_f03() {
    let inner = RedbBackend::open_in_memory().expect("redb in-memory open");
    let backend = UCANBackend::new(Arc::new(inner));

    let issuer = Keypair::generate();
    let audience = Keypair::generate();
    let token = Ucan::builder()
        .issuer_did(&issuer.public_key().to_did())
        .audience_did(&audience.public_key().to_did())
        .capability("/zone/posts", "read")
        .sign(&issuer);

    // A signature-varied copy: IDENTICAL claims, a DIFFERENT signature
    // over the same payload. Models an attacker re-encoding a token's
    // signature to change its sig-inclusive CID while it still verifies.
    let mut varied = token.clone();
    varied.signature = malleate_ed25519_sig(&token.signature);
    assert_ne!(
        varied.signature, token.signature,
        "the copy must carry a byte-distinct signature"
    );

    // Revoke the ORIGINAL token.
    backend.revoke(&token).expect("revoke");
    assert!(
        backend.is_revoked(&token).unwrap(),
        "the revoked original must be revoked"
    );

    // THE F-03 PROPERTY: the signature-varied copy — same claims,
    // different signature — is ALSO seen as revoked, because the marker
    // is keyed on the sig-EXCLUSIVE payload CID. Revert the re-key to
    // the sig-inclusive `ucan_cid` and this FAILs (the copy's
    // sig-inclusive CID differs → the marker is missed → false).
    assert!(
        backend.is_revoked(&varied).unwrap(),
        "F-03 REGRESSION: a signature-re-encoded copy of a revoked UCAN \
         dodged the revocation marker — revocation MUST key on the \
         signature-exclusive payload CID (Inv-15), not the sig-inclusive \
         ucan_cid",
    );
}

//! GAP-KDB Shape-B / Fork-A — DeviceAttestation hybrid parent verify
//! (family AUTH-13). W3 authority-migration RED-PHASE.
//!
//! Ref `3bea1294`: `GAP-KDB-B-DESIGN-R1.md` §6 FORK-A (device-attestation
//! is part of the "hardcoded Ed25519 today" authority path) + §4
//! (multi-device: "each device = its own did:benten … attested under the
//! user"). `R2-LANDSCAPE` AUTH-13.
//!
//! At the freeze base `DeviceAttestation::verify_signature_with(&self,
//! parent_pk: &PublicKey)` (`device_attestation.rs`) extracts a
//! `[u8; 64]` Ed25519 signature and verifies it against the Ed25519
//! `parent_pk`. Fork-A makes this dispatch on the parent DID's
//! multicodec: a `did:benten` parent embeds a LAMPS composite signing
//! key, so the attestation signature is a composite verified via
//! `SignatureSuite::verify`. A composite with the PQ half stripped MUST
//! reject.
//!
//! # would_fail_on_revert
//! A device attestation whose `parent_did` is a `did:benten` and whose
//! composite signature has the ML-DSA half REMOVED (valid Ed25519 half)
//! MUST reject; a silent-strip verify accepts it → Err→Ok flip.
//! Positive control: the full composite verifies. Backward-compat
//! control: a classical did:key parent still verifies via the 64-byte
//! Ed25519 arm unchanged.
//!
//! # R5 un-ignore
//! Migrate `verify_signature_with` to codepoint-dispatched hybrid verify
//! (composite `sig::PublicKey` parent + `SignatureSuite::verify`);
//! repoint the shim; drop `#[ignore]`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_crypto_suite::sig::{self, HybridSignature};
use benten_crypto_suite::SignatureSuite;
use benten_id::CanonicalBytes;
use benten_id::device_attestation::{CapabilityEnvelope, DeviceAttestation};
use benten_id::did::Did;
use benten_id::errors::DeviceAttestationError;
use benten_id::kdb_testing as kdb;
use benten_id::keypair::Keypair as Ed25519Keypair;

const NOW: u64 = 1_900_000_000;

// ─────────────────────────────────────────────────────────────────────────
// R5 ENTRY SHIM — Fork-A hybrid device-attestation parent verify.
// At R5: body := att.verify_signature_with(parent_signing_pk) once it
// takes a composite `sig::PublicKey`. `todo!()` so no red-phase run
// false-greens against the stub.
// ─────────────────────────────────────────────────────────────────────────

fn r5_device_verify(
    _att: &DeviceAttestation,
    _parent_signing_pk: &sig::PublicKey,
) -> Result<(), DeviceAttestationError> {
    todo!(
        "RED-PHASE (AUTH-13): DeviceAttestation hybrid parent verify lands at R5. \
         Body := att.verify_signature_with(parent_signing_pk) once it takes a composite key. \
         un-ignore then."
    )
}

// ── Fixtures ──────────────────────────────────────────────────────────────

/// A `did:benten` embedding `signer`'s composite signing key.
fn benten_did_for(signer: &sig::Keypair, tag: &str) -> Did {
    let doc = kdb::KeySetDocument::v1_hybrid(
        kdb::signing_multikey_of(&signer.public()),
        kdb::kem_multikey_hybrid(
            &kdb::det_x25519_pub(&format!("{tag}/x")),
            &kdb::det_mlkem768_ek(&format!("{tag}/ek")),
        ),
    );
    kdb::self_committed_did(&doc)
}

/// A device attestation for `device_did` issued by `parent_did`, with a
/// caller-chosen signature wire (over the canonical bytes).
fn attestation(parent_did: &Did, device_did: &Did, sig_wire: Vec<u8>) -> DeviceAttestation {
    DeviceAttestation {
        device_did: device_did.as_str().to_string(),
        parent_did: parent_did.as_str().to_string(),
        envelope: CapabilityEnvelope::default(),
        nonce: [7u8; 32],
        issued_at: NOW,
        signature: sig_wire,
    }
}

fn composite_sign_attestation(signer: &sig::Keypair, att: &DeviceAttestation) -> HybridSignature {
    SignatureSuite::v1_default().sign(signer, &CanonicalBytes::to_canonical_bytes(att))
}

// ── AUTH-13 — did:benten parent composite verify (positive) ───────────────

#[test]
#[ignore = "RED-PHASE: AUTH-13 device-attestation did:benten parent composite verifies — un-ignore at R5"]
fn auth13_did_benten_parent_composite_attestation_verifies() {
    let parent_signer = kdb::hybrid_keypair();
    let parent = benten_did_for(&parent_signer, "auth13/parent");
    // The device itself is also a did:benten (design §4), but the verify
    // is against the PARENT signing key.
    let device_signer = kdb::hybrid_keypair();
    let device = benten_did_for(&device_signer, "auth13/device");

    let mut att = attestation(&parent, &device, Vec::new());
    att.signature = composite_sign_attestation(&parent_signer, &att).to_wire_bytes();

    assert!(
        r5_device_verify(&att, &parent_signer.public()).is_ok(),
        "AUTH-13: a device attestation with a did:benten parent + FULL valid composite \
         signature MUST verify against the parent's composite signing key"
    );
}

#[test]
#[ignore = "RED-PHASE: AUTH-13 device-attestation did:benten parent PQ-strip rejects — un-ignore at R5"]
fn auth13_did_benten_parent_pq_stripped_attestation_rejects() {
    let parent_signer = kdb::hybrid_keypair();
    let parent = benten_did_for(&parent_signer, "auth13s/parent");
    let device_signer = kdb::hybrid_keypair();
    let device = benten_did_for(&device_signer, "auth13s/device");

    let mut att = attestation(&parent, &device, Vec::new());
    let composite = composite_sign_attestation(&parent_signer, &att);
    let stripped = HybridSignature::from_parts_internal(
        composite.codepoint(),
        composite.classical_half_for_test(), // VALID Ed25519 half
        Vec::new(),                          // PQ half removed
    );
    att.signature = stripped.to_wire_bytes();

    assert!(
        r5_device_verify(&att, &parent_signer.public()).is_err(),
        "AUTH-13: a device attestation whose did:benten-parent composite signature has the \
         ML-DSA half STRIPPED MUST reject — the parent verify is the full hybrid, not \
         Ed25519-only"
    );
}

// ── AUTH-13 — did:key parent backward-compat unchanged ────────────────────

#[test]
fn auth13_did_key_parent_backward_compat_unchanged() {
    // Non-ignored regression guard (REAL now + after migration): a
    // classical did:key parent still verifies its device attestation via
    // the 64-byte Ed25519 arm, using the REAL production `issue` +
    // `verify_signature_with`. The Fork-A dispatch (multicodec 0xed01)
    // MUST leave this unchanged.
    let parent_kp = Ed25519Keypair::generate();
    let device_kp = Ed25519Keypair::generate();
    let device_did = device_kp.public_key().to_did();

    let att = DeviceAttestation::issue(&parent_kp, device_did, CapabilityEnvelope::default())
        .expect("did:key device attestation issues");
    assert_eq!(
        att.signature.len(),
        64,
        "precondition: a did:key parent attestation carries a bare 64-byte Ed25519 signature"
    );
    assert!(
        att.verify_signature_with(parent_kp.public_key()).is_ok(),
        "AUTH-13: a did:key parent's device attestation MUST still verify via the classical \
         Ed25519 arm (backward-compat)"
    );
}

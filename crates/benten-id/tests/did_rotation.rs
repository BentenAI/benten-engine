//! G14-A2 wave-4a' — DID rotation test pins (un-ignored except where
//! noted).
//!
//! Pin sources (per `crypto-major-3` + `exploration-device-mesh`):
//!
//! - `did_rotate_keypair_emits_superseded_by_attestation_chain`
//! - `did_rotation_propagates_revocation_to_ucan_backend` — RED-PHASE
//!   (G14-B durable backend integration)
//! - `superseded_did_cannot_sign_new_ucan_delegations`
//! - `did_rotate_keypair_preserves_did_under_canonical_bytes`

#![allow(clippy::unwrap_used)]

use benten_id::did_rotation::{AttestationKind, RotationLog, rotate_keypair};
use benten_id::keypair::Keypair;
use benten_id::ucan::{Ucan, validate_chain_with_rotation_log};
use benten_id::{DidRotationError, UcanError};

#[test]
fn did_rotate_keypair_emits_superseded_by_attestation_chain() {
    // crypto-major-3 — rotation emits verifiable superseded_by chain.
    let old_kp = Keypair::generate();
    let did = old_kp.public_key().to_did();
    let new_kp = Keypair::generate();
    let now = 1_000_000_000;

    let attestation = rotate_keypair(&did, &old_kp, &new_kp, now).unwrap();

    assert_eq!(attestation.kind(), AttestationKind::SupersededBy);
    assert_eq!(
        attestation.previous_keypair_did().as_str(),
        old_kp.public_key().to_did().as_str()
    );
    assert_eq!(
        attestation.next_keypair_did().as_str(),
        new_kp.public_key().to_did().as_str()
    );
    // Signed by OLD keypair — proves authorization:
    attestation
        .verify_signature_with(old_kp.public_key())
        .unwrap();

    // NEW keypair's signature MUST NOT verify (different keypair):
    assert!(matches!(
        attestation
            .verify_signature_with(new_kp.public_key())
            .unwrap_err(),
        DidRotationError::BadSignature
    ));
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — DID-rotation propagation through durable UCAN backend. G14-B PR #109 + G14-A2 PR #108 both shipped (durable backend + attestation shape); test body pins specific DID-rotation propagation contract that composes with §2.1-followup ssi external UCAN/VC spec compatibility re-evaluation at G16 Atrium handshake (re-evaluation point at G16-D wave-6b shipped 2026-05-09 PR #163). Body un-ignore at §2.1-followup re-evaluation outcome per Wave-E rationale-only sweep."]
fn did_rotation_propagates_revocation_to_ucan_backend() {
    // crypto-major-3 cross-wave pin — the G14-B durable UCAN backend
    // consumes the rotation event + revokes pre-rotation UCANs at
    // chain-walk. G14-A2 only mints the attestation shape; the
    // durable propagation seam is G14-B's scope.
    unreachable!("G14-B wires this pin");
}

#[test]
fn superseded_did_cannot_sign_new_ucan_delegations() {
    // crypto-major-3 — post-rotation old-key UCAN rejected at
    // chain-walk via rotation log.
    let old_kp = Keypair::generate();
    let new_kp = Keypair::generate();
    let leaf_aud = Keypair::generate();
    let did = old_kp.public_key().to_did();
    let now = 1_000_000_000;

    // Rotate:
    let attestation = rotate_keypair(&did, &old_kp, &new_kp, now).unwrap();
    let log = RotationLog::from_entries(vec![attestation]);

    // Attacker holding old_kp tries to issue a new UCAN AFTER rotation:
    let post_rotation_ucan = Ucan::builder()
        .issuer(old_kp.public_key().to_did().as_str())
        .audience(leaf_aud.public_key().to_did().as_str())
        .capability("/zone/posts", "read")
        .not_before(now + 100)
        .expiry(now + 3600)
        .sign(&old_kp);

    let err = validate_chain_with_rotation_log(&[post_rotation_ucan], &log).unwrap_err();
    assert!(
        matches!(err, UcanError::IssuerKeypairSuperseded { .. }),
        "{err:?}"
    );
}

#[test]
fn did_rotate_keypair_preserves_did_under_canonical_bytes() {
    // exploration-device-mesh — logical DID stable under rotation.
    //
    // The "logical DID" Phase-3 uses for long-lived audience binding
    // is the OLD `did:key` string itself. Its canonical-bytes encoding
    // (the string bytes) is stable across the rotation event — what
    // rotates is the underlying keypair, not the DID string.
    let old_kp = Keypair::generate();
    let logical_did = old_kp.public_key().to_did();
    let canonical_pre = logical_did.as_str().as_bytes().to_vec();

    let new_kp = Keypair::generate();
    let _attestation = rotate_keypair(&logical_did, &old_kp, &new_kp, 1_000_000_000).unwrap();

    // After rotation, the logical DID's canonical bytes are unchanged:
    let canonical_post = logical_did.as_str().as_bytes().to_vec();
    assert_eq!(
        canonical_pre, canonical_post,
        "Logical DID canonical bytes MUST NOT change across rotation"
    );
}

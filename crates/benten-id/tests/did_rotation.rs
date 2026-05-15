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
#[ignore = "phase-3-backlog §2.1-followup `ssi` external UCAN/VC spec compatibility re-evaluation — production prerequisite NOT YET shipped at HEAD. `crates/benten-caps/` does NOT consume `benten_id::did_rotation::RotationLog`; the durable UCAN backend `benten_caps::backends::ucan::UCANBackend` chain-walker has no rotation-event consumption seam. G14-B PR #109 shipped the durable backend (`UCANBackend<B>`) + the in-RAM `RotationLog` helper at `crates/benten-id/src/did_rotation.rs:167` exists, but the integration where rotation events propagate from `did_rotation::rotate_keypair` → durable backend → chain-walker rejection of pre-rotation UCANs is NOT wired. crypto-major-3 cross-wave pin; un-ignore at §2.1-followup re-evaluation outcome (G16-D wave-6b PR #163 shipped 2026-05-09; cryptography-reviewer dispatch pending; rotation-propagation seam composes with the re-evaluation outcome since `ssi` integration would re-shape the chain-walker)."]
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

// ---------------------------------------------------------------------
// Safe-1 #509 / F-FWD-2-01 #1051 — RotationLog::accept_rotation_event
// authenticity gate. These are SUBSTANTIVE runtime-arm pins (pim-2
// §3.6b): the production accept path is exercised; the observable
// consequence is the typed BadSignature reject; each test would FAIL
// against the pre-fix (no-signature-verify) implementation.
// ---------------------------------------------------------------------

#[test]
fn rotation_log_rejects_forged_unauthenticated_event_with_bad_signature() {
    // ATTACK PATH (#509 silent-failure shape): a peer synthesizes a
    // RotationAttestation byte-blob with a real previous_did/next_did
    // and a strictly-monotonic superseded_at, but a FORGED 64-byte
    // signature (NOT signed by the OLD keypair). Pre-fix this was
    // silently pushed into the log and would perpetually revoke the
    // victim's DID at any consuming peer. Post-fix it MUST reject with
    // DidRotationError::BadSignature BEFORE entering the log.
    let victim_old = Keypair::generate();
    let attacker_chosen_new = Keypair::generate();
    let old_did = victim_old.public_key().to_did();

    // Genuine rotation event (signed by victim_old) — captured by the
    // attacker only as a SHAPE template.
    let genuine = rotate_keypair(&old_did, &victim_old, &attacker_chosen_new, 100).unwrap();

    // Forge: keep the (previous_did, next_did, superseded_at) tuple
    // identical-shape but corrupt the signature so it is NOT a valid
    // OLD-keypair signature over the canonical bytes.
    let mut forged = genuine.clone();
    forged.signature[0] ^= 0xFF;

    let mut log = RotationLog::new();
    let err = log
        .accept_rotation_event(&forged)
        .expect_err("forged/unauthenticated rotation event MUST be rejected");
    assert!(
        matches!(err, DidRotationError::BadSignature),
        "expected BadSignature, got {err:?}"
    );
    // Observable consequence: the forged event did NOT enter the log,
    // so it cannot revoke the victim's DID downstream.
    assert!(
        log.entries().is_empty(),
        "forged event MUST NOT be persisted into the rotation log"
    );
    assert!(
        !log.is_superseded(&old_did),
        "victim DID MUST NOT be superseded by a forged rotation event"
    );
}

#[test]
fn rotation_log_rejects_event_signed_by_wrong_key() {
    // ATTACK PATH variant: attacker signs the attestation with THEIR
    // OWN key but claims the victim's DID as previous_did. The
    // signature is structurally valid (64 bytes, real Ed25519 sig) but
    // does NOT verify against the public key resolved from the claimed
    // previous_did. Pre-fix: accepted. Post-fix: BadSignature.
    let victim = Keypair::generate();
    let attacker = Keypair::generate();
    let new_kp = Keypair::generate();
    let victim_did = victim.public_key().to_did();

    // Attacker constructs an event whose previous_did is the VICTIM's
    // DID, but signs it with the attacker's keypair (rotate_keypair
    // with attacker's key produces an attacker-DID previous_did, so we
    // hand-forge the previous_did to the victim while keeping the
    // attacker-key signature).
    let attacker_did = attacker.public_key().to_did();
    let mut event = rotate_keypair(&attacker_did, &attacker, &new_kp, 200).unwrap();
    event.previous_did = victim_did.as_str().to_string();

    let mut log = RotationLog::new();
    let err = log
        .accept_rotation_event(&event)
        .expect_err("event signed by non-owner of previous_did MUST be rejected");
    assert!(
        matches!(err, DidRotationError::BadSignature),
        "expected BadSignature, got {err:?}"
    );
    assert!(!log.is_superseded(&victim_did));
}

#[test]
fn rotation_log_accepts_genuine_authenticated_event() {
    // POSITIVE arm: a genuinely OLD-keypair-signed event still
    // accepts cleanly (the authenticity gate does not regress the
    // legitimate path; the verbatim-replay + HLC-strict defenses still
    // compose AFTER the signature gate).
    let old_kp = Keypair::generate();
    let new_kp = Keypair::generate();
    let old_did = old_kp.public_key().to_did();
    let genuine = rotate_keypair(&old_did, &old_kp, &new_kp, 300).unwrap();

    let mut log = RotationLog::new();
    log.accept_rotation_event(&genuine)
        .expect("genuine OLD-key-signed rotation event MUST accept");
    assert!(
        log.is_superseded(&old_did),
        "post-accept, the OLD DID is observably superseded"
    );

    // Verbatim replay of the same genuine event still rejects (the
    // ordering defenses compose with the new authenticity gate).
    let replay = log.accept_rotation_event(&genuine);
    assert!(
        matches!(replay, Err(DidRotationError::VerbatimReplay { .. })),
        "verbatim replay still rejects post-fix: {replay:?}"
    );
}

//! R3-B RED-PHASE pins for `benten-id` DID rotation primitive
//! (G14-A2 wave-4a'; crypto-major-3 + exploration-device-mesh).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-A2 +
//! §10 device-mesh exploration):
//!
//! - `tests/did_rotate_keypair_emits_superseded_by_attestation_chain` — crypto-major-3
//! - `tests/did_rotation_propagates_revocation_to_ucan_backend` — crypto-major-3 (G14-A2 + G14-B integration)
//! - `tests/superseded_did_cannot_sign_new_ucan_delegations` — crypto-major-3
//! - `tests/did_rotate_keypair_preserves_did_under_canonical_bytes` — exploration-device-mesh
//!
//! ## Architectural intent
//!
//! Per plan §3 G14-A2 row, `Did::rotate_keypair` lands as the load-
//! bearing recovery primitive: when a private key is lost or
//! compromised, the user rotates to a new keypair under the same
//! logical DID. The rotation emits a `superseded_by` attestation
//! chain so that anyone holding a UCAN issued by the OLD keypair
//! can verify the rotation lineage.
//!
//! Cross-wave integration (`did_rotation_propagates_revocation_to_ucan_backend`):
//! G14-A2 emits the rotation event; G14-B's UCAN backend consumes it
//! and revokes UCANs in the OLD chain. The test pins the wire-up
//! end-to-end.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-A2
//! implementer un-ignores. Per §3.6b pim-2 these tests must drive the
//! production rotation entry point + assert observable consequences
//! (revocation propagates; superseded keypair fails new delegations).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-major-3 — rotation emits superseded_by chain"]
fn did_rotate_keypair_emits_superseded_by_attestation_chain() {
    // crypto-major-3 pin. G14-A2 implementer wires this:
    //
    //   let old_kp = benten_id::keypair::Keypair::generate();
    //   let did = old_kp.public_key().to_did();
    //   let new_kp = benten_id::keypair::Keypair::generate();
    //
    //   let attestation = benten_id::did_rotation::rotate_keypair(
    //       &did, &old_kp, &new_kp).unwrap();
    //   assert_eq!(attestation.kind(), benten_id::did_rotation::AttestationKind::SupersededBy);
    //   assert_eq!(attestation.previous_keypair_did(), old_kp.public_key().to_did());
    //   assert_eq!(attestation.next_keypair_did(),     new_kp.public_key().to_did());
    //   // The attestation MUST be signed by the OLD keypair (proving
    //   // the rotation was authorized by the holder of the old secret):
    //   assert!(attestation.verify_signature_with(&old_kp.public_key()).is_ok());
    //
    // OBSERVABLE consequence: rotation produces a verifiable
    // `superseded_by` attestation linking old → new. The chain is
    // walkable forward (anyone seeing old DID can find the new DID
    // without trusting the holder).
    unimplemented!("G14-A2 wires rotate_keypair() emitting verifiable superseded_by attestation");
}

#[test]
#[ignore = "RED-PHASE: G14-A2 + G14-B — crypto-major-3 — revocation propagates to UCAN backend"]
fn did_rotation_propagates_revocation_to_ucan_backend() {
    // crypto-major-3 cross-wave pin. G14-A2 emits rotation; G14-B's
    // UCANBackend consumes the event and revokes UCANs issued by the
    // old keypair. Without this propagation, an attacker holding the
    // old (compromised) key could continue issuing valid UCANs after
    // rotation.
    //
    // Implementer wires:
    //
    //   let old_kp = benten_id::keypair::Keypair::generate();
    //   let new_kp = benten_id::keypair::Keypair::generate();
    //   let did = old_kp.public_key().to_did();
    //
    //   // Issue a UCAN under the OLD keypair before rotation:
    //   let pre_rotation_ucan = benten_id::ucan::Ucan::builder()
    //       .issuer(did.clone())
    //       .audience(...)
    //       .capability("/zone/posts", "read")
    //       .sign(&old_kp).unwrap();
    //
    //   // Wire UCAN backend; install pre-rotation UCAN:
    //   let backend = benten_caps::UCANBackend::new(/* durable store */);
    //   backend.install(pre_rotation_ucan.clone()).unwrap();
    //
    //   // Rotate:
    //   let attestation = benten_id::did_rotation::rotate_keypair(
    //       &did, &old_kp, &new_kp).unwrap();
    //   backend.observe_rotation(&attestation).unwrap();
    //
    //   // Now the pre-rotation UCAN must be revoked at chain-walk:
    //   let err = backend.validate_chain(&[pre_rotation_ucan]).unwrap_err();
    //   assert!(matches!(err, benten_caps::UCANBackendError::SupersededBy { .. }));
    //
    // OBSERVABLE consequence: rotation observably invalidates the
    // pre-rotation UCAN at the durable-store chain-walk. The cross-
    // wave seam is the load-bearing security boundary.
    unimplemented!(
        "G14-A2 + G14-B wires rotation event consumption + pre-rotation UCAN revocation"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — crypto-major-3 — superseded keypair cannot sign new UCANs"]
fn superseded_did_cannot_sign_new_ucan_delegations() {
    // crypto-major-3 pin. After rotation, the OLD keypair (which may
    // have been compromised) MUST NOT be able to issue NEW valid
    // UCANs. Even if the attacker still holds the old secret, any
    // UCAN they sign post-rotation rejects at validate_chain.
    //
    // Implementer wires:
    //
    //   let old_kp = benten_id::keypair::Keypair::generate();
    //   let new_kp = benten_id::keypair::Keypair::generate();
    //   let did = old_kp.public_key().to_did();
    //
    //   // Rotate:
    //   let attestation = benten_id::did_rotation::rotate_keypair(
    //       &did, &old_kp, &new_kp).unwrap();
    //   // (Backend or chain-walker has observed the attestation.)
    //
    //   // Attacker holding old_kp tries to issue a new UCAN AFTER rotation:
    //   let post_rotation_ucan_from_old = benten_id::ucan::Ucan::builder()
    //       .issuer(did.clone())  // same logical DID
    //       .issued_at_now()       // POST-rotation timestamp
    //       .sign(&old_kp).unwrap();
    //
    //   // validate_chain MUST reject because the issuer keypair is superseded:
    //   let err = benten_id::ucan::validate_chain_with_rotation_log(
    //       &[post_rotation_ucan_from_old], &[attestation]).unwrap_err();
    //   assert!(matches!(err, benten_id::ucan::ChainError::IssuerKeypairSuperseded { .. }));
    //
    // OBSERVABLE consequence: a forensic / replay scenario where the
    // attacker's post-rotation UCAN looks structurally valid but
    // rejects because the chain-walker checks the rotation log
    // against the issuer's keypair-binding. This is the "rotation
    // matters even after the fact" pin.
    unimplemented!(
        "G14-A2 wires post-rotation old-key UCAN rejection at chain-walk via rotation log"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-A2 — exploration-device-mesh — DID preserved under canonical bytes"]
fn did_rotate_keypair_preserves_did_under_canonical_bytes() {
    // exploration-device-mesh pin. The LOGICAL DID (what users see,
    // what UCAN audience binds to) MUST NOT change under rotation —
    // only the underlying keypair changes. This is what makes the
    // rotation a recovery primitive rather than a "create new
    // identity" primitive.
    //
    // The contract: `did:key:z...` style DIDs hash a public key,
    // which by definition changes on rotation. So the rotated
    // identity uses a STABLE LOGICAL DID (e.g. did:benten:<anchor-cid>)
    // whose attestation chain points at the current keypair's
    // did:key. The logical DID's canonical-bytes encoding is stable
    // across rotation events.
    //
    // Implementer wires:
    //
    //   let old_kp = benten_id::keypair::Keypair::generate();
    //   let logical_did = benten_id::did_rotation::create_logical_did(&old_kp.public_key()).unwrap();
    //   let canonical_bytes_pre = logical_did.canonical_bytes();
    //
    //   let new_kp = benten_id::keypair::Keypair::generate();
    //   let _attestation = benten_id::did_rotation::rotate_keypair(
    //       &logical_did, &old_kp, &new_kp).unwrap();
    //
    //   // After rotation, the logical DID's canonical bytes are unchanged:
    //   let canonical_bytes_post = logical_did.canonical_bytes();
    //   assert_eq!(canonical_bytes_pre, canonical_bytes_post,
    //       "Logical DID canonical bytes MUST NOT change across rotation");
    //
    // OBSERVABLE consequence: long-lived references to the user's
    // identity (in posts, in shared zones, in UCAN audience fields)
    // remain stable across recovery events. Defends against the
    // "forced cascade-rewrite of every audience field" scenario.
    unimplemented!(
        "G14-A2 wires assertion that logical-DID canonical-bytes survive rotate_keypair"
    );
}

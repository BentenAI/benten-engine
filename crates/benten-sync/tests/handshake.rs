//! R3-C RED-PHASE pins for DID-based mutual-auth handshake (G16-D
//! wave-6b; per r2-test-landscape §2.4 G16-D + plan §3 G16-D row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-D rows
//!   `handshake_did_based_mutual_auth_round_trip` +
//!   `handshake_rejects_invalid_signature` +
//!   `handshake_ucan_grant_exchange_establishes_per_peer_cap_set`.
//! - plan §3 G16-D row.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-D wave-6b lands DID handshake"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-D wave-6b — plan §3 G16-D — DID-based mutual auth round-trip"]
fn handshake_did_based_mutual_auth_round_trip() {
    // plan §3 G16-D pin. G16-D implementer wires this:
    //
    //   use benten_sync::handshake::Handshake;
    //   use benten_id::keypair::Keypair;
    //   let kp_a = Keypair::generate();
    //   let kp_b = Keypair::generate();
    //
    //   // peer_a initiates handshake; peer_b responds:
    //   let frame_a_to_b = Handshake::initiate(&kp_a, kp_b.public_key().to_did()).unwrap();
    //   let frame_b_to_a = Handshake::respond(&kp_b, &frame_a_to_b).unwrap();
    //
    //   // peer_a verifies peer_b's response:
    //   let session = Handshake::finalise(&kp_a, &frame_b_to_a).unwrap();
    //   assert_eq!(session.local_did(), kp_a.public_key().to_did());
    //   assert_eq!(session.remote_did(), kp_b.public_key().to_did());
    //   assert!(session.is_authenticated());
    //
    // OBSERVABLE consequence: a clean handshake produces a session
    // object where both peers' DIDs are mutually authenticated.
    unimplemented!("G16-D wires DID-based mutual-auth handshake round-trip");
}

#[test]
#[ignore = "RED-PHASE: G16-D wave-6b — plan §3 G16-D — handshake rejects invalid signature"]
fn handshake_rejects_invalid_signature() {
    // plan §3 G16-D pin. A handshake frame with a tampered signature
    // MUST reject with a typed error.
    //
    //   let kp_a = Keypair::generate();
    //   let kp_b = Keypair::generate();
    //   let kp_c = Keypair::generate();  // attacker
    //
    //   let frame_a_to_b = Handshake::initiate(&kp_a, kp_b.public_key().to_did()).unwrap();
    //   // Attacker tampers — replays under kp_c's signature:
    //   let tampered = frame_a_to_b.replace_signature_with(kp_c.sign(&[]));
    //   match Handshake::respond(&kp_b, &tampered) {
    //       Err(HandshakeError::InvalidSignature { .. }) => {}
    //       other => panic!("expected InvalidSignature, got {other:?}"),
    //   }
    //
    // OBSERVABLE consequence: a tampered signature fails handshake
    // with a typed error; defends against handshake-replay attacks.
    unimplemented!("G16-D wires handshake invalid-signature rejection");
}

#[test]
#[ignore = "RED-PHASE: G16-D wave-6b — plan §3 G16-D — UCAN grant exchange at handshake"]
fn handshake_ucan_grant_exchange_establishes_per_peer_cap_set() {
    // plan §3 G16-D pin. After mutual-auth, peers exchange UCAN
    // grants that establish each peer's effective cap-set within
    // the Atrium.
    //
    //   let session = run_clean_handshake(&kp_a, &kp_b);
    //   let grant_a_to_b = session.local_grant_to_remote().unwrap();
    //   assert!(grant_a_to_b.includes_cap("/zone/posts", "read"));
    //   let grant_b_to_a = session.remote_grant_to_local().unwrap();
    //   // peer_a's effective cap-set within the Atrium is now
    //   // bounded by the intersection of:
    //   //   1. peer_a's local cap policy
    //   //   2. peer_b's grant to peer_a
    //   let effective = session.effective_cap_set();
    //   assert!(effective.is_authenticated());
    //   assert!(effective.intersection_validates_against_ucan_chain());
    //
    // OBSERVABLE consequence: post-handshake session carries the
    // per-peer cap-set derived from UCAN grant exchange; defends
    // against missing-grant-establishment attack class.
    unimplemented!("G16-D wires UCAN grant exchange + per-peer cap-set establishment at handshake");
}

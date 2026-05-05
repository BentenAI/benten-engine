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

#[test]
#[ignore = "RED-PHASE: G16-D wave-6b — ds-r4-3 — handshake rejects replay within bounded HLC window"]
fn handshake_rejects_replay_within_bounded_window() {
    // ds-r4-3 (R4 large-council Round 1 distributed-systems lens) pin.
    // R1 ds-15 (handshake DID-based mutual-auth replay-rejection
    // within bounded window) was triaged into 'distribute across G16
    // row briefs' but the distinct replay-rejection-within-bounded-
    // window content was not preserved. The current handshake tests
    // cover invalid-signature rejection but NOT bounded-window replay
    // protection for an otherwise-valid handshake frame replayed
    // within the acceptance window.
    //
    // Standard handshake property; cheap to add now (G14-pre-D HLC
    // supports the bounded-window mechanism); expensive to retrofit
    // if a replay-attack class is found post-Phase-3.
    //
    //   use benten_sync::handshake::{Handshake, HandshakeError};
    //   use benten_core::hlc::Hlc;
    //
    //   let kp_a = Keypair::generate();
    //   let kp_b = Keypair::generate();
    //
    //   // peer_a initiates a valid handshake at HLC T1:
    //   let frame_t1 = Handshake::initiate(&kp_a, kp_b.public_key().to_did()).unwrap();
    //   let session_t1 = Handshake::respond(&kp_b, &frame_t1).unwrap();
    //   assert!(session_t1.is_authenticated());
    //
    //   // Adversary replays the SAME frame_t1 to peer_b (e.g. captured
    //   // off-wire) within the bounded acceptance window:
    //   let replay_result = Handshake::respond(&kp_b, &frame_t1);
    //   match replay_result {
    //       Err(HandshakeError::ReplayWithinBoundedWindow {
    //           original_hlc,
    //           replay_hlc,
    //           window_ms,
    //       }) => {
    //           // The error carries observable diagnostic state:
    //           assert!(replay_hlc > original_hlc);
    //           assert!(window_ms > 0);
    //       }
    //       other => panic!("expected ReplayWithinBoundedWindow, got {other:?}"),
    //   }
    //
    //   // Stable error code:
    //   assert_eq!(
    //       replay_result.unwrap_err().code(),
    //       ErrorCode::E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW,
    //   );
    //
    //   // Outside the bounded window (post-window-expiry), the handshake
    //   // either still rejects (nonce-tracked) or accepts as a fresh
    //   // session — implementer chooses but the bounded-window
    //   // assertion is load-bearing for the in-window replay attack.
    //
    // OBSERVABLE consequence: a captured-off-wire handshake frame
    // replayed within the bounded HLC acceptance window fails with a
    // typed error variant carrying the original + replay HLC + window
    // size. Composes G14-pre-D HLC for bounded-window math + G16-D
    // handshake state machine. Defends against the handshake-replay
    // attack class that R1 ds-15 named.
    unimplemented!(
        "G16-D wires HandshakeError::ReplayWithinBoundedWindow + nonce/HLC-bounded acceptance window"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-D wave-6b — net-r4-r1-3 — handshake synchronizes revocation state BEFORE subscribing data"]
fn atrium_handshake_synchronizes_revocation_state_before_subscribing_data() {
    // net-r4-r1-3 (R4 large-council Round 1 networking lens) pin.
    // R1 net-blocker-3 specific_action named TWO recommended pins:
    // (a) `mst_proto_revocation_typed_message_applied_before_data_from_same_peer_batch`
    // (drain-priority — covered by mst_revocation_priority.rs) and
    // (b) `atrium_handshake_synchronizes_revocation_state_before_subscribing_data`
    // (handshake-state-synchronization — NOT covered until R4-FP/R3-C).
    //
    // The two are distinct: drain-priority handles in-flight
    // reordering once data is flowing; pre-subscription synchronization
    // handles the AT-REST state of the receiver's revocation cache
    // BEFORE any data arrives. Without the latter, a receiver that
    // handshakes + immediately subscribes can miss revocations queued
    // at the sender that haven't yet propagated through the regular
    // sync stream — a TOCTOU between handshake-completion and
    // revocation-set-snapshot.
    //
    //   use benten_sync::handshake::Handshake;
    //   use benten_sync::atrium::Atrium;
    //
    //   // peer_a has a backlog of 5-minute-old revocations queued in
    //   // its outbox (peer_b was offline; revocation events haven't
    //   // drained yet). peer_a's revocation set carries N entries:
    //   let mut peer_a = test_peer(peer_a_did);
    //   peer_a.atrium_revoke_for(peer_b_did, "/zone/posts/private/*").await.unwrap();
    //   // Revocation queued; not yet drained to peer_b.
    //
    //   // peer_b comes online + handshakes:
    //   let mut peer_b = test_peer(peer_b_did);
    //   let session = Handshake::run(&peer_a, &peer_b).await.unwrap();
    //
    //   // ASSERTION: handshake completion delivers a snapshot of all
    //   // revocations applicable to peer_b's peer-DID + device-DID
    //   // BEFORE the local Engine is permitted to open data subscriptions
    //   // on this Atrium session:
    //   assert!(session.revocation_set_synchronized());
    //   let synced_revs = session.synchronized_revocations_for_local_peer();
    //   assert!(synced_revs.iter().any(|r|
    //       r.target_peer_did() == peer_b_did
    //       && r.path().starts_with("/zone/posts/private")));
    //
    //   // Subscription opens are GATED on revocation-set-synchronization:
    //   assert!(peer_b.subscription_open_permitted_for_session(&session));
    //
    //   // Now subscribe to /zone/posts. Data from /zone/posts/private
    //   // is filtered out at delivery (per G14-D F6) because peer_b's
    //   // local revocation cache already has the entry from
    //   // handshake-time snapshot:
    //   let events: Vec<_> = peer_b.atrium_subscribe(&session, "/zone/posts").await.collect().await;
    //   for event in &events {
    //       assert!(!event.path().starts_with("/zone/posts/private"),
    //           "data from revoked sub-zone must be filtered \
    //            (revocation synced at handshake)");
    //   }
    //
    // OBSERVABLE consequence: handshake completion guarantees the
    // local revocation cache is at least as fresh as the sender's
    // revocation set at handshake-time, so post-handshake subscriptions
    // never observe data from sub-zones the receiver was already
    // revoked from. Defends against the TOCTOU window between
    // handshake-completion + revocation-set-snapshot that R1
    // net-blocker-3 + R4 net-r4-r1-3 named.
    unimplemented!(
        "G16-D wires handshake-phase revocation-set snapshot synchronization gate before subscription opens"
    );
}

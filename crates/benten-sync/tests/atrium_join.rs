//! R3-C RED-PHASE pins for atrium join + revoke (G16-D wave-6b; per
//! r2-test-landscape §2.4 G16-D + plan §3 G16-D row + plan §4 seed +
//! exit-criterion 15).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-D rows
//!   `atrium_join_flow_end_to_end` +
//!   `atrium_revoke_peer_terminates_active_subscriptions`.
//! - plan §3 G16-D row.
//! - plan §4 seed (atrium join flow seed planted in plan).
//! - exit-criterion 15 (atrium-revoke + active-subscription
//!   termination).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-D wave-6b lands atrium join + revoke"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-D wave-6b — plan §4 seed — atrium join flow end-to-end"]
fn atrium_join_flow_end_to_end() {
    // plan §4 seed pin. G16-D implementer wires this:
    //
    //   let inviter_engine = test_engine();
    //   let invitee_engine = test_engine();
    //
    //   // Inviter creates a fresh atrium + invitation:
    //   let atrium = inviter_engine.create_atrium("test-atrium").unwrap();
    //   let invite = atrium.create_invite_for_peer(invitee_engine.peer_did()).unwrap();
    //
    //   // Invitee accepts the invite:
    //   let invitee_atrium = invitee_engine.accept_atrium_invite(&invite).await.unwrap();
    //
    //   // Both engines now see the same atrium membership:
    //   let inviter_peers = atrium.list_peers();
    //   assert!(inviter_peers.contains(&invitee_engine.peer_did()));
    //   let invitee_peers = invitee_atrium.list_peers();
    //   assert!(invitee_peers.contains(&inviter_engine.peer_did()));
    //
    // OBSERVABLE consequence: the join flow produces mutual atrium
    // membership end-to-end; defends against half-joined state.
    unimplemented!("G16-D wires atrium join flow end-to-end");
}

#[test]
#[ignore = "RED-PHASE: G16-D + G14-D wave-6b — exit-criterion 15 — atrium revoke peer terminates active subscriptions"]
fn atrium_revoke_peer_terminates_active_subscriptions() {
    // exit-criterion 15 pin (composes with G14-D per-subscriber
    // filtering). When a peer is revoked from an atrium, its
    // active SUBSCRIBE subscriptions terminate.
    //
    //   let inviter_engine = test_engine();
    //   let invitee_engine = test_engine();
    //   let atrium = inviter_engine.create_atrium("test").unwrap();
    //   let invite = atrium.create_invite_for_peer(invitee_engine.peer_did()).unwrap();
    //   let invitee_atrium = invitee_engine.accept_atrium_invite(&invite).await.unwrap();
    //
    //   // Invitee subscribes to a zone:
    //   let mut sub = invitee_engine.subscribe_change_events("/zone/posts").await.unwrap();
    //
    //   // Inviter revokes the invitee:
    //   atrium.revoke_peer(invitee_engine.peer_did()).await.unwrap();
    //
    //   // The active subscription terminates with a typed event:
    //   match sub.next().await {
    //       Some(ChangeEvent::SubscriptionTerminated { reason }) => {
    //           assert_eq!(reason, TerminationReason::PeerRevoked);
    //       }
    //       other => panic!("expected SubscriptionTerminated, got {other:?}"),
    //   }
    //
    // OBSERVABLE consequence: atrium-revoke propagates as
    // subscription-termination event; defends against the failure
    // shape where revoked peers continue receiving events.
    unimplemented!("G16-D + G14-D wire atrium-revoke → subscription-termination end-to-end");
}

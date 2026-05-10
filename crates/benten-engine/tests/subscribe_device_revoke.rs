//! R3-B RED-PHASE pin: SUBSCRIBE subscription path terminates on
//! device DID revocation (G14-D wave-5a; crypto-major-6 +
//! exploration-device-mesh).
//!
//! Pin source: r2-test-landscape §2.2 G14-D row
//! `subscribe_subscription_path_terminated_when_device_did_revoked`;
//! crypto-major-6.
//!
//! ## Architectural intent
//!
//! Per crypto-major-6 + D-PHASE-3-25 multi-device contract, when a
//! device DID is revoked (parent DID emits revocation on loss event),
//! every active SUBSCRIBE subscription path bound to that device DID
//! MUST terminate. Without this, a stolen device that retains its
//! local engine state could continue receiving change events from
//! the user's atriums even after the user revokes it.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the test must drive the
//! production revocation flow + assert the subscription state
//! observably terminates.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — subscription path terminates when device DID revoked. G14-D wave-5a + G16-D wave-6b shipped F6 SUBSCRIBE filtering + on-the-wire device-DID-attestation envelope; test body pins device-DID-revoke termination defensive contract; un-ignore at §2.3 (i) WriteContext threading landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn subscribe_subscription_path_terminated_when_device_did_revoked() {
    // crypto-major-6 pin. G14-D implementer wires this:
    //
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //
    //   // Device subscribes:
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default()).unwrap();
    //
    //   let engine = benten_engine::Engine::open_for_device(...).unwrap();
    //   engine.consume_device_attestation(&attestation).unwrap();
    //   let sub_id = engine.subscribe_for_device(
    //       "/zone/posts",
    //       device_kp.public_key().to_did(),
    //       ...,
    //   ).unwrap();
    //
    //   // First event delivers normally:
    //   engine.write_node(&node1).unwrap();
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 1);
    //
    //   // Parent DID emits device revocation:
    //   let revocation = benten_id::device_attestation::DeviceRevocation::issue(
    //       &parent_kp,
    //       device_kp.public_key().to_did(),
    //       benten_id::device_attestation::RevocationReason::DeviceLoss,
    //   ).unwrap();
    //   engine.consume_device_revocation(&revocation).unwrap();
    //
    //   // Subsequent events do NOT deliver:
    //   engine.write_node(&node2).unwrap();
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 1,
    //       "post-revoke event MUST NOT deliver to revoked-device subscription");
    //
    //   // Subscription state surface reports cancelled-by-device-revoke:
    //   assert_eq!(engine.subscription_state(sub_id),
    //       benten_engine::SubscriptionState::CancelledByDeviceRevocation);
    //
    // OBSERVABLE consequence: subscription path observably terminates
    // post-revocation; subsequent events are not delivered; state
    // surface reports the revocation reason. Closes the "stolen
    // device continues to receive forever" attack class.
    unimplemented!(
        "G14-D wires SUBSCRIBE termination on device-revocation event consumption per crypto-major-6"
    );
}

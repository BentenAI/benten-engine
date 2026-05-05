//! R3-B RED-PHASE pin: in-flight event cap recheck race (G14-D
//! wave-5a; stream-r1-1).
//!
//! Pin source: r2-test-landscape §2.2 G14-D row
//! `cap_recheck_delivery_already_in_flight_event_minted_matched_cursor_passed_before_revoke_observable`;
//! stream-r1-1.
//!
//! ## Architectural intent
//!
//! Race condition: an event is MINTED and MATCHED to a subscriber +
//! cursor BEFORE a revoke lands at the cap store. The delivery
//! pipeline must STILL recheck the cap at delivery time (not assume
//! "matched ≡ allowed"). Per stream-r1-1 the recheck point is at
//! delivery, not at match — without this discipline a window opens
//! where revokes silently no-op against in-flight events.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the test must drive the
//! production delivery entry point and observe the race-condition
//! behavior end-to-end (revoke between match + deliver → no
//! delivery).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D — stream-r1-1 — in-flight event cap-recheck race"]
fn cap_recheck_delivery_already_in_flight_event_minted_matched_cursor_passed_before_revoke_observable()
 {
    // stream-r1-1 pin. G14-D implementer wires this with a controlled
    // pause between match + deliver to expose the race:
    //
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let subscriber_kp = ...;
    //   let grant = ... .audience(subscriber_kp.public_key().to_did())
    //                   .capability("/zone/posts", "read") ... ;
    //   engine.caps().install_proof(&grant).unwrap();
    //
    //   let sub_id = engine.subscribe("/zone/posts", subscriber_kp.public_key().to_did(), ...).unwrap();
    //
    //   // Pause delivery pipeline AFTER match BEFORE deliver:
    //   engine.test_pause_delivery_after_match();
    //
    //   // Mint an event:
    //   engine.write_node(&node_in_zone_posts).unwrap();
    //   // Verify it was matched + queued:
    //   assert_eq!(engine.test_in_flight_for_subscription(sub_id).len(), 1);
    //
    //   // Revoke the grant WHILE event is in-flight (matched but not delivered):
    //   engine.caps().revoke(&grant.cid()).unwrap();
    //
    //   // Resume delivery; the per-event recheck fires + skips:
    //   engine.test_resume_delivery();
    //
    //   // Delivery DID NOT fire because the recheck saw the revoke:
    //   assert_eq!(engine.delivered_events_for(sub_id).len(), 0,
    //       "in-flight event MUST recheck cap at delivery time per stream-r1-1");
    //
    // OBSERVABLE consequence: revocation lands AFTER match but
    // BEFORE deliver; the recheck fires at deliver and observably
    // suppresses delivery. Closes the race-condition window where
    // revokes could silently no-op against in-flight events.
    unimplemented!(
        "G14-D wires in-flight event cap-recheck race test exposing match-vs-deliver window"
    );
}

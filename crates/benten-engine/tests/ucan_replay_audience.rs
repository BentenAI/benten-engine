//! R3-B RED-PHASE pin: UCAN proof-chain cross-atrium replay rejection
//! (G14-D wave-5a; CLR-2).
//!
//! Pin source: r2-test-landscape §2.2 G14-D row
//! `ucan_proof_chain_replay_against_different_atrium_peer_rejected_via_audience_binding`;
//! CLR-2 cross-lens cluster.
//!
//! ## Architectural intent
//!
//! Sibling to G14-A1's `ucan_audience_binding_prevents_cross_atrium_replay`
//! (R3-A) but at the engine-side proof-chain consumption seam: a
//! UCAN proof chain assembled for atrium A is REPLAYED against
//! atrium B's engine MUST reject via audience binding. The G14-A1
//! pin tests `validate_chain_for_audience` directly; this G14-D pin
//! tests the engine flow that consumes the chain (WAIT-resume,
//! SUBSCRIBE registration, manifest verify).
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D
//! implementer un-ignores. Per §3.6b pim-2 the test must drive a
//! production engine entry point that consumes the chain and
//! observably rejects the replay.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — cross-atrium UCAN proof-chain replay rejection at engine. G14-D wave-5a shipped F6 SUBSCRIBE + cap_snapshot_hash; test body pins cross-atrium replay-rejection contract that composes with §2.3 (i) WriteContext audience threading (v1-assessment-window). Body un-ignore at §2.3 (i) landing per Wave-E rationale-only sweep."]
fn ucan_proof_chain_replay_against_different_atrium_peer_rejected_via_audience_binding() {
    // CLR-2 cluster pin. G14-D implementer wires this:
    //
    //   // Setup two atriums:
    //   let atrium_a_engine = benten_engine::Engine::open(store_a.path()).unwrap();
    //   let atrium_b_engine = benten_engine::Engine::open(store_b.path()).unwrap();
    //
    //   let issuer = benten_id::keypair::Keypair::generate();
    //   let actor = benten_id::keypair::Keypair::generate();
    //
    //   // UCAN bound to atrium A's audience:
    //   let atrium_a_did = atrium_a_engine.atrium_did();
    //   let ucan_for_a = ... .audience(atrium_a_did) ... ;
    //
    //   // Adversary captures the UCAN; replays it against atrium B:
    //   let invocation = ... .proof_cids(&[ucan_for_a.cid()]) ... ;
    //   let err = atrium_b_engine.consume_invocation(&invocation).unwrap_err();
    //   assert!(matches!(err, benten_engine::EngineError::UcanAudienceMismatch { .. }),
    //       "engine consumption of cross-atrium-replayed UCAN must reject per CLR-2");
    //
    // OBSERVABLE consequence: the engine's invocation-consumption
    // path enforces audience binding even at deep call sites
    // (WAIT-resume / SUBSCRIBE / manifest verify / engine read).
    unimplemented!(
        "G14-D wires engine-side audience-binding rejection for cross-atrium UCAN replay"
    );
}

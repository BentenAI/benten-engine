//! R4-FP RED-PHASE pin: napi serializer for AttributionFrame Phase-3
//! widening (pcds-r4-r1-1 instance-25 PRE-EMPTION).
//!
//! Pin source: `.addl/phase-3/r4-r1-producer-consumer-deep-sweep.json`
//! finding `pcds-r4-r1-1` MAJOR — schema-parity-missing-field (mode 5)
//! at AttributionFrame Phase-3 extension. Same shape as Phase-2b
//! Instance 18 (sandboxDepth widening) caught post-merge by R6-R3
//! r6-r3-pcds-1; pre-empted here at R4 corpus revision time.
//!
//! ## What this pins
//!
//! The napi-side trace projection at `bindings/napi/src/trace.rs:84-107`
//! serializes AttributionFrame to a JSON-shape consumed by the TS
//! `interface AttributionFrame` (see `packages/engine/src/types.ts`).
//!
//! Phase-3 widens AttributionFrame with new producer fields:
//!   - `peer_did_set: Vec<String>` (G16-B wave-6b loro_version_chain)
//!   - `device_did: Option<String>` (G14-D wave-5a sync_replica_attribution)
//!   - `device_cid: Option<String>` (G14-D wave-5a; optional companion)
//!
//! The napi serializer MUST emit these fields with the camelCase JSON
//! keys the TS consumer reads:
//!   - peer_did_set → "peerDidSet"
//!   - device_did   → "deviceDid"
//!   - device_cid   → "deviceCid"
//!
//! ## Pairs with
//!
//!   - `crates/benten-engine/tests/loro_version_chain.rs:94` (producer pin
//!     for peer_did_set)
//!   - `crates/benten-engine/tests/sync_replica_attribution.rs:36` (producer
//!     pin for device_did)
//!   - `packages/engine/test/attribution_frame_widening.test.ts` (TS-side
//!     consumer schema pin + end-to-end runtime pin)
//!   - `crates/benten-eval/tests/invariant_14_fixture_cid.rs` (Phase-3
//!     fixture-CID retrospective extension per pcds-r4-r1-5)
//!
//! ## RED-PHASE discipline
//!
//! The napi serializer at trace.rs does not yet emit these fields.
//! G14-D + G16-B wires the producer; this test pin asserts the napi
//! serializer companion fires the camelCase JSON keys end-to-end.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — napi AttributionFrame serializer emits peerDidSet + deviceDid + deviceCid camelCase keys. G14-D wave-5a + G16-B wave-6b + G16-D wave-6b ALL shipped (PRs #115/#126/#163); test body pins specific napi-side AttributionFrame camelCase serializer contract; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn napi_attribution_frame_serializer_emits_phase_3_peer_did_set_device_did_device_cid_camel_case_keys()
 {
    // pcds-r4-r1-1 LOAD-BEARING napi-companion pin per pim-2 §3.6b.
    //
    // G14-D + G16-B implementer wires this (un-ignored at the LATER of
    // G14-D wave-5a + G16-B wave-6b — both producer fields land before
    // the napi serializer is fully wired):
    //
    //   // Construct an AttributionFrame with the Phase-3 widening:
    //   let mut frame = benten_eval::AttributionFrame::new(
    //       actor_cid, handler_cid, capability_grant_cid, /* sandbox_depth */ 0
    //   );
    //   frame.peer_did_set = vec!["did:key:peer1".into(), "did:key:peer2".into()];
    //   frame.device_did = Some("did:key:devA".into());
    //   frame.device_cid = Some("bafydevice".into());
    //
    //   // Round-trip through the napi serializer:
    //   let json: serde_json::Value = napi_serialize_attribution_frame(&frame);
    //
    //   // OBSERVABLE consequence: camelCase JSON keys present + values
    //   // round-trip verbatim.
    //   assert_eq!(json["peerDidSet"], serde_json::json!(["did:key:peer1", "did:key:peer2"]));
    //   assert_eq!(json["deviceDid"], serde_json::json!("did:key:devA"));
    //   assert_eq!(json["deviceCid"], serde_json::json!("bafydevice"));
    //
    //   // Negative pin: NO snake_case key leakage (the napi boundary
    //   // is the last camelCase translation point):
    //   assert!(json.get("peer_did_set").is_none(),
    //       "napi serializer must NOT emit snake_case key peer_did_set");
    //   assert!(json.get("device_did").is_none(),
    //       "napi serializer must NOT emit snake_case key device_did");
    //
    // Defends against Phase-2b Instance 18 sandboxDepth recurrence: the
    // Rust producer is widened but the napi serializer drops the new
    // field silently (or emits it under a snake_case key the TS consumer
    // never reads). Both failure modes surface here as a hard test fail.
    unimplemented!(
        "G14-D + G16-B wires napi AttributionFrame serializer for peerDidSet + deviceDid + deviceCid camelCase keys"
    );
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — napi AttributionFrame serializer omits Phase-3 widening fields when absent. G14-D + G16-B all shipped; test body pins specific undefined-vs-null pre-emption contract; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn napi_attribution_frame_serializer_omits_phase_3_widening_fields_when_unset() {
    // Companion pre-emption pin: when the Phase-3 widening fields are
    // absent on the Rust producer side (e.g. local-only non-sync-replica
    // path; non-Loro-merged anchor), the napi serializer MUST omit the
    // JSON keys entirely (not emit `null` — undefined-vs-null is the
    // D9 / D14 forward-compat discipline).
    //
    //   // Construct a base-case AttributionFrame WITHOUT the Phase-3
    //   // fields (mirrors a Phase-1/2-shape attribution chain):
    //   let frame = benten_eval::AttributionFrame::new(
    //       actor_cid, handler_cid, capability_grant_cid, 0
    //   );
    //   // peer_did_set defaults to empty Vec; device_did + device_cid default to None.
    //
    //   let json: serde_json::Value = napi_serialize_attribution_frame(&frame);
    //
    //   // OBSERVABLE consequence: Phase-3 keys ABSENT from JSON when
    //   // producer has no value to emit (forward-compat parity with
    //   // pre-Phase-3 trace consumers + canonical-bytes drop-key-when-empty
    //   // discipline per D9):
    //   assert!(
    //       json.get("peerDidSet").is_none()
    //         || json["peerDidSet"] == serde_json::json!([]),
    //       "peerDidSet should be omitted (or empty array) when producer has no peers"
    //   );
    //   assert!(json.get("deviceDid").is_none(),
    //       "deviceDid MUST be omitted (not null) when producer has no device-DID");
    //   assert!(json.get("deviceCid").is_none(),
    //       "deviceCid MUST be omitted (not null) when producer has no device-CID");
    //
    // Defends against the TS consumer reading `null` and treating it as
    // a meaningful value (vs `undefined` which the optional-field shape
    // expects). pim-1 §3.5b HARDENED doc-coupling: the TS .d.ts +
    // canonical-bytes encoding + napi serializer must agree.
    unimplemented!(
        "G14-D + G16-B wires napi AttributionFrame serializer to omit Phase-3 fields when unset"
    );
}

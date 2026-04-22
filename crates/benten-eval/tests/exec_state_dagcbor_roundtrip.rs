//! R3 unit tests for G3-A (§9.1): DAG-CBOR round-trip + CID determinism for
//! ExecutionState envelopes. Also proptest `prop_exec_state_dagcbor_roundtrip`
//! (bijective serialise / deserialise with CID stability).
//!
//! TDD red-phase: encode / decode do not yet exist on `ExecutionStateEnvelope`.
//! Tests fail to compile until G3-A lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.2 + §3 proptest).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::{AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload};
use proptest::prelude::*;

fn zero_cid() -> Cid {
    Cid::from_bytes(&[0u8; benten_core::CID_LEN]).expect("zero cid")
}

fn sample_payload() -> ExecutionStatePayload {
    ExecutionStatePayload {
        attribution_chain: vec![AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
        }],
        pinned_subgraph_cids: vec![zero_cid()],
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero_cid(),
        frame_stack: Vec::new(),
        frame_index: 0,
    }
}

#[test]
fn exec_state_dagcbor_roundtrip() {
    let envelope = ExecutionStateEnvelope::new(sample_payload()).expect("construct");

    let bytes = envelope.to_dagcbor().expect("encode");
    let decoded = ExecutionStateEnvelope::from_dagcbor(&bytes).expect("decode");

    // Round-trip structurally equal.
    assert_eq!(decoded.schema_version, envelope.schema_version);
    assert_eq!(decoded.payload_cid, envelope.payload_cid);
    assert_eq!(
        decoded.payload.resumption_principal_cid,
        envelope.payload.resumption_principal_cid
    );
    assert_eq!(
        decoded.payload.pinned_subgraph_cids,
        envelope.payload.pinned_subgraph_cids
    );
}

#[test]
fn exec_state_cid_deterministic() {
    // Two independent encodes of the same payload must produce byte-identical
    // bytes and identical envelope CIDs.
    let e1 = ExecutionStateEnvelope::new(sample_payload()).expect("e1");
    let e2 = ExecutionStateEnvelope::new(sample_payload()).expect("e2");

    let b1 = e1.to_dagcbor().expect("b1");
    let b2 = e2.to_dagcbor().expect("b2");
    assert_eq!(
        b1, b2,
        "byte-level determinism required for DAG-CBOR round-trip"
    );

    let cid1 = e1.envelope_cid().expect("cid1");
    let cid2 = e2.envelope_cid().expect("cid2");
    assert_eq!(cid1, cid2, "envelope CID must be stable across re-encodes");
}

#[test]
fn envelope_payload_cid_matches_hash_of_payload_bytes() {
    let envelope = ExecutionStateEnvelope::new(sample_payload()).expect("construct");
    let recomputed = envelope.recompute_payload_cid().expect("recompute");
    assert_eq!(
        envelope.payload_cid, recomputed,
        "envelope.payload_cid must equal cid(dagcbor(payload))"
    );
}

#[test]
fn envelope_with_mismatched_payload_cid_detectable() {
    // Constructing with an intentionally-wrong payload_cid and verifying via
    // `recompute_payload_cid` must surface the mismatch (used by the resume
    // protocol step 1).
    let payload = sample_payload();
    let wrong_cid = Cid::from_bytes(&[0u8; benten_core::CID_LEN]).unwrap(); // any fixed value
    let envelope = ExecutionStateEnvelope {
        schema_version: 1,
        payload_cid: wrong_cid,
        payload: payload.clone(),
    };
    let recomputed = envelope.recompute_payload_cid().expect("recompute");
    assert_ne!(
        envelope.payload_cid, recomputed,
        "mismatched payload_cid must be detectable by recompute_payload_cid"
    );
}

// ---- Proptest: round-trip + CID stability --------------------------------

proptest! {
    /// `prop_exec_state_dagcbor_roundtrip`: bijective serialise / deserialise
    /// with CID stability across arbitrary attribution-chain depths and
    /// pinned-subgraph counts. Case count controlled by PROPTEST_CASES env.
    #[test]
    fn prop_exec_state_dagcbor_roundtrip(
        chain_depth in 1usize..8,
        pin_count in 0usize..16,
    ) {
        let frame = AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
        };
        let attribution_chain = vec![frame; chain_depth];
        let pinned_subgraph_cids = vec![zero_cid(); pin_count];

        let payload = ExecutionStatePayload {
            attribution_chain,
            pinned_subgraph_cids,
            context_binding_snapshots: Vec::new(),
            resumption_principal_cid: zero_cid(),
            frame_stack: Vec::new(),
            frame_index: 0,
        };
        let envelope = ExecutionStateEnvelope::new(payload).expect("construct");

        let bytes1 = envelope.to_dagcbor().expect("encode1");
        let bytes2 = envelope.to_dagcbor().expect("encode2");
        prop_assert_eq!(&bytes1, &bytes2, "encode is deterministic");

        let decoded =
            ExecutionStateEnvelope::from_dagcbor(&bytes1).expect("decode");
        let re_encoded = decoded.to_dagcbor().expect("re-encode");
        prop_assert_eq!(bytes1, re_encoded, "re-encode must be bijective");
    }
}

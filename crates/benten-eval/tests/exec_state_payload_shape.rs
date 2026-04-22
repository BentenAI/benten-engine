//! R3 unit tests for G3-A / ucca-1 / ucca-4 (§9.1): `ExecutionStatePayload`
//! struct shape — FROZEN interface.
//!
//! Locked 6-field shape per plan §9.1:
//!   - attribution_chain: Vec<AttributionFrame>  (NOT a 3-tuple)
//!   - pinned_subgraph_cids: Vec<Cid>  (sorted + deduped)
//!   - context_binding_snapshots: Vec<(String, Cid, Vec<u8>)>  (inline bytes, not re-fetch)
//!   - resumption_principal_cid: Cid  (atk-1 mitigation)
//!   - frame_stack: Vec<Frame>
//!   - frame_index: usize
//!
//! TDD red-phase: `ExecutionStatePayload` does not yet exist. Tests fail to
//! compile until G3-A lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::{AttributionFrame, ExecutionStatePayload, Frame};

fn zero_cid() -> Cid {
    // R5 G3-A note: `from_bytes` on an all-zero buffer fails CID-header
    // validation; the zero-digest CID is the intended fixture.
    Cid::from_blake3_digest([0u8; 32])
}

fn unit_cid(byte: u8) -> Cid {
    let mut bytes = [0u8; benten_core::CID_LEN];
    // Skip the multihash header bytes; set the last byte of the digest.
    let last = bytes.len() - 1;
    bytes[last] = byte;
    // Preserve the multicodec prefix that from_bytes validates against.
    bytes[0] = 0x01; // cidv1
    bytes[1] = 0x71; // dag-cbor
    bytes[2] = 0x1e; // blake3
    bytes[3] = 0x20; // length
    Cid::from_bytes(&bytes).expect("unit cid")
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_payload_has_attribution_chain_not_3_tuple() {
    let payload = ExecutionStatePayload {
        attribution_chain: vec![
            AttributionFrame {
                actor_cid: zero_cid(),
                handler_cid: zero_cid(),
                capability_grant_cid: zero_cid(),
            },
            AttributionFrame {
                actor_cid: zero_cid(),
                handler_cid: zero_cid(),
                capability_grant_cid: zero_cid(),
            },
        ],
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero_cid(),
        frame_stack: Vec::new(),
        frame_index: 0,
    };

    // The field type is Vec<AttributionFrame>, not a fixed 3-tuple.
    assert_eq!(payload.attribution_chain.len(), 2);
    let first: &AttributionFrame = &payload.attribution_chain[0];
    let _ = first;
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_payload_pinned_subgraph_cids_sorted_deduped() {
    // Constructor must enforce sorted + deduped invariant on pinned_subgraph_cids.
    let a = unit_cid(1);
    let b = unit_cid(2);
    let c = unit_cid(3);
    // Intentional duplicates and out-of-order input.
    let input = vec![c, a, b, a, c];
    let payload = ExecutionStatePayload::new_with_pinned(input);

    // Result is sorted byte-lexicographic and unique.
    assert_eq!(
        payload.pinned_subgraph_cids,
        vec![a, b, c],
        "pinned_subgraph_cids must be sorted + deduped at construction"
    );
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_payload_context_binding_snapshots_inline_bytes() {
    // Must be Vec<(String, Cid, Vec<u8>)> — inline bytes snapshotted, NOT
    // Vec<(String, Cid)> (re-fetch variant). This is the CID-substitution
    // mitigation from §9.1.
    let payload = ExecutionStatePayload {
        attribution_chain: Vec::new(),
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: vec![("input".to_string(), zero_cid(), vec![0xDE, 0xAD])],
        resumption_principal_cid: zero_cid(),
        frame_stack: Vec::new(),
        frame_index: 0,
    };
    let (name, cid, bytes) = &payload.context_binding_snapshots[0];
    assert_eq!(name, "input");
    assert_eq!(*cid, zero_cid());
    assert_eq!(bytes, &vec![0xDE, 0xAD]);
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_payload_has_resumption_principal_cid() {
    // ucca-4: resumption_principal_cid is a REQUIRED field (not Option).
    let payload = ExecutionStatePayload {
        attribution_chain: Vec::new(),
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: unit_cid(7),
        frame_stack: Vec::new(),
        frame_index: 0,
    };
    assert_eq!(payload.resumption_principal_cid, unit_cid(7));
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_payload_has_frame_stack_and_frame_index() {
    // Frame stack is a Vec<Frame> and frame_index is usize.
    let frame: Frame = Frame::root_for_test();
    let payload = ExecutionStatePayload {
        attribution_chain: Vec::new(),
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero_cid(),
        frame_stack: vec![frame],
        frame_index: 0,
    };
    assert_eq!(payload.frame_stack.len(), 1);
    assert_eq!(payload.frame_index, 0usize);
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_payload_all_6_fields_present() {
    // Catch-all: constructing with all 6 fields named in order must compile.
    // If a field is added or removed, this fails to compile and the signal
    // is unambiguous.
    let payload = ExecutionStatePayload {
        attribution_chain: Vec::new(),
        pinned_subgraph_cids: Vec::new(),
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero_cid(),
        frame_stack: Vec::new(),
        frame_index: 0,
    };
    // Field-count sentinel — if a 7th field is added additively, this count
    // does not change and the test still passes. The compile-time shape is
    // the primary guarantee.
    assert!(payload.attribution_chain.is_empty());
    assert!(payload.pinned_subgraph_cids.is_empty());
    assert!(payload.context_binding_snapshots.is_empty());
    assert_eq!(payload.resumption_principal_cid, zero_cid());
    assert!(payload.frame_stack.is_empty());
    assert_eq!(payload.frame_index, 0);
}

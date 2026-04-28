//! R3 unit tests for G3-A (§9.1): `ExecutionStateEnvelope` struct shape —
//! FROZEN interface.
//!
//! Locked shape per plan §9.1:
//! ```rust
//! pub struct ExecutionStateEnvelope {
//!     pub schema_version: u8,
//!     pub payload_cid: Cid,
//!     pub payload: ExecutionStatePayload,
//! }
//! ```
//!
//! TDD red-phase: `ExecutionStateEnvelope` does not yet exist. Tests fail to
//! compile until G3-A lands.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.5.2).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_eval::{AttributionFrame, ExecutionStateEnvelope, ExecutionStatePayload};

fn zero_cid() -> Cid {
    // R5 G3-A note: `from_bytes` on an all-zero buffer fails CID-header
    // validation; the zero-digest CID is the intended fixture.
    Cid::from_blake3_digest([0u8; 32])
}

fn minimal_payload() -> ExecutionStatePayload {
    ExecutionStatePayload {
        attribution_chain: vec![AttributionFrame {
            actor_cid: zero_cid(),
            handler_cid: zero_cid(),
            capability_grant_cid: zero_cid(),
            sandbox_depth: 0,
        }],
        pinned_subgraph_cids: vec![zero_cid()],
        context_binding_snapshots: Vec::new(),
        resumption_principal_cid: zero_cid(),
        frame_stack: Vec::new(),
        frame_index: 0,
    }
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_envelope_shape_matches_spec() {
    // Constructing with every named field must compile. Any rename / removal
    // breaks this file.
    let envelope = ExecutionStateEnvelope {
        schema_version: 1,
        payload_cid: zero_cid(),
        payload: minimal_payload(),
    };

    assert_eq!(envelope.schema_version, 1);
    assert_eq!(envelope.payload_cid, zero_cid());
    // Payload access is structural; `attribution_chain` is a Vec (not a tuple).
    assert_eq!(envelope.payload.attribution_chain.len(), 1);
}

/// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
/// Does NOT validate firing semantics (those land in Phase 2b).
#[test]
fn exec_state_schema_version_is_1_in_phase_2a() {
    // Every newly constructed envelope (via the canonical constructor or
    // ExecutionStatePayload::into_envelope path) MUST carry schema_version=1
    // in Phase 2a. Phase 2+ bumps additively.
    let envelope = ExecutionStateEnvelope::new(minimal_payload()).expect("construct");
    assert_eq!(
        envelope.schema_version, 1,
        "schema_version must be 1 in Phase 2a (canonical constructor contract)"
    );
}

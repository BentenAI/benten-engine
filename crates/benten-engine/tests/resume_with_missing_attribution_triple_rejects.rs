//! Phase 2a R3 security — ExecutionState frame must carry required
//! attribution-triple (sec-r1-1 / §9.1 attribution-triple-required-in-frame).
//!
//! **Attack class.** The plan §9.1 locks the payload shape at R1 close:
//! `attribution_chain: Vec<AttributionFrame>` where
//! `AttributionFrame { actor_cid, handler_cid, capability_grant_cid }`.
//! All three fields REQUIRED (not `Option<Cid>`). A resume path that
//! treats a missing field as "default" or "synthesized on demand" would
//! let an attacker submit bytes whose payload encodes a chain where a
//! frame's `capability_grant_cid` is absent/null — and the engine silently
//! re-synthesises an authority, laundering the original grant out of the
//! audit trail.
//!
//! **Prerequisite.** Same as atk-1 — attacker has the raw state bytes.
//! Also requires the attacker's DAG-CBOR re-serialiser: produce payload
//! bytes where `attribution_chain[N].capability_grant_cid` is omitted or
//! the whole frame is truncated to only actor+handler.
//!
//! **Attack sequence.**
//!  1. Obtain legitimate bytes via `suspend_to_bytes`.
//!  2. Decode the DAG-CBOR payload, drop `capability_grant_cid` from a
//!     frame (truncating to just `actor_cid` + `handler_cid`).
//!  3. Re-encode the envelope (we deliberately leave `payload_cid` at the
//!     ORIGINAL value — the integrity check must STILL refuse, even when
//!     the inner shape no longer matches the canonical
//!     `AttributionFrame`).
//!  4. Call `resume_from_bytes_unauthenticated(forged_bytes, signal)`.
//!
//! **Impact.** Resume runs with a synthesized grant; audit log records
//! writes as authorised by a grant that was never presented.
//!
//! **Mitigation in code.** `AttributionFrame` is a plain `serde::Deserialize`
//! struct with three REQUIRED `Cid` fields (no `Option<Cid>`). The
//! serde_ipld_dagcbor decoder rejects frames missing any of the three
//! before resume_from_bytes_inner reaches step 1; the typed error surfaces
//! as `E_SERIALIZE` (decode-failure path) and never as a panic. See
//! `crates/benten-eval/src/exec_state.rs` for the struct shape and
//! `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` for
//! the decode→typed-error mapping.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a). R6 fix-pass:
//! un-ignored once `AttributionFrame` shipped with all three CID fields
//! required and the resume decoder path was wired through G3-A + G3-B.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Cid, Value};
use benten_engine::{Engine, SuspensionOutcome};
use benten_errors::ErrorCode;
use benten_eval::ExecutionStateEnvelope;
use serde::Serialize;

/// Tampered attribution frame missing the third (`capability_grant_cid`)
/// field. This intentionally diverges from the canonical
/// `benten_eval::AttributionFrame` struct shape so that a DAG-CBOR
/// round-trip through the canonical decoder rejects with a typed error.
#[derive(Serialize)]
struct TamperedFrame {
    actor_cid: Cid,
    handler_cid: Cid,
    // capability_grant_cid intentionally omitted — this is the attack.
}

/// Tampered payload shape. Field names + ordering mirror
/// `benten_eval::ExecutionStatePayload` so the only delta the decoder
/// can observe is the truncated frame.
#[derive(Serialize)]
struct TamperedPayload {
    attribution_chain: Vec<TamperedFrame>,
    pinned_subgraph_cids: Vec<Cid>,
    context_binding_snapshots: Vec<(String, Cid, Vec<u8>)>,
    resumption_principal_cid: Cid,
    frame_stack: Vec<TamperedFrameStack>,
    frame_index: usize,
}

/// Mirror of `benten_eval::Frame` so the rest of the payload decodes
/// canonically — only the attribution frame triple is shrunk.
#[derive(Serialize)]
struct TamperedFrameStack {
    tag: String,
}

/// Tampered envelope wrapper mirroring `benten_eval::ExecutionStateEnvelope`.
#[derive(Serialize)]
struct TamperedEnvelope {
    schema_version: u8,
    payload_cid: Cid,
    payload: TamperedPayload,
}

/// sec-r1-1: attacker submits bytes whose AttributionFrame lacks one of the
/// three required CID fields. Decoder must reject at the serde layer; resume
/// must surface a typed decode-failure code (Serialize / ExecStateTampered)
/// before any side-effect — and must NOT panic.
#[test]
fn resume_with_missing_attribution_triple_rejects() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Step 1: produce a real envelope so we can read off the legitimate
    // CIDs and build a parallel "tampered" structure with consistent
    // sub-fields. Using a real suspension over our test helper means the
    // resumption_principal_cid + chain-head CIDs are well-formed, so any
    // decoder rejection is provably attributable to the missing
    // capability_grant_cid field — not to a malformed sibling field.
    let alice = benten_engine::testing::principal_cid("alice");
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler(
            "missing-triple",
        ))
        .expect("register WAIT handler");
    let suspended = match engine
        .call_as_with_suspension("missing-triple", "run", benten_core::Node::empty(), &alice)
        .expect("call_as_with_suspension")
    {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("WAIT handler must suspend"),
    };
    let bytes = engine
        .suspend_to_bytes(&suspended)
        .expect("suspend_to_bytes");

    // Decode once via the canonical envelope to harvest the head-of-chain
    // CIDs (so our tampered frame still names valid actors / handlers).
    let canonical = ExecutionStateEnvelope::from_dagcbor(&bytes).expect("decode envelope");
    let head = canonical
        .payload
        .attribution_chain
        .first()
        .expect("fixture sanity: legitimate envelope must have a chain head");
    let actor_cid = head.actor_cid;
    let handler_cid = head.handler_cid;

    // Step 2: build a parallel tampered envelope whose attribution frame
    // is MISSING the capability_grant_cid field. We carry the rest of the
    // payload across verbatim so the decoder's failure can ONLY come from
    // the truncated frame shape.
    let tampered = TamperedEnvelope {
        schema_version: canonical.schema_version,
        // Deliberately preserve the canonical payload_cid: when the
        // decoder accepts the (smaller) shape its recompute would
        // mismatch the claimed CID, surfacing E_EXEC_STATE_TAMPERED.
        // When the decoder rejects the smaller shape outright it
        // surfaces E_SERIALIZE. Both are typed failures; both pass the
        // sec-r1-1 contract. The test accepts either.
        payload_cid: canonical.payload_cid,
        payload: TamperedPayload {
            attribution_chain: vec![TamperedFrame {
                actor_cid,
                handler_cid,
                // capability_grant_cid intentionally absent.
            }],
            pinned_subgraph_cids: canonical.payload.pinned_subgraph_cids.clone(),
            context_binding_snapshots: canonical.payload.context_binding_snapshots.clone(),
            resumption_principal_cid: canonical.payload.resumption_principal_cid,
            frame_stack: canonical
                .payload
                .frame_stack
                .iter()
                .map(|f| TamperedFrameStack { tag: f.tag.clone() })
                .collect(),
            frame_index: canonical.payload.frame_index,
        },
    };
    let tampered_bytes = serde_ipld_dagcbor::to_vec(&tampered).expect("encode tampered envelope");

    assert_ne!(
        tampered_bytes, bytes,
        "fixture sanity: tampered bytes must differ from canonical bytes"
    );

    // Step 3: hand the tampered bytes to resume. Must surface a typed
    // error (Serialize on outright decode rejection, or ExecStateTampered
    // on payload-CID recompute mismatch) — and MUST NOT panic.
    let err = engine
        .resume_from_bytes_unauthenticated(&tampered_bytes, Value::text("attack-signal"))
        .expect_err("frame with missing triple must deny");
    assert!(
        matches!(
            err.code(),
            ErrorCode::Serialize | ErrorCode::ExecStateTampered
        ),
        "missing-triple frame must surface a typed decode-failure code \
         (Serialize or ExecStateTampered); got {:?}",
        err.code()
    );
}

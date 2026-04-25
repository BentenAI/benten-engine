//! Phase 2a R3 security — resume-protocol step 1 (atk-1 / sec-r1-1).
//!
//! **Attack class.** `ExecutionState` is persisted as DAG-CBOR bytes
//! (§9.1) with an envelope carrying `payload_cid` = CID of the inner
//! payload bytes. If the resume path does NOT recompute `payload_cid` from
//! the decoded payload bytes and compare against the envelope's claimed
//! `payload_cid`, an attacker who obtains the serialised bytes can flip
//! any field inside the payload (e.g. swap `attribution_chain[0].actor_cid`
//! to their own) and have the engine trust the forged attribution.
//!
//! **Prerequisite.** Attacker possesses the raw `Vec<u8>` produced by
//! `Engine::suspend_to_bytes(handle)`. In Phase-2a this surfaces via any
//! backup, crash-dump, or shared storage path.
//!
//! **Attack sequence.**
//!  1. Run handler to suspension, call `engine.suspend_to_bytes(handle)` to
//!     get `bytes: Vec<u8>`.
//!  2. Decode the envelope; mutate `payload.attribution_chain[0].actor_cid`
//!     to a DIFFERENT valid CID (a second principal); re-encode.
//!  3. Call `engine.resume_from_bytes_unauthenticated(tampered_bytes,
//!     signal)`.
//!  4. Expected mitigation: `E_EXEC_STATE_TAMPERED` fires before any
//!     side-effect happens. The mutation changed the canonical payload
//!     bytes so step 1's `recompute_payload_cid` mismatches the envelope's
//!     claimed `payload_cid`.
//!
//! **Impact.** Audit-trail forgery; the forged attribution would let the
//! attacker's CID "own" writes the legitimate handler initiated.
//!
//! **Recommended mitigation.** Resume step 1 (§9.1 4-step protocol):
//! recompute `payload_cid` from payload bytes; reject mismatch with
//! `E_EXEC_STATE_TAMPERED`. The check is the envelope's CID integrity —
//! free with the content-addressing machinery Benten already has.
//!
//! Wave-3c R4b fix-pass: this test was previously `#[ignore]`d with a
//! `panic!("red-phase: ...")` body. G3-A landed `ExecutionStateEnvelope`
//! with `from_dagcbor` / `to_dagcbor` / `recompute_payload_cid`, and G3-B
//! landed `suspend_to_bytes` / `resume_from_bytes_unauthenticated` — so
//! the atk-1 attribution-tamper attack is now end-to-end pinned (not just
//! the random byte-flip via `resume_decode_failure_not_panic.rs`).
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
use benten_engine::Engine;
use benten_engine::SuspensionOutcome;
use benten_errors::ErrorCode;
use benten_eval::ExecutionStateEnvelope;

/// atk-1: attacker mutates `attribution_chain[0].actor_cid` inside the
/// persisted ExecutionState payload and calls resume. Resume step 1 (§9.1)
/// must reject with `E_EXEC_STATE_TAMPERED` before any side effect.
#[test]
fn resume_with_tampered_attribution_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Suspend a WAIT-composing handler to obtain a real envelope.
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("tamper-attr"))
        .expect("register WAIT handler");
    let suspended = match engine
        .call_with_suspension("tamper-attr", "run", benten_core::Node::empty())
        .expect("call_with_suspension")
    {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("WAIT handler must suspend"),
    };
    let bytes = engine
        .suspend_to_bytes(&suspended)
        .expect("suspend_to_bytes");

    // Decode → mutate the head-of-chain actor_cid → re-encode. Critically
    // we do NOT touch `envelope.payload_cid` — the integrity check in
    // step 1 catches the payload mutation precisely BECAUSE the envelope
    // still claims the original CID.
    let mut envelope = ExecutionStateEnvelope::from_dagcbor(&bytes).expect("decode envelope");
    let original_cid = envelope.payload_cid;

    // Substitute a different valid CID into attribution_chain[0].actor_cid.
    // BLAKE3 of a sentinel byte string yields a CIDv1-shape Cid that is
    // NOT equal to whatever the synthesised attribution chain holds.
    let attacker_cid = Cid::from_blake3_digest(*blake3::hash(b"atk-1:attacker").as_bytes());
    if let Some(frame) = envelope.payload.attribution_chain.first_mut() {
        assert_ne!(
            frame.actor_cid, attacker_cid,
            "fixture sanity: attacker CID must differ from the legitimate \
             actor CID, otherwise the mutation would be a no-op and the \
             tamper assertion would silently pass"
        );
        frame.actor_cid = attacker_cid;
    } else {
        // Phase-2a synthesised envelopes seed a single attribution frame;
        // an empty chain would defeat the test premise.
        panic!(
            "fixture sanity: attribution_chain unexpectedly empty — atk-1 \
             requires a head-of-chain frame to mutate"
        );
    }

    let tampered_bytes = envelope.to_dagcbor().expect("re-encode tampered envelope");
    assert_ne!(
        tampered_bytes, bytes,
        "mutation must change the canonical bytes (otherwise step 1 cannot \
         distinguish tamper from genuine bytes)"
    );

    let err = engine
        .resume_from_bytes_unauthenticated(&tampered_bytes, benten_core::Value::text("sig"))
        .expect_err("tampered attribution must be rejected before any side effect");
    assert_eq!(
        err.code(),
        ErrorCode::ExecStateTampered,
        "atk-1 attribution-tamper must fire E_EXEC_STATE_TAMPERED via the \
         step-1 payload_cid recompute (envelope still claims {} but the \
         re-encoded payload hashes differently). Got: {err:?}",
        original_cid.to_base32()
    );
}

/// Sanity: confirm the crate's capability-policy builder + create_principal
/// path compiles and runs without the (future) WAIT API. This keeps the
/// test file from silently regressing its compilation setup.
#[test]
fn fixture_setup_path_still_compiles() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();
    let alice = engine
        .create_principal("alice")
        .expect("principal creation succeeds");
    // Sanity read (no WAIT involved).
    let _maybe = engine.get_node(&alice).unwrap();
    // Node::empty() construction path for later use.
    let _ = benten_core::Node::empty();
}

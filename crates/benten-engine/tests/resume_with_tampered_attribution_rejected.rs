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
//!  2. Flip one byte inside the inner payload (NOT the envelope's
//!     `payload_cid` field). Because DAG-CBOR is canonical, any bit flip
//!     changes the content's re-hashed CID.
//!  3. Call `engine.resume_from_bytes(tampered_bytes, signal)`.
//!  4. Expected mitigation: `E_EXEC_STATE_TAMPERED` fires before any
//!     side-effect happens.
//!
//! **Impact.** Audit-trail forgery; the forged attribution has the
//! attacker's handler_cid "owning" writes the handler actually initiated.
//!
//! **Recommended mitigation.** Resume step 1 (§9.1 4-step protocol):
//! recompute `payload_cid` from payload bytes; reject mismatch with
//! `E_EXEC_STATE_TAMPERED`. The check is the envelope's CID integrity —
//! free with the content-addressing machinery Benten already has.
//!
//! **Red-phase contract.** G3-A has not yet landed `ExecutionStateEnvelope`,
//! `suspend_to_bytes`, or `resume_from_bytes`. Test is `#[ignore]`d with a
//! pending marker. Body documents the assertion shape so R5 G3-A has a
//! concrete target. A sanity test in the same file verifies that the
//! fixture setup path (WAIT handler registration) already compiles.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::Engine;

/// atk-1: attacker flips a byte inside the persisted ExecutionState payload
/// and calls resume. Resume step 1 (§9.1) must reject with
/// `E_EXEC_STATE_TAMPERED` before any side effect.
#[test]
#[ignore = "phase-2a-pending: Engine::call_with_suspension / suspend_to_bytes / resume_from_bytes + ExecutionStateEnvelope land in G3-A per plan §9.1. Drop #[ignore] once the WAIT suspend/resume API is live."]
fn resume_with_tampered_attribution_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Setup: register a handler that SUSPENDS on an external signal.
    // Phase-1 DSL does not yet expose WAIT via SubgraphSpec builder — the
    // registration below is the G3-A target shape. Once the `.wait(...)`
    // builder lands, populate it here.
    let _alice = engine.create_principal("alice").unwrap();

    // Target API (G3-A):
    //
    //     let sg = benten_engine::SubgraphSpec::builder()
    //         .handler_id("wait-handler")
    //         .wait(|w| w.signal("external:signal"))
    //         .respond()
    //         .build();
    //     let handler_id = engine.register_subgraph(sg).unwrap();
    //
    //     let suspended = engine
    //         .call_with_suspension(&handler_id, "run", Node::empty())
    //         .expect("setup: call suspends")
    //         .unwrap_suspended();
    //     let mut bytes = engine
    //         .suspend_to_bytes(suspended)
    //         .expect("setup: serialise");
    //
    //     // Flip a byte at a known offset inside the payload (skip the
    //     // envelope header).
    //     let offset = find_payload_byte_offset(&bytes);
    //     bytes[offset] ^= 0x01;
    //
    //     let outcome = engine.resume_from_bytes(bytes, Node::empty());
    //     let err = outcome.expect_err("tampered bytes must be rejected");
    //     assert_eq!(err.code().as_str(), "E_EXEC_STATE_TAMPERED");
    //
    // Red-phase placeholder: the API shapes don't exist yet, so explicit
    // panic keeps the test red with a clear red-phase pointer.
    panic!(
        "red-phase: call_with_suspension / suspend_to_bytes / \
         resume_from_bytes not yet present. G3-A to land; see plan §9.1 \
         resume protocol step 1 for the E_EXEC_STATE_TAMPERED firing \
         contract."
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
    let _ = Node::empty();
}

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
//!  2. Decode the DAG-CBOR payload, delete a frame's `capability_grant_cid`
//!     field (or truncate a frame to just `actor_cid` + `handler_cid`).
//!  3. Re-hash (the `payload_cid` in the envelope) — the tamper is
//!     consistent at the envelope level.
//!  4. Call `resume_from_bytes(forged_bytes, signal)`.
//!
//! **Impact.** Resume runs with a synthesized grant; audit log records
//! writes as authorised by a grant that was never presented.
//!
//! **Recommended mitigation.** `AttributionFrame` struct-shape via
//! `#[serde(deny_unknown_fields)]` + REQUIRED `Cid` (not `Option<Cid>`)
//! for all three fields. DAG-CBOR decoder refuses frames with missing
//! fields at the serde layer; resume step 1 re-verifies via
//! `payload_cid` recompute; forgeries that shrink the frame shape fail
//! to decode before any cap check runs.
//!
//! **Red-phase contract.** G3-A lands `AttributionFrame` + the
//! `#[serde(deny_unknown_fields)]` derive. Test asserts that a frame
//! lacking `capability_grant_cid` fails to decode (surfacing a typed
//! `E_EXEC_STATE_TAMPERED` or `E_EXEC_STATE_INVALID`). Until G3-A lands,
//! the test is `#[ignore]`d with a pending marker.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// sec-r1-1: attacker submits bytes whose AttributionFrame lacks one of the
/// three required CID fields. Decoder must reject at the serde layer; resume
/// must surface `E_EXEC_STATE_TAMPERED` before any side-effect.
#[test]
#[ignore = "phase-2a-pending: AttributionFrame + resume decoder land in G3-A per plan §9.1. Drop #[ignore] once AttributionFrame's required-field shape is serde-derived."]
fn resume_with_missing_attribution_triple_rejects() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    let _alice = engine.create_principal("alice").unwrap();

    // Target API path (G3-A):
    //
    //     let legit_bytes = suspend_and_serialize(&engine, "wait-handler");
    //
    //     // Decode, delete one frame's capability_grant_cid, re-encode.
    //     let tampered = strip_grant_cid_from_first_frame(legit_bytes);
    //
    //     let outcome = engine.resume_from_bytes(tampered, signal_value());
    //     let err = outcome.expect_err("frame with missing triple must deny");
    //     let code = err.code().as_str();
    //     assert!(
    //         code == "E_EXEC_STATE_TAMPERED" || code == "E_EXEC_STATE_INVALID",
    //         "missing triple must deny with a typed decode-failure code; \
    //          got {code}"
    //     );
    //
    // Sanity on currently-available APIs: principal + policy wiring compile.

    panic!(
        "red-phase: AttributionFrame required-field decoder not yet \
         present. G3-A to land with #[serde(deny_unknown_fields)] / \
         required Cid fields per plan §9.1."
    );
}

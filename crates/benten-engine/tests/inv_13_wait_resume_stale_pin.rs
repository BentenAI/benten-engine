//! Phase 2a R3 security — Inv-13 WAIT-resume stale pin (code-as-graph
//! Major #4 / §9.11 row 5).
//!
//! R4 qa-r4-10 cross-reference: R2 §4.7 lists this under
//! `crates/benten-engine/tests/integration/wait_resume_stale_pin.rs`. The
//! file-split is kept; this header names the R2 anchor.
//!
//! **Attack class.** A handler carrying WAIT is registered, invoked,
//! driven to suspension. Before resume, the version-chain CURRENT of one
//! of the transitively-invoked subgraphs MOVES (new version registered,
//! CURRENT pointer updated). On resume, the persisted
//! `pinned_subgraph_cids` entry no longer matches the anchor's CURRENT.
//!
//! Phase 2a §9.11 row 5: `E_RESUME_SUBGRAPH_DRIFT` fires BEFORE any write
//! attempt — it's NOT an Inv-13 immutability firing (that's for WRITE
//! paths); it's a resume-step-3 pre-check.
//!
//! **Prerequisite.** Multi-subgraph workflow with CALL, suspend mid-
//! walk, operator publishes a new version of one of the transitively-
//! referenced subgraphs between suspend and resume.
//!
//! **Attack sequence.**
//!  1. Register handler A; A calls handler B via CALL.
//!  2. Invoke A, suspend partway through. Payload
//!     `pinned_subgraph_cids` contains both A's CID and B's CID.
//!  3. Re-register B under a new version (new subgraph CID). CURRENT
//!     pointer for B's handler_id moves.
//!  4. Call `engine.resume_from_bytes(bytes, signal)`.
//!  5. Mitigation: resume step 3 (§9.1) re-verifies each pinned CID
//!     against the registered-subgraph table; the pin for B fails ⇒
//!     `E_RESUME_SUBGRAPH_DRIFT` fires before any write.
//!
//! **Impact (without mitigation).** Resume silently executes against the
//! NEW version of B instead of the one the suspension was authorised for.
//! Code-as-graph bait-and-switch: operator publishes v2 of B while Alice's
//! workflow is suspended; Alice's resume unwittingly runs the new code.
//!
//! **Recommended mitigation.** Resume step 3 in the 4-step protocol (§9.1)
//! — every `pinned_subgraph_cids` entry re-verified against the
//! registered-subgraph table; mismatch = `E_RESUME_SUBGRAPH_DRIFT` BEFORE
//! any write or cap re-check.
//!
//! **Red-phase contract.** G3-A lands the pinned_subgraph_cids field +
//! resume step 3. Until then `#[ignore]`d with a pending marker.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// code-as-graph Major #4 / §9.11 row 5: WAIT-resume with a stale pin
/// fires `E_RESUME_SUBGRAPH_DRIFT` BEFORE any write attempt.
#[test]
#[ignore = "phase-2a-pending: pinned_subgraph_cids + resume step 3 land in G3-A per plan §9.1. Drop #[ignore] once resume_from_bytes re-verifies pinned CIDs."]
fn inv_13_wait_resume_stale_pin_fires_resume_subgraph_drift() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    let _alice = engine.create_principal("alice").unwrap();

    // Target API path (G3-A):
    //
    //     // Register B v1.
    //     let b_v1 = SubgraphSpec::builder()
    //         .handler_id("b")
    //         .write(|w| w.label("post").requires("store:post:write"))
    //         .respond()
    //         .build();
    //     let b_id = engine.register_subgraph(b_v1).unwrap();
    //     let b_v1_cid = engine.subgraph_cid_for(&b_id).unwrap();
    //
    //     // Register A, which CALLs B.
    //     let a = SubgraphSpec::builder()
    //         .handler_id("a")
    //         .wait(|w| w.signal("external:signal"))
    //         .call(|c| c.handler_id(&b_id))
    //         .respond()
    //         .build();
    //     let a_id = engine.register_subgraph(a).unwrap();
    //
    //     // Invoke A, suspend.
    //     let suspended = engine
    //         .call_with_suspension(&a_id, "run", Node::empty())
    //         .unwrap()
    //         .unwrap_suspended();
    //     let bytes = engine.suspend_to_bytes(suspended).unwrap();
    //
    //     // Between suspend and resume, register B v2 (new version).
    //     let b_v2 = SubgraphSpec::builder()
    //         .handler_id("b")
    //         .write(|w| w.label("post").requires("store:post:write"))
    //         .write(|w| w.label("audit").property("version", Value::Int(2)))
    //         .respond()
    //         .build();
    //     let _ = engine.register_subgraph(b_v2).unwrap();
    //     let b_v2_cid = engine.subgraph_cid_for(&b_id).unwrap();
    //     assert_ne!(b_v1_cid, b_v2_cid, "version bump changes CID");
    //
    //     // Attempt resume; pinned_subgraph_cids still references v1.
    //     let outcome = engine.resume_from_bytes(bytes, signal_value());
    //     let err = outcome.expect_err("stale pin must fail before write");
    //     assert_eq!(err.code().as_str(), "E_RESUME_SUBGRAPH_DRIFT");

    let _ = engine;
    panic!(
        "red-phase: pinned_subgraph_cids + resume step 3 not yet present. \
         G3-A to land per plan §9.1 resume protocol."
    );
}

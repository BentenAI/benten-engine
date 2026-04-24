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
///
/// G11-A Wave 1: the 4-step resume protocol's step 3 (pinned-subgraph
/// CID drift check) landed in G3-A and is now exercised end-to-end via
/// the `testing_force_reregister_with_different_cid` helper — which
/// simulates a version bump between suspend and resume. The resume
/// path must observe the drift and surface
/// `E_RESUME_SUBGRAPH_DRIFT` BEFORE any cap re-check or write.
#[test]
fn inv_13_wait_resume_stale_pin_fires_resume_subgraph_drift() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Register a WAIT-bearing handler whose CID will be pinned inside the
    // persisted envelope.
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("pinned"))
        .expect("register");

    let outcome = engine
        .call_with_suspension("pinned", "run", benten_core::Node::empty())
        .expect("call_with_suspension");
    let handle = outcome
        .unwrap_suspended()
        .expect("WAIT-bearing handler must suspend");
    let bytes = engine.suspend_to_bytes(&handle).expect("suspend_to_bytes");

    // Between suspend and resume, "version-bump" the handler so its
    // registered CID no longer matches the pin inside the envelope.
    engine
        .testing_force_reregister_with_different_cid("pinned")
        .expect("force pin drift");

    let err = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("signal"))
        .expect_err("stale pin must fail before write");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::ResumeSubgraphDrift,
        "§9.11 row 5: pinned subgraph drift fires E_RESUME_SUBGRAPH_DRIFT"
    );
}

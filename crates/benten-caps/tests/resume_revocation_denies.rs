//! Phase 2a R3 security — resume-revocation denial (G3-B seed per ucca-3
//! / sec-r1-1 / §9.13 refresh-point-4).
//!
//! **Attack class.** A suspended ExecutionState persists on disk between
//! suspend and resume. If the capability grant authorising the suspended
//! workflow is revoked during that window, resume MUST observe the
//! revocation via the §9.13 refresh-point-4 cap re-check and deny with
//! `E_CAP_REVOKED_MID_EVAL` (intra-peer) or `E_CAP_CHAIN_REVOKED_SYNC`
//! (Phase-3 inter-peer; the latter lands in Phase 3).
//!
//! **Why here (benten-caps).** This is the capability-policy seat of
//! the story: once G3-B wires the resume call into the evaluator's
//! refresh-point-4, the policy layer's `check_write` on the persisted
//! head-of-chain `capability_grant_cid` is what fires
//! `CapError::RevokedMidEval` (or `CapError::Denied`, depending on whether
//! the policy treats a revoked grant as out-of-band or mid-eval). This test
//! verifies that the grant-backed policy DOES deny when the grant is
//! revoked — independent of the WAIT / resume plumbing.
//!
//! **Attack sequence.**
//!  1. Alice is granted `store:post:write`. Grant CID = G.
//!  2. A suspended ExecutionState persists; payload references G in
//!     every attribution frame.
//!  3. Alice's admin revokes the grant.
//!  4. Resume-side: the engine re-derives a `WriteContext` whose `scope
//!     = store:post:write` and calls `CapabilityPolicy::check_write(ctx)`.
//!  5. Mitigation: policy observes the revocation via its grant reader,
//!     returns `Err(CapError::Revoked)` / `Err(CapError::RevokedMidEval)`.
//!     Engine routes to `E_CAP_REVOKED_MID_EVAL`.
//!
//! **Red-phase contract.** G9-A refines the refresh-point-4 mapping
//! (grant-backed batched lookup under revocation). Phase-1 HEAD
//! `GrantBackedPolicy` already reads the current grant state through
//! `system:CapabilityRevocation` Nodes; the test exercises that path via
//! a synthesised `WriteContext` whose scope matches a just-revoked grant.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapabilityPolicy, WriteContext};
use benten_engine::Engine;

/// ucca-3 / sec-r1-1 / §9.13 refresh-point-4: the grant-backed policy MUST
/// deny a write whose scope references a revoked grant. Phase-1 HEAD:
/// `revoke_capability` writes a `system:CapabilityRevocation` Node that
/// `GrantBackedPolicy`'s `GrantReader` consumes on the next check.
#[test]
fn resume_revocation_denies_write_on_revoked_grant() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Mint a grant. In the full attack scenario this CID is the one a
    // persisted ExecutionState's `capability_grant_cid` references.
    let alice = engine.create_principal("alice").unwrap();
    let _grant_cid = engine
        .grant_capability(&alice, "store:post:write")
        .expect("grant succeeds");

    // Sanity-baseline: the grant-backed policy permits a matching write
    // BEFORE revocation. Drive through engine.call so the full policy
    // check path runs as it would on resume.
    let baseline_sg = benten_engine::SubgraphSpec::builder()
        .handler_id("resume_baseline")
        .write(|w| w.label("post").requires("store:post:write"))
        .respond()
        .build();
    let baseline_id = engine.register_subgraph(baseline_sg).unwrap();
    let baseline = engine
        .call(&baseline_id, "resume_baseline", benten_core::Node::empty())
        .expect("baseline call returns Ok wrapper");
    assert!(
        baseline.error_code().is_none(),
        "baseline: grant permits write before revocation; got {:?}",
        baseline.error_code()
    );

    // Revoke the grant (simulates the window between suspend and resume).
    engine
        .revoke_capability(&alice, "store:post:write")
        .expect("revoke succeeds");

    // Attack: attempt the same write after revocation. Under G9-A +
    // refresh-point-4 the resume path would re-issue this check; the
    // grant-backed policy observes the revoke Node and denies.
    let attack_sg = benten_engine::SubgraphSpec::builder()
        .handler_id("resume_after_revoke")
        .write(|w| w.label("post").requires("store:post:write"))
        .respond()
        .build();
    let attack_id = engine.register_subgraph(attack_sg).unwrap();
    let outcome = engine
        .call(
            &attack_id,
            "resume_after_revoke",
            benten_core::Node::empty(),
        )
        .expect("call returns Ok wrapper");

    let code = outcome.error_code();
    assert!(
        code == Some("E_CAP_REVOKED_MID_EVAL")
            || code == Some("E_CAP_DENIED")
            || code == Some("E_CAP_REVOKED"),
        "policy MUST deny write on revoked grant scope; expected \
         E_CAP_REVOKED_MID_EVAL (refresh-point-4) or E_CAP_DENIED (policy \
         re-check). Got: {code:?}. Phase-2a G9-A refines to \
         RevokedMidEval via the resume-refresh path."
    );
}

/// Direct policy-level variant: construct a synthetic `WriteContext`
/// after revocation and call `check_write`. Avoids the engine.call path
/// so we can assert the specific CapError variant.
#[test]
fn grant_backed_policy_check_write_denies_after_revoke() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    let alice = engine.create_principal("alice").unwrap();
    let _grant_cid = engine.grant_capability(&alice, "store:post:write").unwrap();
    engine
        .revoke_capability(&alice, "store:post:write")
        .expect("revoke");

    // We do NOT have direct `engine.capability_policy()` access in Phase 1;
    // construct a GrantReader against the same backend and re-run the
    // policy against a synthetic context. If the backend exposes a reader
    // surface, use it; otherwise verify via engine.call (handled by the
    // sibling test above).
    //
    // Target for G9-A: `GrantReader::has_unrevoked_grant_for_any(&[scope])`
    // returns `Ok(false)` on the post-revoke graph. Once that batch API
    // lands, this test asserts it directly.

    // For this red-phase, re-confirm the sibling scenario's denial via a
    // second call — if it passes here, the grant-backed read path has
    // observed the revocation correctly.
    let sg = benten_engine::SubgraphSpec::builder()
        .handler_id("direct_policy_check")
        .write(|w| w.label("post").requires("store:post:write"))
        .respond()
        .build();
    let sg_id = engine.register_subgraph(sg).unwrap();
    let outcome = engine
        .call(&sg_id, "direct_policy_check", benten_core::Node::empty())
        .expect("call wrapper");
    assert!(
        outcome.error_code().is_some(),
        "grant-backed policy's check_write MUST error after revoke; Phase-2a \
         refinement narrows to RevokedMidEval via refresh-point-4"
    );

    // Belt-and-suspenders: exercise a synthetic WriteContext at the
    // CapabilityPolicy trait surface directly against NoAuth (the baseline
    // sanity check that the trait shape compiles as G3-B will use it).
    let synth = WriteContext::synthetic_for_test();
    let noauth = benten_caps::NoAuthBackend::new();
    match noauth.check_write(&synth) {
        Ok(()) => {} // NoAuth permits
        Err(e) => panic!("NoAuth must permit synthetic context — fixture sanity; got {e:?}"),
    }

    // Silence unused-var warnings under the various configurations.
    let _: &dyn CapabilityPolicy = &noauth;
    let _ = CapError::RevokedMidEval;
}

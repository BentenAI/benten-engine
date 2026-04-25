//! Phase 2a R3 security — resume-protocol step 4 (atk-1 / sec-r1-1 / ucca-3).
//!
//! R4 qa-r4-10 cross-reference: R2 §4.5 lists this under the collective
//! filename `crates/benten-engine/tests/integration/resume_revocation_denies.rs`.
//! Phase-2a keeps the per-scenario file split (better organization); this
//! header cross-references to the R2 landscape so auditors can still locate
//! the test via the landscape's file-column.
//!
//! **Attack class.** Compromise #1 (Phase-1) named three TOCTOU refresh
//! points: transaction commit, CALL entry, ITERATE batch boundary. Phase 2a
//! §9.13 adds resume as the fourth refresh point: a suspended handler's
//! capability_grant_cid may be revoked while the state bytes sit on disk
//! (AI assistant multi-turn workflow, Garden approval queue). Without a
//! resume-time cap re-check, the resumed handler runs under stale authority.
//!
//! **Prerequisite.** Legitimate holder suspends a handler carrying a cap
//! grant. Between suspend and resume, the grant is revoked (via
//! `system:CapabilityRevocation` Node). The suspended state's persisted
//! `capability_grant_cid` still references the now-revoked grant.
//!
//! **Attack sequence.**
//!  1. Alice holds a grant for the `wait:resume` scope (the synthetic scope
//!     the resume-step-4 protocol consults — see
//!     `engine_wait::resume_from_bytes_inner` step 4). Alice registers a
//!     handler containing a WAIT, invokes it, drives to suspension.
//!  2. Alice persists the state via `engine.suspend_to_bytes(handle)`.
//!  3. A separate actor (admin) revokes Alice's grant.
//!  4. Alice calls `engine.resume_from_bytes_unauthenticated(bytes, signal)`.
//!  5. Mitigation: resume step 4 (§9.1) re-calls
//!     `CapabilityPolicy::check_write` with a synthesized `WriteContext`
//!     scoped to `wait:resume`; the revocation is observed; the engine
//!     surfaces `E_CAP_REVOKED_MID_EVAL` BEFORE the terminal-OK envelope
//!     would be produced.
//!
//! **Impact.** Without step-4, the resume would surface a terminal-OK
//! Outcome attributed to Alice's revoked grant — an audit-trail bypass.
//!
//! **Recommended mitigation.** Per §9.13 refresh point #4 +
//! `Engine::resume_from_bytes_*`, the 4-step resume protocol's step 4 calls
//! `CapabilityPolicy::check_write` with a synthesized `WriteContext` whose
//! `scope = "wait:resume"`. Any denial = `E_CAP_REVOKED_MID_EVAL`.
//!
//! Wave-3c R4b fix-pass: this test was previously a `#[ignore]`d red-phase
//! placeholder. G3-B landed `Engine::resume_from_bytes_unauthenticated`
//! with the step-4 cap re-check (see `engine_wait::resume_from_bytes_inner`
//! lines 492-511); G9-A landed the GrantBackedPolicy + revoke-aware
//! GrantReader. The end-to-end suspend → revoke → resume → deny path is
//! now exercised through the engine surface (the sibling
//! `benten-caps/tests/resume_revocation_denies.rs` only exercised the
//! synthetic-WriteContext policy layer).
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, SuspensionOutcome};
use benten_errors::ErrorCode;

/// ucca-3 / atk-1 refresh-point-4: resume from bytes whose persisted grant
/// has been revoked mid-window must fire `E_CAP_REVOKED_MID_EVAL` BEFORE
/// any side-effect.
#[test]
fn resume_with_revoked_grant_denies() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    // Mint a grant for the resume-protocol step-4 synthetic scope. The
    // engine's `resume_from_bytes_inner` synthesises a WriteContext with
    // `scope = "wait:resume"` (see `engine_wait.rs` ~line 500); the
    // grant-backed policy permits when an unrevoked grant exists for that
    // scope, denies otherwise.
    let alice = engine.create_principal("alice").unwrap();
    let _grant_cid = engine
        .grant_capability(&alice, "wait:resume")
        .expect("grant succeeds");

    // Suspend a WAIT handler. The persisted envelope's attribution chain
    // names alice; resume step 4 will re-derive a WriteContext for
    // `wait:resume` and consult the grant-backed policy.
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("revoke-flow"))
        .expect("register WAIT handler");

    let suspended = match engine
        .call_as_with_suspension("revoke-flow", "run", benten_core::Node::empty(), &alice)
        .expect("call_as_with_suspension")
    {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => panic!("WAIT handler must suspend"),
    };
    let bytes = engine
        .suspend_to_bytes(&suspended)
        .expect("suspend_to_bytes");

    // Sanity-baseline: BEFORE revocation, the same envelope's resume must
    // succeed — otherwise the post-revoke denial below could be a false
    // positive (always-deny independent of revocation state).
    let baseline = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("sig"))
        .expect("baseline: pre-revocation resume must succeed");
    assert!(
        baseline.is_ok_edge(),
        "baseline: unrevoked-grant resume must produce an OK edge; got {:?}",
        baseline.edge_taken()
    );

    // Revoke alice's grant — simulates the window between suspend and
    // resume during which an admin (or a Garden approval flow) flips
    // authority.
    engine
        .revoke_capability(&alice, "wait:resume")
        .expect("revoke succeeds");

    // Resume now: step 4 must observe the revocation and deny BEFORE the
    // terminal-OK envelope would be produced.
    let err = engine
        .resume_from_bytes_unauthenticated(&bytes, benten_core::Value::text("sig"))
        .expect_err("revoked grant must deny resume at step 4");
    assert_eq!(
        err.code(),
        ErrorCode::CapRevokedMidEval,
        "post-revoke resume must fire E_CAP_REVOKED_MID_EVAL via the §9.13 \
         refresh-point-4 path. Got: {err:?}"
    );
}

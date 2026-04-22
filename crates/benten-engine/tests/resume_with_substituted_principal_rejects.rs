//! Phase 2a R3 security — resume-protocol step 2 principal binding
//! (atk-1 / ucca-4 / sec-r1-1).
//!
//! **Attack class.** The ExecutionState payload carries a
//! `resumption_principal_cid` field — the principal identity that owns the
//! resume authority. Without this binding, any actor who obtains the state
//! bytes can call `resume_from_bytes` and execute the remainder of a
//! different actor's workflow under that actor's authority.
//!
//! **Prerequisite.** Attacker Eve obtains state bytes produced by Alice's
//! suspension. In Phase 2a this is any shared-storage path (backup, crash
//! dump); in Phase 3 it's any peer on the sync transport.
//!
//! **Attack sequence.**
//!  1. Alice suspends a multi-turn workflow. Payload:
//!     `resumption_principal_cid = alice_cid`.
//!  2. Eve obtains the bytes.
//!  3. Eve calls `engine.resume_from_bytes_as(bytes, signal, &eve_cid)`
//!     (the resume API's principal-bearing variant).
//!  4. Mitigation: step 2 of the §9.1 4-step resume protocol asserts
//!     `persisted.resumption_principal_cid == caller_cid` — mismatch =
//!     `E_RESUME_ACTOR_MISMATCH`.
//!
//! **Impact.** Phase-6 AI-assistant workflow hijacking: Eve completes
//! Alice's conversation under Alice's delegated grants. Phase-7 Garden
//! approval: Eve "resumes" Alice's vote.
//!
//! **Recommended mitigation.** `resume_from_bytes_as(bytes, signal,
//! claimed_principal)` checks `persisted.resumption_principal_cid ==
//! claimed_principal` by CID-equality (content-addressed) before step 3
//! (pinned subgraph CIDs) or step 4 (cap re-check).
//!
//! **Red-phase contract.** G3-A lands `resumption_principal_cid` on the
//! payload + `resume_from_bytes_as` API. Until then, `#[ignore]`d with a
//! pending pointer. Fixture setup (create_principal for both Alice and
//! Eve) compiles today.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// ucca-4 / atk-1: Eve calls resume on Alice's state. Resume step 2 must
/// fire `E_RESUME_ACTOR_MISMATCH` before any side-effect.
#[test]
#[ignore = "phase-2a-pending: resume_from_bytes_as + resumption_principal_cid field land in G3-A + G3-B per plan §9.1 step 2. Drop #[ignore] once the principal-bearing resume API is live."]
fn resume_with_substituted_principal_rejects() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    let alice_cid = engine.create_principal("alice").unwrap();
    let eve_cid = engine.create_principal("eve").unwrap();
    assert_ne!(
        alice_cid, eve_cid,
        "principals must hash to distinct CIDs — fixture sanity"
    );

    // Target API path (G3-A + G3-B):
    //
    //     let handler_id = register_wait_handler(&engine, "wait-handler");
    //     let suspended = engine
    //         .call_with_suspension_as(&handler_id, "run", Node::empty(), &alice_cid)
    //         .unwrap()
    //         .unwrap_suspended();
    //     let bytes = engine.suspend_to_bytes(suspended).unwrap();
    //
    //     // Eve tries to resume Alice's suspension.
    //     let outcome = engine.resume_from_bytes_as(bytes, signal_value(), &eve_cid);
    //     let err = outcome.expect_err("eve must not resume alice's suspension");
    //     assert_eq!(err.code().as_str(), "E_RESUME_ACTOR_MISMATCH");

    panic!(
        "red-phase: resume_from_bytes_as + resumption_principal_cid field \
         not yet present. G3-A + G3-B to land per plan §9.1 step 2."
    );
}

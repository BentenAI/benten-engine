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
//! **Mitigation in code.** `resume_from_bytes_as(bytes, signal,
//! claimed_principal)` checks `persisted.resumption_principal_cid ==
//! claimed_principal` by CID-equality (content-addressed) before step 3
//! (pinned subgraph CIDs) or step 4 (cap re-check). See
//! `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner`
//! step 2.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a). R6 fix-pass:
//! un-ignored once `resume_from_bytes_as` + `resumption_principal_cid`
//! landed (G3-A + G3-B). The §9.1 4-step protocol is now live and the
//! step-2 firing edge is asserted end-to-end.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SuspensionOutcome};
use benten_errors::ErrorCode;

/// ucca-4 / atk-1: Eve calls resume on Alice's state. Resume step 2 must
/// fire `E_RESUME_ACTOR_MISMATCH` before any side-effect.
#[test]
fn resume_with_substituted_principal_rejects() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Two distinct principals — alice owns the suspension; eve attempts
    // the unauthorized resume. `principal_cid` derives a deterministic
    // CID from the name so this test does not depend on any backing
    // identity store, only on CID-equality semantics in step 2.
    let alice_cid = benten_engine::testing::principal_cid("alice");
    let eve_cid = benten_engine::testing::principal_cid("eve");
    assert_ne!(
        alice_cid, eve_cid,
        "principals must hash to distinct CIDs — fixture sanity"
    );

    // Register a wait handler so `call_as_with_suspension` produces a
    // genuine suspended envelope rather than a Complete short-circuit.
    engine
        .register_subgraph(benten_engine::testing::minimal_wait_handler("atk1"))
        .expect("register wait handler");

    // Alice suspends a workflow. Step 0 of the protocol stamps
    // `resumption_principal_cid = alice_cid` into the envelope.
    let outcome = engine
        .call_as_with_suspension("atk1", "run", Node::empty(), &alice_cid)
        .expect("alice's call_with_suspension_as succeeds");
    let handle = match outcome {
        SuspensionOutcome::Suspended(h) => h,
        SuspensionOutcome::Complete(_) => {
            panic!("minimal_wait_handler must Suspend, not Complete")
        }
    };
    let bytes = engine
        .suspend_to_bytes(&handle)
        .expect("serialise alice's suspension");

    // Eve attempts to resume Alice's suspension under her own principal.
    // Step 2 of the resume protocol must fire E_RESUME_ACTOR_MISMATCH
    // before any side-effect.
    let err = engine
        .resume_from_bytes_as(&bytes, Value::text("attack-signal"), &eve_cid)
        .expect_err("eve must not resume alice's suspension");
    assert_eq!(
        err.code(),
        ErrorCode::ResumeActorMismatch,
        "resume-principal mismatch must fire E_RESUME_ACTOR_MISMATCH; got {err:?}"
    );

    // Sanity: alice CAN resume her own suspension (proves the mismatch
    // wasn't a categorical refuse-all and the bytes are well-formed).
    let alice_resume = engine
        .resume_from_bytes_as(&bytes, Value::text("legit-signal"), &alice_cid)
        .expect("alice's own resume must succeed");
    assert!(
        alice_resume.is_ok_edge(),
        "alice's own resume must route OK; got {alice_resume:?}"
    );
}

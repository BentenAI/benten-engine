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
//!  1. Alice holds `store:post:write`. Registers a handler containing a
//!     WAIT, invokes it, drives to suspension.
//!  2. Alice persists the state via `engine.suspend_to_bytes(handle)`.
//!  3. A separate actor (admin) revokes Alice's grant.
//!  4. Alice calls `engine.resume_from_bytes(bytes, signal)`.
//!  5. Mitigation: resume step 4 (§9.1) re-calls
//!     `CapabilityPolicy::check_write` with the persisted
//!     head-of-chain `capability_grant_cid`; the revocation is observed;
//!     `E_CAP_REVOKED_MID_EVAL` fires BEFORE any write.
//!
//! **Impact.** Write executes under revoked authority — capability-bound
//! audit-trail attributes the write to Alice's revoked grant, which the
//! audit log treats as authorised.
//!
//! **Recommended mitigation.** Per §9.13 refresh point #4 +
//! `Engine::resume_from_bytes`, the 4-step resume protocol's step 4 calls
//! `CapabilityPolicy::check_write` with a freshly-derived `WriteContext`
//! using the persisted head-of-chain `capability_grant_cid`. Any denial =
//! `E_CAP_REVOKED_MID_EVAL`.
//!
//! **Red-phase contract.** G3-B lands the resume API + the refresh-point-4
//! wiring. Until then, this test stays `#[ignore]`d; the body documents
//! the target assertion and the pre-existing revoke API still compiles.
//!
//! R3 writer: `rust-test-writer-security` (Phase 2a).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::Engine;

/// ucca-3 / atk-1 refresh-point-4: resume from bytes whose persisted
/// capability_grant_cid has been revoked must fire `E_CAP_REVOKED_MID_EVAL`
/// BEFORE any side-effect.
#[test]
#[ignore = "phase-2a-pending: suspend/resume API + refresh-point-4 wiring land in G3-B per plan §9.13. Drop #[ignore] once resume_from_bytes consults CapabilityPolicy::check_write."]
fn resume_with_revoked_grant_denies() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .unwrap();

    let alice = engine.create_principal("alice").unwrap();
    let grant_cid = engine.grant_capability(&alice, "store:post:write").unwrap();

    // Target API path (G3-B):
    //
    //     let sg = SubgraphSpec::builder()
    //         .handler_id("wait-before-write")
    //         .wait(|w| w.signal("external:signal"))
    //         .write(|w| w.label("post").requires("store:post:write"))
    //         .respond()
    //         .build();
    //     let handler_id = engine.register_subgraph(sg).unwrap();
    //     let suspended = engine
    //         .call_with_suspension(&handler_id, "run", Node::empty())
    //         .unwrap()
    //         .unwrap_suspended();
    //     let bytes = engine.suspend_to_bytes(suspended).unwrap();
    //
    //     // Revoke the grant between suspend and resume.
    //     engine.revoke_capability(&grant_cid).unwrap();
    //
    //     let outcome = engine.resume_from_bytes(bytes, signal_value());
    //     let err = outcome.expect_err("revoked grant must deny resume");
    //     assert_eq!(err.code().as_str(), "E_CAP_REVOKED_MID_EVAL");
    //
    // Sanity on currently-available APIs: the grant CID is well-formed and
    // the revoke path compiles (revocation API exists in Phase 1).
    assert!(
        grant_cid.to_string().starts_with("bafy"),
        "grant CID shape: {grant_cid}"
    );

    panic!(
        "red-phase: Engine::resume_from_bytes + refresh-point-4 cap \
         re-verification not yet present. G3-B to land; see §9.13."
    );
}

//! Phase-4-Foundation R4-FP-1 — T4 regression-guard: user-initiated
//! admin UI write succeeds (defense isn't over-strict).
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-1 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T4
//! test-pin plan ("Regression-guard").
//!
//! ## What this pin establishes
//!
//! The T4 chain-validator defense (siblings:
//! `admin_ui_v0_background_write_must_trace_to_user_root.rs`,
//! `admin_ui_did_cannot_mint_root_grant.rs`,
//! `cap_policy_chain_validation_at_write_boundary.rs`) must reject
//! writes WITHOUT user-root chains. This pin asserts the same flow
//! with a VALID user-initiated chain SUCCEEDS — confirming the defense
//! doesn't over-fire and break legitimate admin UI activity.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer over-strict — rejects every admin-UI-DID write
//! regardless of chain. User can't use admin UI v0 (no legitimate path
//! to write). This regression-guard pins the OK arm; pairs with the
//! deny pins to form the boundary-condition test set per pim-2 §3.6b
//! per-finding granularity.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G24-B-FP wires user-initiated path verification; un-ignore at G24-B-FP landing. Pin source: r4-triage §1 r4-tc-1 + threat-model §T4 regression-guard."]
fn admin_ui_v0_user_initiated_write_with_user_root_chain_succeeds() {
    // G24-B-FP wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let user_did = harness.user_did();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // User-initiated action: user-DID mints a root grant for the
    //   // store:notes:write scope; delegates via UCAN to admin-UI-DID;
    //   // admin UI dispatches write under the chain.
    //   let user_root_grant = harness.mint_root_grant_as_user(
    //       /* audience */ admin_ui_did.clone(),
    //       /* scope */ "store:notes:write",
    //   ).unwrap();
    //   let user_rooted_chain = harness.build_chain_from(
    //       &user_root_grant,
    //       admin_ui_did.clone(),
    //   );
    //
    //   // Dispatch user-initiated write through call_as; chain validator
    //   // fires; chain traces to user-root via user_root_grant; write
    //   // succeeds.
    //   let result = harness.dispatch_user_initiated_write(
    //       admin_ui_did.clone(),
    //       "store:notes:write",
    //       vec![0u8; 32],
    //       user_rooted_chain,
    //   );
    //
    //   // LOAD-BEARING boundary: defense isn't over-strict. User-root
    //   // chain must trace cleanly; write succeeds.
    //   let cid = result.expect(
    //       "T4 regression-guard: user-initiated write with valid \
    //        user-root chain MUST succeed — defense must NOT over-fire \
    //        and block legitimate admin UI activity"
    //   );
    //   assert!(!cid.as_bytes().is_empty(),
    //       "Successful write must return non-empty CID");
    //
    //   // Verify the write committed (audit log shows user-initiated
    //   // outcome — NOT denied):
    //   let audit = harness.audit_log_since_last_dispatch();
    //   assert!(
    //       audit.iter().any(|r| r.principal == admin_ui_did
    //           && r.outcome.is_committed()
    //           && r.chain_root == user_did),
    //       "T4 regression-guard: audit log MUST record user-rooted \
    //        admin-UI-DID write as COMMITTED with user-DID as chain root"
    //   );
    //
    // OBSERVABLE consequence: boundary verified — defense is structural,
    // not over-strict; pair with deny pins establishes full T4 closure.
    unimplemented!(
        "G24-B-FP wires user-initiated-write-succeeds regression-guard \
         (T4 OK arm). Substantive end-to-end: real mint + delegation + \
         dispatch + audit-log COMMITTED check."
    );
}

//! Phase-4-Foundation R4-FP-1 — T7 LOAD-BEARING pin: admin UI v0
//! private namespace isolated from other plugins.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md` §T7
//! ("Per-plugin private-namespace integrity") + CLAUDE.md baked-in #18
//! private-namespaces section.
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #18 (private namespaces): admin UI v0's
//! writes to `admin-ui-private` go to a DID-scoped namespace; namespace
//! cap is held by admin-UI-DID with `shares: none`. Engine refuses to
//! issue cross-plugin caps for private namespaces — gives plugins a
//! sovereign space without breaking cross-plugin sharing semantics.
//!
//! End-to-end LOAD-BEARING per threat-model §T7 test-pin plan: an
//! "other plugin" attempts to gain access to admin-UI-DID's
//! `private:admin-ui-private:*` scope. Cap-policy MUST refuse.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires the unit-level scope-prefix canonicalization (T7
//! defense step 3) but forgets the structural delegation refusal at
//! grant-time. A malicious plugin issues itself a cap-delegation for
//! `private:admin-ui-private:auto_save`; cap-store admits it; cross-
//! plugin namespace escape succeeds at runtime. End-to-end pin catches
//! the gap that the unit-level pin alone misses.
//!
//! Per pim-2 §3.6b: substantive end-to-end production-runtime arm —
//! real engine + 2 plugin DIDs + delegation attempt + observable
//! consequence (write under hostile plugin DID is rejected at scope-
//! resolution).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G24-A wires private-namespace isolation end-to-end; un-ignore at G24-A landing. Pin source: r4-triage §2 r4-tc-5 + threat-model §T7 LOAD-BEARING."]
fn admin_ui_v0_private_namespace_isolated_from_other_plugins_end_to_end() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let user_did = harness.user_did();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //   let hostile_plugin_did = harness.mint_plugin_did("hostile-plugin");
    //
    //   // User installs admin UI v0 with shares=none for its private NS.
    //   harness.install_plugin_with_user_consent(
    //       admin_ui_did.clone(),
    //       common::admin_ui_v0_manifest_with_private_namespace(),
    //   ).unwrap();
    //
    //   // Admin UI writes to its private namespace — legitimate.
    //   let legit_cid = harness
    //       .dispatch_user_initiated_write(
    //           admin_ui_did.clone(),
    //           "private:admin-ui-private:auto_save",
    //           vec![0u8; 32],
    //           harness.user_rooted_chain_for(&admin_ui_did, "private:admin-ui-private:*"),
    //       )
    //       .expect("Admin UI must be able to write to its own private namespace");
    //
    //   // Hostile plugin attempts to issue itself a cap-delegation for
    //   // the same private NS scope. Cap-store + cap-policy MUST refuse
    //   // the delegation at issue-time per T7 defense step 2.
    //   let delegate_attempt = harness.delegate_cap_from(
    //       /* delegator */ admin_ui_did.clone(),
    //       /* audience */ hostile_plugin_did.clone(),
    //       /* scope */ "private:admin-ui-private:auto_save",
    //   );
    //   let err = delegate_attempt.expect_err(
    //       "T7 LOAD-BEARING: cross-plugin delegation for private NS \
    //        MUST be refused at issue-time — `shares: none` is the \
    //        structural defense"
    //   );
    //   assert!(
    //       matches!(err.code(), ErrorCode::E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN),
    //       "T7: must surface typed E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN; got {:?}",
    //       err.code()
    //   );
    //
    //   // Defense-in-depth: even with a (forged) chain, hostile plugin
    //   // attempts a direct write to admin-ui-private — cap-policy
    //   // backend's prefix canonicalization rejects (T7 defense step 3):
    //   let write_attempt = harness.dispatch_local_write_via_call_as(
    //       hostile_plugin_did.clone(),
    //       "private:admin-ui-private:auto_save",
    //       vec![0u8; 4],
    //       harness.forged_chain_for(&hostile_plugin_did),
    //   );
    //   let err2 = write_attempt.expect_err(
    //       "T7 step 3: hostile plugin write to admin-ui-private MUST \
    //        be REJECTED at scope-prefix canonicalization"
    //   );
    //   assert!(
    //       matches!(err2.code(),
    //           ErrorCode::E_CAP_DENIED
    //           | ErrorCode::E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN
    //       ),
    //       "T7: must surface typed denial for cross-plugin private NS \
    //        write attempt; got {:?}",
    //       err2.code()
    //   );
    //
    //   // Observable: admin UI's legitimate write committed; hostile
    //   // plugin's attempt did NOT write the same key.
    //   let stored = harness.read_private_namespace(
    //       &admin_ui_did, "auto_save"
    //   ).unwrap();
    //   assert_eq!(stored.cid(), legit_cid,
    //       "Hostile plugin must NOT have overwritten the admin-UI private NS key");
    //
    // OBSERVABLE consequence: cross-plugin namespace escape blocked at
    // both delegation-time + scope-resolution boundaries.
    unimplemented!(
        "G24-A wires admin UI v0 private-namespace isolation end-to-end \
         (T7 LOAD-BEARING). Substantive: real 2-plugin install + \
         delegation refusal + scope-prefix enforcement + observable \
         data-isolation check."
    );
}

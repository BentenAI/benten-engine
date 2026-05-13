//! Phase-4-Foundation R4-FP-1 — T10-uninstall (a) pin: plugin uninstall
//! revokes caps the plugin held.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-uninstall defense step 3(a) + plan §3 G24-D-FP-1.
//!
//! ## What this pin establishes
//!
//! Per pim-2-amendment §3.6b sub-rule 4 per-finding granularity:
//! T10-uninstall (a) = revoke every cap admin-UI-DID held. This is the
//! EXPLICIT per-finding pin (the existing
//! `plugin_uninstall_revokes_all_delegated_caps.rs` covers (a) + (b)
//! umbrella but doesn't establish the (a) sub-arm with FAILS-IF-NO-OP
//! observability per sub-rule 4).
//!
//! Couples to §13.11 UCAN revocation observance closure (PR #199;
//! `Engine::revoke_capability_by_grant_cid` already at HEAD).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires the cascade arm (b: revoke delegated TO others)
//! but forgets the direct arm (a: revoke caps the plugin HELD). Plugin
//! is uninstalled; its own cap-store entries remain valid; a re-
//! installed plugin (or stale code path) can still authenticate.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_uninstall_revokes_every_cap_with_audience_equals_plugin_did() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D-FP-1 wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::uninstall_plugin;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let plugin_did = stub_plugin_did();
    //   let user_did = stub_user_did();
    //
    //   // Install: user-DID issues 3 distinct grants to plugin-DID
    //   // (different scopes).
    //   common::manifest_fixtures::install_test_plugin(
    //       &mut engine, plugin_did.clone(),
    //       vec!["store:notes:read", "store:notes:write", "host:time:now"]
    //   ).unwrap();
    //
    //   // Baseline: all 3 grants active.
    //   let baseline = engine.cap_store().active_grants_for_audience(&plugin_did);
    //   assert_eq!(baseline.len(), 3,
    //       "Baseline: 3 grants must be active pre-uninstall");
    //
    //   // Uninstall.
    //   uninstall_plugin(&plugin_did, &mut engine).unwrap();
    //
    //   // T10-uninstall (a) LOAD-BEARING: every grant with
    //   // audience=plugin_did MUST be revoked.
    //   let after = engine.cap_store().active_grants_for_audience(&plugin_did);
    //   assert!(
    //       after.is_empty(),
    //       "T10-uninstall (a): plugin-DID's held caps MUST be revoked; \
    //        {} caps still active in cap-store",
    //       after.len()
    //   );
    //
    //   // Defense-in-depth: revocation goes through the typed
    //   // Engine::revoke_capability_by_grant_cid surface (PR #199)
    //   // — observe the revocation log:
    //   let revoke_log = engine.cap_store().revocation_log_since_test_start();
    //   assert_eq!(revoke_log.len(), 3,
    //       "T10-uninstall (a): all 3 grants must surface in \
    //        revocation log (not skipped via cap-store backdoor)");
    //   assert!(
    //       revoke_log.iter().all(|r| r.audience == plugin_did),
    //       "T10-uninstall (a): all revocations must be for plugin-DID; \
    //        contamination of other plugins' caps is a separate bug"
    //   );
    //
    // OBSERVABLE consequence: typed per-grant revocation via PR #199
    // surface; revocation log auditable.
    panic!(
        "RED-PHASE: G24-D-FP-1 must wire direct cap-revocation in \
         uninstall_plugin (T10-uninstall (a) per-finding-granular pin). \
         Substantive: 3 distinct grants + uninstall + zero-remaining + \
         revocation log via PR #199 surface."
    );
}

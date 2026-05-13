//! Phase-4-Foundation R4-FP-1 — T10-uninstall (b) pin: plugin uninstall
//! cascade-revokes caps the plugin delegated to OTHER plugins.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-uninstall defense step 3(b) + plan §3 G24-D-FP-1 + CLAUDE.md
//! baked-in #18 Layer 3 cascade discipline.
//!
//! ## What this pin establishes
//!
//! Per pim-2-amendment §3.6b sub-rule 4 per-finding granularity:
//! T10-uninstall (b) = cascade-revoke every cap the plugin delegated
//! to OTHER plugins. Without cascade, downstream plugins retain stale
//! caps after the source plugin is uninstalled.
//!
//! Distinct from (a) "revoke caps plugin HELD" — (b) walks
//! grants WHERE issuer = plugin_did, cascade.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires (a) revoke held caps but skips cascade. Plugin A
//! is uninstalled; A had delegated caps to plugin B; B's downstream
//! caps remain valid. Hostile re-install path could exploit stale
//! delegations. Per CLAUDE.md baked-in #18 Layer 3: the manifest
//! envelope's transitivity guarantee requires cascade.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[test]
#[ignore = "RED-PHASE: G24-D-FP-1 wires cascade-revocation in uninstall_plugin; un-ignore at G24-D-FP-1 landing. Pin source: r4-triage §1 r4-tc-3 + threat-model §T10-uninstall (b) per-finding-granular."]
fn plugin_uninstall_cascade_revokes_caps_delegated_to_other_plugins() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D-FP-1 wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::uninstall_plugin;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let plugin_a = stub_plugin_did();
    //   let plugin_b = common::manifest_fixtures::stub_plugin_did_b();
    //   let user_did = stub_user_did();
    //
    //   // Install plugin A with shares=any + plugin B.
    //   common::manifest_fixtures::install_test_plugin_with_shares_any(
    //       &mut engine, plugin_a.clone(),
    //       vec!["store:notes:read"]
    //   ).unwrap();
    //   common::manifest_fixtures::install_test_plugin(
    //       &mut engine, plugin_b.clone(),
    //       vec![],
    //   ).unwrap();
    //
    //   // Plugin A delegates cap to plugin B (within envelope per shares=any).
    //   let delegation_cid = engine.delegate_cap(
    //       /* delegator */ plugin_a.clone(),
    //       /* audience */ plugin_b.clone(),
    //       /* scope */ "store:notes:read",
    //   ).unwrap();
    //
    //   // Baseline: B has the delegated cap.
    //   let b_baseline = engine.cap_store().active_grants_for_audience(&plugin_b);
    //   assert!(
    //       b_baseline.iter().any(|g| g.issuer == plugin_a),
    //       "Baseline: plugin B must have an active grant issued by plugin A"
    //   );
    //
    //   // Uninstall plugin A.
    //   uninstall_plugin(&plugin_a, &mut engine).unwrap();
    //
    //   // T10-uninstall (b): cap A→B MUST be cascade-revoked.
    //   let b_after = engine.cap_store().active_grants_for_audience(&plugin_b);
    //   assert!(
    //       !b_after.iter().any(|g| g.issuer == plugin_a),
    //       "T10-uninstall (b): cap A→B MUST be cascade-revoked when A \
    //        is uninstalled; {} grants from A still active for B",
    //       b_after.iter().filter(|g| g.issuer == plugin_a).count()
    //   );
    //
    //   // Defense-in-depth: revocation log shows cascade explicitly
    //   // (audience = plugin_b, NOT plugin_a):
    //   let revoke_log = engine.cap_store().revocation_log_since_test_start();
    //   assert!(
    //       revoke_log.iter().any(|r|
    //           r.grant_cid == delegation_cid
    //           && r.audience == plugin_b
    //           && r.cascade_source == Some(plugin_a.clone())),
    //       "T10-uninstall (b): revocation log MUST tag the cascade \
    //        source for forensic auditability"
    //   );
    //
    // OBSERVABLE consequence: Layer 3 transitivity guarantee preserved;
    // downstream plugins lose delegated caps cleanly on uninstall.
    panic!(
        "RED-PHASE: G24-D-FP-1 must wire cascade-revocation in \
         uninstall_plugin (T10-uninstall (b) per-finding-granular pin). \
         Substantive: 2 plugins + delegation A→B + uninstall A + \
         cascade-revocation assertion + forensic cascade-source log tag."
    );
}

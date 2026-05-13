//! Phase-4-Foundation R4-FP-1 — T10-uninstall (c) LOAD-BEARING pin:
//! plugin uninstall terminates subscriptions.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-uninstall defense step 3(c) + plan §3 G24-D-FP-1.
//!
//! ## What this pin establishes
//!
//! Per pim-2-amendment §3.6b sub-rule 4 per-finding granularity:
//! T10-uninstall has 3 sub-arms (a) revoke held caps, (b) cascade-
//! revoke delegated caps, (c) terminate live subscriptions. This
//! pin is the (c) arm — LOAD-BEARING per threat-model §T10 test-pin
//! plan + couples to T12 cross-process amplification defense.
//!
//! New seam (per threat-model §T10 defense step 3):
//! `crates/benten-platform-foundation/src/plugin_lifecycle.rs::
//! uninstall_plugin` MUST terminate every live subscription whose
//! subscriber DID was this plugin.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires uninstall to revoke caps but forgets to terminate
//! subscriptions. Subscription callback continues delivering events
//! post-uninstall; plugin-DID receives change-stream events for data
//! it no longer has cap to read. Cross-process amplification under
//! D-4F-4 (a) thin-client makes this HIGH severity per threat-model
//! §T12.

#![allow(clippy::unwrap_used)]

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[test]
#[ignore = "RED-PHASE: G24-D-FP-1 wires subscription termination in uninstall_plugin; un-ignore at G24-D-FP-1 landing. Pin source: r4-triage §1 r4-tc-3 + threat-model §T10-uninstall (c) LOAD-BEARING."]
fn plugin_uninstall_terminates_active_subscriptions_for_subscriber_did() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // G24-D-FP-1 wave wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_lifecycle::uninstall_plugin;
    //
    //   let mut engine = common::manifest_fixtures::test_engine_with_user_did();
    //   let plugin_did = stub_plugin_did();
    //
    //   // Install plugin with delegated cap + register an active
    //   // subscription under plugin_did.
    //   common::manifest_fixtures::install_test_plugin(
    //       &mut engine, plugin_did.clone(), vec!["store:notes:read"]
    //   ).unwrap();
    //   let sub_handle = engine
    //       .subscribe_change_events_as(&plugin_did, "store:notes:read")
    //       .unwrap();
    //
    //   // Baseline: subscription delivers event from a write.
    //   engine.write_as(
    //       stub_user_did(), "store:notes:write", vec![1, 2, 3]
    //   ).unwrap();
    //   let baseline = engine.collect_events(&sub_handle);
    //   assert!(
    //       !baseline.is_empty(),
    //       "Baseline: subscription delivers events pre-uninstall"
    //   );
    //
    //   // Uninstall: per T10-uninstall (c), subscriptions MUST be
    //   // terminated. Future surface signature:
    //   //   plugin_lifecycle::uninstall_plugin(plugin_did, &mut engine)
    //   //     -> Result
    //   uninstall_plugin(&plugin_did, &mut engine).unwrap();
    //
    //   // Post-uninstall write: subscription must NOT deliver.
    //   engine.write_as(
    //       stub_user_did(), "store:notes:write", vec![4, 5, 6]
    //   ).unwrap();
    //   let post = engine.collect_events(&sub_handle);
    //
    //   // LOAD-BEARING: zero events post-uninstall.
    //   assert!(
    //       post.is_empty(),
    //       "T10-uninstall (c) LOAD-BEARING: subscriptions MUST be \
    //        terminated on uninstall; got {} stale events — cross-\
    //        process amplification (T12 defense compromised)",
    //       post.len()
    //   );
    //
    //   // Defense-in-depth: subscription registry is empty for the
    //   // uninstalled plugin-DID:
    //   let active = engine.active_subscriptions_for(&plugin_did);
    //   assert!(active.is_empty(),
    //       "T10-uninstall (c): subscription registry must be empty \
    //        for uninstalled plugin-DID");
    //
    // OBSERVABLE consequence: T10-uninstall (c) + T12 cross-process
    // amplification defense both verified.
    panic!(
        "RED-PHASE: G24-D-FP-1 must wire subscription termination in \
         uninstall_plugin (T10-uninstall (c) LOAD-BEARING). \
         Substantive: subscribe + baseline + uninstall + post-write + \
         zero-event assertion + subscription registry empty."
    );
}

//! Phase-4-Foundation R4-FP-1 — T10 LOAD-BEARING pin: admin UI v0
//! uninstall revokes active-session subscriptions.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §2 MAJOR row
//! r4-tc-5 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10 ("Module ecosystem tooling attack surface") test-pin plan
//! ("Load-bearing end-to-end").
//!
//! ## What this pin establishes
//!
//! End-to-end LOAD-BEARING: admin UI v0 is the first plugin / default
//! module per CLAUDE.md baked-in #18. When admin UI v0 is uninstalled:
//! (a) every cap admin-UI-DID held is revoked; (b) every cap admin-UI-
//! DID delegated to OTHER plugins is cascade-revoked; (c) every LIVE
//! subscription whose subscriber DID was admin-UI-DID is terminated.
//!
//! Pair with the unit-level T10 pins in
//! `crates/benten-platform-foundation/tests/`:
//! - `plugin_uninstall_terminates_subscriptions.rs` (T10-uninstall c)
//! - `plugin_uninstall_revokes_held_caps.rs` (T10-uninstall a)
//! - `plugin_uninstall_cascade_revokes_delegated_caps.rs` (T10-uninstall b)
//!
//! per pim-2 §3.6b sub-rule 4 per-finding granularity (T10-uninstall
//! a + b + c collapsed into one file was the BLOCKER r4-tc-3 cause;
//! this end-to-end is the umbrella over the per-finding splits).
//!
//! Couples to §13.11 UCAN revocation observance closure (PR #199 minimal
//! seam-fix; already at HEAD via `Engine::revoke_capability_by_grant_cid`).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires uninstall to revoke caps from cap-store but
//! forgets to terminate live subscriptions. Subscription callback
//! continues firing post-uninstall; admin-UI-DID receives change-
//! stream events for data it no longer has cap to read. Cross-process
//! amplification under D-4F-4 (a) thin-client makes this a critical
//! HIGH gap per threat-model §T12.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "RED-PHASE: G24-D-FP-1 wires uninstall_plugin cascade end-to-end; un-ignore at G24-D-FP-1 landing. Pin source: r4-triage §2 r4-tc-5 + threat-model §T10 LOAD-BEARING end-to-end."]
fn admin_ui_v0_uninstall_revokes_active_session_subscriptions_end_to_end() {
    // G24-D-FP-1 wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let user_did = harness.user_did();
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //
    //   // Install admin UI v0 with user consent + delegated caps +
    //   // some active subscriptions for live UI updates.
    //   harness.install_plugin_with_user_consent(
    //       admin_ui_did.clone(),
    //       common::admin_ui_v0_manifest_with_subscriptions(),
    //   ).unwrap();
    //
    //   let sub_handle = harness
    //       .subscribe_change_events_as(admin_ui_did.clone(), "store:notes:read")
    //       .expect("subscription must register while plugin is installed");
    //
    //   // Validate baseline: subscription delivers an event from a
    //   // write to the subscribed scope.
    //   harness.dispatch_user_initiated_write(
    //       user_did.clone(),
    //       "store:notes:write",
    //       vec![1, 2, 3],
    //       harness.user_root_chain_for("store:notes:write"),
    //   ).unwrap();
    //   let baseline_events = harness.collect_events(&sub_handle);
    //   assert!(
    //       !baseline_events.is_empty(),
    //       "Baseline: subscription must deliver event before uninstall"
    //   );
    //
    //   // Uninstall admin UI v0.
    //   harness.uninstall_plugin(admin_ui_did.clone()).unwrap();
    //
    //   // LOAD-BEARING: all 3 cascade arms (T10-uninstall a + b + c).
    //
    //   // (a) Caps admin-UI-DID held: all revoked.
    //   let held_after = harness.cap_store().active_caps_for(&admin_ui_did);
    //   assert!(
    //       held_after.is_empty(),
    //       "T10-uninstall (a): admin-UI-DID's held caps MUST be revoked; \
    //        {} caps still active",
    //       held_after.len()
    //   );
    //
    //   // (b) Caps admin-UI-DID delegated to OTHERS: all cascade-revoked.
    //   let issued_after = harness.cap_store().active_caps_issued_by(&admin_ui_did);
    //   assert!(
    //       issued_after.is_empty(),
    //       "T10-uninstall (b): admin-UI-DID's downstream delegations \
    //        MUST be cascade-revoked; {} grants still active",
    //       issued_after.len()
    //   );
    //
    //   // (c) Active subscriptions: terminated. No event delivery
    //   // post-uninstall (LOAD-BEARING — defense-in-depth with T12).
    //   harness.dispatch_user_initiated_write(
    //       user_did.clone(),
    //       "store:notes:write",
    //       vec![4, 5, 6],
    //       harness.user_root_chain_for("store:notes:write"),
    //   ).unwrap();
    //   let post_uninstall_events = harness.collect_events(&sub_handle);
    //   assert!(
    //       post_uninstall_events.is_empty(),
    //       "T10-uninstall (c) LOAD-BEARING: subscriptions MUST be \
    //        terminated on uninstall; got {} stale events post-uninstall \
    //        — cross-process amplification gap (T12 defense compromised)",
    //       post_uninstall_events.len()
    //   );
    //
    //   // PluginUninstalled change-event emitted for downstream observers:
    //   let plugin_events = harness.captured_plugin_lifecycle_events();
    //   assert!(
    //       plugin_events.iter().any(|e| e.is_uninstalled(&admin_ui_did)),
    //       "T10-uninstall: PluginUninstalled change-event MUST be \
    //        emitted for downstream observers"
    //   );
    //
    // OBSERVABLE consequence: complete cascade-revoke + subscription
    // termination; defense-in-depth with T12 amplification gap.
    unimplemented!(
        "G24-D-FP-1 wires admin UI v0 uninstall end-to-end \
         (T10 LOAD-BEARING). Substantive: real install + subscribe + \
         write-roundtrip baseline + uninstall + all 3 cascade arms \
         (caps held + delegated + subscriptions) + PluginUninstalled \
         event emission."
    );
}

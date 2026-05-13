//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for dogfood path (e):
//! install admin UI v0 on a 2nd device via signed manifest in ≤3 clicks;
//! user can decline a single cap.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 15 (LOAD-BEARING §3.6f substantive); closes ux-r1-1 + ux-r1-2
//! (install-flow UX + per-cap consent granularity).
//!
//! ## Per pim-18 §3.6f LOAD-BEARING substantive shape
//!
//! Production-runtime arms:
//! 1. **Real signed `PluginManifest` consumed** — content-addressed
//!    via `compute_content_cid`; peer-DID signature verified.
//! 2. **User-DID signs install record** on 2nd device — per D-4F-12
//!    user-as-source model (CLAUDE.md baked-in #18 implementation
//!    refinements).
//! 3. **Per-cap decline path works** — user un-checks one `requires`
//!    entry; install proceeds with reduced cap-set; declined cap is
//!    NOT granted (verified via cap-policy check after install).
//! 4. **Install reachable in ≤3 clicks** per ux-r1-2 acceptance.
//! 5. **Replay-once-installed** — admin UI on 2nd device dispatches
//!    a workflow created on 1st device → trace matches.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A + G24-D + G24-F wire this; depends on Family F3 PluginManifest FULL (wave-7). Pin source: r2-test-landscape.md §2.6 row 15 + ux-r1-2. LOAD-BEARING per pim-18 §3.6f: real signed manifest + user-DID-signed install record + per-cap decline + ≤3 clicks + post-install replay. Would FAIL if install flow shape-only-shaped."]
fn dogfood_path_e_install_admin_ui_on_2nd_device_ux_acceptance() {
    // G24-A + G24-D + G24-F wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_manifest::PluginManifest;
    //
    //   // Spawn 2-device harness; admin UI installed on device A only:
    //   let harness_a = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let harness_b = harness_a.spawn_blank_second_device();
    //
    //   // (1) Author + sign a real admin UI manifest (would normally
    //   // ship pre-bundled; here we author it inline for the test):
    //   let manifest: PluginManifest = harness_a.export_admin_ui_v0_manifest();
    //   let manifest_cid = manifest.compute_content_cid();
    //   // Peer-DID signature is verifiable independent of install
    //   // record per D-4F-12:
    //   assert!(
    //       manifest.verify_peer_signature().is_ok(),
    //       "Manifest peer-DID signature MUST verify before install"
    //   );
    //
    //   // (4) UX-acceptance click count per ux-r1-2:
    //   let click_recorder = harness_b.start_click_recording();
    //   let install_flow = harness_b
    //       .navigate_to_plugin_install()
    //       .open_manifest_via_atrium_share(manifest_cid);
    //
    //   // (3) Per-cap decline path — user un-checks 'host:notifications:emit':
    //   install_flow.uncheck_cap("host:notifications:emit");
    //   let install_record_cid = install_flow.consent_and_install().unwrap();
    //   let click_count = click_recorder.stop();
    //   assert!(
    //       click_count <= 3,
    //       "Dogfood path (e): install reachable in ≤3 clicks per \
    //        ux-r1-2; saw {}",
    //       click_count,
    //   );
    //
    //   // (2) User-DID signs install record on device B per D-4F-12:
    //   let install_record = harness_b.load_install_record(&install_record_cid);
    //   assert_eq!(
    //       install_record.consenting_user_did,
    //       harness_b.user_did(),
    //       "Install record consenting_user_did MUST equal device B's \
    //        user-DID (user-as-source per D-4F-12)"
    //   );
    //   assert!(
    //       harness_b.verify_user_signature_on_install_record(
    //           &install_record
    //       ).is_ok(),
    //       "Install record MUST carry valid user-DID signature on \
    //        device B per CLAUDE.md baked-in #18 Layer 2"
    //   );
    //
    //   // (3 cont.) Declined cap NOT granted — try the action; expect denial:
    //   let admin_ui_did_on_b = harness_b.admin_ui_plugin_did();
    //   let emit_result = harness_b.cap_policy_check(
    //       &admin_ui_did_on_b, "host:notifications:emit"
    //   );
    //   assert!(
    //       emit_result.is_err(),
    //       "Declined cap 'host:notifications:emit' MUST NOT be granted \
    //        to admin-UI-DID on device B per ux-r1-2; cap check \
    //        unexpectedly succeeded"
    //   );
    //
    //   // (5) Replay-once-installed — create workflow on A, dispatch on B:
    //   let workflow_cid = harness_a.create_workflow("created_on_a");
    //   let trace_a = harness_a.trace_capture(|h| {
    //       h.dispatch_workflow_by_cid(workflow_cid).unwrap()
    //   });
    //   harness_b.await_sync(workflow_cid, std::time::Duration::from_secs(5));
    //   let trace_b = harness_b.trace_capture(|h| {
    //       h.dispatch_workflow_by_cid(workflow_cid).unwrap()
    //   });
    //   assert_eq!(
    //       trace_a.canonical_event_bytes(),
    //       trace_b.canonical_event_bytes(),
    //       "Dogfood path (e): post-install replay on device B MUST \
    //        match device A trace per pim-18 §3.6f determinism arm"
    //   );
    //
    // OBSERVABLE consequence: install-on-2nd-device works under real
    // manifest schema + real Atrium share + per-cap consent UX.
    // Defends against the failure shape where install ships as a
    // shape-only consent prompt that doesn't actually narrow caps.
    unimplemented!(
        "G24-A + G24-D + G24-F wire dogfood path (e): install-on-2nd-device \
         with 5-arm production-runtime check (real signed manifest + \
         user-DID install record + per-cap decline + ≤3 clicks + \
         replay) per pim-18 §3.6f"
    );
}

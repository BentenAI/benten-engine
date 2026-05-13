//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for dogfood path (f):
//! install a non-admin plugin via the same manifest flow; user-DID signs
//! install record (D-4F-12).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 16 (LOAD-BEARING §3.6f substantive); closes ux-r1-1 + D-4F-12
//! (workflow ↔ plugin unification: same install flow for admin UI and
//! arbitrary plugins).
//!
//! ## Per pim-18 §3.6f LOAD-BEARING substantive shape
//!
//! Production-runtime arms:
//! 1. **Non-admin plugin uses identical manifest flow** — no special-case
//!    code path for "first plugin (admin UI)" vs. "subsequent plugins".
//! 2. **User-DID signs install record** for the 2nd plugin per D-4F-12.
//! 3. **Plugin loads + executes** post-install — invoke a plugin handler
//!    via `Engine::call_as` with the plugin-DID principal.
//! 4. **Per-plugin DID is distinct** from admin-UI-DID; plugin's
//!    `requires` envelope is enforced (cap check denies out-of-envelope
//!    invocation).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A + G24-D wires this; depends on Family F3 PluginManifest FULL (wave-7). Pin source: r2-test-landscape.md §2.6 row 16 + D-4F-12. LOAD-BEARING per pim-18 §3.6f: identical flow as admin UI + user-DID signs + plugin loads + per-plugin envelope enforced."]
fn dogfood_path_f_install_2nd_plugin_ux_acceptance() {
    // G24-A + G24-D wires this. Substantive shape:
    //
    //   use benten_platform_foundation::plugin_manifest::{
    //       PluginManifest, CapRequirement, SharesPolicy, SharesPolicyDefault,
    //   };
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // Author a non-admin plugin manifest (e.g., a journaling plugin):
    //   let journaling_manifest = PluginManifest {
    //       plugin_name: "journaling-v0".into(),
    //       content_cid: harness.author_test_plugin_content("journaling"),
    //       peer_did: harness.peer_did(),
    //       peer_signature: harness.peer_sign(&[0u8; 32]),
    //       requires: vec![
    //           CapRequirement { scope: "store:journal:read+write".into() },
    //       ],
    //       shares: SharesPolicy {
    //           default: SharesPolicyDefault::None,
    //           rules: None,
    //       },
    //       renderer_config: None,
    //       composes_plugins: None,
    //       accepts_content: None,
    //       requires_schema_authors: None,
    //       requires_plugin_authors: None,
    //   };
    //
    //   // (1) Install flow is the SAME as admin UI install — no special-case:
    //   let admin_ui_install_flow_class = harness
    //       .admin_ui_install_flow_typeid();
    //   let journaling_install_flow_class = harness
    //       .plugin_install_flow_typeid(&journaling_manifest);
    //   assert_eq!(
    //       admin_ui_install_flow_class, journaling_install_flow_class,
    //       "Dogfood path (f): non-admin plugin install MUST use the \
    //        SAME install-flow code path as admin UI per D-4F-12 \
    //        workflow↔plugin unification"
    //   );
    //
    //   // (2) User-DID signs the install record per D-4F-12:
    //   let install_record_cid = harness
    //       .install_plugin_via_admin_ui(journaling_manifest.clone())
    //       .unwrap();
    //   let install_record = harness.load_install_record(&install_record_cid);
    //   assert_eq!(
    //       install_record.consenting_user_did,
    //       harness.user_did(),
    //       "Install record consenting_user_did MUST equal user-DID per \
    //        D-4F-12 user-as-source signing model"
    //   );
    //   assert!(
    //       harness.verify_user_signature_on_install_record(&install_record).is_ok(),
    //       "Install record MUST carry valid user-DID signature per D-4F-12"
    //   );
    //
    //   // (3) Plugin loads + executes post-install (production-runtime arm):
    //   let journaling_plugin_did = harness.plugin_did_for(&journaling_manifest);
    //   let outcome = harness.engine_call_as(
    //       &journaling_plugin_did,
    //       "journal_append",
    //       serde_json::json!({"text": "first entry"}),
    //   ).unwrap();
    //   assert!(
    //       outcome.is_success(),
    //       "Plugin invocation post-install MUST succeed for in-envelope cap"
    //   );
    //
    //   // (4) Per-plugin envelope enforced — out-of-envelope cap denied:
    //   let denied = harness.engine_call_as(
    //       &journaling_plugin_did,
    //       "store:notes:read",
    //       serde_json::json!({}),
    //   );
    //   assert!(
    //       denied.is_err(),
    //       "Per-plugin envelope MUST enforce: journaling plugin lacks \
    //        store:notes:read cap; invocation succeeded unexpectedly"
    //   );
    //
    //   // (4 cont.) Plugin DID is distinct from admin UI's plugin DID:
    //   let admin_ui_did = harness.admin_ui_plugin_did();
    //   assert_ne!(
    //       admin_ui_did, journaling_plugin_did,
    //       "Per-plugin DIDs MUST be distinct per CLAUDE.md baked-in #18"
    //   );
    //
    // OBSERVABLE consequence: 2nd-plugin install reuses admin UI's
    // install flow as one unified surface; user-DID signs both install
    // records uniformly; per-plugin envelopes enforce independently.
    // Defends against the failure shape where admin UI is a special-case
    // plugin that gets a parallel install path.
    unimplemented!(
        "G24-A + G24-D wire dogfood path (f): 2nd-plugin install with \
         4-arm production-runtime check (same flow + user-DID-signed + \
         plugin-loads-and-executes + per-plugin envelope enforced) per \
         pim-18 §3.6f"
    );
}

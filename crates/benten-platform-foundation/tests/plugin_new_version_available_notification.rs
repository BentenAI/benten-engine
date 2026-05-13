//! G24-D row pin — pull-not-push notification (plugin-arch-r1-13).
//!
//! Per docs/PLUGIN-MANIFEST.md §4: PULL-not-PUSH updates. Admin UI
//! surfaces a `E_PLUGIN_NEW_VERSION_AVAILABLE` notification (NOT a
//! hard-reject) when a newer descendant of the installed CID is
//! discovered via atrium.

use benten_errors::ErrorCode;
use benten_platform_foundation::module_ecosystem::new_version_available_code;

#[test]
fn new_version_notification_surfaces_typed_pull_not_push_code_at_engine_boundary() {
    // SUBSTANTIVE per pim-2 §3.6b: at HEAD `new_version_available_code()`
    // is the engine-boundary anchor that the admin UI surfaces as the
    // pull-not-push notification. Asserting the typed return defends
    // against rename / collapse to a different code family. Would-FAIL
    // if the anchor returned a wrong/stale ErrorCode (e.g.,
    // PluginManifestInvalid).
    assert_eq!(
        new_version_available_code(),
        ErrorCode::PluginNewVersionAvailable,
        "pull-not-push notification anchor MUST return typed \
         PluginNewVersionAvailable; would-FAIL if family-shifted"
    );
    // Round-trip via the string form to defend the string contract.
    assert_eq!(
        ErrorCode::PluginNewVersionAvailable.as_static_str(),
        "E_PLUGIN_NEW_VERSION_AVAILABLE"
    );
}

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    End-to-end pull-not-push notification surface: atrium peer announces v2 CID; \
    receiver's plugin_lifecycle checks if v2 is a DAG-descendant of any installed \
    plugin's anchor; if yes, emit PluginNewVersionAvailable change-event to admin UI. \
    Couples to G24-D-FP-1 plugin_lifecycle hardening (uninstall + upgrade + \
    pull-notify in same lifecycle module). Named destination: plan §3 G24-D-FP-1. \
    HARD RULE 12 clause-(b) BELONGS-NAMED-NOW."]
#[test]
fn discovering_newer_version_in_atrium_surfaces_new_version_available_event_end_to_end() {
    // Phase 4-Foundation R5 G24-D-FP-1 surface: plugin_lifecycle ships
    // the discover-then-notify path. At HEAD only the typed anchor
    // exists; end-to-end atrium-discovery wiring lands at G24-D-FP-1.
    panic!("G24-D-FP-1 wires end-to-end atrium-discovery pull-not-push notify path");
}

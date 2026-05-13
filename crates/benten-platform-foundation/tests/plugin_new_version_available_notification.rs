//! G24-D row pin — pull-not-push notification (plugin-arch-r1-13).
//!
//! Per docs/PLUGIN-MANIFEST.md §4: PULL-not-PUSH updates. Admin UI
//! surfaces a `E_PLUGIN_NEW_VERSION_AVAILABLE` notification (NOT a
//! hard-reject) when a newer descendant of the installed CID is
//! discovered via atrium.

mod common;

#[test]
#[ignore = "RED-PHASE: G24-D wave wires pull-model new-version notification; un-ignore at G24-D landing"]
fn discovering_newer_version_in_atrium_surfaces_new_version_available_notification() {
    let _v1 = common::manifest_fixtures::stub_cid_one();
    let _v2 = common::manifest_fixtures::stub_cid_two();

    // Future surface: atrium peer announces v2; receiver's
    // plugin_lifecycle checks if v2 is a descendant of any installed
    // plugin's anchor; if yes, emit E_PLUGIN_NEW_VERSION_AVAILABLE
    // event to admin UI (NOT auto-install per pull-not-push model).
    panic!("RED-PHASE: G24-D wave must wire pull-model new-version notification");
}

//! G24-D-FP-1 row pin — uninstall clears private namespace.
//!
//! Per plan §3.5.1 G24-D-FP-1 acceptance pin #2: delete private
//! namespace data (`private:<plugin_did>:*` rows). Touches T7 defense:
//! a re-installed plugin with the same DID should NOT inherit stale
//! private data from a prior install.

mod common;

use common::manifest_fixtures::stub_plugin_did;

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    uninstall_plugin's private-namespace data teardown arm; walks all storage \
    rows under `private:<plugin_did>:*` scope-prefix + deletes them. Named \
    destination: plan §3 G24-D-FP-1 (plugin_lifecycle uninstall cascade + \
    private-NS teardown + library-entry removal). HARD RULE 12 clause-(b) \
    BELONGS-NAMED-NOW: plan row pre-exists."]
#[test]
fn uninstall_deletes_private_namespace_rows_for_plugin_did() {
    let _plugin = stub_plugin_did();

    // Phase 4-Foundation R5 G24-D-FP-1 surface (NOT at G24-D primary):
    // uninstall_plugin walks all storage rows under the
    // `private:<plugin_did>:*` scope-prefix and deletes them.
    //
    // FAILS-IF-NO-OP because re-install would inherit stale state,
    // breaking the T7 isolation guarantee.
    panic!("G24-D-FP-1 wires private namespace teardown in uninstall_plugin");
}

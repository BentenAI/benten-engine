//! G24-D-FP-1 row pin — uninstall clears private namespace.
//!
//! Per plan §3.5.1 G24-D-FP-1 acceptance pin #2: delete private
//! namespace data (`private:<plugin_did>:*` rows). Touches T7 defense:
//! a re-installed plugin with the same DID should NOT inherit stale
//! private data from a prior install.

mod common;

use common::manifest_fixtures::stub_plugin_did;

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn uninstall_deletes_private_namespace_rows_for_plugin_did() {
    let _plugin = stub_plugin_did();

    // Future surface: uninstall_plugin walks all storage rows under
    // the `private:<plugin_did>:*` scope-prefix and deletes them.
    //
    // FAILS-IF-NO-OP because re-install would inherit stale state,
    // breaking the T7 isolation guarantee.
    panic!("RED-PHASE: G24-D-FP-1 must wire private namespace teardown");
}

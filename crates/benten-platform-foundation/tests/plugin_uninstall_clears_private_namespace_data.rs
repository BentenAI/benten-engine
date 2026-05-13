//! G24-D-FP-1 row pin — uninstall clears private namespace.
//!
//! Per plan §3.5.1 G24-D-FP-1 acceptance pin #2: delete private
//! namespace data (`private:<plugin_did>:*` rows). Touches T7 defense:
//! a re-installed plugin with the same DID should NOT inherit stale
//! private data from a prior install.
//!
//! Substantive after G24-D-FP-1 wire-up: exercises the
//! `PrivateNamespaceTeardown` port via [`InMemoryUninstallCascade`]
//! which walks `private:<plugin_did>:*` scope-prefix and deletes
//! every row.

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_id::did::Did;
use benten_id::plugin_did::PluginDidStore;
use benten_platform_foundation::plugin_library::{LibraryEntry, PluginLibrary};
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryUninstallCascade, UninstallContext, uninstall_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
};
use common::manifest_fixtures::stub_plugin_did;

fn fake_manifest(plugin_name: &str, cid: Cid) -> PluginManifest {
    PluginManifest {
        plugin_name: plugin_name.to_string(),
        content_cid: cid,
        peer_did: Did::from_string_unchecked("did:key:zAuthor".to_string()),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement::new("store:notes:read")],
        shares: SharesPolicy {
            default: SharesPolicyDefault::None,
            rules: None,
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

#[test]
fn uninstall_deletes_private_namespace_rows_for_plugin_did() {
    let plugin_did = stub_plugin_did();
    let other_plugin = Did::from_string_unchecked("did:key:zOtherPlugin".to_string());

    let cid = Cid::from_blake3_digest([4u8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(LibraryEntry {
        manifest_cid: cid,
        manifest: fake_manifest("doomed", cid),
        plugin_did: plugin_did.clone(),
        installed_at_nanos: 1,
    });
    let mut store = PluginDidStore::new();

    // Private-namespace data: 3 rows for doomed plugin + 1 row for
    // distractor plugin.
    let mut private = InMemoryUninstallCascade::new();
    private.insert_private_row(
        format!("private:{}:notes/draft-1", plugin_did.as_str()),
        b"hello".to_vec(),
    );
    private.insert_private_row(
        format!("private:{}:notes/draft-2", plugin_did.as_str()),
        b"world".to_vec(),
    );
    private.insert_private_row(
        format!("private:{}:journal/2024", plugin_did.as_str()),
        b"journal".to_vec(),
    );
    private.insert_private_row(
        format!("private:{}:notes/distractor", other_plugin.as_str()),
        b"distractor".to_vec(),
    );

    // Baseline.
    assert_eq!(
        private.private_rows_for(&plugin_did).len(),
        3,
        "Baseline: 3 rows for doomed plugin pre-uninstall"
    );
    assert_eq!(
        private.private_rows_for(&other_plugin).len(),
        1,
        "Baseline: 1 row for distractor plugin"
    );

    // Uninstall.
    let mut cascade = InMemoryUninstallCascade::new();
    let mut subs = InMemoryUninstallCascade::new();
    let mut ctx = UninstallContext {
        cap_revoker: &mut cascade,
        private_ns: &mut private,
        subscriptions: &mut subs,
    };
    let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).expect("uninstall ok");

    assert_eq!(
        outcome.private_namespace_rows_deleted, 3,
        "G24-D-FP-1: outcome counter reflects 3 rows deleted"
    );

    // T7 isolation guarantee: doomed plugin's private-NS empty;
    // distractor's preserved.
    assert!(
        private.private_rows_for(&plugin_did).is_empty(),
        "T7 isolation: re-install with same DID MUST NOT inherit stale private-NS data"
    );
    assert_eq!(
        private.private_rows_for(&other_plugin).len(),
        1,
        "T7 isolation: other plugins' private-NS MUST NOT be contaminated"
    );
}

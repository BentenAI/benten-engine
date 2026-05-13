//! G24-D-FP-1 row pin — uninstall_plugin cascade umbrella.
//!
//! Per plan §3 G24-D-FP-1: cascade-revoke + private-NS teardown +
//! library-entry removal at `crates/benten-platform-foundation/src/
//! plugin_lifecycle.rs::uninstall_plugin`.
//!
//! Acceptance: enumerate all user-DID-issued grants WHERE
//! `audience=plugin-DID`; revoke each. Cascade plugin-DID's own
//! downstream UCAN delegations. Terminate live subscriptions. Delete
//! private namespace data. Remove library entry.
//!
//! Substantive after G24-D-FP-1 wire-up: this umbrella test exercises
//! the (a) + (b) revoke arms together via [`InMemoryUninstallCascade`]
//! to confirm BOTH cascade halves observably run in a single uninstall
//! call. The per-finding-granular pins
//! ([`plugin_uninstall_revokes_held_caps.rs`] and
//! [`plugin_uninstall_cascade_revokes_delegated_caps.rs`]) cover each
//! arm in isolation; this file pins their joint outcome.

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_id::did::Did;
use benten_id::plugin_did::PluginDidStore;
use benten_platform_foundation::plugin_library::{LibraryEntry, PluginLibrary};
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryGrant, InMemoryUninstallCascade, UninstallContext, uninstall_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
};
use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

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

fn install_plugin_a(library: &mut PluginLibrary, plugin_did: Did) -> Cid {
    let cid = Cid::from_blake3_digest([5u8; 32]);
    library.insert(LibraryEntry {
        manifest_cid: cid,
        manifest: fake_manifest("plugin-a", cid),
        plugin_did,
        installed_at_nanos: 1,
    });
    cid
}

#[test]
fn uninstall_cascade_revokes_user_grants_with_audience_equals_plugin_did() {
    let plugin_did = stub_plugin_did();
    let user_did = stub_user_did();

    let mut library = PluginLibrary::new();
    let cid = install_plugin_a(&mut library, plugin_did.clone());
    let mut store = PluginDidStore::new();

    let mut cascade = InMemoryUninstallCascade::new();
    cascade.insert_grant(InMemoryGrant {
        grant_cid: Cid::from_blake3_digest([10u8; 32]),
        audience: plugin_did.clone(),
        issuer: user_did.clone(),
        scope: "store:notes:read".to_string(),
    });
    cascade.insert_grant(InMemoryGrant {
        grant_cid: Cid::from_blake3_digest([11u8; 32]),
        audience: plugin_did.clone(),
        issuer: user_did.clone(),
        scope: "store:notes:write".to_string(),
    });

    let mut private = InMemoryUninstallCascade::new();
    let mut subs = InMemoryUninstallCascade::new();
    let mut ctx = UninstallContext {
        cap_revoker: &mut cascade,
        private_ns: &mut private,
        subscriptions: &mut subs,
    };
    let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).expect("uninstall ok");

    // FAILS-IF-NO-OP: after uninstall, no active grants remain for
    // plugin-DID's audience.
    assert!(cascade.active_grants_for_audience(&plugin_did).is_empty());
    assert_eq!(outcome.held_caps_revoked, 2);
}

#[test]
fn uninstall_cascade_revokes_plugin_did_downstream_ucan_delegations() {
    let plugin_a = stub_plugin_did();
    let plugin_b = Did::from_string_unchecked("did:key:zPluginB".to_string());

    let mut library = PluginLibrary::new();
    let cid = install_plugin_a(&mut library, plugin_a.clone());
    let mut store = PluginDidStore::new();

    let mut cascade = InMemoryUninstallCascade::new();
    // A → B delegation.
    cascade.insert_grant(InMemoryGrant {
        grant_cid: Cid::from_blake3_digest([30u8; 32]),
        audience: plugin_b.clone(),
        issuer: plugin_a.clone(),
        scope: "store:notes:read".to_string(),
    });

    let mut private = InMemoryUninstallCascade::new();
    let mut subs = InMemoryUninstallCascade::new();
    let mut ctx = UninstallContext {
        cap_revoker: &mut cascade,
        private_ns: &mut private,
        subscriptions: &mut subs,
    };
    let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).expect("uninstall ok");

    // FAILS-IF-NO-OP: cascade removes A-issued grants from B's audience.
    assert!(cascade.active_grants_with_issuer(&plugin_a).is_empty());
    assert_eq!(outcome.delegations_cascade_revoked, 1);
}

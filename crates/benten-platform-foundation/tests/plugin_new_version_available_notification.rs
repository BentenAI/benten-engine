//! G24-D row pin — pull-not-push notification (plugin-arch-r1-13).
//!
//! Per docs/PLUGIN-MANIFEST.md §4: PULL-not-PUSH updates. Admin UI
//! surfaces a `E_PLUGIN_NEW_VERSION_AVAILABLE` notification (NOT a
//! hard-reject) when a newer descendant of the installed CID is
//! discovered via atrium.
//!
//! G24-D-FP-1 substantive wiring: the `discover_new_version` function
//! at `plugin_lifecycle.rs` is the engine-boundary anchor that the
//! atrium-side peer-discovery wiring calls on each peer announce. The
//! atrium-side runtime wiring (subscribe to peer-announcement topic;
//! invoke `discover_new_version` per announce) is owned by the sync
//! crate and lands when foundation is wired into the sync runtime;
//! the foundation-side anchor is verified substantively here.

#![allow(clippy::unwrap_used)]

use benten_core::Cid;
use benten_core::version_chain::DagVersionChain;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_platform_foundation::module_ecosystem::new_version_available_code;
use benten_platform_foundation::plugin_library::{LibraryEntry, PluginLibrary};
use benten_platform_foundation::plugin_lifecycle::{
    NewVersionDiscoveryOutcome, discover_new_version,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
};

fn fake_manifest(plugin_name: &str, cid: Cid) -> PluginManifest {
    PluginManifest {
        plugin_name: plugin_name.to_string(),
        content_cid: cid,
        peer_did: Did::from_string_for_test_fixture("did:key:zAuthor".to_string()),
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

#[test]
fn discovering_newer_version_in_atrium_surfaces_new_version_available_event_end_to_end() {
    // Set up an installed v1 of a plugin.
    let v1 = Cid::from_blake3_digest([1u8; 32]);
    let v2 = Cid::from_blake3_digest([2u8; 32]);
    let unrelated = Cid::from_blake3_digest([99u8; 32]);

    let mut library = PluginLibrary::new();
    library.insert(LibraryEntry {
        manifest_cid: v1,
        manifest: fake_manifest("my-plugin", v1),
        plugin_did: Did::from_string_for_test_fixture("did:key:zPluginV1".to_string()),
        installed_at_nanos: 1,
    });

    // DAG-version-chain rooted at v1; v2 is a child.
    let mut chain = DagVersionChain::new(v1);
    chain.add_version(v1, v2).unwrap();

    // Simulated atrium peer announces v2. discover_new_version walks
    // the version chain + finds v2 is a descendant of installed v1.
    let outcome = discover_new_version(&library, v2, &chain);
    match outcome {
        NewVersionDiscoveryOutcome::NewVersionAvailable {
            announced_cid,
            plugin_name,
        } => {
            assert_eq!(announced_cid, v2);
            assert_eq!(plugin_name, "my-plugin");
        }
        NewVersionDiscoveryOutcome::NoChange => {
            panic!(
                "FAILS-IF-NO-OP: descendant CID announce MUST emit \
                 NewVersionAvailable per plugin-arch-r1-13 pull-not-push"
            );
        }
    }

    // Distractor: an unrelated CID announce MUST NOT surface a
    // notification (avoid notification spam from unrelated peers).
    let unrelated_outcome = discover_new_version(&library, unrelated, &chain);
    assert_eq!(
        unrelated_outcome,
        NewVersionDiscoveryOutcome::NoChange,
        "Unrelated peer announce MUST NOT emit new-version notification"
    );

    // Same-version re-announce (peer telling us about v1 we already
    // have) MUST NOT emit a notification.
    let same_outcome = discover_new_version(&library, v1, &chain);
    assert_eq!(
        same_outcome,
        NewVersionDiscoveryOutcome::NoChange,
        "Re-announce of already-installed CID MUST NOT emit new-version notification"
    );
}

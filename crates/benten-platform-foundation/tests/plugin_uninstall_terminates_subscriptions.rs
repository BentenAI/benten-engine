//! Phase-4-Foundation R4-FP-1 — T10-uninstall (c) LOAD-BEARING pin:
//! plugin uninstall terminates subscriptions.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-uninstall defense step 3(c) + plan §3 G24-D-FP-1.
//!
//! ## What this pin establishes
//!
//! Per pim-2-amendment §3.6b sub-rule 4 per-finding granularity:
//! T10-uninstall has 3 sub-arms (a) revoke held caps, (b) cascade-
//! revoke delegated caps, (c) terminate live subscriptions. This
//! pin is the (c) arm — LOAD-BEARING per threat-model §T10 test-pin
//! plan + couples to T12 cross-process amplification defense.
//!
//! Substantive after G24-D-FP-1 wire-up: exercises the
//! `SubscriptionRegistry` port via [`InMemoryUninstallCascade`] which
//! faithfully reproduces the engine-side `Engine::on_change_as_with_cursor`
//! registry semantics — namely, every subscription whose subscriber
//! DID equals the uninstalled plugin-DID is terminated and the
//! registry reports zero active subscriptions for that DID.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires uninstall to revoke caps but forgets to terminate
//! subscriptions. Subscription callback continues delivering events
//! post-uninstall; plugin-DID receives change-stream events for data
//! it no longer has cap to read. Cross-process amplification under
//! D-4F-4 (a) thin-client makes this HIGH severity per threat-model
//! §T12.

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_id::did::Did;
use benten_id::plugin_did::PluginDidStore;
use benten_platform_foundation::plugin_library::{LibraryEntry, PluginLibrary};
use benten_platform_foundation::plugin_lifecycle::{
    InMemorySubscription, InMemoryUninstallCascade, SubscriptionRegistry, UninstallPorts,
    uninstall_plugin,
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
fn plugin_uninstall_terminates_active_subscriptions_for_subscriber_did() {
    let plugin_did = stub_plugin_did();
    let other_plugin = Did::from_string_unchecked("did:key:zOtherPlugin".to_string());

    let cid = Cid::from_blake3_digest([3u8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(LibraryEntry {
        manifest_cid: cid,
        manifest: fake_manifest("doomed", cid),
        plugin_did: plugin_did.clone(),
        installed_at_nanos: 1,
    });
    let mut store = PluginDidStore::new();

    // Subscription registry has 2 subscriptions for the doomed plugin
    // + 1 for an unrelated plugin (distractor).
    let mut subs = InMemoryUninstallCascade::new();
    subs.insert_subscription(InMemorySubscription {
        subscriber: plugin_did.clone(),
        scope: "store:notes:read".to_string(),
    });
    subs.insert_subscription(InMemorySubscription {
        subscriber: plugin_did.clone(),
        scope: "host:time:now".to_string(),
    });
    subs.insert_subscription(InMemorySubscription {
        subscriber: other_plugin.clone(),
        scope: "store:notes:read".to_string(),
    });

    // Baseline: 2 active for doomed plugin, 1 for distractor.
    assert_eq!(subs.active_subscription_count(&plugin_did), 2);
    assert_eq!(subs.active_subscription_count(&other_plugin), 1);

    // Uninstall: T10-uninstall (c) terminates subscriptions.
    let mut cascade = InMemoryUninstallCascade::new();
    let mut private = InMemoryUninstallCascade::new();
    let mut ctx = UninstallPorts {
        cap_revoker: &mut cascade,
        private_ns: &mut private,
        subscriptions: &mut subs,
    };
    let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).expect("uninstall ok");

    assert_eq!(
        outcome.subscriptions_terminated, 2,
        "T10-uninstall (c) LOAD-BEARING: outcome counter reflects 2 terminations"
    );

    // LOAD-BEARING: registry has zero active subscriptions for
    // uninstalled plugin-DID; distractor untouched.
    assert_eq!(
        subs.active_subscription_count(&plugin_did),
        0,
        "T10-uninstall (c) LOAD-BEARING: subscription registry MUST be empty for uninstalled plugin-DID"
    );
    assert_eq!(
        subs.active_subscription_count(&other_plugin),
        1,
        "T10-uninstall (c): distractor subscriptions for OTHER plugins MUST NOT be terminated"
    );
}

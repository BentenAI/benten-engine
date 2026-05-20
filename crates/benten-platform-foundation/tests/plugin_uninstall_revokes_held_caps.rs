//! Phase-4-Foundation R4-FP-1 — T10-uninstall (a) pin: plugin uninstall
//! revokes caps the plugin held.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-uninstall defense step 3(a) + plan §3 G24-D-FP-1.
//!
//! ## What this pin establishes
//!
//! Per pim-2-amendment §3.6b sub-rule 4 per-finding granularity:
//! T10-uninstall (a) = revoke every cap admin-UI-DID held. Substantive
//! after G24-D-FP-1 wire-up: exercises the `CapRevoker` port via
//! [`InMemoryUninstallCascade`] which faithfully reproduces the
//! engine-side `Engine::revoke_capability_by_grant_cid` semantics
//! (PR #199) — namely, every grant whose audience equals the
//! uninstalled plugin-DID is revoked and surfaces in the revocation
//! log; revocations DO NOT contaminate other plugins' caps.
//!
//! Couples to §13.11 UCAN revocation observance closure (PR #199;
//! `Engine::revoke_capability_by_grant_cid` already at HEAD).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires the cascade arm (b: revoke delegated TO others)
//! but forgets the direct arm (a: revoke caps the plugin HELD). Plugin
//! is uninstalled; its own cap-store entries remain valid; a re-
//! installed plugin (or stale code path) can still authenticate.

#![allow(clippy::unwrap_used)]

mod common;

use benten_core::Cid;
use benten_id::did::Did;
use benten_id::plugin_did::PluginDidStore;
use benten_platform_foundation::plugin_library::{LibraryEntry, PluginLibrary};
use benten_platform_foundation::plugin_lifecycle::{
    InMemoryGrant, InMemoryUninstallCascade, UninstallPorts, uninstall_plugin,
};
use benten_platform_foundation::plugin_manifest::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
};
use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

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
fn plugin_uninstall_revokes_every_cap_with_audience_equals_plugin_did() {
    let plugin_did = stub_plugin_did();
    let user_did = stub_user_did();
    let other_plugin = Did::from_string_for_test_fixture("did:key:zOtherPlugin".to_string());

    let cid = Cid::from_blake3_digest([7u8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(LibraryEntry {
        manifest_cid: cid,
        manifest: fake_manifest("doomed", cid),
        plugin_did: plugin_did.clone(),
        installed_at_nanos: 1,
    });
    let mut store = PluginDidStore::new();

    let mut cascade = InMemoryUninstallCascade::new();
    // 3 distinct grants issued by user-DID to plugin-DID.
    for (i, scope) in ["store:notes:read", "store:notes:write", "host:time:now"]
        .iter()
        .enumerate()
    {
        cascade.insert_grant(InMemoryGrant {
            grant_cid: Cid::from_blake3_digest([i as u8 + 10; 32]),
            audience: plugin_did.clone(),
            issuer: user_did.clone(),
            scope: (*scope).to_string(),
        });
    }
    // Distractor: grant for a DIFFERENT plugin must NOT be revoked.
    cascade.insert_grant(InMemoryGrant {
        grant_cid: Cid::from_blake3_digest([99u8; 32]),
        audience: other_plugin.clone(),
        issuer: user_did.clone(),
        scope: "store:notes:read".to_string(),
    });

    // Baseline: all 3 grants active for plugin-DID.
    let baseline = cascade.active_grants_for_audience(&plugin_did);
    assert_eq!(
        baseline.len(),
        3,
        "Baseline: 3 grants must be active pre-uninstall"
    );

    // Uninstall.
    let mut private = InMemoryUninstallCascade::new();
    let mut subs = InMemoryUninstallCascade::new();
    let mut ctx = UninstallPorts {
        cap_revoker: &mut cascade,
        private_ns: &mut private,
        subscriptions: &mut subs,
    };
    let outcome = uninstall_plugin(&mut library, &mut store, &mut ctx, &cid).expect("uninstall ok");

    // T10-uninstall (a) LOAD-BEARING: every grant with
    // audience=plugin_did MUST be revoked.
    let after = cascade.active_grants_for_audience(&plugin_did);
    assert!(
        after.is_empty(),
        "T10-uninstall (a): plugin-DID's held caps MUST be revoked; \
         {} caps still active",
        after.len()
    );
    assert_eq!(
        outcome.held_caps_revoked, 3,
        "T10-uninstall (a): outcome counter must reflect direct revocations"
    );

    // Defense-in-depth: revocation log shows 3 entries with audience =
    // plugin_did, cascade_source = None (direct revocation).
    let log_entries: Vec<_> = cascade
        .revocation_log()
        .iter()
        .filter(|r| r.audience == plugin_did)
        .collect();
    assert_eq!(
        log_entries.len(),
        3,
        "T10-uninstall (a): all 3 grants must surface in revocation log"
    );
    assert!(
        log_entries.iter().all(|r| r.cascade_source.is_none()),
        "T10-uninstall (a): direct revocations are NOT cascade-tagged"
    );

    // Defense-in-depth: distractor untouched.
    let other_after = cascade.active_grants_for_audience(&other_plugin);
    assert_eq!(
        other_after.len(),
        1,
        "T10-uninstall (a): other plugins' caps MUST NOT be contaminated"
    );
}

//! Phase-4-Foundation R4-FP-1 — T10-uninstall (b) pin: plugin uninstall
//! cascade-revokes caps the plugin delegated to OTHER plugins.
//!
//! Pin source: `.addl/phase-4-foundation/r4-triage.md` §1 BLOCKER row
//! r4-tc-3 + `.addl/phase-4-foundation/admin-ui-v0-threat-model.md`
//! §T10-uninstall defense step 3(b) + plan §3 G24-D-FP-1 + CLAUDE.md
//! baked-in #18 Layer 3 cascade discipline.
//!
//! ## What this pin establishes
//!
//! Per pim-2-amendment §3.6b sub-rule 4 per-finding granularity:
//! T10-uninstall (b) = cascade-revoke every cap the plugin delegated
//! to OTHER plugins. Without cascade, downstream plugins retain stale
//! caps after the source plugin is uninstalled.
//!
//! Distinct from (a) "revoke caps plugin HELD" — (b) walks
//! grants WHERE issuer = plugin_did, cascade.
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer wires (a) revoke held caps but skips cascade. Plugin A
//! is uninstalled; A had delegated caps to plugin B; B's downstream
//! caps remain valid. Hostile re-install path could exploit stale
//! delegations. Per CLAUDE.md baked-in #18 Layer 3: the manifest
//! envelope's transitivity guarantee requires cascade.

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
fn plugin_uninstall_cascade_revokes_caps_delegated_to_other_plugins() {
    let plugin_a = stub_plugin_did();
    let plugin_b = Did::from_string_for_test_fixture("did:key:zPluginB".to_string());
    let user_did = stub_user_did();

    let cid_a = Cid::from_blake3_digest([1u8; 32]);
    let mut library = PluginLibrary::new();
    library.insert(LibraryEntry {
        manifest_cid: cid_a,
        manifest: fake_manifest("plugin-a", cid_a),
        plugin_did: plugin_a.clone(),
        installed_at_nanos: 1,
    });
    let mut store = PluginDidStore::new();

    let mut cascade = InMemoryUninstallCascade::new();
    // A delegates two caps to B (within envelope).
    let delegation_1_cid = Cid::from_blake3_digest([21u8; 32]);
    let delegation_2_cid = Cid::from_blake3_digest([22u8; 32]);
    cascade.insert_grant(InMemoryGrant {
        grant_cid: delegation_1_cid,
        audience: plugin_b.clone(),
        issuer: plugin_a.clone(),
        scope: "store:notes:read".to_string(),
    });
    cascade.insert_grant(InMemoryGrant {
        grant_cid: delegation_2_cid,
        audience: plugin_b.clone(),
        issuer: plugin_a.clone(),
        scope: "host:time:now".to_string(),
    });
    // Distractor: user-DID directly grants B a cap; that grant MUST NOT
    // be cascade-revoked (different issuer).
    let user_grant_cid = Cid::from_blake3_digest([55u8; 32]);
    cascade.insert_grant(InMemoryGrant {
        grant_cid: user_grant_cid,
        audience: plugin_b.clone(),
        issuer: user_did.clone(),
        scope: "store:notes:write".to_string(),
    });

    // Baseline: B has 3 active grants (2 from A + 1 from user-DID).
    let b_baseline = cascade.active_grants_for_audience(&plugin_b);
    assert_eq!(
        b_baseline.len(),
        3,
        "Baseline: B must have 3 active grants pre-uninstall (2 from A + 1 from user-DID)"
    );

    // Uninstall A.
    let mut private = InMemoryUninstallCascade::new();
    let mut subs = InMemoryUninstallCascade::new();
    let mut ctx = UninstallPorts {
        cap_revoker: &mut cascade,
        private_ns: &mut private,
        subscriptions: &mut subs,
    };
    let outcome =
        uninstall_plugin(&mut library, &mut store, &mut ctx, &cid_a).expect("uninstall ok");

    // T10-uninstall (b): caps A→B MUST be cascade-revoked.
    assert_eq!(
        outcome.delegations_cascade_revoked, 2,
        "T10-uninstall (b): outcome counter reflects 2 cascade-revocations"
    );
    let b_after_revoke = cascade.active_grants_for_audience(&plugin_b);
    // User-DID-issued grant must remain.
    assert_eq!(
        b_after_revoke.len(),
        1,
        "T10-uninstall (b): user-DID-issued grant to B must remain; got {} active",
        b_after_revoke.len()
    );
    assert_eq!(
        b_after_revoke[0].issuer, user_did,
        "T10-uninstall (b): remaining grant's issuer MUST be user-DID"
    );

    // Defense-in-depth: revocation log tags cascade-source explicitly.
    let cascade_entries: Vec<_> = cascade
        .revocation_log()
        .iter()
        .filter(|r| r.cascade_source == Some(plugin_a.clone()))
        .collect();
    assert_eq!(
        cascade_entries.len(),
        2,
        "T10-uninstall (b): revocation log MUST tag cascade-source (plugin_a) for forensic auditability"
    );
    assert!(
        cascade_entries.iter().all(|r| r.audience == plugin_b),
        "T10-uninstall (b): cascade-revoked grants' audience is plugin_b"
    );
}

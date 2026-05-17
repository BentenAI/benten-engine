//! G27-D — cap-policy fires on plugin-delegated write WITHIN the
//! manifest envelope.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.17 G27-D row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-D entry
//! + CLAUDE.md baked-in #18 layer-(c) (runtime delegation within
//! manifest envelope).
//!
//! ## The within-envelope path (CLAUDE.md #18 layer-c)
//!
//! When plugin A delegates a UCAN cap to plugin B for a scope that
//! the source plugin's manifest `shares` policy PERMITS, the cap-
//! policy MUST observe + permit the audience-side write. This is
//! the "within envelope" path: the runtime delegation matches the
//! manifest's pre-installed consent envelope, so no per-action
//! user prompt fires (per the Phase 6 AI-agent ergonomics goal).
//!
//! ## Pin shape — substantive end-to-end (pim-2 §3.6b)
//!
//! 1. Construct source `PluginManifest` for plugin A with explicit
//!    `shares` rule permitting `store:notes:write` to plugin B.
//! 2. Mint a grant for `store:notes:write` (plugin A holds the cap).
//! 3. `manifest_scope::check_scope_within_envelope` for the
//!    audience-side `(cap_scope, target_plugin_did, source_manifest)`
//!    triple returns `Ok(())` — the envelope-CHECK passes.
//! 4. The audience-side `GrantBackedPolicy::check_write` also permits
//!    (grant lookup succeeds + manifest envelope check is the
//!    sibling step).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer ships `manifest_scope::check_scope_within_envelope`
//! that ALWAYS returns Ok regardless of policy default. Sibling pin
//! (`...denies_plugin_delegated_write_outside_manifest_envelope`)
//! FAILS — its assertion expects Err. So both pins as a pair
//! constrain the implementer to faithfully consult the manifest's
//! `shares` envelope.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::manifest_scope::check_scope_within_envelope;
use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, GrantBackedPolicy, GrantReader};
use benten_id::did::Did;
use benten_platform_foundation::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault, SharesRule, SharesTarget,
};

struct MockGrants {
    grants: Vec<String>,
}

impl GrantReader for MockGrants {
    fn has_unrevoked_grant_for_scope(&self, scope: &str) -> Result<bool, CapError> {
        Ok(self.grants.iter().any(|g| g == scope))
    }
}

fn plugin_a_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkPluginA".to_string())
}

fn plugin_b_did() -> Did {
    Did::from_string_unchecked("did:key:z6MkPluginB".to_string())
}

fn source_manifest_with_shares_rule() -> PluginManifest {
    PluginManifest {
        plugin_name: "plugin-a".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        peer_did: plugin_a_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement {
            scope: "store:notes:write".to_string(),
        }],
        shares: SharesPolicy {
            default: SharesPolicyDefault::Matching,
            // Explicit rule: A shares store:notes:write to B specifically.
            rules: Some(vec![SharesRule {
                cap_pattern: "store:notes:write".to_string(),
                target: SharesTarget::PluginDid(plugin_b_did()),
            }]),
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// G27-D within-envelope: cap-policy permits the audience-side write
/// when the delegation fits the source manifest's `shares` envelope.
#[test]
fn cap_policy_fires_on_plugin_delegated_write_within_manifest_envelope() {
    let plugin_b = plugin_b_did();
    let source_manifest = source_manifest_with_shares_rule();

    // Step 1: source manifest's envelope-CHECK permits the cap-scope
    // for the target plugin-DID.
    check_scope_within_envelope("store:notes:write", &plugin_b, &source_manifest).expect(
        "G27-D within-envelope: manifest_scope::check_scope_within_envelope MUST permit when \
         source manifest's `shares` rule covers the cap + target",
    );

    // Step 2: cap-policy permits the audience-side write (source
    // grant exists; envelope is consulted as sibling check by the
    // caller wiring at G24-D-FP-3 wave).
    let grants = Arc::new(MockGrants {
        grants: vec!["store:notes:write".to_string()],
    });
    let policy = GrantBackedPolicy::new(grants);

    let ctx = CapWriteContext {
        label: "notes".into(),
        scope: "store:notes:write".into(),
        ..Default::default()
    };

    policy
        .check_write(&ctx)
        .expect("G27-D within-envelope: cap-policy permits the audience-side write");
}

/// G27-D within-envelope: `shares: Any` policy admits delegation
/// without a specific rule. Verifies the default-Any short-circuit.
#[test]
fn shares_any_default_admits_audience_write_within_envelope() {
    let plugin_b = plugin_b_did();
    let mut manifest = source_manifest_with_shares_rule();
    manifest.shares = SharesPolicy {
        default: SharesPolicyDefault::Any,
        rules: None,
    };

    check_scope_within_envelope("store:notes:write", &plugin_b, &manifest)
        .expect("shares: Any → permit");
}

/// Compile-time witness: F3 stub `PluginManifest` is reachable.
#[test]
fn within_envelope_stub_manifest_reachable_compile_witness() {
    fn _accepts_plugin_manifest(_m: &benten_platform_foundation::PluginManifest) {}
    let _: fn(&benten_platform_foundation::PluginManifest) = _accepts_plugin_manifest;
}

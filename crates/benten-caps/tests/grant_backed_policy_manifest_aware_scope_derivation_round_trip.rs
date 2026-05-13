//! G27-D — manifest-aware scope derivation round-trip.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.17 G27-D row +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-D entry
//! + plugin-arch-r1-10 (manifest scope-string grammar pin) + Ben
//! D-4F-1 FULL plugin manifest ratification + CLAUDE.md baked-in #18
//! "Implementation refinements" layered-consent model.
//!
//! ## What this pin verifies (un-ignored at G27-D)
//!
//! Under the FULL plugin manifest (G24-D), the cap-policy scope
//! derivation must consult the manifest `requires` / `shares` halves
//! to map plugin-DID-keyed scope shapes through the policy. The
//! `crates/benten-caps/src/manifest_scope.rs` module (G27-D) wires a
//! pure function `manifest_requires_to_scope(manifest, plugin_did)`
//! that produces the canonical scope-string set per
//! plugin-arch-r1-10 grammar:
//!
//! - `private:<plugin_did>:*`             (private-namespace caps)
//! - `requires:<plugin_did>:<path>`       (manifest `requires` half)
//! - `shares:<plugin_did>:<path>`         (manifest `shares` half)
//!
//! ## Round-trip pin shape (substantive — pim-2 §3.6b + pim-18 §3.6f)
//!
//! 1. Construct a `PluginManifest` (via F3 fixture) with both
//!    `private:`-shaped + plain-shaped `requires` entries.
//! 2. Invoke `manifest_requires_to_scope` — verify every output
//!    string matches the plugin-arch-r1-10 canonical grammar prefix.
//! 3. Mint grants for each derived scope; assert a write under the
//!    derived scope permits via the existing `GrantBackedPolicy`
//!    explicit-scope short-circuit (G27-B lift).
//! 4. Inverse arm — a scope-string OUTSIDE the manifest envelope
//!    has no matching grant → policy DENIES.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;

use benten_caps::manifest_scope::{
    REQUIRES_PREFIX, manifest_requires_to_scope, manifest_shares_to_scope,
};
use benten_caps::{CapError, CapabilityPolicy, GrantBackedPolicy, GrantReader, WriteContext};
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

fn plugin_did_alpha() -> Did {
    Did::from_string_unchecked("did:key:z6MkAlpha".to_string())
}

fn manifest_with_mixed_requires() -> PluginManifest {
    PluginManifest {
        plugin_name: "test-plugin".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        peer_did: plugin_did_alpha(),
        peer_signature: vec![0u8; 64],
        requires: vec![
            CapRequirement {
                scope: "store:notes:read".to_string(),
            },
            CapRequirement {
                scope: "private:admin-ui-private:scratch".to_string(),
            },
            CapRequirement {
                scope: "host:time:now".to_string(),
            },
        ],
        shares: SharesPolicy {
            default: SharesPolicyDefault::Matching,
            rules: Some(vec![SharesRule {
                cap_pattern: "store:notes:write".to_string(),
                target: SharesTarget::Any,
            }]),
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// G27-D primary round-trip pin: manifest's `requires` / `shares`
/// halves map to canonical scope strings that round-trip through
/// `GrantBackedPolicy::check_write`.
#[test]
fn manifest_aware_scope_derivation_round_trip() {
    let plugin_did = plugin_did_alpha();
    let manifest = manifest_with_mixed_requires();

    // Step 1: derive scope strings from `requires` half.
    let derived = manifest_requires_to_scope(&manifest, &plugin_did);
    assert_eq!(
        derived.len(),
        3,
        "expected one derived scope per `requires` entry"
    );

    // plugin-arch-r1-10 grammar: every derived scope must be one of
    // `requires:`/`shares:`/`private:` prefixed.
    for scope in &derived {
        assert!(
            scope.starts_with("requires:")
                || scope.starts_with("shares:")
                || scope.starts_with("private:"),
            "G27-D grammar (plugin-arch-r1-10): manifest-derived scopes \
             must match canonical plugin-DID-keyed shape; got {scope}"
        );
    }

    // Step 2: round-trip through the policy.
    let grants = Arc::new(MockGrants {
        grants: derived.clone(),
    });
    let policy = GrantBackedPolicy::new(grants);
    let scope = derived
        .iter()
        .find(|s| s.starts_with(REQUIRES_PREFIX))
        .expect("at least one requires-prefixed scope")
        .clone();
    let ctx = WriteContext {
        label: String::new(),
        scope: scope.clone(),
        ..Default::default()
    };
    policy
        .check_write(&ctx)
        .expect("G27-D round-trip: manifest-derived scope must permit when grant present");

    // Step 3: inverse arm — scope outside envelope denies.
    let empty_grants = Arc::new(MockGrants { grants: vec![] });
    let policy_2 = GrantBackedPolicy::new(empty_grants);
    let ctx_2 = WriteContext {
        label: String::new(),
        scope: scope.clone(),
        ..Default::default()
    };
    let err = policy_2.check_write(&ctx_2).expect_err("no grant → deny");
    assert!(
        matches!(err, CapError::Denied { .. }),
        "G27-D inverse: scope without grant must deny; got {err:?}"
    );

    // Step 4: shares half also produces canonical scope strings.
    let shares_derived = manifest_shares_to_scope(&manifest, &plugin_did);
    assert_eq!(
        shares_derived,
        vec!["shares:did:key:z6MkAlpha:store:notes:write"],
        "G27-D shares-half: manifest_shares_to_scope emits canonical \
         `shares:<plugin_did>:<cap_pattern>` form"
    );
}

/// Compile-time witness: F3 stub `PluginManifest` is reachable from
/// `benten-caps` tests via the dev-dep declared in Cargo.toml. This
/// is the cross-family dependency that G27-D R3 pins must compile
/// against (per r2-test-landscape §4 helper inventory item #2).
#[test]
fn plugin_manifest_stub_reachable_compile_witness() {
    fn _accepts_plugin_manifest(_m: &benten_platform_foundation::PluginManifest) {}
    let _: fn(&benten_platform_foundation::PluginManifest) = _accepts_plugin_manifest;
}

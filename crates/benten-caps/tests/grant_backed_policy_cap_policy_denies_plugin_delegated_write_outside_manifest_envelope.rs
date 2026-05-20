//! G27-D — cap-policy DENIES plugin-delegated write OUTSIDE the
//! manifest envelope (LOAD-BEARING substantive).
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.17 G27-D row
//! ("LOAD-BEARING substantive") +
//! `.addl/phase-4-foundation/00-implementation-plan.md` §3 G27-D entry
//! + CLAUDE.md baked-in #18 layer-(c) (runtime delegation MUST fit
//! source manifest's `shares` policy or be DENIED) + arch-r1-3
//! ratified split of `E_PLUGIN_MANIFEST_SIGNATURE_INVALID` adjacent
//! ErrorCodes (`E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` lands
//! at G24-D per plan §3 G24-D ErrorCode list).
//!
//! ## The outside-envelope path (CLAUDE.md #18 layer-c, deny side)
//!
//! Plugin A delegates a UCAN cap to plugin B for a scope that the
//! source plugin's manifest `shares` policy DOES NOT permit. The cap-
//! policy MUST DENY — even though the delegation chain mathematically
//! traces to user-root via the source grant, the delegation EXCEEDS
//! the manifest envelope the user consented to at install. This is
//! the LOAD-BEARING half of CLAUDE.md #18 layer-(c): without this
//! denial, the manifest envelope is a paper guarantee.
//!
//! ## Pin shape — substantive end-to-end (pim-2 §3.6b)
//!
//! 1. Construct source `PluginManifest` for plugin A with
//!    `shares: { default: None, rules: [] }` (CONSERVATIVE default —
//!    plugin A delegates NOTHING via manifest).
//! 2. `manifest_scope::check_scope_within_envelope` for the audience-
//!    side `(cap_scope, target_plugin_did, source_manifest)` triple
//!    returns Err — the envelope check fails CLOSED.
//! 3. The audience-side write fails via the wired envelope-check
//!    pathway (G24-D-FP-3 wave). At G27-D level the assertion is on
//!    the `check_scope_within_envelope` surface directly.
//! 4. The denial surfaces a typed `ErrorCode::PluginDelegationOutsideManifestEnvelope`
//!    via the engine boundary (verified at G24-D-FP-3 napi-side pin).
//!
//! ## Would-FAIL-if-no-op'd
//!
//! Implementer ships the within-envelope arm only + omits the
//! envelope-CHECK in `manifest_scope::check_scope_within_envelope`
//! (returns Ok unconditionally). Within-envelope sister pin PASSES;
//! this pin's assertion flips from Err to Ok — silent fail-OPEN
//! breaking the layer-(c) consent guarantee. The pair must hold
//! together (pim-2 §3.6b would-FAIL-if-no-op'd 4-axis check).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::CapError;
use benten_caps::manifest_scope::check_scope_within_envelope;
use benten_id::did::Did;
use benten_platform_foundation::{
    CapRequirement, PluginManifest, SharesPolicy, SharesPolicyDefault,
};

fn plugin_a_did() -> Did {
    Did::from_string_for_test_fixture("did:key:z6MkPluginA".to_string())
}

fn plugin_b_did() -> Did {
    Did::from_string_for_test_fixture("did:key:z6MkPluginB".to_string())
}

fn source_manifest_with_no_shares() -> PluginManifest {
    PluginManifest {
        plugin_name: "plugin-a".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        peer_did: plugin_a_did(),
        peer_signature: vec![0u8; 64],
        requires: vec![CapRequirement {
            scope: "store:notes:write".to_string(),
        }],
        // SharesPolicyDefault::None — conservative default; plugin A
        // explicitly delegates NOTHING via the manifest envelope.
        shares: SharesPolicy::none(),
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    }
}

/// G27-D LOAD-BEARING outside-envelope deny: the manifest's `shares:
/// None` envelope DENIES audience-side delegation. Without this
/// denial, the manifest envelope is a paper guarantee.
#[test]
fn cap_policy_denies_plugin_delegated_write_outside_manifest_envelope() {
    let plugin_b = plugin_b_did();
    let source_manifest = source_manifest_with_no_shares();

    // LOAD-BEARING: the envelope-CHECK MUST deny because plugin A's
    // manifest delegates nothing.
    let err = check_scope_within_envelope("store:notes:write", &plugin_b, &source_manifest)
        .expect_err(
            "G27-D outside-envelope: manifest_scope::check_scope_within_envelope MUST deny \
             when source manifest's `shares: None` envelope forbids the cap-scope",
        );

    // arch-r1-3 ratified split: the denial surfaces
    // `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` at the engine
    // boundary. At the `CapError` level we observe `Denied` + the
    // payload names the offending scope.
    match err {
        CapError::Denied { required, .. } => {
            assert_eq!(
                required, "store:notes:write",
                "G27-D outside-envelope: denial payload names the offending scope"
            );
        }
        other => panic!("G27-D outside-envelope: expected CapError::Denied; got {other:?}"),
    }
}

/// G27-D outside-envelope: even with a `Matching` default + rule for
/// a DIFFERENT cap-pattern, an unrelated cap-scope is still denied.
/// Defends against the "rule wildcard widens unintentionally" mode.
#[test]
fn outside_envelope_denies_when_rule_targets_unrelated_cap_pattern() {
    let plugin_b = plugin_b_did();
    let manifest = PluginManifest {
        shares: SharesPolicy {
            default: SharesPolicyDefault::Matching,
            rules: Some(vec![benten_platform_foundation::SharesRule {
                cap_pattern: "store:other:read".to_string(), // unrelated cap
                target: benten_platform_foundation::SharesTarget::Any,
            }]),
        },
        ..source_manifest_with_no_shares()
    };

    let err = check_scope_within_envelope("store:notes:write", &plugin_b, &manifest)
        .expect_err("G27-D outside-envelope: unrelated rule does not widen envelope");
    assert!(matches!(err, CapError::Denied { .. }));
}

/// Compile-time witness: F3 stub `PluginManifest` + `SharesPolicyDefault::None`
/// are reachable from the test crate.
#[test]
fn outside_envelope_stub_manifest_reachable_compile_witness() {
    let none_default = benten_platform_foundation::SharesPolicyDefault::None;
    let _: &benten_platform_foundation::SharesPolicyDefault = &none_default;
}

//! G24-D row test pin — runtime delegation WITHIN manifest envelope.
//!
//! CLAUDE.md #18 Layer 3 — plugin A delegates UCAN to plugin B; A's
//! manifest `shares` policy permits; chain validates. Per pim-2-
//! amendment §3.6b sub-rule 4: this pin exercises the POSITIVE within-
//! envelope arm specifically (sibling pin outside-envelope-denied
//! covers the NEGATIVE arm).

mod common;

use benten_caps::plugin_delegation::{
    DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
};
use benten_platform_foundation::{SharesPolicy, SharesPolicyDefault, SharesRule, SharesTarget};
use common::manifest_fixtures::{manifest_with_shares_matching_rule, stub_plugin_did};

/// Adapter that lets the manifest's SharesPolicy be consulted by the
/// caps-crate delegation check (mirrors the umbrella test adapter).
struct PolicyAdapter<'a>(&'a SharesPolicy);

impl<'a> SharesPolicyView for PolicyAdapter<'a> {
    fn permits(&self, cap_pattern: &str, target_plugin_did: &benten_id::did::Did) -> bool {
        self.0.permits_delegation(cap_pattern, target_plugin_did)
    }
}

#[test]
fn delegation_within_envelope_admitted_for_explicitly_permitted_cap_and_target() {
    // SUBSTANTIVE per pim-2 §3.6b: build manifest with shares-matching
    // rule "store:notes:read" -> stub_plugin_did(); attempt delegation
    // of EXACTLY that cap to EXACTLY that target; expect
    // DelegationDecision::Permitted. Would-FAIL if rule-pattern check
    // were skipped (would surface OutsideEnvelope).
    let target = stub_plugin_did();
    let manifest_a = manifest_with_shares_matching_rule(
        "store:notes:read",
        SharesTarget::PluginDid(target.clone()),
    );
    let view = PolicyAdapter(&manifest_a.shares);

    let decision = check_delegation_within_envelope("store:notes:read", &target, &view);
    assert_eq!(
        decision,
        DelegationDecision::Permitted,
        "exact cap + exact target MUST admit per shares-matching rule; \
         would-FAIL if rule-pattern check were skipped"
    );
    // into_result also threads cleanly.
    decision.into_result().expect("Permitted -> Ok");
}

#[test]
fn delegation_admitted_under_shares_any_default_for_non_private_cap() {
    // SUBSTANTIVE per pim-2 §3.6b: shares-any default admits any
    // non-private cap. This is a per-finding granular arm sibling to
    // shares-matching-rule above. Would-FAIL if default-Any check
    // missed.
    let target = stub_plugin_did();
    let manifest_a = benten_platform_foundation::PluginManifest {
        plugin_name: "a".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        peer_did: benten_id::did::Did::from_string_for_test_fixture(
            "did:key:z6MkAuthorAny".to_string(),
        ),
        peer_signature: vec![0u8; 64],
        requires: vec![benten_platform_foundation::CapRequirement::new(
            "store:notes:read",
        )],
        shares: SharesPolicy {
            default: SharesPolicyDefault::Any,
            rules: None,
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    let view = PolicyAdapter(&manifest_a.shares);

    let decision = check_delegation_within_envelope("store:notes:read", &target, &view);
    assert_eq!(decision, DelegationDecision::Permitted);
}

#[test]
fn delegation_within_envelope_admits_only_the_specified_rule_not_others() {
    // SUBSTANTIVE boundary per pim-2 §3.6b sub-rule 4: rule for
    // "store:notes:read" does NOT bleed into a different cap. Would-
    // FAIL if envelope check returned Permitted for caps not named in
    // the rules vector.
    let target = stub_plugin_did();
    let manifest_a = benten_platform_foundation::PluginManifest {
        plugin_name: "a".to_string(),
        content_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        peer_did: benten_id::did::Did::from_string_for_test_fixture(
            "did:key:z6MkAuthorMatch".to_string(),
        ),
        peer_signature: vec![0u8; 64],
        requires: vec![benten_platform_foundation::CapRequirement::new(
            "store:notes:read",
        )],
        shares: SharesPolicy {
            default: SharesPolicyDefault::Matching,
            rules: Some(vec![SharesRule {
                cap_pattern: "store:notes:read".to_string(),
                target: SharesTarget::PluginDid(target.clone()),
            }]),
        },
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    let view = PolicyAdapter(&manifest_a.shares);

    // Other cap NOT in rules => OutsideEnvelope.
    let decision_other = check_delegation_within_envelope("store:notes:write", &target, &view);
    assert_eq!(decision_other, DelegationDecision::OutsideEnvelope);
}

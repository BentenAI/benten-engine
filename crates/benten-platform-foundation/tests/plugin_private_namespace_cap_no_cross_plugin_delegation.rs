//! G24-D row + T7 defense — private namespace cap NO cross-plugin
//! delegation.
//!
//! Per docs/PLUGIN-MANIFEST.md §3.2: "any cap-scope of shape
//! `private:<plugin_did>:*` is implicitly `shares: none` regardless of
//! declared policy". `crates/benten-caps/src/plugin_delegation.rs`
//! enforces via `check_delegation_within_envelope` returning
//! `PrivateNamespaceForbidden` regardless of policy answer.
//!
//! Per pim-2-amendment §3.6b sub-rule 4: this pin is finer-grained
//! than the umbrella `private_namespace_cap_unconditionally_denied_cross_plugin`
//! — it specifically exercises the per-finding "declared shares:any
//! does NOT override private:*" structural guarantee with a granular
//! plugin-DID-keyed namespace.

mod common;

use benten_caps::plugin_delegation::{
    DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
    is_private_namespace_cap,
};
use benten_errors::ErrorCode;
use benten_platform_foundation::{CapRequirement, SharesPolicy, SharesPolicyDefault};
use common::manifest_fixtures::{minimal_manifest, stub_plugin_did};

struct PolicyAdapter<'a>(&'a SharesPolicy);

impl<'a> SharesPolicyView for PolicyAdapter<'a> {
    fn permits(&self, cap_pattern: &str, target_plugin_did: &benten_id::did::Did) -> bool {
        self.0.permits_delegation(cap_pattern, target_plugin_did)
    }
}

#[test]
fn private_namespace_cap_rejects_cross_plugin_delegation_even_with_shares_any() {
    // SUBSTANTIVE per pim-2 §3.6b + pim-2-amendment sub-rule 4:
    // construct a manifest that ATTEMPTS to declare shares: any for a
    // private-namespace cap; ASSERT rejection at check_delegation_
    // within_envelope is `PrivateNamespaceForbidden` regardless. The
    // private-namespace check is STRUCTURAL — must precede the policy
    // consultation. Would-FAIL if the rule check ordered policy
    // BEFORE the private-namespace short-circuit.
    let owner_did = stub_plugin_did();
    let target = benten_id::did::Did::from_string_unchecked("did:key:z6MkOtherPlugin".to_string());

    // Manifest declares shares: Any for a private:<owner_did>:* cap.
    let mut m = minimal_manifest();
    let private_scope = format!("private:{}:notes_state", owner_did.as_str());
    m.requires = vec![CapRequirement {
        scope: private_scope.clone(),
    }];
    m.shares = SharesPolicy {
        default: SharesPolicyDefault::Any,
        rules: None,
    };

    // Structural pre-check: detector recognizes private:*.
    assert!(
        is_private_namespace_cap(&private_scope),
        "private:* scope MUST be detected"
    );

    let view = PolicyAdapter(&m.shares);
    let decision = check_delegation_within_envelope(&private_scope, &target, &view);

    // SUBSTANTIVE: PrivateNamespaceForbidden even though shares: any
    // (the structural defense per T7).
    assert_eq!(
        decision,
        DelegationDecision::PrivateNamespaceForbidden,
        "private:* scope MUST be PrivateNamespaceForbidden even with \
         shares: any — structural rule per T7"
    );

    // Typed-error coupling: into_result threads the typed error.
    let err = decision
        .into_result()
        .expect_err("PrivateNamespaceForbidden -> Err");
    assert_eq!(err, ErrorCode::PluginPrivateNamespaceDelegationForbidden);
}

#[test]
fn non_private_scope_is_not_treated_as_private_namespace_boundary() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: a non-private cap
    // (`store:*`) is NOT private-namespace; the check returns the
    // policy answer (Permitted under shares: any). Would-FAIL if the
    // private-namespace detector over-matched (e.g., misclassified
    // `store:...` as private).
    let m = minimal_manifest();
    let target = stub_plugin_did();
    let view = PolicyAdapter(&SharesPolicy {
        default: SharesPolicyDefault::Any,
        rules: None,
    });
    let _ = m; // boundary pin uses fresh policy view
    assert!(!is_private_namespace_cap("store:notes:read"));

    let decision = check_delegation_within_envelope("store:notes:read", &target, &view);
    assert_eq!(decision, DelegationDecision::Permitted);
}

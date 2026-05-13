//! G24-D row + T7 defense — private namespace cap NO cross-plugin
//! delegation.
//!
//! Per docs/PLUGIN-MANIFEST.md §3.2: "any cap-scope of shape
//! `private:<plugin_did>:*` is implicitly `shares: none` regardless of
//! declared policy". `crates/benten-caps/src/private_namespace_policy.rs`
//! enforces.

mod common;

use benten_platform_foundation::{CapRequirement, SharesPolicy, SharesPolicyDefault};
use common::manifest_fixtures::{minimal_manifest, stub_plugin_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn private_namespace_cap_rejects_cross_plugin_delegation_even_with_shares_any() {
    // Construct a manifest that ATTEMPTS to declare shares: any
    // for a private namespace cap. Defense must reject delegation
    // regardless of the policy declaration — the rule is structural.
    let plugin_did = stub_plugin_did();
    let mut m = minimal_manifest();
    m.requires = vec![CapRequirement {
        scope: format!("private:{}:notes_state", plugin_did.as_str()),
    }];
    m.shares = SharesPolicy {
        default: SharesPolicyDefault::Any,
        rules: None,
    };

    // Future surface: even with shares: any, delegate_cap on a
    // private:<plugin_did>:* scope is rejected with
    // ErrorCode::PluginPrivateNamespaceDelegationForbidden.
    //
    // FAILS-IF-NO-OP because the private-namespace check is the
    // structural defense per T7.
    panic!(
        "RED-PHASE: G24-D wave must wire private_namespace_policy::reject_cross_plugin_delegation"
    );
}

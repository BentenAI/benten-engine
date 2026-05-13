//! G24-D-FP-1 row pin — uninstall_plugin cascade.
//!
//! Per plan §3 G24-D-FP-1: cascade-revoke + private-NS teardown +
//! library-entry removal at `crates/benten-platform-foundation/src/
//! plugin_lifecycle.rs::uninstall_plugin`.
//!
//! Acceptance: enumerate all user-DID-issued grants WHERE
//! `audience=plugin-DID`; revoke each. Cascade plugin-DID's own
//! downstream UCAN delegations. Terminate live subscriptions. Delete
//! private namespace data. Remove from manifest store. Remove library
//! entry. Emit PluginUninstalled change-event.

mod common;

use common::manifest_fixtures::{stub_plugin_did, stub_user_did};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn uninstall_cascade_revokes_user_grants_with_audience_equals_plugin_did() {
    let _plugin = stub_plugin_did();
    let _user = stub_user_did();

    // Future surface: plugin_lifecycle::uninstall_plugin(plugin_did:
    // &Did, engine: &mut Engine) -> Result. Walks user-DID-issued
    // UCAN grants where audience = plugin_did; revokes each via
    // Engine::revoke_capability_by_grant_cid (shipped at PR #199).
    //
    // FAILS-IF-NO-OP because after uninstall, the cap-store should
    // be empty of any grants with that audience.
    panic!("RED-PHASE: G24-D-FP-1 must wire uninstall_plugin cascade-revoke");
}

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn uninstall_cascade_revokes_plugin_did_downstream_ucan_delegations() {
    let _plugin = stub_plugin_did();

    // Future surface: uninstall_plugin also walks grants WHERE issuer
    // = plugin_did; revokes each. This cascades the plugin's own
    // delegations (e.g., plugin A -> plugin B delegations) per
    // CLAUDE.md #18 Layer 3.
    //
    // FAILS-IF-NO-OP because plugin-issued grants must also be
    // revoked or downstream plugins retain stale caps.
    panic!("RED-PHASE: G24-D-FP-1 must wire cascade of plugin-DID downstream delegations");
}

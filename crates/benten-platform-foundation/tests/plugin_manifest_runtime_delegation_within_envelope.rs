//! G24-D row test pin — runtime delegation WITHIN manifest envelope.
//!
//! CLAUDE.md #18 Layer 3 — plugin A delegates UCAN to plugin B; A's
//! manifest `shares` policy permits; chain validates.

mod common;

use benten_platform_foundation::SharesTarget;
use common::manifest_fixtures::{manifest_with_shares_matching_rule, stub_plugin_did};

#[test]
#[ignore = "RED-PHASE: G24-D wave wires plugin_delegation chain validation; un-ignore at G24-D landing"]
fn plugin_delegation_within_envelope_admitted_by_manifest_envelope_chain_validator() {
    let _manifest_a = manifest_with_shares_matching_rule(
        "store:notes:read",
        SharesTarget::PluginDid(stub_plugin_did()),
    );

    // Future surface: `crates/benten-caps/src/plugin_delegation.rs::
    // delegate_cap(from: PluginDid, to: PluginDid, cap_scope: &str,
    // source_manifest: &PluginManifest)` -> Result. Inspects
    // source_manifest.shares; admits delegation if cap_scope matches
    // any rule's cap_pattern AND target permits to-plugin.
    //
    // FAILS-IF-NO-OP because validator must consult source manifest's
    // shares policy.
    panic!(
        "RED-PHASE: G24-D wave must wire plugin_delegation::delegate_cap consulting source manifest envelope"
    );
}

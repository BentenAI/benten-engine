//! G24-D row test pin — runtime delegation OUTSIDE manifest envelope.
//!
//! CLAUDE.md #18 Layer 3 — plugin A attempts to delegate cap to plugin
//! B but A's manifest shares = none; chain validator REJECTS with
//! `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE`.

mod common;

use common::manifest_fixtures::minimal_manifest;

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn plugin_delegation_outside_envelope_denied_with_e_plugin_delegation_outside_manifest_envelope() {
    let _manifest_a = minimal_manifest(); // shares: none

    // Future surface: delegate_cap fails because manifest.shares =
    // SharesPolicyDefault::None. The cap chain validator returns
    // ErrorCode::PluginDelegationOutsideManifestEnvelope (new
    // ErrorCode minted atomically Rust+TS per §3.5g at G24-D).
    //
    // FAILS-IF-NO-OP because no-op validator would admit any chain
    // that signature-verifies (T8 attack class regression).
    panic!(
        "RED-PHASE: G24-D wave must wire E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE rejection"
    );
}

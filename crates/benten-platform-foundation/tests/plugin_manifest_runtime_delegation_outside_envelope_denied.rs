//! G24-D row test pin — runtime delegation OUTSIDE manifest envelope.
//!
//! CLAUDE.md #18 Layer 3 — plugin A attempts to delegate cap to plugin
//! B but A's manifest shares = none; chain validator REJECTS with
//! `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE`. Per pim-2-amendment
//! §3.6b sub-rule 4: this pin exercises the NEGATIVE outside-envelope
//! arm specifically (sibling pin within-envelope-admitted covers the
//! POSITIVE arm).

mod common;

use benten_caps::plugin_delegation::{
    DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
};
use benten_errors::ErrorCode;
use benten_platform_foundation::SharesPolicy;
use common::manifest_fixtures::{minimal_manifest, stub_plugin_did};

struct PolicyAdapter<'a>(&'a SharesPolicy);

impl<'a> SharesPolicyView for PolicyAdapter<'a> {
    fn permits(&self, cap_pattern: &str, target_plugin_did: &benten_id::did::Did) -> bool {
        self.0.permits_delegation(cap_pattern, target_plugin_did)
    }
}

#[test]
fn delegation_outside_envelope_surfaces_typed_outside_envelope_error_code() {
    // SUBSTANTIVE per pim-2 §3.6b + pim-2-amendment sub-rule 4:
    // manifest with shares: none denies delegation; decision converts
    // to typed ErrorCode::PluginDelegationOutsideManifestEnvelope.
    // Would-FAIL if a no-op validator admitted any chain that
    // signature-verified (T8 attack class regression).
    let manifest_a = minimal_manifest(); // shares: none by minimal_manifest
    let target = stub_plugin_did();
    let view = PolicyAdapter(&manifest_a.shares);

    let decision = check_delegation_within_envelope("store:notes:read", &target, &view);
    assert_eq!(
        decision,
        DelegationDecision::OutsideEnvelope,
        "shares=none MUST surface OutsideEnvelope"
    );

    // Substantive typed-error coupling: into_result returns the typed
    // ErrorCode (would-FAIL if the conversion arm was wrong).
    let err = decision.into_result().expect_err("OutsideEnvelope -> Err");
    assert_eq!(err, ErrorCode::PluginDelegationOutsideManifestEnvelope);
}

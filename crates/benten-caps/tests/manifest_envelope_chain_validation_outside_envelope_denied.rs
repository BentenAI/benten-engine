//! G24-D-FP-2 LOAD-BEARING pin — denies chains that signature-verify
//! cleanly but don't fit source manifest's shares policy.
//!
//! Per T8 defense narrative (sec-4f-r1-3): regression case where
//! cap-backend validates UCAN signatures, forgets to check manifest
//! envelope, admits a delegation outside the envelope.
//!
//! Per pim-2 §3.6b + pim-18 §3.6f: load-bearing SUBSTANTIVE pin —
//! drives production-source arm with actual hostile chain, asserts
//! the typed `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE` surfaces.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use benten_caps::manifest_envelope_chain_validation::{
    ChainValidationOutcome, DelegationStep, ManifestEnvelopeLookup, UserDidRegistry,
    validate_chain_with_manifest_envelope,
};
use benten_caps::plugin_delegation::SharesPolicyView;
use benten_errors::ErrorCode;
use benten_id::did::Did;

struct NonePermit;
impl SharesPolicyView for NonePermit {
    fn permits(&self, _cap: &str, _target: &Did) -> bool {
        false
    }
}

struct DenyAllLookup {
    known: HashSet<String>,
}
impl ManifestEnvelopeLookup for DenyAllLookup {
    type View<'a>
        = &'a NonePermit
    where
        Self: 'a;

    fn lookup<'a>(&'a self, plugin_did: &Did) -> Option<Self::View<'a>> {
        static NONE_P: NonePermit = NonePermit;
        if self.known.contains(plugin_did.as_str()) {
            Some(&NONE_P)
        } else {
            None
        }
    }
}

struct UserRegistry {
    users: HashSet<String>,
}
impl UserDidRegistry for UserRegistry {
    fn is_user_did(&self, did: &Did) -> bool {
        self.users.contains(did.as_str())
    }
}

#[test]
fn ucan_chain_outside_manifest_envelope_denied_load_bearing() {
    let user = Did::from_string_for_test_fixture("did:key:z6MkUser".into());
    let plugin_a = Did::from_string_for_test_fixture("did:key:z6MkPluginA".into());
    let plugin_b = Did::from_string_for_test_fixture("did:key:z6MkPluginB".into());

    // Hostile chain: plugin-A's manifest installed but `shares=None`
    // for the requested cap. Plugin-A would-be-delegation to plugin-B
    // MUST be rejected at the envelope boundary.
    let chain = vec![
        DelegationStep {
            issuer_did: user.clone(),
            audience_did: plugin_a.clone(),
            cap_pattern: "store:notes:write".into(),
        },
        DelegationStep {
            issuer_did: plugin_a.clone(),
            audience_did: plugin_b,
            cap_pattern: "store:notes:write".into(),
        },
    ];

    let mut known = HashSet::new();
    known.insert(plugin_a.as_str().to_string());
    let lookup = DenyAllLookup { known };

    let mut users = HashSet::new();
    users.insert(user.as_str().to_string());
    let registry = UserRegistry { users };

    let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &registry);
    match outcome.clone() {
        ChainValidationOutcome::StepOutsideEnvelope {
            issuer_did,
            cap_pattern,
        } => {
            assert_eq!(
                issuer_did, plugin_a,
                "T8 defense: must surface the offending issuer plugin-DID"
            );
            assert_eq!(
                cap_pattern, "store:notes:write",
                "T8 defense: must surface the offending cap pattern"
            );
        }
        other => panic!(
            "T8 defense (sec-4f-r1-3): chain outside source manifest envelope MUST \
             be rejected with StepOutsideEnvelope; got {other:?}"
        ),
    }

    // OBSERVABLE: typed-error mapping is the documented Layer-3 deny.
    let err = outcome.into_result().expect_err("outside envelope denies");
    assert_eq!(
        err,
        ErrorCode::PluginDelegationOutsideManifestEnvelope,
        "T8 defense: typed error must be E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE"
    );
}

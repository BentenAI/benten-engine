//! G24-D-FP-2 pin — manifest-envelope chain-validator seam admits
//! a chain whose every delegation step fits the source manifest's
//! shares policy.
//!
//! Surface: `crates/benten-caps/src/manifest_envelope_chain_validation.rs`.
//! Per plan §3.5.1 G24-D-FP-2 acceptance tests + T8 defense.
//!
//! ## Substantive arm
//!
//! Build a 2-step delegation chain user → plugin-A → plugin-B where
//! plugin-A's manifest `shares` policy admits the delegation. Walk via
//! `validate_chain_with_manifest_envelope`. Assert `Admitted`.
//!
//! WOULD-FAIL-IF-NO-OP because a no-op stub that returned
//! `RootNotUserDid` / `Empty` / panics would not produce `Admitted`.

#![allow(clippy::unwrap_used)]

use std::collections::{HashMap, HashSet};

use benten_caps::manifest_envelope_chain_validation::{
    ChainValidationOutcome, DelegationStep, ManifestEnvelopeLookup, UserDidRegistry,
    validate_chain_with_manifest_envelope,
};
use benten_caps::plugin_delegation::SharesPolicyView;
use benten_id::did::Did;

struct AllPermit;
impl SharesPolicyView for AllPermit {
    fn permits(&self, _cap: &str, _target: &Did) -> bool {
        true
    }
}

struct StaticLookup {
    permits_for: HashSet<String>,
}

impl ManifestEnvelopeLookup for StaticLookup {
    type View<'a>
        = &'a AllPermit
    where
        Self: 'a;

    fn lookup<'a>(&'a self, plugin_did: &Did) -> Option<Self::View<'a>> {
        static ALL: AllPermit = AllPermit;
        if self.permits_for.contains(plugin_did.as_str()) {
            Some(&ALL)
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
fn ucan_chain_within_manifest_envelope_admitted_at_chain_validator() {
    let user = Did::from_string_for_test_fixture("did:key:z6MkUser".into());
    let plugin_a = Did::from_string_for_test_fixture("did:key:z6MkPluginA".into());
    let plugin_b = Did::from_string_for_test_fixture("did:key:z6MkPluginB".into());

    let chain = vec![
        DelegationStep {
            issuer_did: user.clone(),
            audience_did: plugin_a.clone(),
            cap_pattern: "store:notes:write".into(),
        },
        DelegationStep {
            issuer_did: plugin_a.clone(),
            audience_did: plugin_b.clone(),
            cap_pattern: "store:notes:write".into(),
        },
    ];

    let mut permits_for = HashSet::new();
    permits_for.insert(plugin_a.as_str().to_string());
    let lookup = StaticLookup { permits_for };

    let mut users = HashSet::new();
    users.insert(user.as_str().to_string());
    let registry = UserRegistry { users };

    let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &registry);
    assert_eq!(
        outcome,
        ChainValidationOutcome::Admitted,
        "T8 defense: chain whose every step fits source manifest envelope MUST be admitted"
    );

    // Boundary: convert-to-result is Ok(()).
    assert!(outcome.into_result().is_ok());
}

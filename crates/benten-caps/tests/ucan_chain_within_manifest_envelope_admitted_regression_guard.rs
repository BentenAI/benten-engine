//! G24-D-FP-2 regression-guard — false-positive guard.
//!
//! Per T8 defense plan: ensure the envelope check doesn't over-reject.
//! A chain where every delegation step DOES fit the source plugin's
//! manifest `shares` policy MUST be admitted.
//!
//! **R4b-FP-1** un-ignored. Per L1 finding r4b-l1-5 + r4b-l1-8 this is
//! kept as a thin defense-in-depth shim alongside the substantive
//! sibling `manifest_envelope_chain_validation_within_envelope_admitted.rs`
//! — both surfaces exercise the admit path so a regression has TWO
//! independent fail-points to catch it.

#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

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
fn legitimate_chain_within_manifest_envelope_admitted_no_false_positive() {
    // R4b-FP-1 substantive — defense-in-depth false-positive guard.
    // Substance: build a 2-step user → A → B chain where A's manifest
    // shares policy permits (AllPermit); assert
    // validate_chain_with_manifest_envelope returns Admitted.
    //
    // Would-FAIL-IF-OVER-REJECT: a regression that adds envelope check
    // but breaks valid chains.
    let user = Did::from_string_unchecked("did:key:z6MkRegGuardUser".into());
    let plugin_a = Did::from_string_unchecked("did:key:z6MkRegGuardA".into());
    let plugin_b = Did::from_string_unchecked("did:key:z6MkRegGuardB".into());

    let chain = vec![
        DelegationStep {
            issuer_did: user.clone(),
            audience_did: plugin_a.clone(),
            cap_pattern: "store:notes:read".into(),
        },
        DelegationStep {
            issuer_did: plugin_a.clone(),
            audience_did: plugin_b.clone(),
            cap_pattern: "store:notes:read".into(),
        },
    ];

    let mut users = HashSet::new();
    users.insert(user.as_str().to_string());
    let user_reg = UserRegistry { users };

    let mut permits_for = HashSet::new();
    permits_for.insert(plugin_a.as_str().to_string());
    let lookup = StaticLookup { permits_for };

    let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &user_reg);
    assert_eq!(
        outcome,
        ChainValidationOutcome::Admitted,
        "legitimate within-envelope chain MUST be admitted — would-FAIL \
         if envelope check over-rejects (defense-in-depth false-positive guard)"
    );
}

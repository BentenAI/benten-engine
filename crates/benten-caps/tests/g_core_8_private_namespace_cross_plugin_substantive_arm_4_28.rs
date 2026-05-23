//! ADDL Phase-4-Meta-Core R3-B5 / TF-8 (benten-caps lane) — §4.28
//! private-namespace cross-plugin delegation policy SUBSTANTIVE arm
//! + §3.6e stranded-pin retarget.
//!
//! ## RED-PHASE — un-ignore at G-CORE-8
//!
//! §4.28: the stranded pins
//! `private_namespace_cross_plugin_delegation_denied.rs` +
//! `private_namespace_scope_prefix_canonicalization.rs` cite a
//! `private_namespace_policy::reject_cross_plugin` symbol that does
//! NOT exist at HEAD — the surface shipped under different names
//! (`benten_caps::validate_chain_with_manifest_envelope` +
//! `benten_caps::is_private_namespace_cap` +
//! `benten_caps::private_namespace_scope_admits_actor`). §4.28 / §3.6e
//! require these stranded pins be RE-TARGETED to the actually-shipped
//! surface (reviewer verifies landing-status, not just spec-pin
//! presence) at G-CORE-8 — NOT left citing a phantom symbol.
//!
//! This file is the retargeted SUBSTANTIVE arm written against the
//! shipped public surface. It is RED-phase because the §4.28
//! disposition (retarget-or-fold) lands at G-CORE-8; on un-ignore the
//! G-CORE-8 implementer either keeps this retargeted arm + DELETES the
//! two phantom-symbol stranded pins, or folds their case into the
//! `manifest_envelope_chain_validation` family per §4.28.
//!
//! ## §3.6g prior-phase pim-N pre-flight checklist (LITERAL):
//!   - pim-2-amendment (§3.6b sub-rule-4): exercises the SPECIFIC
//!     cross-plugin private-namespace refusal step (production
//!     `validate_chain_with_manifest_envelope` call-site, observable
//!     `ChainValidationOutcome` reject, would-FAIL if private-namespace
//!     scopes were delegable cross-plugin).
//!   - pim-12 (§3.6e): RED-PHASE staged-pin + the explicit
//!     stranded-phantom-symbol retarget obligation named above.
//!   - pim-18 (§3.6f): substantive arm against the shipped surface,
//!     NOT the phantom `private_namespace_policy::reject_cross_plugin`.
//!   - §3.13: no shared process-scoped static (discharged structurally).
//!
//! Pins: G-CORE-8 · C8 · couples §5.5 manifest-envelope-chain-validation.
//! R2 map: TF-8 §4.28 private-namespace cross-plugin substantive arm.

use benten_caps::is_private_namespace_cap;

#[test]
fn private_namespace_cap_is_recognized_by_shipped_surface() {
    // The SHIPPED classifier (`benten_caps::is_private_namespace_cap`)
    // is the canonical recognizer the §4.28 stranded pins must
    // retarget to (they currently cite the phantom
    // `private_namespace_policy::reject_cross_plugin`).
    //
    // A `private:<did>:*` scope IS a private-namespace cap.
    assert!(
        is_private_namespace_cap("private:did:key:z6MkPluginA:*"),
        "shipped surface: a private:<did>:* scope must be classified \
         a private-namespace cap (retarget anchor for §4.28)"
    );
    // A non-private store scope is NOT.
    assert!(
        !is_private_namespace_cap("store:notes:read"),
        "shipped surface: a non-private scope must NOT be classified \
         private-namespace"
    );

    // G-CORE-8 §4.28 LANDED: the substantive cross-plugin private-
    // namespace refusal arm is exercised through the SHIPPED
    // `validate_chain_with_manifest_envelope` chain validator — a
    // chain that delegates a `private:<plugin-A>:*` scope cross-plugin
    // (user → plugin-A → plugin-B audience) surfaces
    // `ChainValidationOutcome::PrivateNamespaceLeaked` which maps to
    // `ErrorCode::PluginPrivateNamespaceDelegationForbidden` via
    // `into_result`.
    //
    // The stranded `private_namespace_policy::reject_cross_plugin`
    // phantom-symbol pin (named in `docs/future/phase-4-backlog.md
    // §4.28`) is retargeted to this substantive arm + the
    // chain-validator's PrivateNamespaceLeaked path. The phantom-
    // symbol deletion (the legacy 2-pin retarget) is the §3.6e
    // wave-completion obligation; it lands as part of the §4.28
    // retarget per HARD-RULE-12 named-now disposition.
    use benten_caps::manifest_envelope_chain_validation::{
        ChainValidationOutcome, DelegationStep, ManifestEnvelopeLookup, UserDidRegistry,
        validate_chain_with_manifest_envelope,
    };
    use benten_caps::plugin_delegation::SharesPolicyView;
    use benten_id::did::Did;
    use std::collections::HashSet;

    struct AllPermit;
    impl SharesPolicyView for AllPermit {
        fn permits(&self, _cap: &str, _target: &Did) -> bool {
            true
        }
    }
    struct PermitLookup<'a> {
        view: &'a AllPermit,
        keys: HashSet<String>,
    }
    impl<'a> ManifestEnvelopeLookup for PermitLookup<'a> {
        type View<'b>
            = &'b AllPermit
        where
            Self: 'b;
        fn lookup<'b>(&'b self, plugin_did: &Did) -> Option<Self::View<'b>> {
            if self.keys.contains(plugin_did.as_str()) {
                Some(self.view)
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

    let user = Did::from_string_for_test_fixture("did:key:z6MkUserA".to_string());
    let plugin_a = Did::from_string_for_test_fixture("did:key:z6MkPluginA".to_string());
    let plugin_b = Did::from_string_for_test_fixture("did:key:z6MkPluginB".to_string());

    let mut users = HashSet::new();
    users.insert(user.as_str().to_string());
    let registry = UserRegistry { users };

    let mut keys = HashSet::new();
    keys.insert(plugin_a.as_str().to_string());
    let permit = AllPermit;
    let lookup = PermitLookup {
        view: &permit,
        keys,
    };

    // The attack: plugin-A's `private:<plugin-A>:*` scope is delegated
    // cross-plugin to plugin-B. Layer-3 envelope validation MUST reject
    // — private-namespace scopes are NOT cross-plugin delegatable per
    // CLAUDE.md baked-in #18.
    let attack_chain = vec![
        DelegationStep {
            issuer_did: user.clone(),
            audience_did: plugin_a.clone(),
            cap_pattern: format!("private:{}:*", plugin_a.as_str()),
        },
        DelegationStep {
            issuer_did: plugin_a.clone(),
            audience_did: plugin_b.clone(),
            cap_pattern: format!("private:{}:*", plugin_a.as_str()),
        },
    ];
    let outcome = validate_chain_with_manifest_envelope(&attack_chain, &lookup, &registry);
    assert!(
        matches!(
            outcome,
            ChainValidationOutcome::PrivateNamespaceLeaked { .. }
        ),
        "§4.28 G-CORE-8: cross-plugin delegation of a private-namespace \
         cap MUST surface PrivateNamespaceLeaked (CLAUDE.md baked-in #18 \
         private-namespace invariant). Got: {outcome:?}"
    );
    // The mapping to the typed code completes the substantive arm.
    let err_code = outcome.into_result().expect_err("MUST reject");
    assert_eq!(
        err_code,
        benten_errors::ErrorCode::PluginPrivateNamespaceDelegationForbidden,
        "§4.28 G-CORE-8: typed code mapping for the cross-plugin \
         private-namespace refusal"
    );
}

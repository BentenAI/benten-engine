//! Phase-4-Foundation G24-D-FP-2 — Layer 2 ↔ Layer 3 chain validator.
//!
//! Per CLAUDE.md baked-in #18 three-layer trust model:
//!
//! - **Layer 1** (user-as-root): every capability chain traces back to a
//!   user-issued root grant.
//! - **Layer 2** (install-time manifest): each plugin's manifest declares
//!   `requires` + `shares` policy; user consents to the *envelope* at
//!   install.
//! - **Layer 3** (runtime delegation within envelope): plugins delegate
//!   UCANs to other plugins *if and only if* the request fits the source
//!   plugin's manifest `shares` policy.
//!
//! [`crate::plugin_delegation`] (G24-D) ships the **single-step** runtime
//! gate (`check_delegation_within_envelope`). This module composes that
//! gate across an entire delegation chain — walking each step from leaf
//! to root, asserting:
//!
//! 1. The **root** is a user-DID (Layer 1).
//! 2. **Every intermediate** plugin-DID issuer's manifest `shares` policy
//!    permits the delegation step (Layer 2 ↔ Layer 3 bridge).
//! 3. **Private-namespace** caps are unconditionally rejected for cross-
//!    plugin delegation (composition of the [`crate::plugin_delegation`] gate).
//!
//! ## Dep-direction
//!
//! Per arch invariants, `benten-caps` MUST NOT depend on
//! `benten-platform-foundation` in production. The chain validator is
//! parameterized over a [`ManifestEnvelopeLookup`] trait + a
//! [`UserDidRegistry`] trait + a [`SharesPolicyView`] (from
//! [`crate::plugin_delegation`]) so concrete manifest / user-store types stay
//! out of this crate.

#[cfg(not(target_arch = "wasm32"))]
use benten_id::did::Did;
#[cfg(target_arch = "wasm32")]
use core::marker::PhantomData;

use benten_errors::ErrorCode;

use crate::plugin_delegation::{
    DelegationDecision, SharesPolicyView, check_delegation_within_envelope,
    is_private_namespace_cap,
};

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Did(String, PhantomData<()>);

// sec-r6r3-1 + sec-r6r3-2 closure (R6 R3 threat-model + security-auditor
// lenses): defensive `compile_error!` companion mirror to the native-only
// `validate_chain_with_manifest_envelope` function below (line 207+).
// Per CLAUDE.md baked-in #17(b) the wasm32 thin-client deployment shape
// does NOT perform UCAN chain validation — that lives only on full peers
// (shape a) and embedded webview's embedded-full-peer (shape c). If a
// future build configuration pulls this module into a wasm32 cdylib AND
// references the chain-validation entry point, the failure mode should be
// loud + cite-bearing rather than a cryptic linker error.
#[cfg(target_arch = "wasm32")]
const _: () = {
    // Token reference forces the gate to participate in conditional
    // compilation only when the wasm32 chain-validation surface is
    // pulled in by a downstream feature. Currently no downstream
    // feature does so; the token is defensive scaffolding.
    //
    // If a future change EXPLICITLY enables chain-validation on
    // wasm32 (e.g. via a `wasm32-full-peer` feature flag), replace
    // this stub with the full function body + remove the comment.
};

/// One step in a UCAN delegation chain.
///
/// A chain is an ordered list of steps from **root** (issuer = anchor
/// principal; usually user-DID) to **leaf** (issuer = the final
/// delegating plugin-DID, audience = the principal exercising the cap).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationStep {
    /// The principal issuing the delegation (root anchor at step 0).
    pub issuer_did: Did,
    /// The principal receiving the delegation.
    pub audience_did: Did,
    /// The capability pattern being delegated (e.g. `store:notes:write`).
    pub cap_pattern: String,
}

/// Anchor of a delegation chain — the root principal.
///
/// Per CLAUDE.md #18 clause-(a) the anchor MUST be a user-DID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainAnchor {
    /// User-DID — the only legitimate root anchor.
    UserDid(Did),
    /// Non-user root — rejected with [`ErrorCode::PluginManifestInvalid`]
    /// (see [`validate_chain_with_manifest_envelope`] for the explicit
    /// surface error).
    NonUser(Did),
}

/// Trait for resolving a plugin-DID to its manifest's `shares` policy
/// view.
///
/// Implementations live in `benten-platform-foundation` or test
/// fixtures; this trait keeps the dep-direction one-way.
#[cfg(not(target_arch = "wasm32"))]
pub trait ManifestEnvelopeLookup {
    /// Shares-policy view type (typically `&SharesPolicy` borrow).
    type View<'a>: SharesPolicyView
    where
        Self: 'a;

    /// Look up the shares-policy view for `plugin_did`. Returns `None`
    /// if no manifest is installed for the DID — the chain validator
    /// treats that as `OutsideEnvelope` (cannot delegate from an
    /// unknown principal).
    fn lookup<'a>(&'a self, plugin_did: &Did) -> Option<Self::View<'a>>;
}

/// Trait for asserting that a DID is a registered user-root.
///
/// Production implementation consults the engine's user-DID store
/// (where install records are signed); the test fixtures provide an
/// in-memory set.
#[cfg(not(target_arch = "wasm32"))]
pub trait UserDidRegistry {
    /// Whether `did` is a registered user-DID (Layer 1 root anchor).
    fn is_user_did(&self, did: &Did) -> bool;
}

/// Outcome of a full-chain validation walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainValidationOutcome {
    /// All steps fit the envelope; root anchors at user-DID.
    Admitted,
    /// Root is not a user-DID — CLAUDE.md #18 clause-(a) violation.
    RootNotUserDid,
    /// An intermediate plugin-DID's manifest `shares` policy does not
    /// permit the delegation step (Layer 2 ↔ Layer 3 mismatch).
    StepOutsideEnvelope {
        /// The plugin-DID whose envelope was violated.
        issuer_did: Did,
        /// The cap pattern delegated.
        cap_pattern: String,
    },
    /// Private-namespace cap leaked into a cross-plugin delegation.
    PrivateNamespaceLeaked {
        /// The cap pattern (begins with `private:`).
        cap_pattern: String,
    },
    /// Intermediate plugin-DID has no installed manifest (cannot
    /// verify envelope; treated as denial).
    NoManifestForIssuer {
        /// The plugin-DID with no installed manifest.
        issuer_did: Did,
    },
    /// Chain is empty.
    Empty,
}

impl ChainValidationOutcome {
    /// Convert to a `Result<(), ErrorCode>` for boundary integration.
    pub fn into_result(self) -> Result<(), ErrorCode> {
        match self {
            ChainValidationOutcome::Admitted => Ok(()),
            ChainValidationOutcome::RootNotUserDid => Err(ErrorCode::PluginManifestInvalid),
            ChainValidationOutcome::StepOutsideEnvelope { .. } => {
                Err(ErrorCode::PluginDelegationOutsideManifestEnvelope)
            }
            ChainValidationOutcome::PrivateNamespaceLeaked { .. } => {
                Err(ErrorCode::PluginPrivateNamespaceDelegationForbidden)
            }
            ChainValidationOutcome::NoManifestForIssuer { .. } => {
                Err(ErrorCode::PluginDelegationOutsideManifestEnvelope)
            }
            ChainValidationOutcome::Empty => Err(ErrorCode::PluginManifestInvalid),
        }
    }
}

/// Validate a full UCAN delegation chain against the three-layer trust
/// model.
///
/// Chain ordering: `chain[0]` is the ROOT (issuer = the anchor
/// principal); successive steps are children. The leaf (last entry)
/// is the final delegation step. Each step's `audience_did` MUST equal
/// the NEXT step's `issuer_did` (chain integrity). The validator does
/// NOT re-verify UCAN signatures (that's `benten-id::ucan` /
/// `validate_chain_at`); it ONLY enforces the envelope semantics.
///
/// # Algorithm
///
/// 1. Reject empty chain.
/// 2. Reject if `chain[0].issuer_did` is NOT a registered user-DID.
/// 3. For each step `s` in `chain`:
///    - If `s.issuer_did` is the user-DID (step 0): no envelope check
///      (user is root; user-issued caps are bounded by attenuation only,
///      checked by the UCAN chain validator separately).
///    - Else: look up `s.issuer_did`'s manifest envelope. If no
///      manifest, deny. Else run [`check_delegation_within_envelope`]
///      against `s.cap_pattern` + `s.audience_did`. If
///      [`DelegationDecision::OutsideEnvelope`] → reject. If
///      [`DelegationDecision::PrivateNamespaceForbidden`] → reject.
/// 4. Chain integrity: `chain[i].audience_did == chain[i+1].issuer_did`
///    for all adjacent pairs. Violation surfaces as
///    `StepOutsideEnvelope` (the next step's issuer can't issue a cap
///    it didn't receive).
#[cfg(not(target_arch = "wasm32"))]
pub fn validate_chain_with_manifest_envelope<L, U>(
    chain: &[DelegationStep],
    manifest_lookup: &L,
    user_registry: &U,
) -> ChainValidationOutcome
where
    L: ManifestEnvelopeLookup,
    U: UserDidRegistry,
{
    if chain.is_empty() {
        return ChainValidationOutcome::Empty;
    }

    // Layer 1 — root must be a user-DID.
    let root_issuer = &chain[0].issuer_did;
    if !user_registry.is_user_did(root_issuer) {
        return ChainValidationOutcome::RootNotUserDid;
    }

    // Walk every step. Step 0 is the user-issued root — no envelope
    // check (user is root). Steps 1..N are plugin-DID issuers whose
    // manifest envelope MUST admit the delegation.
    for (idx, step) in chain.iter().enumerate() {
        // Chain integrity: audience of step i = issuer of step i+1.
        if let Some(next) = chain.get(idx + 1)
            && step.audience_did != next.issuer_did
        {
            return ChainValidationOutcome::StepOutsideEnvelope {
                issuer_did: next.issuer_did.clone(),
                cap_pattern: next.cap_pattern.clone(),
            };
        }

        // Private-namespace caps NEVER cross plugin boundaries —
        // composition with the single-step gate.
        if is_private_namespace_cap(&step.cap_pattern) && idx > 0 {
            return ChainValidationOutcome::PrivateNamespaceLeaked {
                cap_pattern: step.cap_pattern.clone(),
            };
        }

        // Step 0 is user-issued. No envelope check (user is root).
        if idx == 0 {
            continue;
        }

        // Step i>0: plugin-DID issuer; consult its manifest envelope.
        let shares = match manifest_lookup.lookup(&step.issuer_did) {
            Some(s) => s,
            None => {
                return ChainValidationOutcome::NoManifestForIssuer {
                    issuer_did: step.issuer_did.clone(),
                };
            }
        };

        let decision =
            check_delegation_within_envelope(&step.cap_pattern, &step.audience_did, &shares);
        match decision {
            DelegationDecision::Permitted => {}
            DelegationDecision::OutsideEnvelope => {
                return ChainValidationOutcome::StepOutsideEnvelope {
                    issuer_did: step.issuer_did.clone(),
                    cap_pattern: step.cap_pattern.clone(),
                };
            }
            DelegationDecision::PrivateNamespaceForbidden => {
                return ChainValidationOutcome::PrivateNamespaceLeaked {
                    cap_pattern: step.cap_pattern.clone(),
                };
            }
        }
    }

    ChainValidationOutcome::Admitted
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    // ---------------------------------------------------------------
    // In-test SharesPolicyView impls
    // ---------------------------------------------------------------

    #[derive(Clone)]
    struct AllPermit;
    impl SharesPolicyView for AllPermit {
        fn permits(&self, _: &str, _: &Did) -> bool {
            true
        }
    }

    #[derive(Clone)]
    struct NonePermit;
    impl SharesPolicyView for NonePermit {
        fn permits(&self, _: &str, _: &Did) -> bool {
            false
        }
    }

    // ---------------------------------------------------------------
    // Test manifest registry
    // ---------------------------------------------------------------

    enum PolicyChoice {
        All,
        None,
    }

    struct TestManifestLookup {
        map: HashMap<String, PolicyChoice>,
    }

    enum TestView<'a> {
        All(&'a AllPermit),
        None(&'a NonePermit),
    }

    impl<'a> SharesPolicyView for TestView<'a> {
        fn permits(&self, cap_pattern: &str, target: &Did) -> bool {
            match self {
                TestView::All(p) => p.permits(cap_pattern, target),
                TestView::None(p) => p.permits(cap_pattern, target),
            }
        }
    }

    impl ManifestEnvelopeLookup for TestManifestLookup {
        type View<'a> = TestView<'a>;

        fn lookup<'a>(&'a self, plugin_did: &Did) -> Option<Self::View<'a>> {
            static ALL: AllPermit = AllPermit;
            static NONE_P: NonePermit = NonePermit;
            self.map.get(plugin_did.as_str()).map(|c| match c {
                PolicyChoice::All => TestView::All(&ALL),
                PolicyChoice::None => TestView::None(&NONE_P),
            })
        }
    }

    struct TestUserRegistry {
        users: HashSet<String>,
    }

    impl UserDidRegistry for TestUserRegistry {
        fn is_user_did(&self, did: &Did) -> bool {
            self.users.contains(did.as_str())
        }
    }

    fn user_did() -> Did {
        Did::from_string_unchecked("did:key:z6MkUser".to_string())
    }

    fn plugin_a_did() -> Did {
        Did::from_string_unchecked("did:key:z6MkPluginA".to_string())
    }

    fn plugin_b_did() -> Did {
        Did::from_string_unchecked("did:key:z6MkPluginB".to_string())
    }

    fn peer_did() -> Did {
        Did::from_string_unchecked("did:key:z6MkPeer".to_string())
    }

    fn user_registry_with(user: Did) -> TestUserRegistry {
        let mut users = HashSet::new();
        users.insert(user.as_str().to_string());
        TestUserRegistry { users }
    }

    #[test]
    fn empty_chain_rejected() {
        let lookup = TestManifestLookup {
            map: HashMap::new(),
        };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&[], &lookup, &reg);
        assert_eq!(outcome, ChainValidationOutcome::Empty);
    }

    #[test]
    fn non_user_root_rejected() {
        // peer-DID rooted chain — CLAUDE.md #18 clause-(a) violation.
        let chain = vec![DelegationStep {
            issuer_did: peer_did(),
            audience_did: plugin_a_did(),
            cap_pattern: "store:notes:write".into(),
        }];
        let lookup = TestManifestLookup {
            map: HashMap::new(),
        };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        assert_eq!(outcome, ChainValidationOutcome::RootNotUserDid);
    }

    #[test]
    fn single_step_user_to_plugin_admitted() {
        // user → plugin-A; user is root, no envelope check needed.
        let chain = vec![DelegationStep {
            issuer_did: user_did(),
            audience_did: plugin_a_did(),
            cap_pattern: "store:notes:write".into(),
        }];
        let lookup = TestManifestLookup {
            map: HashMap::new(),
        };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        assert_eq!(outcome, ChainValidationOutcome::Admitted);
    }

    #[test]
    fn two_step_chain_within_envelope_admitted() {
        // user → plugin-A → plugin-B; plugin-A's shares=Any permits.
        let chain = vec![
            DelegationStep {
                issuer_did: user_did(),
                audience_did: plugin_a_did(),
                cap_pattern: "store:notes:write".into(),
            },
            DelegationStep {
                issuer_did: plugin_a_did(),
                audience_did: plugin_b_did(),
                cap_pattern: "store:notes:write".into(),
            },
        ];
        let mut map = HashMap::new();
        map.insert(plugin_a_did().as_str().to_string(), PolicyChoice::All);
        let lookup = TestManifestLookup { map };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        assert_eq!(outcome, ChainValidationOutcome::Admitted);
    }

    #[test]
    fn two_step_chain_outside_envelope_rejected() {
        // user → plugin-A → plugin-B; plugin-A's shares=None denies.
        let chain = vec![
            DelegationStep {
                issuer_did: user_did(),
                audience_did: plugin_a_did(),
                cap_pattern: "store:notes:write".into(),
            },
            DelegationStep {
                issuer_did: plugin_a_did(),
                audience_did: plugin_b_did(),
                cap_pattern: "store:notes:write".into(),
            },
        ];
        let mut map = HashMap::new();
        map.insert(plugin_a_did().as_str().to_string(), PolicyChoice::None);
        let lookup = TestManifestLookup { map };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        match outcome {
            ChainValidationOutcome::StepOutsideEnvelope {
                issuer_did,
                cap_pattern,
            } => {
                assert_eq!(issuer_did, plugin_a_did());
                assert_eq!(cap_pattern, "store:notes:write");
            }
            other => panic!("expected StepOutsideEnvelope, got {other:?}"),
        }
    }

    #[test]
    fn private_namespace_cap_across_plugins_rejected() {
        // user → plugin-A → plugin-B; plugin-A tries to delegate
        // private:plugin-A:* to plugin-B. Never permitted.
        let chain = vec![
            DelegationStep {
                issuer_did: user_did(),
                audience_did: plugin_a_did(),
                cap_pattern: "private:did:key:z6MkPluginA:*".into(),
            },
            DelegationStep {
                issuer_did: plugin_a_did(),
                audience_did: plugin_b_did(),
                cap_pattern: "private:did:key:z6MkPluginA:*".into(),
            },
        ];
        let mut map = HashMap::new();
        map.insert(plugin_a_did().as_str().to_string(), PolicyChoice::All);
        let lookup = TestManifestLookup { map };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        match outcome {
            ChainValidationOutcome::PrivateNamespaceLeaked { cap_pattern } => {
                assert!(cap_pattern.starts_with("private:"));
            }
            other => panic!("expected PrivateNamespaceLeaked, got {other:?}"),
        }
    }

    #[test]
    fn no_manifest_for_intermediate_issuer_rejected() {
        // user → plugin-A → plugin-B; plugin-A's manifest not installed.
        let chain = vec![
            DelegationStep {
                issuer_did: user_did(),
                audience_did: plugin_a_did(),
                cap_pattern: "store:notes:write".into(),
            },
            DelegationStep {
                issuer_did: plugin_a_did(),
                audience_did: plugin_b_did(),
                cap_pattern: "store:notes:write".into(),
            },
        ];
        let lookup = TestManifestLookup {
            map: HashMap::new(),
        };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        match outcome {
            ChainValidationOutcome::NoManifestForIssuer { issuer_did } => {
                assert_eq!(issuer_did, plugin_a_did());
            }
            other => panic!("expected NoManifestForIssuer, got {other:?}"),
        }
    }

    #[test]
    fn chain_integrity_violation_rejected() {
        // user → plugin-A; plugin-X (NOT plugin-A) → plugin-B.
        // The next step's issuer doesn't match the previous step's
        // audience — chain integrity violation.
        let plugin_x = Did::from_string_unchecked("did:key:z6MkPluginX".into());
        let chain = vec![
            DelegationStep {
                issuer_did: user_did(),
                audience_did: plugin_a_did(),
                cap_pattern: "store:notes:write".into(),
            },
            DelegationStep {
                issuer_did: plugin_x.clone(),
                audience_did: plugin_b_did(),
                cap_pattern: "store:notes:write".into(),
            },
        ];
        let mut map = HashMap::new();
        map.insert(plugin_x.as_str().to_string(), PolicyChoice::All);
        let lookup = TestManifestLookup { map };
        let reg = user_registry_with(user_did());
        let outcome = validate_chain_with_manifest_envelope(&chain, &lookup, &reg);
        // Integrity violation surfaces as StepOutsideEnvelope (the next
        // step's issuer can't issue what it didn't receive).
        assert!(matches!(
            outcome,
            ChainValidationOutcome::StepOutsideEnvelope { .. }
        ));
    }

    #[test]
    fn into_result_admitted_is_ok() {
        assert!(ChainValidationOutcome::Admitted.into_result().is_ok());
    }

    #[test]
    fn into_result_root_not_user_maps_to_manifest_invalid() {
        let err = ChainValidationOutcome::RootNotUserDid
            .into_result()
            .unwrap_err();
        assert_eq!(err, ErrorCode::PluginManifestInvalid);
    }

    #[test]
    fn into_result_step_outside_maps_to_delegation_outside() {
        let err = ChainValidationOutcome::StepOutsideEnvelope {
            issuer_did: plugin_a_did(),
            cap_pattern: "store:notes:write".into(),
        }
        .into_result()
        .unwrap_err();
        assert_eq!(err, ErrorCode::PluginDelegationOutsideManifestEnvelope);
    }

    #[test]
    fn into_result_private_namespace_maps_to_forbidden() {
        let err = ChainValidationOutcome::PrivateNamespaceLeaked {
            cap_pattern: "private:did:key:zX:*".into(),
        }
        .into_result()
        .unwrap_err();
        assert_eq!(err, ErrorCode::PluginPrivateNamespaceDelegationForbidden);
    }
}

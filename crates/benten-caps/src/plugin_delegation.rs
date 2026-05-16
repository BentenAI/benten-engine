//! Phase-4-Foundation G24-D — runtime UCAN delegation within the
//! plugin manifest envelope.
//!
//! Per CLAUDE.md baked-in #18 Layer 3 (runtime delegation):
//!
//! > Plugins delegate UCANs to each other freely *if and only if* the
//! > request fits the source plugin's manifest `shares` policy. The
//! > `CapabilityPolicy` backend validates the chain at access-time:
//! > chain traces to user-root + each delegation step fits source
//! > plugin's policy + requested cap is within attenuation envelope.
//!
//! This module hosts the **runtime delegation check** —
//! `check_delegation_within_envelope` — that fires at the moment one
//! plugin tries to delegate to another. The Layer-2-↔-Layer-3 chain
//! validator (full UCAN-chain walk with envelope re-check at each
//! hop) lands as `manifest_envelope_chain_validation.rs` at G24-D-FP-2.
//!
//! Private-namespace caps (`private:<plugin_did>:*`) are
//! unconditionally denied for cross-plugin delegation — that check
//! fires here before consulting the manifest's `shares` policy.

#[cfg(not(target_arch = "wasm32"))]
use benten_id::did::Did;
#[cfg(target_arch = "wasm32")]
use core::marker::PhantomData;

use benten_errors::ErrorCode;

// On wasm32 we don't have `benten_id::Did` available (per crate
// dep-direction); declare a transparent shim so the function signatures
// compile on both targets. Production thin-clients don't run delegation
// checks; the path is full-peer-only.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Did(String, PhantomData<()>);

/// Whether a cap scope names a private namespace (`private:*`).
///
/// Private-namespace caps are ALWAYS denied cross-plugin delegation
/// regardless of the source manifest's `shares` policy — the engine
/// refuses to cross the namespace boundary. The `<plugin_did>` in
/// `private:<plugin_did>:*` is the namespace owner; only the owning
/// plugin (whose plugin-DID matches) may use the cap, and never
/// delegate it.
#[must_use]
pub fn is_private_namespace_cap(cap_scope: &str) -> bool {
    cap_scope.starts_with("private:")
}

/// Decision returned by `check_delegation_within_envelope`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationDecision {
    /// Delegation permitted — caller proceeds to issue the UCAN.
    Permitted,
    /// Delegation outside the source plugin's manifest envelope.
    OutsideEnvelope,
    /// Private-namespace cap — never delegable cross-plugin.
    PrivateNamespaceForbidden,
}

impl DelegationDecision {
    /// Convert to a `Result<()`/typed-`ErrorCode>` for the cap-policy
    /// boundary.
    pub fn into_result(self) -> Result<(), ErrorCode> {
        match self {
            DelegationDecision::Permitted => Ok(()),
            DelegationDecision::OutsideEnvelope => {
                Err(ErrorCode::PluginDelegationOutsideManifestEnvelope)
            }
            DelegationDecision::PrivateNamespaceForbidden => {
                Err(ErrorCode::PluginPrivateNamespaceDelegationForbidden)
            }
        }
    }
}

/// The minimal abstract shape of a manifest's `shares` policy that
/// this crate consumes. Defined here as a trait so consumers can
/// stub it in tests without dragging in the concrete
/// `SharesPolicy` shape.
///
/// G27-D introduced a target-conditional native-only prod dep edge
/// `benten-caps → benten-platform-foundation` (for `PluginManifest::
/// requires` walk at scope-derivation time per plan §3 G27-D). The
/// dep edge is Cargo-cycle-safe because the reverse direction
/// (`platform-foundation → benten-caps`) stays dev-only. With this
/// prod-edge direction available (batch-3 onwards), the blanket
/// `impl SharesPolicyView for SharesPolicy` ships in
/// `benten-platform-foundation::plugin_manifest` directly so
/// production callers can pass `&manifest.shares` without a per-test
/// `PolicyAdapter` newtype wrapper.
pub trait SharesPolicyView {
    /// Whether the policy permits delegating `cap_pattern` to
    /// `target_plugin_did`.
    fn permits(&self, cap_pattern: &str, target_plugin_did: &Did) -> bool;
}

// Reference-blanket impl so `&T: SharesPolicyView` when `T:
// SharesPolicyView`. Lets `ManifestEnvelopeLookup::View<'a>` borrows
// return references without duplicating impls.
#[cfg(not(target_arch = "wasm32"))]
impl<T: SharesPolicyView + ?Sized> SharesPolicyView for &T {
    fn permits(&self, cap_pattern: &str, target_plugin_did: &Did) -> bool {
        (*self).permits(cap_pattern, target_plugin_did)
    }
}

// Blanket impl per G24-D-FP-2 mr-1 closure: production + test callers
// pass `&manifest.shares` directly without a per-test `PolicyAdapter`
// newtype wrapper. Lands at batch-3 assembly because G27-D's prod
// dep edge `benten-caps → benten-platform-foundation` is now on the
// same branch tree (Cargo-cycle-safe: foundation declares caps as
// dev-only). Native-only mirroring the rest of plugin_delegation.
#[cfg(not(target_arch = "wasm32"))]
impl SharesPolicyView for benten_platform_foundation::SharesPolicy {
    fn permits(&self, cap_pattern: &str, target_plugin_did: &Did) -> bool {
        self.permits_delegation(cap_pattern, target_plugin_did)
    }
}

/// Whether a cap-resource string names the SANDBOX-execution
/// authority dimension.
///
/// COLLAPSE / #1241 (F2 capability-predicate completion): the literal
/// CLAUDE.md baked-in #17 thin-shape predicate is the inbound
/// principal's UCAN `cap.resource` — *"a `runs_sandbox=false`
/// principal must not exercise `host:sandbox:*`"*. This is the
/// resource-prefix routing test that discriminates that dimension.
/// It is a structural prefix match on the PUBLIC cap-schema (not a
/// secret compare), so ct-eq UNIFORMITY does not apply here (the
/// project's ct-eq commitment is for identity/secret compares; this
/// is cap-schema routing — same rationale as the device-envelope
/// seam's existing inline note).
#[must_use]
pub fn is_sandbox_exec_cap(cap_resource: &str) -> bool {
    cap_resource.starts_with("host:sandbox:")
}

/// COLLAPSE J8 — the ONE generalized envelope-ceiling predicate.
///
/// DECISION-RECORD-trust-model-reframe.md §4 (RATIFIED): the
/// device-attestation envelope is no longer a distinct trust-root; it
/// is one signed *ceiling* the single chain-validation seam ANDs into
/// the principal's effective authority. design-A-spine §2 + impl-design
/// §3 establish that the device envelope and the plugin-manifest
/// `shares` envelope are *the same abstraction*; per
/// build-constraint iii (DECISION-RECORD §4) the J8-caveat and the
/// #669 manifest ceiling-check are **ONE code path / one mechanism /
/// one mini-review / one closure pin** — NOT parallel pipes (recreating
/// parallelism is precisely the #707 / META-#1140 regression the whole
/// COLLAPSE exists to kill).
///
/// This enum is that one mechanism's input. Both COLLAPSE callers
/// route through [`ceiling_admits_cap`]:
/// - the inbound-sync seam (`benten_engine::manifest_envelope_recheck::
///   envelope_ceiling_admits_row`) supplies [`EnvelopeCeiling::Device`]
///   + the inbound writer's effective cap-resources;
/// - the plugin-manifest delegation gate
///   ([`check_delegation_within_envelope`], consumed by
///   `Engine::delegate_capability`) supplies
///   [`EnvelopeCeiling::PluginShares`].
#[cfg(not(target_arch = "wasm32"))]
pub enum EnvelopeCeiling<'a, P: SharesPolicyView + ?Sized> {
    /// A signed device `CapabilityEnvelope` ceiling. The `runs_sandbox`
    /// dimension is the load-bearing CLAUDE.md #17 thin-shape property:
    /// a `runs_sandbox=false` ceiling forbids any `host:sandbox:*`
    /// cap-resource. (Broader envelope dimensions — `holds_zones` /
    /// `runs_atrium_peer` — are construction-time guarded via
    /// `benten_id::device_attestation::DeviceAttestation::issue_with_runtime_check`
    /// + `envelope_widens`; the runtime ceiling-AND enforces the
    /// `runs_sandbox` execution-authority dimension a sync row's
    /// cap-resource can actually exercise.)
    Device {
        /// `true` iff the attested device may execute SANDBOX modules.
        runs_sandbox: bool,
    },
    /// A plugin-manifest `shares` policy ceiling. Admits a cap-resource
    /// iff (a) it is not a private-namespace cap AND (b) the source
    /// manifest's `shares` policy permits delegating it to
    /// `target_plugin_did`.
    PluginShares {
        /// The source plugin's `shares` policy view.
        source_shares: &'a P,
        /// The delegation target plugin-DID.
        target_plugin_did: &'a Did,
    },
}

/// The ONE generalized envelope-ceiling check (COLLAPSE J8 / #669 /
/// #1241). ANDs `cap_resource` against `ceiling`.
///
/// `Ok(())` ⇒ the ceiling admits a principal exercising `cap_resource`.
/// `Err(DelegationDecision)` ⇒ the ceiling forbids it (the variant
/// carries the precise reason for typed-error mapping at the boundary).
///
/// **#1241 / F2 capability-predicate completion:** for the device
/// ceiling this discriminates on the literal `cap_resource` (the
/// CLAUDE.md #17 predicate as written) — NOT a synthetic
/// `{zone}:write` zone-scope. A `runs_sandbox=false` ceiling rejects a
/// `host:sandbox:*` cap-resource *regardless of which zone the row
/// targets* (closes the sec-review-1238 F2 SHAPE-not-substance gap;
/// the prior P3 zone-scoped predicate was verified INERT at HEAD per
/// `F2-exploitability-investigation.md`, so this is strictly-more-
/// enforcement, not a regression).
///
/// **One mechanism, two callers** (build-constraint iii): both the
/// device-envelope sync seam and the plugin-manifest delegation gate
/// call THIS function — there is no parallel device-vs-manifest pipe.
#[cfg(not(target_arch = "wasm32"))]
pub fn ceiling_admits_cap<P: SharesPolicyView + ?Sized>(
    ceiling: &EnvelopeCeiling<'_, P>,
    cap_resource: &str,
) -> Result<(), DelegationDecision> {
    match ceiling {
        EnvelopeCeiling::Device { runs_sandbox } => {
            // J8 / CLAUDE.md #17: a `runs_sandbox=false`-attested
            // principal MUST NOT exercise `host:sandbox:*` — the literal
            // cap.resource predicate (#1241 completion). This is the
            // FIRST place the inbound-sync surface enforces it on the
            // writer's actual cap-resource rather than a zone proxy.
            if is_sandbox_exec_cap(cap_resource) && !*runs_sandbox {
                return Err(DelegationDecision::OutsideEnvelope);
            }
            Ok(())
        }
        EnvelopeCeiling::PluginShares {
            source_shares,
            target_plugin_did,
        } => {
            if is_private_namespace_cap(cap_resource) {
                return Err(DelegationDecision::PrivateNamespaceForbidden);
            }
            if source_shares.permits(cap_resource, target_plugin_did) {
                Ok(())
            } else {
                Err(DelegationDecision::OutsideEnvelope)
            }
        }
    }
}

/// Check whether a runtime delegation step is within the source
/// plugin's manifest envelope.
///
/// Returns `Permitted` if:
/// 1. The cap is NOT a private-namespace cap, AND
/// 2. The source plugin's manifest `shares` policy permits delegating
///    `cap_pattern` to `target_plugin_did`.
///
/// Returns the corresponding deny code otherwise.
///
/// **COLLAPSE #669 unification:** this is now a thin caller of the ONE
/// [`ceiling_admits_cap`] mechanism with an
/// [`EnvelopeCeiling::PluginShares`] ceiling — the SAME function the
/// device-envelope inbound-sync seam calls (build-constraint iii). The
/// plugin-manifest envelope decision is no longer a parallel pipe; it
/// is a second caller of the unified ceiling-check.
#[cfg(not(target_arch = "wasm32"))]
pub fn check_delegation_within_envelope<P: SharesPolicyView>(
    cap_pattern: &str,
    target_plugin_did: &Did,
    source_shares: &P,
) -> DelegationDecision {
    let ceiling = EnvelopeCeiling::PluginShares {
        source_shares,
        target_plugin_did,
    };
    match ceiling_admits_cap(&ceiling, cap_pattern) {
        Ok(()) => DelegationDecision::Permitted,
        Err(decision) => decision,
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::*;

    struct AllPermit;
    impl SharesPolicyView for AllPermit {
        fn permits(&self, _: &str, _: &Did) -> bool {
            true
        }
    }
    struct NonePermit;
    impl SharesPolicyView for NonePermit {
        fn permits(&self, _: &str, _: &Did) -> bool {
            false
        }
    }

    fn target_did() -> Did {
        Did::from_string_unchecked("did:key:z6MkTarget".to_string())
    }

    #[test]
    fn private_namespace_always_denied_regardless_of_policy() {
        let decision = check_delegation_within_envelope(
            "private:did:key:z6MkSomePlugin:*",
            &target_did(),
            &AllPermit,
        );
        assert_eq!(decision, DelegationDecision::PrivateNamespaceForbidden);
    }

    #[test]
    fn non_private_cap_consults_policy_permitted() {
        let decision =
            check_delegation_within_envelope("store:notes:read", &target_did(), &AllPermit);
        assert_eq!(decision, DelegationDecision::Permitted);
    }

    #[test]
    fn non_private_cap_consults_policy_denied() {
        let decision =
            check_delegation_within_envelope("store:notes:read", &target_did(), &NonePermit);
        assert_eq!(decision, DelegationDecision::OutsideEnvelope);
    }

    // ---- COLLAPSE J8 / #1241 / #669 unified ceiling-check ----

    #[test]
    fn unified_ceiling_device_runs_sandbox_false_rejects_sandbox_exec_cap_resource() {
        // #1241 / F2: the literal CLAUDE.md #17 predicate — the
        // inbound writer's *cap.resource* is `host:sandbox:*` and the
        // attested ceiling is `runs_sandbox=false`. MUST reject,
        // regardless of any zone proxy.
        let ceiling: EnvelopeCeiling<'_, AllPermit> = EnvelopeCeiling::Device {
            runs_sandbox: false,
        };
        let err = ceiling_admits_cap(&ceiling, "host:sandbox:exec")
            .expect_err("runs_sandbox=false ceiling MUST reject host:sandbox:* cap-resource");
        assert_eq!(err, DelegationDecision::OutsideEnvelope);
    }

    #[test]
    fn unified_ceiling_device_runs_sandbox_true_admits_sandbox_exec_cap_resource() {
        let ceiling: EnvelopeCeiling<'_, AllPermit> =
            EnvelopeCeiling::Device { runs_sandbox: true };
        ceiling_admits_cap(&ceiling, "host:sandbox:exec")
            .expect("runs_sandbox=true ceiling MUST admit host:sandbox:* cap-resource");
    }

    #[test]
    fn unified_ceiling_device_runs_sandbox_false_admits_ordinary_cap_resource() {
        // Negative control: the predicate is scoped to the
        // sandbox-execution dimension; it must not over-reject
        // ordinary cap-resources.
        let ceiling: EnvelopeCeiling<'_, AllPermit> = EnvelopeCeiling::Device {
            runs_sandbox: false,
        };
        ceiling_admits_cap(&ceiling, "store:notes:write")
            .expect("runs_sandbox=false ceiling MUST NOT block an ordinary cap-resource");
    }

    #[test]
    fn unified_ceiling_plugin_shares_is_second_caller_of_one_mechanism() {
        // #669: the plugin-manifest `shares` ceiling routes through
        // the SAME `ceiling_admits_cap` — proving one mechanism / two
        // callers (build-constraint iii; not a parallel pipe).
        let permit = EnvelopeCeiling::PluginShares {
            source_shares: &AllPermit,
            target_plugin_did: &target_did(),
        };
        ceiling_admits_cap(&permit, "store:notes:read").expect("AllPermit shares admits");

        let deny = EnvelopeCeiling::PluginShares {
            source_shares: &NonePermit,
            target_plugin_did: &target_did(),
        };
        assert_eq!(
            ceiling_admits_cap(&deny, "store:notes:read"),
            Err(DelegationDecision::OutsideEnvelope)
        );

        // Private-namespace caps are forbidden cross-plugin via the
        // same unified path.
        assert_eq!(
            ceiling_admits_cap(&permit, "private:did:key:z6MkX:*"),
            Err(DelegationDecision::PrivateNamespaceForbidden)
        );
    }

    #[test]
    fn check_delegation_within_envelope_is_thin_caller_of_unified_check() {
        // Regression: the legacy #669 entrypoint must keep its exact
        // semantics now that it delegates to the unified mechanism.
        assert_eq!(
            check_delegation_within_envelope("store:notes:read", &target_did(), &AllPermit),
            DelegationDecision::Permitted
        );
        assert_eq!(
            check_delegation_within_envelope("store:notes:read", &target_did(), &NonePermit),
            DelegationDecision::OutsideEnvelope
        );
        assert_eq!(
            check_delegation_within_envelope("private:did:key:z6MkP:*", &target_did(), &AllPermit),
            DelegationDecision::PrivateNamespaceForbidden
        );
    }
}

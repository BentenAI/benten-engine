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
/// G27-D introduced a target-conditional native-only prod dep on
/// `benten-platform-foundation` (for `PluginManifest::requires` walk
/// at scope-derivation time per plan §3 G27-D); the dep edge is
/// Cargo-cycle-safe because the reverse `platform-foundation →
/// benten-caps` direction stays dev-only. The platform-foundation
/// crate provides an `impl SharesPolicyView for SharesPolicy`
/// blanket at the manifest-envelope-chain-validation G24-D-FP-2 wave.
pub trait SharesPolicyView {
    /// Whether the policy permits delegating `cap_pattern` to
    /// `target_plugin_did`.
    fn permits(&self, cap_pattern: &str, target_plugin_did: &Did) -> bool;
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
#[cfg(not(target_arch = "wasm32"))]
pub fn check_delegation_within_envelope<P: SharesPolicyView>(
    cap_pattern: &str,
    target_plugin_did: &Did,
    source_shares: &P,
) -> DelegationDecision {
    if is_private_namespace_cap(cap_pattern) {
        return DelegationDecision::PrivateNamespaceForbidden;
    }
    if source_shares.permits(cap_pattern, target_plugin_did) {
        DelegationDecision::Permitted
    } else {
        DelegationDecision::OutsideEnvelope
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
}

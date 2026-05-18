//! Refinement-audit-2026-05 Wave-E HELD #1197/#1146 — Layer-3
//! plugin-manifest `shares`-envelope enforcement port for
//! [`crate::engine_caps::EngineCapsHandle::delegate_capability`].
//!
//! **What this seam does**
//!
//! CLAUDE.md baked-in #18 Layer 3 (runtime delegation within the
//! manifest envelope):
//!
//! > Plugins delegate UCANs to each other freely *if and only if* the
//! > request fits the source plugin's manifest `shares` policy. The
//! > `CapabilityPolicy` backend validates the chain at access-time:
//! > chain traces to user-root + each delegation step fits source
//! > plugin's policy + requested cap is within attenuation envelope.
//!
//! Pre-this-wave, `delegate_capability` consulted a hard-coded
//! `AllPermit` shim — every cross-plugin delegation was admitted
//! regardless of the source plugin's signed, user-consented manifest
//! `shares` policy. The single-step envelope check existed but its
//! policy view was always-true: Layer 3 was **paper-only at HEAD**
//! (META #669; Safe-3 #596). This port wires the *real* manifest
//! `shares` policy through to the production delegation path.
//!
//! **Dep-direction discipline (arch-r1-1 thinness)**
//!
//! `benten-engine` does NOT depend on `benten-platform-foundation` in
//! production (pinned by
//! `crates/benten-platform-foundation/tests/arch_n_benten_platform_foundation_dep_direction.rs`).
//! The engine therefore cannot look up a `PluginManifest` directly.
//! It consumes THIS port; the concrete implementation lives in the
//! engine-adapter crate that owns the plugin library (typically
//! `benten-platform-foundation`'s glue type that wraps `ManifestStore`
//! + manifest-by-CID resolution + `benten_caps::plugin_delegation`).
//! Mirrors the established
//! [`crate::manifest_envelope_recheck::ManifestEnvelopeRechecker`] port
//! pattern (Seam 3 at the sync merge boundary).
//!
//! **Fail-CLOSED contract**
//!
//! When a resolver IS installed and the source delegation principal is
//! a plugin-DID (i.e. NOT a user-root), the resolver MUST positively
//! `Admitted` the request. `Denied` and `NoManifest` BOTH reject the
//! delegation (no manifest for a plugin-principal ⇒ cannot verify the
//! envelope ⇒ deny — never fail-OPEN). Only an explicit `Admitted`,
//! or a `NotPluginPrincipal` classification (the source is the
//! user-root, which Layer 1 already anchors), permits the delegation.

use benten_errors::ErrorCode;

/// Outcome of resolving a single runtime delegation step against the
/// source principal's manifest `shares` policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DelegationResolution {
    /// The source principal is the user-root (or otherwise not a
    /// plugin-DID with a manifest envelope). Layer 1 already anchors
    /// user-issued caps; the manifest-envelope gate does not apply.
    /// Delegation proceeds (subject to the always-on private-namespace
    /// clause + attenuation, checked by the caller).
    NotPluginPrincipal,
    /// The source plugin's signed manifest `shares` policy permits
    /// delegating this cap to this target. Admit.
    Admitted,
    /// The source plugin's manifest `shares` policy does NOT permit
    /// this delegation. Reject (fail-CLOSED).
    Denied,
    /// The source principal looks like a plugin-DID but no installed,
    /// user-consented manifest could be resolved for it. The envelope
    /// cannot be verified — reject (fail-CLOSED; never admit an
    /// un-verifiable cross-plugin delegation).
    NoManifest,
}

impl DelegationResolution {
    /// Map to the boundary `Result`. `Admitted` / `NotPluginPrincipal`
    /// proceed; `Denied` / `NoManifest` reject with the typed
    /// [`ErrorCode::PluginDelegationOutsideManifestEnvelope`].
    ///
    /// # Errors
    ///
    /// [`ErrorCode::PluginDelegationOutsideManifestEnvelope`] on
    /// `Denied` or `NoManifest`.
    pub fn into_result(self) -> Result<(), ErrorCode> {
        match self {
            DelegationResolution::NotPluginPrincipal | DelegationResolution::Admitted => Ok(()),
            DelegationResolution::Denied | DelegationResolution::NoManifest => {
                Err(ErrorCode::PluginDelegationOutsideManifestEnvelope)
            }
        }
    }
}

/// Port the engine-adapter crate implements to drive Layer-3
/// manifest-`shares`-envelope enforcement from inside
/// [`crate::engine_caps::EngineCapsHandle::delegate_capability`].
///
/// The default implementation [`NoopSharesPolicyResolver`] classifies
/// every principal as [`DelegationResolution::NotPluginPrincipal`] —
/// observably identical to the pre-wave `AllPermit` behavior. Engines
/// built WITHOUT a configured resolver therefore behave exactly as
/// before; operators / the platform layer install a real adapter via
/// [`crate::engine::Engine::set_shares_policy_resolver`].
pub trait SharesPolicyResolver: Send + Sync {
    /// Resolve whether `source_principal_did` may delegate `cap_pattern`
    /// to `target_plugin_did` under its installed manifest `shares`
    /// policy.
    ///
    /// `source_principal_did` is the `actor` recorded on the resolved
    /// source `system:CapabilityGrant` Node — the principal that issued
    /// (or holds) the grant being delegated from.
    ///
    /// Implementations CONSULT the install-record / manifest store:
    /// - If the principal is the registered user-DID (Layer 1 root) →
    ///   [`DelegationResolution::NotPluginPrincipal`].
    /// - If the principal is a plugin-DID with an installed,
    ///   user-consented manifest → run the manifest `shares` policy and
    ///   return [`DelegationResolution::Admitted`] /
    ///   [`DelegationResolution::Denied`].
    /// - If the principal is a plugin-DID with NO resolvable manifest →
    ///   [`DelegationResolution::NoManifest`] (fail-CLOSED).
    fn resolve_delegation(
        &self,
        source_principal_did: &str,
        cap_pattern: &str,
        target_plugin_did: &str,
    ) -> DelegationResolution;
}

/// Default resolver — classifies every principal as
/// [`DelegationResolution::NotPluginPrincipal`].
///
/// Behavior is observably identical to the pre-Wave-E `AllPermit`
/// shim: when no real adapter is installed the manifest-`shares` gate
/// does not constrain delegation (the always-on private-namespace
/// clause + Layer-1 user-root anchor + attenuation still apply). A
/// production engine that hosts a plugin library installs a real
/// adapter so the Layer-3 gate fires.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopSharesPolicyResolver;

impl SharesPolicyResolver for NoopSharesPolicyResolver {
    fn resolve_delegation(
        &self,
        _source_principal_did: &str,
        _cap_pattern: &str,
        _target_plugin_did: &str,
    ) -> DelegationResolution {
        DelegationResolution::NotPluginPrincipal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_classifies_everything_not_plugin_principal() {
        let r = NoopSharesPolicyResolver;
        assert_eq!(
            r.resolve_delegation("did:key:zSource", "store:notes:read", "did:key:zTarget"),
            DelegationResolution::NotPluginPrincipal
        );
    }

    #[test]
    fn into_result_admit_paths_ok() {
        assert!(
            DelegationResolution::NotPluginPrincipal
                .into_result()
                .is_ok()
        );
        assert!(DelegationResolution::Admitted.into_result().is_ok());
    }

    #[test]
    fn into_result_deny_paths_fail_closed() {
        assert_eq!(
            DelegationResolution::Denied.into_result(),
            Err(ErrorCode::PluginDelegationOutsideManifestEnvelope)
        );
        // NoManifest is fail-CLOSED — an un-verifiable cross-plugin
        // delegation must reject, never admit.
        assert_eq!(
            DelegationResolution::NoManifest.into_result(),
            Err(ErrorCode::PluginDelegationOutsideManifestEnvelope)
        );
    }
}

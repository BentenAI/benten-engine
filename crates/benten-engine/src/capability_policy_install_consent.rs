//! **R6 R2 FP-B (L6-r6r2-l6-1 closure)** — engine-side adapter that
//! bridges a [`benten_caps::CapabilityPolicy`] to
//! [`benten_platform_foundation::install_consent::InstallConsentPolicy`].
//!
//! ## Why this adapter lives here (not in `benten-platform-foundation`)
//!
//! Per [`benten_platform_foundation::install_consent`] module-doc:
//! `benten-caps` already depends on `benten-platform-foundation` (G27-D
//! manifest-aware scope derivation), so the reverse dependency would
//! create a cycle. The cycle-safe shape: define the narrow per-hook
//! trait in `benten-platform-foundation` and the engine-side adapter
//! ("`CapabilityPolicyInstallConsent`") in the engine crate which
//! already depends on both. This file is that adapter.
//!
//! ## Wire-in
//!
//! `crate::production_engine_builder::ProductionEngineBuilder` threads a `CapabilityPolicyInstallConsent`
//! wrapping the configured `CapabilityPolicy` into the install pipeline's
//! Step 3c so a custom policy's `check_install_consent` is honored at
//! runtime (the L6-r6r2-l6-1 finding's substantive close — pre-FP-B the
//! adapter was named in the module-doc but did not exist).
//!
//! ## Sealed-by-convention
//!
//! Per CLAUDE.md baked-in commitment #7 (sealed-discipline refinement),
//! the [`benten_platform_foundation::install_consent::InstallConsentPolicy`]
//! trait is sealed-by-convention; this adapter is the canonical
//! engine-side impl + the typical consumer route.

use std::sync::Arc;

use benten_caps::CapabilityPolicy;
use benten_errors::ErrorCode;
use benten_platform_foundation::install_consent::InstallConsentPolicy;

/// Engine-side adapter wrapping a `CapabilityPolicy` to honor the
/// install-pipeline's [`InstallConsentPolicy`] port.
///
/// The wrapped policy's
/// [`CapabilityPolicy::check_install_consent`] is called on every
/// install-attempt; any `CapError` denial maps to the typed
/// [`ErrorCode::PluginInstallConsentDenied`] (the install pipeline's
/// canonical denial code per §S3a's
/// [`benten_platform_foundation::plugin_lifecycle::install_plugin`]
/// shape).
///
/// Construct with [`CapabilityPolicyInstallConsent::new`]. Production
/// callers typically have the engine's configured policy as
/// `Arc<dyn CapabilityPolicy>` already; this adapter takes the same
/// `Arc` so the install path observes the same policy state as the
/// `check_write` per-row path.
pub struct CapabilityPolicyInstallConsent {
    policy: Arc<dyn CapabilityPolicy>,
}

impl CapabilityPolicyInstallConsent {
    /// Wrap a `CapabilityPolicy` so it can serve as an
    /// `InstallConsentPolicy` port for the install pipeline.
    #[must_use]
    pub fn new(policy: Arc<dyn CapabilityPolicy>) -> Self {
        Self { policy }
    }
}

impl InstallConsentPolicy for CapabilityPolicyInstallConsent {
    fn check_install_consent(
        &self,
        install_record_signing_payload_hash: &[u8; 32],
        plugin_did_str: &str,
    ) -> Result<(), ErrorCode> {
        self.policy
            .check_install_consent(install_record_signing_payload_hash, plugin_did_str)
            .map_err(|_cap_err| ErrorCode::PluginInstallConsentDenied)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests + benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_caps::NoAuthBackend;

    /// Adapter wrapping the default `NoAuthBackend` admits every
    /// install consent (the default `check_install_consent` impl
    /// returns `Ok(())`).
    #[test]
    fn adapter_admits_when_wrapped_policy_admits() {
        let policy: Arc<dyn CapabilityPolicy> = Arc::new(NoAuthBackend);
        let adapter = CapabilityPolicyInstallConsent::new(policy);
        let hash = [0u8; 32];
        assert!(
            adapter
                .check_install_consent(&hash, "did:key:zTest")
                .is_ok()
        );
    }

    /// Adapter constructs from an `Arc<dyn CapabilityPolicy>` — pins
    /// the canonical wire shape that the install pipeline expects.
    #[test]
    fn adapter_constructs_from_dyn_capability_policy() {
        let policy: Arc<dyn CapabilityPolicy> = Arc::new(NoAuthBackend);
        let _adapter = CapabilityPolicyInstallConsent::new(policy);
    }
}

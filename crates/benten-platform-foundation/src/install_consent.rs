//! R6 R1 FP-F4 §S3a (Row D-3-a closure) — install-time consent policy
//! port for the install pipeline.
//!
//! Why a local trait here (rather than `&dyn benten_caps::CapabilityPolicy`):
//! `benten-caps` depends on `benten-platform-foundation` (G27-D consumes
//! `PluginManifest` + `CapRequirement` for the manifest-aware scope
//! derivation), so a direct dep the other way creates a cycle. The
//! cycle-safe shape: define a narrow per-hook trait in this crate,
//! and (engine-side / glue-side) provide a blanket impl over
//! `CapabilityPolicy` so production wiring is a single line.
//!
//! This preserves the F4 design intent: the install pipeline routes
//! through a `&dyn` policy port (CRITIC-2 F-1.2 — NOT via an
//! `Engine::capability_policy()` accessor). The Class B β sealed
//! boundary is preserved; the trait is "sealed-by-convention" (no
//! external impls expected; consumers route through the engine glue).

use benten_errors::ErrorCode;

/// **R6 R1 FP-F4 §S3a** — install-time consent hook trait.
///
/// Called at install-pipeline step 3c (BEFORE the cap-cascade). The
/// implementation typically delegates to the engine's configured
/// `benten_caps::CapabilityPolicy::check_install_consent`; the engine-
/// side adapter `CapabilityPolicyInstallConsent` (NOT in this crate
/// because of the dep-cycle constraint) wraps the policy.
///
/// Implementations return `Ok(())` to admit, or any `Err(_)` to deny;
/// `plugin_lifecycle::install_plugin` maps the denial to typed
/// `ErrorCode::PluginInstallConsentDenied`.
pub trait InstallConsentPolicy {
    /// Permit or deny the pending install.
    ///
    /// `install_record_signing_payload_hash` is the canonical 32-byte
    /// BLAKE3 hash of the install record's signing payload (the same
    /// identity the §4.37 replay-defense store keys on).
    /// `plugin_did_str` is the install record's plugin-DID string.
    ///
    /// # Errors
    ///
    /// Any `ErrorCode` to deny — `plugin_lifecycle::install_plugin`
    /// remaps the denial to `PluginInstallConsentDenied`. The default
    /// of every wrapped `CapabilityPolicy::check_install_consent` is
    /// `Ok(())` per the §8-E "admit-all default" contract.
    fn check_install_consent(
        &self,
        install_record_signing_payload_hash: &[u8; 32],
        plugin_did_str: &str,
    ) -> Result<(), ErrorCode>;
}

/// Admit-all default consent policy — wires through `install_plugin`
/// without enforcing any install-time policy. Equivalent to wrapping
/// `benten_caps::NoAuthBackend` whose `check_install_consent` default
/// returns `Ok(())`. Test fixtures that aren't exercising the
/// install-time consent surface use this.
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmitAllInstallConsent;

impl InstallConsentPolicy for AdmitAllInstallConsent {
    fn check_install_consent(
        &self,
        _install_record_signing_payload_hash: &[u8; 32],
        _plugin_did_str: &str,
    ) -> Result<(), ErrorCode> {
        Ok(())
    }
}

/// Deny-all consent policy — for test fixtures that exercise the
/// negative arm (install rejects via the consent hook surfacing
/// `ErrorCode::PluginInstallConsentDenied`).
#[derive(Debug, Clone, Copy, Default)]
pub struct DenyAllInstallConsent;

impl InstallConsentPolicy for DenyAllInstallConsent {
    fn check_install_consent(
        &self,
        _install_record_signing_payload_hash: &[u8; 32],
        _plugin_did_str: &str,
    ) -> Result<(), ErrorCode> {
        Err(ErrorCode::PluginInstallConsentDenied)
    }
}

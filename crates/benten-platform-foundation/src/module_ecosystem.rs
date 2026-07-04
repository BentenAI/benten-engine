//! Phase-4-Foundation G24-D — module ecosystem auxiliary helpers.
//!
//! Upgrade-time consent + trust + version-notification helpers that
//! sit alongside the canonical install path at
//! [`crate::plugin_lifecycle::install_plugin`] (the 11-step pipeline
//! with full Layer-1 cap cascade + Layer-2 consent + Layer-3 manifest
//! envelope per CLAUDE.md #18).
//!
//! Per `docs/PLUGIN-MANIFEST.md` §4 lifecycle.
//!
//! ## Phase-4-Meta-Core G-CORE-0 deletion note (HARD-RULE-12 clause-(a))
//!
//! The legacy `install_plugin` + `install_plugin_persisting_did`
//! precursors that previously lived on this surface were the G24-D
//! canary; they bypassed Layer-2 consent + Layer-1 cap cascade and
//! were `#[deprecated]` at R6-FP-A pending the migration of 4 legacy
//! integration tests. G-CORE-0 (the Phase-4-Meta-Core opening wave,
//! plan §1.A.FROZEN item 7) deleted both functions + their associated
//! `InstallResult` / `InstallerShape` supports per the v1
//! platform-shippable assessment requirement that two install paths
//! with different security envelopes cannot coexist in the public
//! surface. The canonical install path is
//! [`crate::plugin_lifecycle::install_plugin`].
//!
//! ## Install flow (§4.1)
//!
//! See [`crate::plugin_lifecycle::install_plugin`].
//!
//! ## Uninstall flow (§4.2) — uninstall_plugin seam is in
//! `plugin_lifecycle.rs` (G24-D-FP-1 follow-up).
//!
//! ## Upgrade flow (§4.3) — cap-change-triggered fresh consent.
//!
//! See [`decide_upgrade_consent`] +
//! [`verify_upgrade_author_continuity`] below; upgrade installs route
//! through [`crate::plugin_lifecycle::install_plugin`].

use crate::plugin_manifest::{InstallRecord, PluginManifest};
use benten_errors::ErrorCode;
use benten_id::did::Did;

/// Verify an install record was signed by the consenting user-DID.
///
/// This is the second gate — the cap chain validation (Layer 1 trace
/// to user-root) lives in `benten-caps`; this function only verifies
/// the install-record signature itself.
///
/// # Errors
///
/// `E_PLUGIN_INSTALL_RECORD_USER_SIGNATURE_INVALID`.
pub fn verify_install_record(record: &InstallRecord) -> Result<(), ErrorCode> {
    record.verify_user_signature()
}

/// Upgrade outcome — does the upgrade require fresh user consent?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeConsentDecision {
    /// `requires` is a strict subset of the prior version — silent
    /// upgrade allowed.
    Silent,
    /// `requires` grew (or changed) — user must re-consent.
    ConsentRequired,
}

/// Decide consent for an upgrade per ratification #8 (cap-change-
/// triggered fresh consent).
///
/// Silent within-lineage upgrade IFF the new `requires` set is a
/// strict subset of the old (or identical). Otherwise fresh consent.
///
/// Note: scope widening (e.g. `store:notes:read` → `store:*:read`)
/// counts as growth at this layer — we use exact-match-on-scope-
/// strings; the cap-policy backend may interpret hierarchically but
/// for the consent gate exact-string-set is the conservative default.
#[must_use]
pub fn decide_upgrade_consent(
    old: &PluginManifest,
    new: &PluginManifest,
) -> UpgradeConsentDecision {
    let old_set: std::collections::HashSet<&str> =
        old.requires.iter().map(|c| c.scope.as_str()).collect();
    let new_set: std::collections::HashSet<&str> =
        new.requires.iter().map(|c| c.scope.as_str()).collect();
    if new_set.is_subset(&old_set) {
        UpgradeConsentDecision::Silent
    } else {
        UpgradeConsentDecision::ConsentRequired
    }
}

/// Verify peer-DID continuity across a within-lineage upgrade.
///
/// Per `docs/PLUGIN-MANIFEST.md` §4.3: peer-DID change at upgrade =
/// re-install (user re-consents per T10-upgrade attack defense).
///
/// # Errors
///
/// `E_PLUGIN_AUTHOR_NOT_TRUSTED` if peer-DID differs.
pub fn verify_upgrade_author_continuity(
    old: &PluginManifest,
    new: &PluginManifest,
) -> Result<(), ErrorCode> {
    if old.peer_did != new.peer_did {
        return Err(ErrorCode::PluginAuthorNotTrusted);
    }
    Ok(())
}

/// Notify-on-new-version: returns the typed code to surface to the
/// admin UI when a peer broadcasts a new version CID for a
/// plugin-name the user has installed. The admin UI consumes this to
/// surface "new version available" prompts (pull-not-push per
/// plugin-arch-r1-13).
#[must_use]
pub fn new_version_available_code() -> ErrorCode {
    ErrorCode::PluginNewVersionAvailable
}

/// Check that a candidate plugin-author DID is in the user's
/// trust-list. Returns the typed code if not — caller surfaces the
/// first-install consent prompt.
///
/// # Errors
///
/// `E_PLUGIN_AUTHOR_NOT_TRUSTED` if `author_did` is not in `trust_list`.
pub fn check_author_trust(author_did: &Did, trust_list: &[Did]) -> Result<(), ErrorCode> {
    if trust_list.iter().any(|d| d == author_did) {
        Ok(())
    } else {
        Err(ErrorCode::PluginAuthorNotTrusted)
    }
}

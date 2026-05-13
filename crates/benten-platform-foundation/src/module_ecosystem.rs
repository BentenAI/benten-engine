//! Phase-4-Foundation G24-D — module ecosystem tooling.
//!
//! Install / uninstall / upgrade / share / discover surface for the
//! plugin manifest lifecycle. Sits ATOP `plugin_manifest` (envelope
//! types), `plugin_library` (durable library subgraph), and
//! `benten_id::plugin_did` (DID minting).
//!
//! Per `docs/PLUGIN-MANIFEST.md` §4 lifecycle.
//!
//! ## Install flow (§4.1)
//!
//! 1. Receiver verifies content-CID matches bytes (catches substitution).
//! 2. Receiver verifies peer-DID signature (catches forgery).
//! 3. Engine mints fresh plugin-DID via OsRng.
//! 4. Heterogeneity check (CLAUDE.md #17 + ds-r1-8): if manifest
//!    requires `host:sandbox:exec` AND installing peer is
//!    thin-compute-surface, reject with
//!    `E_PLUGIN_HETEROGENEITY_INCOMPATIBLE`.
//! 5. Cycle detection runs over `composes_plugins`.
//! 6. User reviews + consents; user-DID signs `InstallRecord`.
//! 7. Entry added to plugin library; active reference updated.
//!
//! ## Uninstall flow (§4.2) — uninstall_plugin seam is in
//! `plugin_lifecycle.rs` (G24-D-FP-1 follow-up).
//!
//! ## Upgrade flow (§4.3) — cap-change-triggered fresh consent.

use crate::plugin_library::{LibraryEntry, PluginLibrary};
use crate::plugin_manifest::{InstallRecord, PluginManifest, detect_composition_cycle};
use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::plugin_did::{PluginDidHandle, PluginDidStore, mint as mint_plugin_did};

/// Result of an install attempt — both the LibraryEntry and the
/// minted plugin-DID handle (returned so caller can persist the
/// secret keypair to the DID store).
pub struct InstallResult {
    /// The newly-inserted library entry.
    pub entry: LibraryEntry,
    /// The minted plugin-DID handle (caller persists into a
    /// `PluginDidStore`).
    ///
    /// G24-D-FP-1: callers that want the uninstall path to observably
    /// revoke the plugin-DID SHOULD use [`install_plugin_persisting_did`]
    /// instead — it persists the handle into the store atomically so
    /// `PluginDidStore::revoke` succeeds at uninstall. The legacy
    /// [`install_plugin`] still returns the handle for older callers.
    pub plugin_did_handle: PluginDidHandle,
}

/// Whether the installing peer is a thin-compute-surface (CLAUDE.md
/// #17 shape b/c) — affects the heterogeneity check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerShape {
    /// Full peer (native Rust; runs_sandbox=true).
    FullPeer,
    /// Thin compute surface (browser wasm32 / edge / Tauri webview).
    ThinClient,
}

/// **DEPRECATED — legacy install path that BYPASSES Layer-2 + Layer-3
/// consent gates per CLAUDE.md #18.** Use
/// [`crate::plugin_lifecycle::install_plugin`] for ALL production
/// installs.
///
/// This precursor was the Phase-4-Foundation G24-D canary; it verifies
/// content-CID + peer-DID signature + heterogeneity + composition cycle
/// ONLY. It does NOT:
///
/// - Verify the `InstallRecord` user-DID signature (Layer-2 consent
///   gate; CLAUDE.md #18 user-as-root anchor).
/// - Check the install-time manifest envelope (Layer-2).
/// - Cascade-mint root grants from user-DID → plugin-DID (Layer-1
///   trace anchor).
/// - Provision the plugin's private namespace.
/// - Run the clock-injected validate seam (`E_UCAN_CLOCK_NOT_INJECTED`).
///
/// Three pre-R4b-FP-1 integration tests still consume this surface for
/// content-CID-mismatch / peer-signature-substitution / heterogeneity
/// arms. Those tests are scheduled for migration to
/// `plugin_lifecycle::install_plugin` in the post-R6 pre-tag sweep;
/// once migrated, this function will be removed entirely (no shim).
///
/// **Future consumers MUST route through `plugin_lifecycle::install_plugin`** —
/// the legacy path is `#[deprecated]` so the compiler surfaces a
/// warning at every call site (including legitimate tests).
///
/// `received_bytes` is the canonical-bytes DAG-CBOR encoding of the
/// manifest that the receiver got over the wire / out-of-band.
///
/// `resolver` is a function from CID → optional manifest for
/// cycle-detection over `composes_plugins`. For top-level installs
/// without composition, return `|_| None`.
///
/// Returns the install result on success, or one of the typed
/// `E_PLUGIN_*` errors on failure.
///
/// # Errors
///
/// See `docs/PLUGIN-MANIFEST.md` §4.1 for the full failure mode list.
#[deprecated(
    since = "0.1.0",
    note = "BYPASSES Layer-2 consent + Layer-1 cap cascade per CLAUDE.md #18. \
            Use `plugin_lifecycle::install_plugin` for all production installs. \
            Deletion + test-migration named at docs/future/phase-4-backlog.md \
            §4.22 (Phase-4-Meta opening wave)."
)]
pub fn install_plugin<F>(
    library: &mut PluginLibrary,
    received_bytes: &[u8],
    expected_cid: &Cid,
    installer_shape: InstallerShape,
    installed_at_nanos: u64,
    resolver: &F,
) -> Result<InstallResult, ErrorCode>
where
    F: Fn(&Cid) -> Option<PluginManifest>,
{
    // 1. Decode manifest from bytes.
    let manifest: PluginManifest = serde_ipld_dagcbor::from_slice(received_bytes)
        .map_err(|_| ErrorCode::PluginManifestInvalid)?;

    // 2. Verify content-CID matches declared (substitution defense).
    if manifest.compute_content_cid() != *expected_cid {
        return Err(ErrorCode::PluginContentCidMismatch);
    }
    if manifest.content_cid != *expected_cid {
        return Err(ErrorCode::PluginContentCidMismatch);
    }

    // 3. Validate envelope structure.
    manifest.validate()?;

    // 4. Verify peer-DID signature (provenance / forgery defense).
    manifest.verify_peer_signature()?;

    // 5. Heterogeneity check.
    if matches!(installer_shape, InstallerShape::ThinClient) && manifest.requires_sandbox_exec() {
        return Err(ErrorCode::PluginHeterogeneityIncompatible);
    }

    // 6. Composition cycle detection.
    detect_composition_cycle(*expected_cid, &manifest, resolver)?;

    // 7. Mint plugin-DID.
    let plugin_did_handle = mint_plugin_did();
    let plugin_did = plugin_did_handle.did().clone();

    // 8. Insert into library.
    let entry = LibraryEntry {
        manifest_cid: *expected_cid,
        manifest: manifest.clone(),
        plugin_did,
        installed_at_nanos,
    };
    library.insert(entry.clone());

    // 9. Update active reference to the freshly-installed CID.
    library.set_active(&manifest.plugin_name, *expected_cid)?;

    Ok(InstallResult {
        entry,
        plugin_did_handle,
    })
}

/// Install a plugin AND persist the freshly-minted plugin-DID handle
/// into the supplied [`PluginDidStore`]. Returns the resulting
/// library entry + the persisted plugin-DID.
///
/// G24-D-FP-1 ergonomic seam: the legacy [`install_plugin`] returns a
/// loose [`PluginDidHandle`] for the caller to persist manually; in
/// the umbrella uninstall test path we want the install path to
/// atomically persist into the store so the subsequent uninstall
/// observes `plugin_did_revoked=true`. Production callers will route
/// through this seam.
///
/// # Errors
///
/// See [`install_plugin`] — this seam adds NO failure modes beyond
/// what the underlying install path surfaces; the persist step is
/// infallible.
#[deprecated(
    since = "0.1.0",
    note = "BYPASSES Layer-2 consent + Layer-1 cap cascade. Use \
            `plugin_lifecycle::install_plugin` which atomically persists \
            the minted plugin-DID."
)]
pub fn install_plugin_persisting_did<F>(
    library: &mut PluginLibrary,
    plugin_did_store: &mut PluginDidStore,
    received_bytes: &[u8],
    expected_cid: &Cid,
    installer_shape: InstallerShape,
    installed_at_nanos: u64,
    resolver: &F,
) -> Result<LibraryEntry, ErrorCode>
where
    F: Fn(&Cid) -> Option<PluginManifest>,
{
    #[allow(deprecated)]
    let result = install_plugin(
        library,
        received_bytes,
        expected_cid,
        installer_shape,
        installed_at_nanos,
        resolver,
    )?;
    plugin_did_store.insert(result.plugin_did_handle);
    Ok(result.entry)
}

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

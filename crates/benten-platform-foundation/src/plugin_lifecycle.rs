//! Phase-4-Foundation G24-D — plugin uninstall lifecycle.
//!
//! Hosts the `uninstall_plugin` seam per CLAUDE.md baked-in #18 +
//! `docs/PLUGIN-MANIFEST.md` §4.2.
//!
//! The G24-D primary lands the seam SHAPE (function signature + the
//! library/active-reference half). G24-D-FP-1 fills the substantive
//! cap-cascade-revoke + private-namespace teardown across crate
//! boundaries (touches `benten-caps` UCAN grant store + private
//! namespace policy).

use crate::plugin_library::PluginLibrary;
use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use benten_id::plugin_did::PluginDidStore;

/// Result of `uninstall_plugin` — observable counters that callers
/// can pin in tests + observability.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct UninstallOutcome {
    /// Count of user-DID-issued grants that were revoked.
    pub user_grants_revoked: usize,
    /// Count of plugin-DID-issued downstream delegations revoked.
    pub plugin_delegations_revoked: usize,
    /// Count of live SUBSCRIBE subscriptions terminated.
    pub subscriptions_terminated: usize,
    /// Count of private-namespace rows deleted.
    pub private_namespace_rows_deleted: usize,
    /// Whether the library entry was removed.
    pub library_entry_removed: bool,
    /// Whether the plugin-DID was revoked from the store.
    pub plugin_did_revoked: bool,
}

/// Uninstall a plugin by manifest-CID.
///
/// **G24-D primary scope** (this function lands here at canary):
///   - Removes the library entry (couples to `PluginLibrary::remove`,
///     which also clears the active reference if it pointed here).
///   - Revokes the plugin-DID from the `PluginDidStore`.
///
/// **G24-D-FP-1 follow-up scope** (cap-cascade-revoke; lands at FP-1):
///   - Enumerates all user-DID-issued grants WHERE
///     `audience = plugin-DID`; revokes each.
///   - Cascades plugin-DID's own downstream UCAN delegations.
///   - Terminates live SUBSCRIBE subscriptions for this plugin-DID.
///   - Deletes private-namespace rows.
///
/// The G24-D primary returns an `UninstallOutcome` with the
/// cap-cascade counters at zero (since the cascade-revoke seam
/// lands at FP-1). The seam shape is fixed at this wave so FP-1
/// can drop-in its substance without changing the function
/// signature.
///
/// # Errors
///
/// `E_PLUGIN_MANIFEST_INVALID` if `manifest_cid` is not in the library.
pub fn uninstall_plugin(
    library: &mut PluginLibrary,
    plugin_did_store: &mut PluginDidStore,
    manifest_cid: &Cid,
) -> Result<UninstallOutcome, ErrorCode> {
    let entry = library
        .remove(manifest_cid)
        .ok_or(ErrorCode::PluginManifestInvalid)?;

    let plugin_did_revoked = plugin_did_store.revoke(&entry.plugin_did);

    Ok(UninstallOutcome {
        user_grants_revoked: 0,
        plugin_delegations_revoked: 0,
        subscriptions_terminated: 0,
        private_namespace_rows_deleted: 0,
        library_entry_removed: true,
        plugin_did_revoked,
    })
}

/// Surface the plugin-DID associated with a library entry, for the
/// FP-1 cascade-revoke to consult.
#[must_use]
pub fn plugin_did_for_entry(library: &PluginLibrary, manifest_cid: &Cid) -> Option<Did> {
    library.get(manifest_cid).map(|e| e.plugin_did.clone())
}

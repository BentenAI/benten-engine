//! Phase-4-Foundation G24-D — plugin library subgraph.
//!
//! Per CLAUDE.md baked-in #18:
//!
//! > User's full plugin set lives as a "plugin library" subgraph
//! > containing all installed versions + forks; active graph holds
//! > references to specific plugin-versions currently in use. Switching
//! > active version = updating the reference. Keeps old/unused versions
//! > in the library (cheap, content-addressed) without polluting the
//! > active graph.
//!
//! At G24-D wave the library is an in-memory shape (`PluginLibrary`)
//! holding `LibraryEntry` records keyed by content-CID. Phase-4-Foundation
//! production code persists this via redb (parallel to the
//! `ManifestStore`).
//!
//! The library is ALSO durable — it survives across sessions and
//! sync to other devices. Per ratification #2 the CURRENT pointer
//! ("active reference") is per-device-local (Loro Map per-device-
//! keyed) so each device can have its own active plugin-version
//! without forcing other devices to upgrade simultaneously.

use crate::plugin_manifest::PluginManifest;
use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;
use std::collections::HashMap;

/// A single entry in the plugin library — one installed version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    /// Content-CID of this plugin version's manifest.
    pub manifest_cid: Cid,
    /// The manifest body (cached for lookup).
    pub manifest: PluginManifest,
    /// Plugin-DID minted at install of THIS version.
    pub plugin_did: Did,
    /// Timestamp of install (nanos since UNIX epoch; engine-injected).
    pub installed_at_nanos: u64,
}

/// The plugin library — holds every installed version + fork.
///
/// Entries are NOT removed when a new version is installed; old
/// versions remain in the library for rollback / cross-fork merge /
/// historical reference. Removal only happens via explicit uninstall
/// (G24-D-FP-1 `uninstall_plugin`).
#[derive(Debug, Default)]
pub struct PluginLibrary {
    /// All entries keyed by manifest-CID.
    entries: HashMap<Cid, LibraryEntry>,
    /// Per-plugin-name "active reference" — which CID is the CURRENT
    /// pointer for this plugin name. Per ratification #2 this is
    /// per-device-local; in the in-memory shape at G24-D wave we
    /// model the local-half only.
    active: HashMap<String, Cid>,
}

impl PluginLibrary {
    /// New empty library.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert an entry (install path).
    ///
    /// Returns the previous entry at the same CID if any (shouldn't
    /// happen — content-CIDs are unique — but defensive).
    pub fn insert(&mut self, entry: LibraryEntry) -> Option<LibraryEntry> {
        self.entries.insert(entry.manifest_cid, entry)
    }

    /// Lookup an entry by manifest-CID.
    #[must_use]
    pub fn get(&self, manifest_cid: &Cid) -> Option<&LibraryEntry> {
        self.entries.get(manifest_cid)
    }

    /// All installed CIDs (for sync / enumeration).
    pub fn cids(&self) -> impl Iterator<Item = &Cid> {
        self.entries.keys()
    }

    /// All entries.
    pub fn entries(&self) -> impl Iterator<Item = &LibraryEntry> {
        self.entries.values()
    }

    /// Count of installed versions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the library is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Set the active reference for a plugin name.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_MANIFEST_INVALID` if the CID is not in the library.
    pub fn set_active(&mut self, plugin_name: &str, cid: Cid) -> Result<(), ErrorCode> {
        if !self.entries.contains_key(&cid) {
            return Err(ErrorCode::PluginManifestInvalid);
        }
        self.active.insert(plugin_name.to_string(), cid);
        Ok(())
    }

    /// Active CID for a plugin name (CURRENT pointer per ratification #2).
    #[must_use]
    pub fn active(&self, plugin_name: &str) -> Option<&Cid> {
        self.active.get(plugin_name)
    }

    /// All versions of a plugin by name (across all entries with the
    /// same `plugin_name`).
    pub fn versions_of(&self, plugin_name: &str) -> Vec<&LibraryEntry> {
        let mut out: Vec<&LibraryEntry> = self
            .entries
            .values()
            .filter(|e| e.manifest.plugin_name == plugin_name)
            .collect();
        out.sort_by_key(|e| e.installed_at_nanos);
        out
    }

    /// Drop an entry from the library (uninstall path).
    ///
    /// Also clears the active reference if it pointed at the dropped
    /// CID. Returns the dropped entry if present.
    pub fn remove(&mut self, manifest_cid: &Cid) -> Option<LibraryEntry> {
        let dropped = self.entries.remove(manifest_cid)?;
        // If the removed CID was the active ref for its plugin, clear it.
        if let Some(active_cid) = self.active.get(&dropped.manifest.plugin_name)
            && active_cid == manifest_cid
        {
            self.active.remove(&dropped.manifest.plugin_name);
        }
        Some(dropped)
    }
}

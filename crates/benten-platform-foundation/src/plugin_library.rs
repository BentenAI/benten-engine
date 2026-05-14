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
//! >
//! > Versioning via existing Phase-1 anchor + Version Node pattern,
//! > extended to DAG-shape. Linear version-chain extended to support
//! > branches (forks). Anchor → v1 → {v2-mainline, v1.5-fork}; CURRENT
//! > can point at any branch tip.
//!
//! ## R6-FP-D lift — HashMap → real Subgraph + Anchor+Version Node
//!
//! Pre-R6-FP-D the library was a `HashMap<Cid, LibraryEntry>` keyed by
//! manifest-CID with an `active: HashMap<plugin_name, Cid>` companion
//! and a `versions_of(name)` impl that returned entries sorted by
//! `installed_at_nanos` wall-clock. That contradicted CLAUDE.md #18
//! prose: the library was metadata storage, not a typed-field-node
//! subgraph, and version-ordering was wall-clock-keyed rather than
//! content-CID-chained.
//!
//! R6-FP-D (R6 R1 cag-ux-r6-r1-1 + cag-ux-r6-r1-2 closure) lifts the
//! internal representation to:
//!
//! - **A real [`benten_core::Subgraph`]** materialized as the
//!   single-source-of-truth structure for the library's plugin set.
//!   Three node kinds: a `library_root` Read node (the subgraph
//!   anchor); one `anchor::<plugin_name>` Read node per plugin-name
//!   (the per-name Anchor in the Phase-1 sense); one `version::<cid>`
//!   Read node per installed version. Edges connect library_root →
//!   anchor (`ITEM_TYPE` from the schema-vocabulary, since the library
//!   conceptually contains plugin-anchors), anchor → version
//!   (`VERSION_OF`, a library-local edge label), and anchor →
//!   active-version (`CURRENT`, the active-reference per CLAUDE.md #18).
//!
//! - **A per-plugin-name `benten_core::version::Anchor`** maintained
//!   in a companion map. Each install appends to the anchor via
//!   `benten_core::version::append_version`; the chain history is
//!   content-CID-keyed (the canonical Phase-1 surface), so fork-
//!   detection via `benten_core::version::VersionError::Branched` is
//!   structurally available. DAG-shape (forks) is supported by allowing multiple
//!   appends from the same prior head via a separate anchor (see
//!   `versions_of` doc — the per-name Anchor is the linear-mainline
//!   anchor; forks are represented as additional Version Nodes with
//!   their own provenance edges in the subgraph but not in the
//!   mainline chain).
//!
//! - **A rich-types `entries: BTreeMap<Cid, LibraryEntry>` index**
//!   retained as the O(1) lookup surface for [`LibraryEntry`] (which
//!   carries the full [`PluginManifest`] body + [`Did`] + install
//!   timestamp). The subgraph + anchor are the canonical STRUCTURE; the
//!   entries map is the projection from CID → rich body. Construction
//!   discipline: every mutator that adds/removes an entry MUST keep
//!   subgraph + anchor + entries in lockstep.
//!
//! No new [`benten_core::PrimitiveKind`] variant is introduced — the
//! library uses the canonical 12-primitive Read kind for every node
//! (CLAUDE.md baked-in #1 12-primitive irreducibility). The structure
//! is encoded in node properties + edge labels, not in new primitives.
//!
//! ## Per-device-local active reference
//!
//! Per ratification #2 the CURRENT pointer ("active reference") is per-
//! device-local — each device can have its own active plugin-version
//! without forcing other devices to upgrade simultaneously. The
//! in-memory shape models the local-half only; Phase-4-Foundation
//! production code persists this via redb (parallel to the
//! [`crate::manifest_store::ManifestStore`]).

use crate::plugin_manifest::PluginManifest;
use benten_core::version::{Anchor, append_version, walk_versions};
use benten_core::{Cid, OperationNode, PrimitiveKind, Subgraph, Value};
use benten_errors::ErrorCode;
use benten_id::did::Did;
use std::collections::{BTreeMap, HashMap};

/// Edge label connecting the library root to a per-plugin-name anchor.
///
/// Reuses the schema-vocabulary [`crate::schema_compiler::VocabEdge::ItemType`]
/// canonical string — the library is conceptually a container of plugin-
/// anchors, mirroring how `FieldList` / `FieldMap` use `ITEM_TYPE` for
/// element relationships. Bound to the const-fn `as_str()` of the source-
/// of-truth `VocabEdge::ItemType` variant per §3.5g cross-language rule-
/// mirror (Rust-side same-language mirror): a future rename of the canonical
/// string in `vocab.rs` cascades here automatically at compile time.
pub const EDGE_LIBRARY_ANCHOR: &str = crate::schema_compiler::VocabEdge::ItemType.as_str();

/// Edge label connecting an anchor to one of its Version Nodes (every
/// installed CID under that plugin-name).
pub const EDGE_VERSION_OF: &str = "VERSION_OF";

/// Edge label connecting an anchor to the CURRENT pointer (active
/// reference per CLAUDE.md #18 + ratification #2 per-device-local).
pub const EDGE_CURRENT: &str = "CURRENT";

/// Property key on a library_anchor Node: the plugin_name string.
pub const PROP_ANCHOR_PLUGIN_NAME: &str = "plugin_name";

/// Property key on a version Node: the manifest_cid (as bytes via
/// [`benten_core::Value::Bytes`]).
pub const PROP_VERSION_MANIFEST_CID: &str = "manifest_cid";

/// Property key on a version Node: the plugin_did (as text).
pub const PROP_VERSION_PLUGIN_DID: &str = "plugin_did";

/// Property key on a version Node: the installed-at timestamp in nanos
/// (as Int).
pub const PROP_VERSION_INSTALLED_AT_NANOS: &str = "installed_at_nanos";

/// Stable id of the library_root Node within the subgraph.
pub const NODE_ID_LIBRARY_ROOT: &str = "library_root";

/// Canonical handler id for the plugin-library subgraph (foundation-
/// owned namespace, parallel to `schema:` prefix used by
/// [`crate::schema_compiler`]).
pub const HANDLER_ID_PLUGIN_LIBRARY: &str = "plugin-library";

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
/// Internally the library is a real [`Subgraph`] (`handler_id =
/// "plugin-library"`) + per-plugin-name [`Anchor`] companions + a
/// rich-types `entries` index. See module-level doc for the full
/// substrate explanation.
///
/// Entries are NOT removed when a new version is installed; old
/// versions remain in the library (as Version Nodes) for rollback /
/// cross-fork merge / historical reference. Removal only happens via
/// explicit uninstall (G24-D-FP-1 `uninstall_plugin`).
#[derive(Debug)]
pub struct PluginLibrary {
    /// Single-source-of-truth subgraph. Built from the library_root +
    /// per-name anchors + per-CID version nodes.
    subgraph: Subgraph,
    /// Per-plugin-name [`Anchor`] (Phase-1 Cid-head-threaded
    /// version-chain). The Anchor's `head` is the CID of the FIRST
    /// installed version; subsequent installs append via
    /// [`append_version`]. Forks (DAG-shape; see CLAUDE.md #18) are
    /// preserved as additional Version Nodes in the subgraph; the
    /// anchor itself tracks the mainline chain only (any append against
    /// a non-tip prior head surfaces `benten_core::version::VersionError::Branched`
    /// which the library currently treats as a non-error fork and
    /// simply skips the linear-chain append — the Version Node still
    /// lands in the subgraph for retention).
    anchors: HashMap<String, Anchor>,
    /// Rich-types projection: O(1) CID → entry. Maintained in lockstep
    /// with the subgraph by every mutator.
    entries: BTreeMap<Cid, LibraryEntry>,
    /// Per-plugin-name "active reference" — which CID is the CURRENT
    /// pointer. Per ratification #2 this is per-device-local; in the
    /// in-memory shape at G24-D wave we model the local-half only.
    /// Encoded in the subgraph via [`EDGE_CURRENT`] edges; this map is
    /// the projection for fast O(1) lookup.
    active: HashMap<String, Cid>,
}

impl Default for PluginLibrary {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginLibrary {
    /// New empty library — instantiates the underlying subgraph with
    /// only the `library_root` node.
    #[must_use]
    pub fn new() -> Self {
        let mut subgraph = Subgraph::new(HANDLER_ID_PLUGIN_LIBRARY);
        // The library_root node is structural: every library has it,
        // whether empty or populated. Read kind — the library is
        // navigable via reads (consumers iterate entries / look up
        // active version).
        subgraph.nodes.push(OperationNode::new(
            NODE_ID_LIBRARY_ROOT,
            PrimitiveKind::Read,
        ));
        Self {
            subgraph,
            anchors: HashMap::new(),
            entries: BTreeMap::new(),
            active: HashMap::new(),
        }
    }

    /// Borrow the underlying subgraph (the canonical structure).
    ///
    /// Consumers that want to walk the library by graph-evaluator
    /// semantics use this surface. The subgraph's handler_id is
    /// [`HANDLER_ID_PLUGIN_LIBRARY`]; nodes are
    /// [`PrimitiveKind::Read`]; edges use [`EDGE_LIBRARY_ANCHOR`] /
    /// [`EDGE_VERSION_OF`] / [`EDGE_CURRENT`].
    #[must_use]
    pub fn as_subgraph(&self) -> &Subgraph {
        &self.subgraph
    }

    /// Borrow the per-plugin-name [`Anchor`] (Phase-1 version-chain
    /// surface). Returns `None` if the plugin-name has no installed
    /// version yet.
    #[must_use]
    pub fn anchor(&self, plugin_name: &str) -> Option<&Anchor> {
        self.anchors.get(plugin_name)
    }

    /// Walk the linear version chain for a plugin-name (Phase-1
    /// [`walk_versions`] semantics). Returns CIDs from oldest to newest
    /// along the mainline. Forks are NOT returned by this walker —
    /// use [`PluginLibrary::versions_of`] for the full set including
    /// forks.
    #[must_use]
    pub fn walk_mainline(&self, plugin_name: &str) -> Vec<Cid> {
        self.anchors
            .get(plugin_name)
            .map(|a| walk_versions(a).collect())
            .unwrap_or_default()
    }

    /// Insert an entry (install path). Updates subgraph + anchor +
    /// entries map in lockstep.
    ///
    /// Returns the previous entry at the same CID if any (shouldn't
    /// happen — content-CIDs are unique — but defensive).
    pub fn insert(&mut self, entry: LibraryEntry) -> Option<LibraryEntry> {
        let manifest_cid = entry.manifest_cid;
        let plugin_name = entry.manifest.plugin_name.clone();
        let prior = self.entries.insert(manifest_cid, entry.clone());

        // Subgraph + anchor maintenance — only on first-insert of THIS
        // (name, cid) pair. Re-insert (defensive case) leaves
        // structure intact.
        if prior.is_none() {
            self.upsert_anchor_node(&plugin_name);
            self.append_version_node(&plugin_name, &entry);
            self.try_append_anchor_chain(&plugin_name, manifest_cid);
        }
        prior
    }

    /// Ensure the per-name anchor node + library-root → anchor edge
    /// exist in the subgraph.
    fn upsert_anchor_node(&mut self, plugin_name: &str) {
        let anchor_id = anchor_node_id(plugin_name);
        if self.subgraph.nodes.iter().any(|n| n.id == anchor_id) {
            return;
        }
        self.subgraph.nodes.push(
            OperationNode::new(&anchor_id, PrimitiveKind::Read).with_property(
                PROP_ANCHOR_PLUGIN_NAME,
                Value::Text(plugin_name.to_string()),
            ),
        );
        self.subgraph.edges.push((
            NODE_ID_LIBRARY_ROOT.to_string(),
            anchor_id,
            EDGE_LIBRARY_ANCHOR.to_string(),
        ));
    }

    /// Add a Version Node + anchor → version edge to the subgraph.
    fn append_version_node(&mut self, plugin_name: &str, entry: &LibraryEntry) {
        let anchor_id = anchor_node_id(plugin_name);
        let version_id = version_node_id(&entry.manifest_cid);
        self.subgraph.nodes.push(
            OperationNode::new(&version_id, PrimitiveKind::Read)
                .with_property(
                    PROP_VERSION_MANIFEST_CID,
                    Value::Bytes(entry.manifest_cid.as_bytes().to_vec()),
                )
                .with_property(
                    PROP_VERSION_PLUGIN_DID,
                    Value::Text(entry.plugin_did.as_str().to_string()),
                )
                .with_property(
                    PROP_VERSION_INSTALLED_AT_NANOS,
                    Value::Int(i64::try_from(entry.installed_at_nanos).unwrap_or(i64::MAX)),
                ),
        );
        self.subgraph
            .edges
            .push((anchor_id, version_id, EDGE_VERSION_OF.to_string()));
    }

    /// Try to append `new_cid` to the per-name [`Anchor`] (Phase-1
    /// chain). On first install for a plugin-name, mint the Anchor. On
    /// subsequent installs, attempt [`append_version`] against the
    /// current mainline tip; if the append surfaces
    /// `benten_core::version::VersionError::Branched`, the install is a fork
    /// (DAG-shape per CLAUDE.md #18) — the Version Node still landed in
    /// the subgraph above; we simply do NOT extend the linear-chain
    /// past the fork point.
    fn try_append_anchor_chain(&mut self, plugin_name: &str, new_cid: Cid) {
        if let Some(anchor) = self.anchors.get(plugin_name) {
            // Find the mainline tip (last entry of walk_versions).
            let tip = walk_versions(anchor).last().unwrap_or(anchor.head);
            // Defensive: re-install same CID is a no-op for the chain.
            if tip == new_cid {
                return;
            }
            // Append. Fork (Branched) is a non-error here — the
            // Version Node already exists in the subgraph above. This
            // is the DAG-shape behavior CLAUDE.md #18 names.
            let _ = append_version(anchor, &tip, &new_cid);
        } else {
            // First install for this plugin-name: mint the Anchor
            // rooted at this CID.
            self.anchors
                .insert(plugin_name.to_string(), Anchor::new(new_cid));
        }
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

    /// Set the active reference for a plugin name. Updates the subgraph
    /// `CURRENT` edge + the projection map.
    ///
    /// # Errors
    ///
    /// `E_PLUGIN_MANIFEST_INVALID` if the CID is not in the library.
    pub fn set_active(&mut self, plugin_name: &str, cid: Cid) -> Result<(), ErrorCode> {
        if !self.entries.contains_key(&cid) {
            return Err(ErrorCode::PluginManifestInvalid);
        }
        self.active.insert(plugin_name.to_string(), cid);
        self.refresh_current_edge(plugin_name, Some(cid));
        Ok(())
    }

    /// Active CID for a plugin name (CURRENT pointer per ratification #2).
    #[must_use]
    pub fn active(&self, plugin_name: &str) -> Option<&Cid> {
        self.active.get(plugin_name)
    }

    /// All versions of a plugin by name (across all entries with the
    /// same `plugin_name`). Returns the full set including forks (per
    /// CLAUDE.md #18 DAG-shape — both mainline and fork Version Nodes
    /// land in the subgraph and are surfaced here).
    ///
    /// Ordering is by `installed_at_nanos` ascending (stable across
    /// equal timestamps via manifest_cid tie-break). The mainline-only
    /// content-CID-chained walk is exposed via
    /// [`PluginLibrary::walk_mainline`].
    pub fn versions_of(&self, plugin_name: &str) -> Vec<&LibraryEntry> {
        let mut out: Vec<&LibraryEntry> = self
            .entries
            .values()
            .filter(|e| e.manifest.plugin_name == plugin_name)
            .collect();
        out.sort_by(|a, b| {
            a.installed_at_nanos
                .cmp(&b.installed_at_nanos)
                .then_with(|| a.manifest_cid.cmp(&b.manifest_cid))
        });
        out
    }

    /// Drop an entry from the library (uninstall path). Updates
    /// subgraph + anchor + entries map in lockstep.
    ///
    /// Also clears the active reference if it pointed at the dropped
    /// CID. Returns the dropped entry if present.
    pub fn remove(&mut self, manifest_cid: &Cid) -> Option<LibraryEntry> {
        let dropped = self.entries.remove(manifest_cid)?;
        let plugin_name = dropped.manifest.plugin_name.clone();

        // Drop the Version Node + its anchor → version edge from the
        // subgraph.
        let version_id = version_node_id(manifest_cid);
        self.subgraph.nodes.retain(|n| n.id != version_id);
        self.subgraph
            .edges
            .retain(|(_, to, label)| !(to == &version_id && label == EDGE_VERSION_OF));

        // If the removed CID was the active ref for its plugin, clear
        // it (also drops the CURRENT edge).
        let was_active = self
            .active
            .get(&plugin_name)
            .is_some_and(|c| c == manifest_cid);
        if was_active {
            self.active.remove(&plugin_name);
            self.refresh_current_edge(&plugin_name, None);
        }

        // If this was the LAST version of this plugin-name, drop the
        // anchor node + library_root → anchor edge + anchor map entry.
        let any_remaining = self
            .entries
            .values()
            .any(|e| e.manifest.plugin_name == plugin_name);
        if !any_remaining {
            let anchor_id = anchor_node_id(&plugin_name);
            self.subgraph.nodes.retain(|n| n.id != anchor_id);
            self.subgraph
                .edges
                .retain(|(from, to, _)| from != &anchor_id && to != &anchor_id);
            self.anchors.remove(&plugin_name);
        }

        Some(dropped)
    }

    /// Set/clear the `CURRENT` edge from the per-name anchor to the
    /// active Version Node. When `target` is `None`, drops any existing
    /// CURRENT edge.
    fn refresh_current_edge(&mut self, plugin_name: &str, target: Option<Cid>) {
        let anchor_id = anchor_node_id(plugin_name);
        // Always drop any existing CURRENT edge first.
        self.subgraph
            .edges
            .retain(|(from, _, label)| !(from == &anchor_id && label == EDGE_CURRENT));
        if let Some(cid) = target {
            let version_id = version_node_id(&cid);
            self.subgraph
                .edges
                .push((anchor_id, version_id, EDGE_CURRENT.to_string()));
        }
    }
}

/// Construct the canonical anchor-node id for a plugin-name.
#[must_use]
pub fn anchor_node_id(plugin_name: &str) -> String {
    format!("anchor::{plugin_name}")
}

/// Construct the canonical version-node id for a manifest CID. Uses
/// hex encoding of the CID bytes for stable ids.
#[must_use]
pub fn version_node_id(cid: &Cid) -> String {
    let mut hex = String::with_capacity(2 + 64);
    hex.push_str("version::");
    for b in cid.as_bytes() {
        use core::fmt::Write;
        let _ = write!(&mut hex, "{b:02x}");
    }
    hex
}

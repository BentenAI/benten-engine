//! Phase-4-Foundation G24-D — workflow ↔ plugin unification.
//!
//! Per CLAUDE.md baked-in #18:
//!
//! > A workflow IS a plugin IS a subgraph. Single shape, distinguished
//! > by **scale + sharing intent** (manifest presence). Promoting a
//! > workflow to a plugin = adding a manifest; no fundamental shape
//! > change.
//!
//! This module is intentionally thin (~50-100 LOC). It surfaces the
//! `promote_workflow_to_plugin` affordance — the only difference
//! between a workflow and a plugin is whether a manifest is attached.
//! Composition: meta-plugins reference sub-plugins recursively via
//! the same `composes_plugins` mechanism that plugins use to compose
//! plugins (no new primitive).

use crate::plugin_manifest::{CapRequirement, PluginManifest, SharesPolicy};
use benten_core::Cid;
use benten_id::did::Did;

/// A "workflow" handle — a subgraph CID without a manifest envelope.
///
/// Workflows are private/local-scope by default — they execute as
/// part of the user's own subgraph walk. They have no shareability
/// semantics, no install record, no plugin-DID. Promoting a workflow
/// to a plugin equips it with a manifest envelope that makes it
/// shareable to other Atriums / peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowHandle {
    /// CID of the subgraph that defines the workflow's operations.
    pub subgraph_cid: Cid,
    /// Human-readable name (for the user's own organization).
    pub name: String,
}

/// Promote a workflow to a plugin by attaching a manifest.
///
/// This is the ONLY shape-change between workflows and plugins per
/// CLAUDE.md #18. The returned `PluginManifest` has the
/// `peer_signature` field empty — the caller (typically the
/// `module_ecosystem::publish_plugin` path) signs after computing
/// `content_cid`.
///
/// The workflow's `subgraph_cid` is seeded into the manifest's
/// `content_cid` slot but is then OVERWRITTEN by the hash-of-manifest-body
/// at line 70 below (`manifest.compute_content_cid()`). The shipped
/// shape: the manifest body (including the seed subgraph_cid + envelope
/// metadata) IS the plugin identity post-promotion — the manifest's
/// `content_cid` after promotion is structurally distinct from the
/// original workflow's `subgraph_cid`.
#[must_use]
pub fn promote_workflow_to_plugin(
    workflow: &WorkflowHandle,
    author_peer_did: Did,
    requires: Vec<CapRequirement>,
    shares: SharesPolicy,
) -> PluginManifest {
    let mut manifest = PluginManifest {
        plugin_name: workflow.name.clone(),
        content_cid: workflow.subgraph_cid,
        peer_did: author_peer_did,
        peer_signature: Vec::new(),
        requires,
        shares,
        renderer_config: None,
        composes_plugins: None,
        accepts_content: None,
        requires_schema_authors: None,
        requires_plugin_authors: None,
    };
    // Compute the real content_cid (hash of the manifest body itself),
    // OVERWRITING the seed subgraph_cid placeholder set at construction.
    // The promotion-time subgraph_cid is NOT recoverable from the post-
    // promotion manifest (would require the caller to retain the
    // original workflow handle); this is intentional — the manifest's
    // identity post-promotion is the manifest body, not the workflow.
    manifest.content_cid = manifest.compute_content_cid();
    manifest
}

/// Whether a subgraph-CID is "workflow-shaped" (no manifest) vs
/// "plugin-shaped" (has a corresponding manifest in the library).
///
/// The detection is at the library/registry level: a workflow is
/// just a subgraph CID with no `LibraryEntry`; a plugin is the
/// inverse.
#[must_use]
pub fn is_promoted_to_plugin<F>(workflow_subgraph_cid: &Cid, library_lookup: F) -> bool
where
    F: Fn(&Cid) -> bool,
{
    library_lookup(workflow_subgraph_cid)
}

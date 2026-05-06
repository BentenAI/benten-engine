//! Phase-3 G14-C wave-4b — durable handler-version chain (Compromise #18).
//!
//! ## What this is
//!
//! Closure of **Compromise #18** (in-memory handler-version chain).
//! Each `register_subgraph` / `register_subgraph_replace` invocation
//! durably persists a `system:HandlerVersion` zone Node carrying:
//!
//! - `handler_id` — the handler's stable id (also used as the
//!   per-handler [`benten_core::version::Anchor`] grouping key).
//! - `version_cid` — the registered subgraph's content-addressed CID.
//! - `predecessor_cid` — the previously-registered CID for the same
//!   handler (omitted on the first registration, present on each
//!   subsequent replace; mirrors `core::version::append_version`'s
//!   `prior_head` arg).
//! - `seq` — monotonic per-handler insertion sequence (0-based; the
//!   first registration is `seq=0`, each replace bumps by 1). The seq
//!   field gives a stable rebuild order on engine open.
//!
//! On `Engine::open`, the crate-internal
//! `rehydrate_handler_version_chains_from_zone` accessor
//! scans the zone, groups by `handler_id`, sorts each group by `seq`
//! ascending, and rebuilds the in-memory newest-first `Vec<Cid>` chain
//! (so `chain[0]` is CURRENT, `chain[chain.len()-1]` is the oldest).
//!
//! ## Design notes (arch-r1-4 BLOCKER + D-C HYBRID)
//!
//! Per **arch-r1-4 BLOCKER + D-C**, the canonical-bytes encoding for
//! each version Node MUST be EXTENSIBLE — additive variant slots for
//! future attribution metadata (Loro merge frame, multi-actor
//! delegation, AttributionFrame extension) MUST land WITHOUT breaking
//! pinned-CID test sites.
//!
//! The encoding is the standard Benten Node DAG-CBOR shape (label +
//! property map; map keys are sorted lexicographically; missing
//! optional properties are omitted from the wire bytes via
//! `skip_serializing_if = "Option::is_none"`). DAG-CBOR's strict
//! sorted-map invariant + serde's omitted-optional encoding mean a
//! future Phase-3 G16-B amendment can add a new property key (e.g.
//! `loro_merge_attribution`) without changing the canonical bytes (or
//! CID) of any chain Node that pre-dates the amendment. The new
//! property key sorts in lexicographically; pre-amendment Nodes lack
//! it; their bytes are unchanged.
//!
//! ## Compromise #18 promotion path
//!
//! Per-handler chains are functionally equivalent to a `core::version::
//! Anchor` rooted at the first version + each subsequent replace
//! calling `append_version(anchor, prior_head, new_head)`. The wave-8f
//! in-memory `BTreeMap<HandlerId, Vec<Cid>>` is preserved as the
//! hot-path cache; the zone Nodes are the durable source of truth.
//! Phase-3 G16-B (Loro merge attribution) extends the property bag
//! additively per the extensibility contract above.

use std::collections::BTreeMap;

use benten_core::{Cid, Node, Value};
use benten_graph::MutexExt;

use crate::engine::Engine;
use crate::error::EngineError;

/// G14-C label used for the durable handler-version-chain side-table.
/// Privileged-write surface (`system:` prefix) — same shape as
/// `system:ModuleManifest` and `system:ModuleBytes`.
pub const HANDLER_VERSION_LABEL: &str = "system:HandlerVersion";

/// Property key carrying the handler id (UTF-8 string). Per-handler
/// rehydration groups Nodes by this key.
pub const HANDLER_ID_PROPERTY: &str = "handler_id";

/// Property key carrying the registered subgraph CID (base32 string).
pub const VERSION_CID_PROPERTY: &str = "version_cid";

/// Property key carrying the predecessor CID (base32 string). OMITTED
/// from canonical bytes when the entry is the first registration for a
/// handler (DAG-CBOR map key absent ≠ present-with-null per
/// crypto-major-1-style discipline; mirrors `ManifestSignature::None`'s
/// `skip_serializing_if = "Option::is_none"` shape).
pub const PREDECESSOR_CID_PROPERTY: &str = "predecessor_cid";

/// Property key carrying the per-handler monotonic insertion sequence
/// (`u64` encoded as `Value::Integer`). Provides a stable rebuild
/// order at engine open even when redb's index iteration order
/// disagrees with insertion order.
pub const SEQUENCE_PROPERTY: &str = "seq";

/// Per-handler version-chain view. Newest-first ordering matches
/// [`Engine::handler_version_chain`]'s in-memory shape; the durable
/// rebuild at engine open sorts by `seq` ascending then reverses.
#[derive(Debug, Clone)]
pub struct HandlerVersionChain {
    /// The chain's anchor — `core::version::Anchor` rooted at the
    /// first registered version. None when the handler has no
    /// versions yet (defensive — the public accessor returns an
    /// empty Vec rather than constructing this struct in that case).
    pub anchor: Option<benten_core::version::Anchor>,
    /// Newest-first list of registered version CIDs.
    pub versions: Vec<Cid>,
}

impl HandlerVersionChain {
    /// CID of the most recently registered subgraph, or `None` when
    /// the chain is empty.
    #[must_use]
    pub fn current_version_cid(&self) -> Option<Cid> {
        self.versions.first().copied()
    }

    /// CID of the chain's anchor (root version), if any. Equal to
    /// `versions.last()` for a fully-rebuilt chain.
    #[must_use]
    pub fn anchor_cid(&self) -> Option<Cid> {
        self.anchor.as_ref().map(|a| a.head)
    }

    /// Newest-first list of all registered version CIDs.
    #[must_use]
    pub fn versions(&self) -> &[Cid] {
        &self.versions
    }

    /// Returns whether the chain contains the given version CID
    /// (linear scan; chains are operator-bounded).
    #[must_use]
    pub fn fetch_version(&self, cid: &Cid) -> Option<Cid> {
        self.versions.iter().find(|c| *c == cid).copied()
    }
}

/// Construct the canonical [`Node`] for one entry in a handler's
/// durable version chain.
///
/// The label is [`HANDLER_VERSION_LABEL`]; the property map carries
/// `handler_id`, `version_cid`, optional `predecessor_cid`, and `seq`.
/// Per arch-r1-4 / D-C the encoding is additively extensible — Phase-3
/// G16-B may add new property keys without breaking pinned-CID test
/// sites.
#[must_use]
pub fn make_version_node(
    handler_id: &str,
    version_cid: &Cid,
    predecessor_cid: Option<&Cid>,
    seq: u64,
) -> Node {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert(
        HANDLER_ID_PROPERTY.to_string(),
        Value::Text(handler_id.to_string()),
    );
    props.insert(
        VERSION_CID_PROPERTY.to_string(),
        Value::Text(version_cid.to_base32()),
    );
    if let Some(prev) = predecessor_cid {
        props.insert(
            PREDECESSOR_CID_PROPERTY.to_string(),
            Value::Text(prev.to_base32()),
        );
    }
    // Per-handler seq encoded as Integer. Phase-3 G16-B may extend
    // the property bag with `loro_merge_attribution` / `multi_actor_*`
    // / etc. without disturbing this shape — DAG-CBOR's sorted-map
    // discipline accepts any additive extension at any position.
    // seq encoded as Int (i64) — operator-bounded chains stay well
    // below i64 max; per-handler chain length grows monotonically
    // with replace events.
    let seq_signed = i64::try_from(seq).unwrap_or(i64::MAX);
    props.insert(SEQUENCE_PROPERTY.to_string(), Value::Int(seq_signed));
    Node::new(vec![HANDLER_VERSION_LABEL.to_string()], props)
}

impl Engine {
    /// Phase-3 G14-C — persist one entry in a handler's durable
    /// version chain.
    ///
    /// Called from [`Engine::register_subgraph`] (first registration,
    /// `predecessor` = `None`, `seq` = 0) and
    /// [`Engine::register_subgraph_replace`] (subsequent registrations,
    /// `predecessor` = previous CURRENT, `seq` = chain.len() at insert
    /// time).
    ///
    /// Idempotent: re-persisting an entry whose canonical Node CID
    /// matches an existing one (same `handler_id` + `version_cid` +
    /// `predecessor_cid` + `seq`) is a no-op via the redb Inv-13 dedup
    /// path.
    ///
    /// # Errors
    ///
    /// [`EngineError::Graph`] when the privileged write surfaces a
    /// backend error.
    pub(crate) fn persist_handler_version_entry(
        &self,
        handler_id: &str,
        version_cid: &Cid,
        predecessor_cid: Option<&Cid>,
        seq: u64,
    ) -> Result<(), EngineError> {
        let node = make_version_node(handler_id, version_cid, predecessor_cid, seq);
        self.backend()
            .put_node_with_context(
                &node,
                &benten_graph::WriteContext::privileged_for_engine_api(),
            )
            .map_err(EngineError::from)?;
        Ok(())
    }

    /// Phase-3 G14-C (Compromise #18 closure) — return the handler's
    /// full durable version-chain, rooted in a `core::version::Anchor`.
    ///
    /// Mirrors [`Self::handler_version_chain`]'s in-memory shape but
    /// also carries the [`benten_core::version::Anchor`] rooted at the
    /// chain's first registered version. The Anchor is constructed
    /// fresh each call (its in-memory `Arc<Mutex>` chain state stays
    /// per-call); persistence is via the `system:HandlerVersion` zone
    /// Nodes the engine wrote at register time.
    ///
    /// Returns `None` when the handler has no registered versions.
    /// Otherwise returns a [`HandlerVersionChain`] whose `versions`
    /// list is newest-first (matches the in-memory chain's
    /// invariant).
    #[must_use]
    pub fn handler_version_chain_with_anchor(
        &self,
        handler_id: &str,
    ) -> Option<HandlerVersionChain> {
        let versions = self.handler_version_chain(handler_id);
        if versions.is_empty() {
            return None;
        }
        // The root (oldest) is the chain's anchor head. Newest-first
        // ordering puts the root at `versions.last()`.
        let root = *versions.last().expect("non-empty chain has a last");
        let anchor = benten_core::version::Anchor::new(root);
        Some(HandlerVersionChain {
            anchor: Some(anchor),
            versions,
        })
    }

    /// Phase-3 G14-C (Compromise #18 closure) — rebuild the in-memory
    /// `handler_version_chain` map from the durable
    /// `system:HandlerVersion` zone.
    ///
    /// Called from `EngineBuilder::assemble` once after the backend
    /// opens + the engine is constructed. Failures during rehydration
    /// log via tracing and are non-fatal.
    ///
    /// Algorithm:
    /// 1. Scan all `system:HandlerVersion` zone Nodes.
    /// 2. Group by `handler_id`.
    /// 3. Sort each group by `seq` ascending.
    /// 4. Reverse + insert into the in-memory chain (newest-first).
    ///
    /// # Errors
    ///
    /// [`EngineError::Graph`] when the backend's `get_by_label` /
    /// `get_node` accessors error. Per-Node decode failures are
    /// logged + skipped rather than aborting the scan.
    pub(crate) fn rehydrate_handler_version_chains_from_zone(&self) -> Result<usize, EngineError> {
        let node_cids = self.backend.get_by_label(HANDLER_VERSION_LABEL)?;
        // Group: handler_id -> Vec<(seq, version_cid)>.
        let mut grouped: BTreeMap<String, Vec<(u64, Cid)>> = BTreeMap::new();
        for node_cid in node_cids {
            let Some(node) = self.backend.get_node(&node_cid)? else {
                continue;
            };
            let handler_id = match node.properties.get(HANDLER_ID_PROPERTY) {
                Some(Value::Text(s)) => s.clone(),
                _ => continue,
            };
            let version_cid_str = match node.properties.get(VERSION_CID_PROPERTY) {
                Some(Value::Text(s)) => s,
                _ => continue,
            };
            let Ok(version_cid) = Cid::from_str(version_cid_str) else {
                continue;
            };
            let seq = match node.properties.get(SEQUENCE_PROPERTY) {
                Some(Value::Int(n)) if *n >= 0 => u64::try_from(*n).unwrap_or(0),
                _ => continue,
            };
            grouped
                .entry(handler_id)
                .or_default()
                .push((seq, version_cid));
        }
        // Sort each group by seq ascending; reverse to newest-first;
        // insert into the in-memory map.
        let count = grouped.len();
        let mut chain_guard = self.handler_version_chain_in_memory_lock();
        for (handler_id, mut entries) in grouped {
            entries.sort_by_key(|(seq, _)| *seq);
            // newest-first
            entries.reverse();
            let cids: Vec<Cid> = entries.into_iter().map(|(_, c)| c).collect();
            chain_guard.insert(handler_id, cids);
        }
        Ok(count)
    }
}

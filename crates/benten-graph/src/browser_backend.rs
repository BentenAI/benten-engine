//! [`BrowserBackend`] — in-RAM thin-client cache for `wasm32-unknown-unknown`
//! browser tabs (Phase-3 R5 wave-3 G13-C; CLAUDE.md baked-in #17).
//!
//! ## Why this backend exists
//!
//! Phase-2b shipped `Engine` hard-bound to [`crate::RedbBackend`], which baked
//! the redb dependency into every layer above — including the
//! `wasm32-unknown-unknown` browser-target build. The Phase-2b retrospective
//! at `.addl/phase-2b/wave-8j-wasm-browser-bundle-bisect.md` § Phase-3
//! follow-up identified this as the load-bearing cause of the 600KB browser-
//! bundle blow-up; the spike at `.addl/phase-3/spike-bundle-cap-empirical.md`
//! recalibrated the cap from the Phase-2b 350KB aspiration to an empirically-
//! anchored 600KB ceiling for Phase-3.
//!
//! G13-A (wave-1 canary) extracted the [`crate::GraphBackend`] umbrella
//! trait so the engine could substitute non-redb backends for the browser
//! target. G13-B (wave-2) cascaded the generic parameter through the
//! engine. G13-C (this wave) lands `BrowserBackend` as the in-RAM
//! [`std::collections::BTreeMap`]-backed thin-client cache that the
//! browser bundle uses in place of `RedbBackend`.
//!
//! ## Thin-client cache scope (CLAUDE.md baked-in #17 — load-bearing)
//!
//! `BrowserBackend` is a THIN-CLIENT CACHE ONLY:
//!
//! - **No durable storage.** All state lives in an in-RAM `BTreeMap` behind
//!   a coarse [`std::sync::Mutex`]. Tab-close drops the cache; persistence
//!   (snapshot cache + manifest store) lives in IndexedDB via the separate
//!   browser-blob-backend infrastructure landed at G18-A.
//! - **No transactions** (returns a no-op transaction-runner marker). The
//!   full peer is the source of truth for atomicity; browser-tab thin-
//!   client writes mirror the full-peer's authoritative ordering via the
//!   thin-client subscription protocol (D-PHASE-3-30, landed at G14-D).
//! - **No subscribers** (returns silent no-op fan-out). Browser tabs
//!   subscribe to the full peer over Server-Sent Events / WebSocket per
//!   G14-D thin-client subscription, NOT to their own local cache. Local
//!   fan-out would either (a) double-fire against the full-peer
//!   subscription or (b) silently change the browser-tab UX. Both are
//!   non-goals.
//! - **No sync state.** Sync is full-peer-only per CLAUDE.md baked-in #17;
//!   browser-side `BrowserBackend` never participates in
//!   [iroh](https://crates.io/crates/iroh) /
//!   [Loro](https://crates.io/crates/loro) / Merkle Search Tree diff.
//!   Those crates ship in `benten-sync` which is `[target.'cfg(not(...
//!   wasm32...))']`-gated.
//! - **`put_node_with_context` BYPASSES cap-recheck at the cache layer.**
//!   The upstream subscription (G14-D) already filters events per
//!   delivered-subscriber's grant; the local cache simply mirrors the
//!   authorized stream. Re-running cap-policy on the cache write path
//!   would double-count rate-limits and would couple the browser bundle
//!   to the cap-policy crate (defying the bundle-cap commitment per
//!   `.addl/phase-3/spike-bundle-cap-empirical.md` §6 per-contributor
//!   budget).
//!
//! ## Object-safety + generic-cascade contract
//!
//! `BrowserBackend` impls the [`crate::GraphBackend`] umbrella trait so the
//! engine consumes it via the *generic-cascade* direction
//! (`EngineGeneric<BrowserBackend>`) per `D-PHASE-3-1` RESOLVED. The trait
//! is intentionally not object-safe; `dyn GraphBackend` is not a supported
//! engine boundary.
//!
//! ## Cargo feature gating
//!
//! `BrowserBackend` is gated behind the `browser-backend` cargo feature
//! on `benten-graph`. Default-features builds (native targets) do NOT
//! compile the module; the feature lights up the in-crate impl + the
//! crate-level re-export at [`crate::lib`] (see the `#[cfg(feature =
//! "browser-backend")]` re-export site).
//!
//! Enabling the feature on a non-wasm32 target compiles cleanly — the
//! backend is target-agnostic at the Rust level (uses only `std`) — but
//! the napi binding's `Engine = EngineGeneric<BrowserBackend>` alias is
//! gated behind the same feature on `benten-engine` so the alias re-points
//! only when the consumer opts in.
//!
//! ## What `BrowserBackend` does NOT do
//!
//! - Does NOT implement subgraph storage (the `s:CID` schema is not used
//!   in the thin-client cache; subgraphs land via the full-peer module
//!   registry and are mirrored to `BrowserBackend` only as Node-shaped
//!   payloads).
//! - Does NOT honor [`crate::DurabilityMode`] — there is no fsync
//!   semantic to honor. The in-RAM `BTreeMap` `insert` is the durable-
//!   write equivalent.
//! - Does NOT enforce content-addressing invariants beyond what callers
//!   already enforce. The full peer validates CIDs at write-time;
//!   `BrowserBackend` mirrors the validated bytes via
//!   [`Self::put_node_with_context`].
//!
//! ## Wave-8j-bisect § Phase-3-followup cite (per pim-1 §3.5b doc-coupling)
//!
//! The originating decision context for this backend lives at
//! `.addl/phase-2b/wave-8j-wasm-browser-bundle-bisect.md` § Phase-3
//! follow-up. That paragraph posited "≤350KB once redb is dropped"; the
//! Phase-3 spike at `.addl/phase-3/spike-bundle-cap-empirical.md` § 6
//! revised the cap to 600KB after the empirical anchor at
//! `crates/benten-graph/src/browser_backend.rs::BrowserBackend` (this
//! file) confirmed the dep-tree fan-out. Future PRs that tighten the cap
//! must update both citation sites in lockstep per pim-1.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use benten_core::{Cid, Edge, Node};

use crate::backend::{KVBackend, ScanResult};
use crate::graph_backend::GraphBackend;
use crate::prefix_helpers::next_prefix;
use crate::store::{
    ChangeSubscriber, EdgeStore, NodeStore, decode_err, edge_key, edge_src_index_key,
    edge_src_index_prefix, edge_tgt_index_key, edge_tgt_index_prefix, node_key,
};
use crate::{GraphError, WriteContext};

/// Marker handle returned by [`BrowserBackend::transaction`].
///
/// Browser thin-client writes are NOT transactionally atomic; the full
/// peer is the source of truth for atomicity per CLAUDE.md baked-in #17.
/// The marker exists so [`BrowserBackend`] satisfies the
/// [`GraphBackend::transaction`] umbrella shape; calling
/// [`BrowserBackend::transaction`] is permitted but does not fence
/// concurrent writes.
///
/// G13-C SHIPS THIS AS A UNIT MARKER — parallel to
/// [`crate::RedbTransactionRunner`] for the umbrella-shape pinning. The
/// thin-client commitment means the runner does NOT grow a closure-based
/// `run<F, R>` execution method in Phase 3.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BrowserTransactionRunner;

impl BrowserTransactionRunner {
    /// Construct a fresh runner handle.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Owned in-RAM snapshot of the [`BrowserBackend`] keyspace at the
/// `snapshot()` call instant.
///
/// The snapshot is an *independent owned clone* of the backing
/// `BTreeMap` — subsequent writes to the live backend cannot mutate the
/// snapshot per the `br-r4-r1-1` / `br-r4-r2-1` contract. This matches
/// the option-i fix-brief recommendation (Mutex-based clone-on-snapshot)
/// over the surprising shape where the snapshot would observe live
/// mutations.
///
/// For a thin-client cache the snapshot cost is bounded by the typical
/// browser-tab cache size (manifests + transient handler graphs +
/// recent subscription deliveries — sub-megabyte in practice), so the
/// clone-on-snapshot tradeoff is preferable to the lock-borrow shape
/// that would prevent concurrent writes during a snapshot's lifetime.
#[derive(Debug, Clone, Default)]
pub struct BrowserSnapshot {
    pairs: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl BrowserSnapshot {
    /// Number of (key, value) pairs in the snapshot.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    /// `true` if the snapshot is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pairs.is_empty()
    }

    /// Look up a key in the snapshot; returns the value bytes if present
    /// at the snapshot instant.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.pairs.get(key).map(Vec::as_slice)
    }

    /// Look up a Node by CID against the snapshot. Returns `Ok(None)` on
    /// a clean miss.
    ///
    /// # Errors
    /// Returns [`GraphError::Core`] (carrying a `CoreError::Serialize`)
    /// if the stored bytes fail to decode as a [`Node`].
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        let key = node_key(cid);
        match self.pairs.get(&key) {
            None => Ok(None),
            Some(bytes) => {
                let node: Node = serde_ipld_dagcbor::from_slice(bytes)
                    .map_err(decode_err)
                    .map_err(GraphError::from)?;
                Ok(Some(node))
            }
        }
    }
}

/// In-RAM thin-client cache backend for `wasm32-unknown-unknown` browser
/// tabs. See module docs for scope, semantics, and the load-bearing
/// thin-client commitment.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "browser-backend")] {
/// use benten_graph::{BrowserBackend, KVBackend};
///
/// let backend = BrowserBackend::new();
/// backend.put(b"k", b"v").unwrap();
/// assert_eq!(backend.get(b"k").unwrap().as_deref(), Some(&b"v"[..]));
/// # }
/// ```
#[derive(Debug, Default)]
pub struct BrowserBackend {
    inner: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
}

impl BrowserBackend {
    /// Construct an empty `BrowserBackend`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(BTreeMap::new()),
        }
    }

    /// Number of entries currently in the in-RAM cache. Test-only —
    /// production callers go through the trait surface.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.lock().map_or(0, |g| g.len())
    }

    /// `true` if the in-RAM cache is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.lock().map_or(true, |g| g.is_empty())
    }
}

#[inline]
fn poisoned() -> GraphError {
    GraphError::Redb("browser-backend: lock poisoned".into())
}

// ---------------------------------------------------------------------------
// KVBackend — byte-level get/put/delete/scan/put_batch
// ---------------------------------------------------------------------------

impl KVBackend for BrowserBackend {
    type Error = GraphError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        let g = self.inner.lock().map_err(|_| poisoned())?;
        Ok(g.get(key).cloned())
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), GraphError> {
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.remove(key);
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, GraphError> {
        let g = self.inner.lock().map_err(|_| poisoned())?;

        // Mirror the InMemoryBackend / RedbBackend prefix semantics:
        //  - empty prefix → full table iter
        //  - non-empty prefix → bounded range [prefix, next_prefix)
        //  - all-0xff prefix → unbounded prefix..
        let pairs: Vec<(Vec<u8>, Vec<u8>)> = if prefix.is_empty() {
            g.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
        } else {
            match next_prefix(prefix) {
                Some(upper) => g
                    .range(prefix.to_vec()..upper)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                None => g
                    .range(prefix.to_vec()..)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            }
        };

        Ok(ScanResult::from_iter(pairs))
    }

    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), GraphError> {
        // Atomic by virtue of holding the coarse `Mutex` for the whole
        // batch — every pair lands or none do (the only failure path is
        // lock poisoning, which fires before any mutation).
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        for (k, v) in pairs {
            g.insert(k.clone(), v.clone());
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// NodeStore — Node ↔ DAG-CBOR ↔ `n:CID` key schema
// ---------------------------------------------------------------------------

impl NodeStore for BrowserBackend {
    type Error = GraphError;

    fn put_node(&self, node: &Node) -> Result<Cid, GraphError> {
        let bytes = node.canonical_bytes().map_err(GraphError::from)?;
        let cid = node.cid().map_err(GraphError::from)?;
        let key = node_key(&cid);
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.insert(key, bytes);
        Ok(cid)
    }

    fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        let key = node_key(cid);
        let g = self.inner.lock().map_err(|_| poisoned())?;
        match g.get(&key) {
            None => Ok(None),
            Some(bytes) => {
                let node: Node = serde_ipld_dagcbor::from_slice(bytes)
                    .map_err(decode_err)
                    .map_err(GraphError::from)?;
                Ok(Some(node))
            }
        }
    }

    fn delete_node(&self, cid: &Cid) -> Result<(), GraphError> {
        let key = node_key(cid);
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.remove(&key);
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// EdgeStore — Edge ↔ DAG-CBOR ↔ `e:CID` + `es:` / `et:` index schema
// ---------------------------------------------------------------------------

impl EdgeStore for BrowserBackend {
    type Error = GraphError;

    fn put_edge(&self, edge: &Edge) -> Result<Cid, GraphError> {
        let bytes = edge.canonical_bytes().map_err(GraphError::from)?;
        let cid = edge.cid().map_err(GraphError::from)?;
        let body_key = edge_key(&cid);
        let src_idx = edge_src_index_key(&edge.source, &cid);
        let tgt_idx = edge_tgt_index_key(&edge.target, &cid);
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.insert(body_key, bytes);
        // Index entries store the edge CID bytes so `edges_from` / `edges_to`
        // can resolve back to the body. Mirror RedbBackend's index payload
        // shape (raw CID bytes) so the snapshot view sees identical layout.
        g.insert(src_idx, cid.as_bytes().to_vec());
        g.insert(tgt_idx, cid.as_bytes().to_vec());
        Ok(cid)
    }

    fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, GraphError> {
        let key = edge_key(cid);
        let g = self.inner.lock().map_err(|_| poisoned())?;
        match g.get(&key) {
            None => Ok(None),
            Some(bytes) => {
                let edge: Edge = serde_ipld_dagcbor::from_slice(bytes)
                    .map_err(decode_err)
                    .map_err(GraphError::from)?;
                Ok(Some(edge))
            }
        }
    }

    fn delete_edge(&self, cid: &Cid) -> Result<(), GraphError> {
        // Read the edge body first so we can clean up the index entries.
        // Idempotent — a missing edge is `Ok(())`.
        let key = edge_key(cid);
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        if let Some(bytes) = g.get(&key).cloned() {
            let edge: Edge = serde_ipld_dagcbor::from_slice(&bytes)
                .map_err(decode_err)
                .map_err(GraphError::from)?;
            g.remove(&edge_src_index_key(&edge.source, cid));
            g.remove(&edge_tgt_index_key(&edge.target, cid));
            g.remove(&key);
        }
        Ok(())
    }

    fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, GraphError> {
        let prefix = edge_src_index_prefix(source);
        let g = self.inner.lock().map_err(|_| poisoned())?;
        let upper = next_prefix(&prefix);
        let iter: Box<dyn Iterator<Item = (&Vec<u8>, &Vec<u8>)>> = match upper {
            Some(u) => Box::new(g.range(prefix.clone()..u)),
            None => Box::new(g.range(prefix.clone()..)),
        };
        let mut edges = Vec::new();
        for (_idx_key, idx_val) in iter {
            // `idx_val` carries the edge CID bytes; resolve the body.
            let edge_cid = Cid::from_bytes(idx_val.as_slice()).map_err(|e| {
                GraphError::Decode(format!("browser-backend edges_from: index entry: {e}"))
            })?;
            if let Some(bytes) = g.get(&edge_key(&edge_cid)) {
                let edge: Edge = serde_ipld_dagcbor::from_slice(bytes)
                    .map_err(decode_err)
                    .map_err(GraphError::from)?;
                edges.push(edge);
            }
        }
        Ok(edges)
    }

    fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, GraphError> {
        let prefix = edge_tgt_index_prefix(target);
        let g = self.inner.lock().map_err(|_| poisoned())?;
        let upper = next_prefix(&prefix);
        let iter: Box<dyn Iterator<Item = (&Vec<u8>, &Vec<u8>)>> = match upper {
            Some(u) => Box::new(g.range(prefix.clone()..u)),
            None => Box::new(g.range(prefix.clone()..)),
        };
        let mut edges = Vec::new();
        for (_idx_key, idx_val) in iter {
            let edge_cid = Cid::from_bytes(idx_val.as_slice()).map_err(|e| {
                GraphError::Decode(format!("browser-backend edges_to: index entry: {e}"))
            })?;
            if let Some(bytes) = g.get(&edge_key(&edge_cid)) {
                let edge: Edge = serde_ipld_dagcbor::from_slice(bytes)
                    .map_err(decode_err)
                    .map_err(GraphError::from)?;
                edges.push(edge);
            }
        }
        Ok(edges)
    }
}

// ---------------------------------------------------------------------------
// GraphBackend umbrella impl
// ---------------------------------------------------------------------------

impl GraphBackend for BrowserBackend {
    type Snapshot = BrowserSnapshot;
    type Error = GraphError;
    type Transaction = BrowserTransactionRunner;

    /// Returns a marker [`BrowserTransactionRunner`]. The thin-client
    /// commitment means there is NO atomic-commit semantic at the
    /// browser cache layer — concurrent writes from different async tasks
    /// may race. The full peer is the source of truth for atomicity per
    /// CLAUDE.md baked-in #17.
    fn transaction(&self) -> Self::Transaction {
        BrowserTransactionRunner::new()
    }

    /// Silent no-op fan-out per CLAUDE.md baked-in #17.
    ///
    /// Browser tabs subscribe to the FULL PEER over Server-Sent Events /
    /// WebSocket per G14-D thin-client subscription. Local-cache writes
    /// are NOT republished to local subscribers — that would either
    /// (a) double-fire against the full-peer subscription or (b)
    /// silently change the browser-tab UX. The umbrella shape is
    /// preserved so the engine can wire IVM views uniformly without
    /// conditional code paths per backend.
    ///
    /// The `subscriber` argument is intentionally dropped immediately:
    /// the browser thin-client cache holds no subscriber list, no fan-out
    /// channel, and no event source.
    fn register_subscriber(&self, _subscriber: Arc<dyn ChangeSubscriber>) {
        // Intentionally empty — see docstring.
    }

    /// Owned [`BrowserSnapshot`] independent of subsequent live writes
    /// per the `br-r4-r1-1` / `br-r4-r2-1` contract.
    ///
    /// Clones the backing `BTreeMap` on entry — for typical browser-tab
    /// cache sizes (sub-megabyte) the clone is cheap and the
    /// independence-from-live-writes guarantee is the load-bearing
    /// shape.
    fn snapshot(&self) -> Self::Snapshot {
        let g = self.inner.lock();
        let pairs = match g {
            Ok(guard) => guard.clone(),
            // A poisoned lock means a previous holder panicked; we still
            // surface a usable (empty) snapshot rather than panicking
            // again at the trait boundary. The inherent KVBackend path
            // surfaces the typed error; trait-level snapshot has no
            // Result shape per `arch-r1-6`.
            Err(_) => BTreeMap::new(),
        };
        BrowserSnapshot { pairs }
    }

    /// Privileged thin-client cache write path.
    ///
    /// **Cap-recheck is BYPASSED at the cache layer per CLAUDE.md
    /// baked-in #17.** The upstream subscription (G14-D) already filters
    /// events per delivered-subscriber's grant; the local cache simply
    /// mirrors the authorized stream. Re-running cap-policy on the cache
    /// write path would double-count rate-limits and would couple the
    /// browser bundle to the cap-policy crate (defying the bundle-cap
    /// commitment per `.addl/phase-3/spike-bundle-cap-empirical.md` §6).
    ///
    /// System-zone label gating from the [`WriteContext`] is honored —
    /// even though the browser thin-client never originates a system-zone
    /// write, the gate is preserved so a buggy thin-client subscription
    /// that delivered a system-zone event without privilege would still
    /// surface the typed [`GraphError::SystemZoneWrite`] at the cache
    /// boundary rather than silently caching the bytes.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone
    ///   label.
    /// - [`GraphError::Core`] (carrying `CoreError::Serialize`) on
    ///   DAG-CBOR encode failure.
    /// - [`GraphError::Redb`] (with a `"browser-backend: lock poisoned"`
    ///   payload) if the cache mutex was poisoned by a prior panicking
    ///   holder.
    fn put_node_with_context(
        &self,
        node: &Node,
        ctx: &WriteContext,
    ) -> Result<Cid, <Self as GraphBackend>::Error> {
        // System-zone gate (preserved per docstring).
        if !ctx.is_privileged {
            for label in &node.labels {
                if label.starts_with("system:") {
                    return Err(GraphError::SystemZoneWrite {
                        label: label.clone(),
                    });
                }
            }
        }

        // Encode + insert. Cap-recheck is intentionally NOT consulted
        // here — see docstring.
        let bytes = node.canonical_bytes().map_err(GraphError::from)?;
        let cid = node.cid().map_err(GraphError::from)?;
        let key = node_key(&cid);
        let mut g = self.inner.lock().map_err(|_| poisoned())?;
        g.insert(key, bytes);
        Ok(cid)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    /// Compile-time witness: BrowserBackend satisfies the
    /// [`GraphBackend`] umbrella per `D-PHASE-3-1` RESOLVED.
    #[test]
    fn browser_backend_satisfies_graph_backend_umbrella() {
        fn assert_graph_backend<B: GraphBackend>() {}
        assert_graph_backend::<BrowserBackend>();
    }

    /// Compile-time witness: `Snapshot: Send + Sync + 'static` per
    /// `arch-r1-6`.
    #[test]
    fn browser_backend_snapshot_is_send_sync_static() {
        fn assert_send_sync_static<T: Send + Sync + 'static>() {}
        assert_send_sync_static::<BrowserSnapshot>();
    }

    #[test]
    fn kv_round_trip() {
        let backend = BrowserBackend::new();
        backend.put(b"n:test", b"data").unwrap();
        assert_eq!(
            backend.get(b"n:test").unwrap().as_deref(),
            Some(&b"data"[..])
        );
        backend.delete(b"n:test").unwrap();
        assert_eq!(backend.get(b"n:test").unwrap(), None);
    }

    #[test]
    fn node_store_round_trip() {
        let backend = BrowserBackend::new();
        let node = canonical_test_node();
        let cid = backend.put_node(&node).unwrap();
        assert_eq!(backend.get_node(&cid).unwrap().as_ref(), Some(&node));
        backend.delete_node(&cid).unwrap();
        assert!(backend.get_node(&cid).unwrap().is_none());
    }

    #[test]
    fn snapshot_independence_smoke() {
        let backend = BrowserBackend::new();
        backend.put(b"k1", b"v1").unwrap();
        let snap = backend.snapshot();
        // Live mutation after snapshot:
        backend.put(b"k2", b"v2").unwrap();
        backend.put(b"k1", b"v1-modified").unwrap();
        assert_eq!(snap.len(), 1);
        assert_eq!(snap.get(b"k1"), Some(&b"v1"[..]));
        assert_eq!(snap.get(b"k2"), None);
    }

    #[test]
    fn register_subscriber_is_silent_no_op() {
        // Registering a subscriber must NOT cause any observable
        // change — there is no fan-out, no error path, no panic.
        struct CountingSub(std::sync::atomic::AtomicUsize);
        impl ChangeSubscriber for CountingSub {
            fn on_change(&self, _event: &crate::ChangeEvent) {
                self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            }
        }

        let backend = BrowserBackend::new();
        let sub = Arc::new(CountingSub(std::sync::atomic::AtomicUsize::new(0)));
        backend.register_subscriber(sub.clone());
        backend.put(b"n:x", b"y").unwrap();
        assert_eq!(sub.0.load(std::sync::atomic::Ordering::SeqCst), 0);
    }

    #[test]
    fn put_node_with_context_bypasses_cap_recheck_at_cache_layer() {
        // Per CLAUDE.md baked-in #17: the cache write path mirrors
        // upstream-authorized bytes — it does NOT consult cap-policy.
        // The observable consequence is that a non-privileged context
        // for a regular-label node succeeds without hitting any
        // cap-policy plug.
        let backend = BrowserBackend::new();
        let node = canonical_test_node();
        let ctx = WriteContext::new("post"); // user authority, non-privileged
        let cid = backend
            .put_node_with_context(&node, &ctx)
            .expect("non-privileged regular-label put_node_with_context succeeds");
        assert_eq!(node.cid().unwrap(), cid);
        assert_eq!(backend.get_node(&cid).unwrap().as_ref(), Some(&node));
    }

    #[test]
    fn put_node_with_context_system_zone_gate_preserved() {
        let backend = BrowserBackend::new();
        let mut sys_node = canonical_test_node();
        sys_node.labels = vec!["system:Critical".into()];
        let ctx = WriteContext::new("system:Critical"); // non-privileged
        let err = backend
            .put_node_with_context(&sys_node, &ctx)
            .expect_err("non-privileged system-zone write rejected");
        assert!(matches!(err, GraphError::SystemZoneWrite { .. }));
    }
}

//! Phase-3 G14-C wave-4b — concrete redb-native [`BlobBackend`] impl.
//!
//! ## What this is
//!
//! Closure of **Compromise #17** (in-memory module-bytes registry).
//! Implements the [`BlobBackend`]
//! trait surface scaffolded at G13-pre-B in
//! `crates/benten-graph/src/backends/blob_backend_trait.rs` over a
//! redb-backed durable side-table (`system:ModuleBytes` zone Nodes).
//!
//! ## Storage shape
//!
//! Each blob is persisted as a Node in the `system:ModuleBytes` zone:
//!
//! - `labels: ["system:ModuleBytes"]`
//! - `properties.blob_cid: Value::Text(cid.to_base32())`
//! - `properties.blob_bytes: Value::Bytes(bytes)`
//!
//! The `system:` zone prefix is a privileged-write surface
//! (`crates/benten-engine/src/system_zones.rs::SYSTEM_ZONE_PREFIXES`),
//! mirroring how `system:ModuleManifest` already persists module
//! manifests at G10-B. Storing blobs as graph Nodes — rather than as
//! a parallel redb side-table the storage layer would need to crack
//! open — preserves CLAUDE.md baked-in commitment #3 (code-as-graph)
//! and keeps the durable surface uniformly inspectable via the
//! existing `get_by_label` accessor.
//!
//! Per CLAUDE.md baked-in commitment #4 (BLAKE3 + DAG-CBOR + CIDv1),
//! blob CIDs are computed as `BLAKE3(blob_bytes)` wrapped in the
//! Benten CIDv1 envelope (matches `Cid::from_blake3_digest`). The
//! authoritative CID-validation gate lives at the engine's
//! `Engine::register_module_bytes` call site per
//! D-PHASE-3-12 RESOLVED — the `BlobBackend::put` impl here recomputes
//! defense-in-depth so an attacker writing into the backend directly
//! cannot poison the cache (the CID stored in `properties.blob_cid`
//! is hash-derived from the bytes).
//!
//! ## Why not a raw redb side-table
//!
//! The trait-level put accepts a caller-supplied CID; storing blobs
//! as system-zone Nodes lets us reuse the existing
//! `put_node_with_context` privileged-write path + the
//! `get_by_label` / `get_node` accessors for rehydration. A parallel
//! redb side-table would duplicate the privileged-write capability
//! plumbing + require new public methods on `RedbBackend` for blob-
//! shaped reads + writes. The system-zone Node shape is structurally
//! the simpler surface.
//!
//! ## Async-shape adapter
//!
//! The [`BlobBackend`]
//! trait returns `impl Future + Send` per D-PHASE-3-7. redb itself is
//! synchronous, so this impl wraps each operation in
//! `core::future::ready(...)`. The browser-side IndexedDB impl
//! (G18-A wave-5a) will use the natively-async IDB API directly.

use core::future::{Future, ready};
use std::sync::Arc;

use benten_core::{Cid, Node, Value};

use crate::backends::blob_backend_trait::BlobBackend;
use crate::redb_backend::RedbBackend;
use crate::store::NodeStore;
use crate::{GraphError, WriteContext};

/// G14-C label used for the durable module-bytes side-table. The label
/// MUST start with `system:` so the `guard_system_zone_node` helper at
/// the redb backend's write boundary requires the privileged
/// [`WriteContext::privileged_for_engine_api`] context to write here.
pub const MODULE_BYTES_LABEL: &str = "system:ModuleBytes";

/// G14-C property key carrying the blob's CID base32 string. Set on
/// every `system:ModuleBytes` Node so a rehydration scan can index by
/// CID without re-canonicalizing the Node.
pub const BLOB_CID_PROPERTY: &str = "blob_cid";

/// G14-C property key carrying the blob's raw bytes. `Value::Bytes`
/// round-trips through DAG-CBOR.
pub const BLOB_BYTES_PROPERTY: &str = "blob_bytes";

/// Hard cap on the number of `system:ModuleBytes` Nodes a single
/// [`RedbBlobBackend::get_sync`] / [`RedbBlobBackend::list_blob_cids`]
/// scan will decode before refusing (#567, META #629 DoS-via-unbounded-
/// decode benten-graph slice).
///
/// Both methods do an O(N) linear scan that decodes every Node body in the
/// `system:ModuleBytes` zone. The zone is system-zone (privileged-write
/// only — see [`MODULE_BYTES_LABEL`]) so an *unprivileged* peer cannot
/// inflate it directly, but the O(N)-decode-per-fetch amplification is a
/// real cost ceiling for legitimately-large module sets and a defense-in-
/// depth bound should a privileged-write path ever be reached unexpectedly.
/// 100k modules is far beyond any realistic deployment (modules are
/// operator-curated, not user-generated) while still bounding the worst-
/// case decode work a single scan can perform. Exceeding it surfaces
/// [`GraphError::DecodeTooLarge`] rather than silently doing unbounded work.
pub const MAX_MODULE_BYTES_ZONE_SCAN: usize = 100_000;

/// Concrete redb-native [`BlobBackend`] implementation closing
/// **Compromise #17**.
///
/// Constructed from an existing `Arc<RedbBackend>` so the same redb
/// database that holds the engine's `system:ModuleManifest` zone +
/// data Nodes also holds the durable module-bytes blobs — one
/// fsync-coherent on-disk store, no separate durability story.
///
/// The handle is `Clone`able cheaply: the inner `Arc<RedbBackend>` is
/// shared. Cloning does NOT clone the redb database — readers and a
/// single writer cooperate through redb's MVCC / single-writer-lock.
#[derive(Clone)]
pub struct RedbBlobBackend {
    backend: Arc<RedbBackend>,
}

impl RedbBlobBackend {
    /// Construct a new [`RedbBlobBackend`] over an existing
    /// `Arc<RedbBackend>` handle.
    ///
    /// The backend handle is the same one the engine uses for its
    /// data + system-zone writes — sharing the redb file means the
    /// blob writes are atomic-with-respect-to system-zone manifests
    /// in the same on-disk database (no cross-store consistency
    /// concern at engine-open time).
    #[must_use]
    pub fn new(backend: Arc<RedbBackend>) -> Self {
        Self { backend }
    }

    /// List the CIDs of every blob currently persisted in the
    /// `system:ModuleBytes` zone. Used by
    /// `Engine::rehydrate_module_bytes_from_zone` at
    /// engine-open time to repopulate the in-memory cache.
    ///
    /// # Errors
    ///
    /// Surfaces [`GraphError`] from the backend's index lookup.
    pub fn list_blob_cids(&self) -> Result<Vec<Cid>, GraphError> {
        // The `system:ModuleBytes` Node CID is the *Node's* CID
        // (which hashes labels + properties), NOT the blob's CID.
        // Callers that want the blob CID must read the Node + decode
        // the `blob_cid` property. The two-CID dance preserves the
        // blob CID = BLAKE3(blob_bytes) invariant per #4 baked-in.
        let node_cids = self.backend.get_by_label(MODULE_BYTES_LABEL)?;
        if node_cids.len() > MAX_MODULE_BYTES_ZONE_SCAN {
            return Err(GraphError::DecodeTooLarge {
                actual: node_cids.len(),
                limit: MAX_MODULE_BYTES_ZONE_SCAN,
            });
        }
        let mut blob_cids = Vec::with_capacity(node_cids.len());
        for node_cid in node_cids {
            let Some(node) = self.backend.get_node(&node_cid)? else {
                continue;
            };
            let Some(Value::Text(cid_str)) = node.properties.get(BLOB_CID_PROPERTY) else {
                continue;
            };
            let Ok(cid) = Cid::from_str(cid_str) else {
                continue;
            };
            blob_cids.push(cid);
        }
        Ok(blob_cids)
    }

    /// Fetch blob bytes by their content-addressed CID — synchronous
    /// inherent counterpart to [`BlobBackend::get`] for callers that
    /// already hold an `Arc<RedbBlobBackend>` and don't want to
    /// drive the future. The async trait method delegates here.
    ///
    /// # Errors
    ///
    /// Surfaces [`GraphError`] from the backend's index + node read.
    pub fn get_sync(&self, cid: &Cid) -> Result<Option<Vec<u8>>, GraphError> {
        // Linear scan over the system:ModuleBytes zone — fine in
        // Phase-3 (operator-bounded module count), Phase-4+ may add
        // a CID-keyed property index if profiling demands. The scan is
        // bounded by `MAX_MODULE_BYTES_ZONE_SCAN` so a pathologically
        // large zone cannot turn a single fetch into unbounded
        // per-Node-decode work (#567, META #629 slice).
        let node_cids = self.backend.get_by_label(MODULE_BYTES_LABEL)?;
        if node_cids.len() > MAX_MODULE_BYTES_ZONE_SCAN {
            return Err(GraphError::DecodeTooLarge {
                actual: node_cids.len(),
                limit: MAX_MODULE_BYTES_ZONE_SCAN,
            });
        }
        let target = cid.to_base32();
        for node_cid in node_cids {
            let Some(node) = self.backend.get_node(&node_cid)? else {
                continue;
            };
            let Some(Value::Text(stored_cid)) = node.properties.get(BLOB_CID_PROPERTY) else {
                continue;
            };
            if stored_cid != &target {
                continue;
            }
            let Some(Value::Bytes(bytes)) = node.properties.get(BLOB_BYTES_PROPERTY) else {
                continue;
            };
            return Ok(Some(bytes.clone()));
        }
        Ok(None)
    }

    /// Persist blob bytes under their content-addressed CID —
    /// synchronous inherent counterpart to [`BlobBackend::put`]. The
    /// async trait method delegates here.
    ///
    /// Defense-in-depth: recomputes `BLAKE3(bytes)` and rejects with
    /// [`BlobError::CidMismatch`] if it does not match the
    /// caller-supplied CID. The authoritative validator lives at
    /// `Engine::register_module_bytes` per D-PHASE-3-12 RESOLVED;
    /// this re-check guards against direct backend writes that
    /// bypass the engine entry point.
    ///
    /// # Errors
    ///
    /// - [`BlobError::CidMismatch`] when `BLAKE3(bytes) != cid`.
    /// - [`BlobError::Graph`] when the redb privileged-write surface
    ///   surfaces a backend error.
    pub fn put_sync(&self, cid: &Cid, bytes: &[u8]) -> Result<(), BlobError> {
        let recomputed = Cid::from_blake3_digest(*blake3::hash(bytes).as_bytes());
        if &recomputed != cid {
            return Err(BlobError::CidMismatch {
                expected: *cid,
                computed: recomputed,
            });
        }

        let mut props: std::collections::BTreeMap<String, Value> =
            std::collections::BTreeMap::new();
        props.insert(BLOB_CID_PROPERTY.to_string(), Value::Text(cid.to_base32()));
        props.insert(
            BLOB_BYTES_PROPERTY.to_string(),
            Value::Bytes(bytes.to_vec()),
        );
        let node = Node::new(vec![MODULE_BYTES_LABEL.to_string()], props);

        // Idempotent write: if a Node with the same canonical content
        // (label + properties) is already present, redb's Inv-13
        // dedup path returns Ok without bumping commit counters.
        // The canonical Node CID here hashes label + properties +
        // `blob_cid` + `blob_bytes`, so two writes of identical
        // (cid, bytes) collapse to a single Node.
        self.backend
            .put_node_with_context(&node, &WriteContext::privileged_for_engine_api())
            .map_err(BlobError::Graph)?;
        Ok(())
    }

    /// Evict the blob stored under `cid` — synchronous inherent
    /// counterpart to [`BlobBackend::delete`]. The async trait method
    /// delegates here.
    ///
    /// Idempotent: a CID with no backing Node returns `Ok(())`. The
    /// `system:ModuleBytes` zone is walked to locate the Node whose
    /// `blob_cid` property matches `cid` (the two-CID dance — the Node's
    /// CID hashes label + properties, NOT the blob bytes); the matching
    /// Node is removed via the backend's privileged `delete_node`.
    ///
    /// # Errors
    ///
    /// Surfaces [`BlobError::Graph`] from the backend's index walk or
    /// node-delete surface.
    pub fn delete_sync(&self, cid: &Cid) -> Result<(), BlobError> {
        let node_cids = self
            .backend
            .get_by_label(MODULE_BYTES_LABEL)
            .map_err(BlobError::Graph)?;
        let target = cid.to_base32();
        for node_cid in node_cids {
            let Some(node) = self.backend.get_node(&node_cid).map_err(BlobError::Graph)? else {
                continue;
            };
            let Some(Value::Text(stored_cid)) = node.properties.get(BLOB_CID_PROPERTY) else {
                continue;
            };
            if stored_cid != &target {
                continue;
            }
            self.backend
                .delete_node(&node_cid)
                .map_err(BlobError::Graph)?;
        }
        Ok(())
    }
}

impl BlobBackend for RedbBlobBackend {
    type Error = BlobError;

    fn get(&self, cid: &Cid) -> impl Future<Output = Result<Option<Vec<u8>>, Self::Error>> + Send {
        let result = self.get_sync(cid).map_err(BlobError::Graph);
        ready(result)
    }

    fn put(&self, cid: &Cid, bytes: &[u8]) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let result = self.put_sync(cid, bytes);
        ready(result)
    }

    fn is_persistent(&self) -> bool {
        true
    }

    fn delete(&self, cid: &Cid) -> impl Future<Output = Result<(), Self::Error>> + Send {
        ready(self.delete_sync(cid))
    }

    fn list_cids(&self) -> impl Future<Output = Result<Vec<Cid>, Self::Error>> + Send {
        ready(self.list_blob_cids().map_err(BlobError::Graph))
    }
}

/// G14-C [`RedbBlobBackend`] error surface. Routes both
/// content-integrity violations (cid-mismatch — defense-in-depth at
/// the trait boundary per D-PHASE-3-12) and underlying redb errors
/// into one typed enum so consumers can `.source()`-chain across
/// the heterogeneous backends.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BlobError {
    /// `BLAKE3(bytes) != cid` at the put boundary. Carries both CIDs
    /// so the operator can identify the mis-paired blob from logs
    /// alone (mirrors the
    /// `EngineError::ModuleManifestCidMismatch` shape — see
    /// `crates/benten-engine/src/error.rs`).
    #[error("blob CID mismatch: expected {expected}, computed {computed}")]
    CidMismatch {
        /// The caller-supplied CID under which the bytes were
        /// supposed to be stored.
        expected: Cid,
        /// The CID `BLAKE3(bytes)` actually produced.
        computed: Cid,
    },

    /// Underlying redb / graph-layer error wrapping the storage call.
    #[error(transparent)]
    Graph(#[from] GraphError),
}

impl BlobError {
    /// Stable error code for cross-language surfacing. Mirrors the
    /// other graph-layer error enums.
    #[must_use]
    pub fn code(&self) -> benten_errors::ErrorCode {
        match self {
            BlobError::CidMismatch { .. } => {
                benten_errors::ErrorCode::Unknown("E_MODULE_BYTES_CID_MISMATCH".into())
            }
            BlobError::Graph(g) => g.code(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn open_backend() -> Arc<RedbBackend> {
        Arc::new(RedbBackend::open_in_memory().unwrap())
    }

    #[test]
    fn round_trip_put_get() {
        let backend = open_backend();
        let blob = RedbBlobBackend::new(backend);
        let bytes = b"hello-blob".to_vec();
        let cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());
        blob.put_sync(&cid, &bytes).unwrap();
        let got = blob.get_sync(&cid).unwrap();
        assert_eq!(got, Some(bytes));
    }

    #[test]
    fn put_rejects_cid_mismatch_defense_in_depth() {
        let backend = open_backend();
        let blob = RedbBlobBackend::new(backend);
        let bytes = b"actual-bytes".to_vec();
        let wrong_cid = Cid::from_blake3_digest(*blake3::hash(b"different-bytes").as_bytes());
        let err = blob.put_sync(&wrong_cid, &bytes).unwrap_err();
        assert!(matches!(err, BlobError::CidMismatch { .. }));
    }

    #[test]
    fn get_returns_none_for_unknown_cid() {
        let backend = open_backend();
        let blob = RedbBlobBackend::new(backend);
        let bytes = b"unrelated".to_vec();
        let cid = Cid::from_blake3_digest(*blake3::hash(&bytes).as_bytes());
        assert!(blob.get_sync(&cid).unwrap().is_none());
    }

    #[test]
    fn is_persistent_true_for_redb_backend() {
        let backend = open_backend();
        let blob = RedbBlobBackend::new(backend);
        // Per D-PHASE-3-7 / CLAUDE.md baked-in #17 — redb-native is
        // a full-peer durable backend.
        assert!(blob.is_persistent());
    }

    #[test]
    fn list_blob_cids_returns_persisted_set() {
        let backend = open_backend();
        let blob = RedbBlobBackend::new(backend);
        let bytes_a = b"first".to_vec();
        let bytes_b = b"second".to_vec();
        let cid_a = Cid::from_blake3_digest(*blake3::hash(&bytes_a).as_bytes());
        let cid_b = Cid::from_blake3_digest(*blake3::hash(&bytes_b).as_bytes());
        blob.put_sync(&cid_a, &bytes_a).unwrap();
        blob.put_sync(&cid_b, &bytes_b).unwrap();
        let cids = blob.list_blob_cids().unwrap();
        assert!(cids.contains(&cid_a));
        assert!(cids.contains(&cid_b));
        assert_eq!(cids.len(), 2);
    }
}

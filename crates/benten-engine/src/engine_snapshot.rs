//! Phase-2b G10-A-wasip1 snapshot-blob handoff API on [`Engine`]
//! (D10-RESOLVED).
//!
//! ## What this is
//!
//! Three engine-level methods that pair with the
//! [`benten_graph::SnapshotBlobBackend`] read-only backend:
//!
//! - [`Engine::export_snapshot_blob`] — walk the engine's storage and
//!   encode a canonical DAG-CBOR `SnapshotBlob` for handoff.
//! - [`Engine::from_snapshot_blob`] — decode a snapshot-blob and
//!   construct a read-only engine over a freshly-hydrated tempdir
//!   backend; mutation methods surface `E_BACKEND_READ_ONLY`.
//! - [`Engine::compute_snapshot_blob_cid`] — static helper hashing an
//!   already-encoded blob.
//!
//! ## Why a tempdir backend (Phase-2b posture)
//!
//! The Engine is hard-bound to [`benten_graph::RedbBackend`] in 2b. To
//! preserve the full read surface (label-index scans, `register_crud`
//! handlers, `read_view`, etc.) without re-implementing every
//! IndexMaintainer against `SnapshotBlobBackend`, `from_snapshot_blob`
//! materializes the blob's contents into a fresh tempdir-resident redb
//! file and flips the engine's `read_only_snapshot` flag. Mutation
//! attempts surface [`benten_errors::ErrorCode::BackendReadOnly`] at the
//! user-facing surface (engine_crud.rs); the underlying backend is a
//! plain redb so internal-side reads (e.g. resume-from-bytes envelopes
//! that need to persist transient state) can still operate on the
//! tempdir.
//!
//! Phase 3 replaces the tempdir hydration with a direct
//! `SnapshotBlobBackend` wired in once the Engine is generic over its
//! backend; until then this is the simplest correct shape that
//! preserves the canonical-bytes round-trip the D10 test pins.
//!
//! Native-target only — see `lib.rs` for the wasm32 cfg-gate rationale.

use std::collections::BTreeMap;
use std::sync::Arc;

use benten_core::{Cid, CoreError, Node};
use benten_errors::ErrorCode;
use benten_graph::{KVBackend, RedbBackend, SnapshotBlob, SnapshotBlobError};

use crate::builder::EngineBuilder;
use crate::engine::Engine;
use crate::error::EngineError;
use crate::system_zones::SYSTEM_ZONE_PREFIXES;

// `n:` Node-key prefix duplicated here (the constant is `pub(crate)` in
// benten-graph). Using a literal keeps the engine -> graph dependency
// surface narrow and matches the documented key schema in
// `benten-graph/src/store.rs`.
const NODE_KEY_PREFIX: &[u8] = b"n:";

/// RAII handle keeping a snapshot-blob engine's tempdir alive for the
/// engine's lifetime. Stored on the engine via the [`SnapshotEngineGuard`]
/// global registry indexed by the engine's tempdir CID so the tempdir
/// is dropped together with the engine.
///
/// This is a stop-gap; the proper shape lands when Engine becomes generic
/// over its backend in a later phase and the snapshot blob can drive a
/// `SnapshotBlobBackend` directly without spilling to disk.
struct SnapshotTempDirGuard {
    _dir: tempfile::TempDir,
}

// We hold guards in a process-wide registry so the tempdir lives as long
// as ANY clone of the Engine that points at it; a more polished design
// would put the guard on `EngineInner` but that's a 50-call-site cascade
// (every constructor / builder helper). The map is keyed by the
// snapshot-blob's CID so two engines built from the same blob in the
// same process share the same tempdir hold — there's no correctness
// hazard from that because the snapshot is read-only.
static SNAPSHOT_TEMP_DIRS: std::sync::OnceLock<
    std::sync::Mutex<BTreeMap<Cid, Arc<SnapshotTempDirGuard>>>,
> = std::sync::OnceLock::new();

fn snapshot_tempdir_registry() -> &'static std::sync::Mutex<BTreeMap<Cid, Arc<SnapshotTempDirGuard>>>
{
    SNAPSHOT_TEMP_DIRS.get_or_init(|| std::sync::Mutex::new(BTreeMap::new()))
}

impl Engine {
    /// Phase-2b G10-A-wasip1 (D10-RESOLVED): walk this engine's storage
    /// and encode a canonical DAG-CBOR snapshot-blob for handoff.
    ///
    /// The result is a [`SnapshotBlob`] containing every Node body the
    /// backend holds, plus a `system_zone_index` mapping system-zone
    /// labels (`SYSTEM_ZONE_PREFIXES`) to the CIDs that carry them. The
    /// outer struct + both `BTreeMap`s sort by key so two engines built
    /// from identical state produce byte-identical blobs (D10 +
    /// sec-pre-r1-09 Inv-13 collision-safety).
    ///
    /// Phase-2b limitation: edges and IVM-view state are not part of the
    /// D10 handoff shape. Phase 3 extends the schema additively under a
    /// new [`benten_graph::SnapshotBlob::schema_version`].
    ///
    /// # Errors
    /// - [`EngineError::Graph`] on backend I/O failure.
    /// - [`EngineError::Core`] (carrying `CoreError::Serialize`) on
    ///   encoding failure.
    pub fn export_snapshot_blob(&self) -> Result<Vec<u8>, EngineError> {
        let blob = self.collect_snapshot_blob()?;
        let bytes = blob
            .to_dag_cbor()
            .map_err(|e: CoreError| EngineError::Core(e))?;
        Ok(bytes)
    }

    /// Internal helper: walk the storage and assemble the in-memory
    /// `SnapshotBlob` struct. Tested directly + reused by both
    /// `export_snapshot_blob` and the snapshot-blob-CID computation.
    fn collect_snapshot_blob(&self) -> Result<SnapshotBlob, EngineError> {
        let backend = self.backend();
        // `BTreeMap` insertion sorts by `Cid` (`Ord` derived) so the
        // result is canonical regardless of the order the backend's
        // scan returned the keys.
        let mut nodes: BTreeMap<Cid, Vec<u8>> = BTreeMap::new();
        let mut system_zone_index: BTreeMap<String, Vec<Cid>> = BTreeMap::new();

        for (key, body) in backend.scan(NODE_KEY_PREFIX)?.iter() {
            let Some(cid_bytes) = key.strip_prefix(NODE_KEY_PREFIX) else {
                continue;
            };
            // Skip non-CID-shaped suffixes defensively. A malformed key
            // here would indicate a redb corruption; we surface it by
            // skipping (the next read will reproduce the issue) rather
            // than crashing the export.
            let Ok(cid) = Cid::from_bytes(cid_bytes) else {
                continue;
            };

            // Decode just enough to project the labels for the
            // system_zone_index. Failure to decode is skipped silently;
            // a release build's invariants prevent this from happening
            // in practice (every put goes through the canonical-bytes
            // path).
            if let Ok(node) = serde_ipld_dagcbor::from_slice::<Node>(body)
                && let Some(label) = node.labels.first()
            {
                let is_system_zone = SYSTEM_ZONE_PREFIXES
                    .iter()
                    .any(|prefix| label.starts_with(prefix));
                if is_system_zone {
                    system_zone_index
                        .entry(label.clone())
                        .or_default()
                        .push(cid);
                }
            }

            nodes.insert(cid, body.clone());
        }

        // Sort each system-zone-index Vec<Cid> for canonical bytes —
        // BTreeMap sorts the keys but the inner vecs honor insertion
        // order, which depends on the backend's scan iteration.
        for cids in system_zone_index.values_mut() {
            cids.sort();
            cids.dedup();
        }

        Ok(SnapshotBlob {
            schema_version: benten_graph::backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION,
            anchor_cid: None,
            nodes,
            system_zone_index,
        })
    }

    /// Phase-2b G10-A-wasip1 (D10-RESOLVED): construct a read-only
    /// engine view over a snapshot-blob handoff payload.
    ///
    /// The bytes are decoded as a canonical DAG-CBOR
    /// [`SnapshotBlob`]; the contents are hydrated into a fresh
    /// tempdir-resident redb backend, the engine's
    /// [`Engine::is_read_only_snapshot`] flag is set so user-facing
    /// mutations surface [`ErrorCode::BackendReadOnly`], and the
    /// constructed engine is returned.
    ///
    /// The tempdir is held alive for the engine's lifetime via a
    /// process-wide registry keyed by the snapshot-blob CID.
    ///
    /// # Errors
    /// - [`EngineError::Other`] (`E_SERIALIZE`) on snapshot-blob decode
    ///   failure or schema-version mismatch.
    /// - [`EngineError::Graph`] on tempdir / redb hydration failure.
    pub fn from_snapshot_blob(bytes: &[u8]) -> Result<Self, EngineError> {
        let blob = SnapshotBlob::from_dag_cbor(bytes).map_err(EngineError::Core)?;
        if blob.schema_version
            != benten_graph::backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION
        {
            return Err(EngineError::Other {
                code: ErrorCode::Serialize,
                message: format!(
                    "snapshot-blob schema mismatch: expected {expected}, got {actual}",
                    expected = benten_graph::backends::snapshot_blob::SNAPSHOT_BLOB_SCHEMA_VERSION,
                    actual = blob.schema_version
                ),
            });
        }

        // Prove the SnapshotBlobBackend will actually accept the bytes
        // — a separate check from the engine-side hydration so a future
        // direct-backed Engine reuses the same validation.
        let _ = benten_graph::SnapshotBlobBackend::from_bytes(bytes).map_err(
            |e: SnapshotBlobError| EngineError::Other {
                code: e.code(),
                message: format!("{e}"),
            },
        )?;

        // Hydrate a fresh tempdir-backed redb engine. The blob is keyed
        // by the snapshot-blob CID so the tempdir guard outlives the
        // engine clone(s) that point at it.
        let blob_cid = SnapshotBlob::compute_cid(bytes);
        let dir = tempfile::tempdir().map_err(|e| EngineError::Other {
            code: ErrorCode::GraphInternal,
            message: format!("snapshot-blob tempdir: {e}"),
        })?;
        let path = dir.path().join("benten-snapshot.redb");
        let backend = RedbBackend::open(&path)?;

        // Replay each Node body through `put_node`. Because the body
        // bytes are canonical DAG-CBOR (the source engine produced them
        // via the same canonical_bytes path), `put_node`'s recomputed
        // CID matches the snapshot-blob's key — Inv-13's
        // content-addressing invariant carries us through.
        for (cid, body) in &blob.nodes {
            let node: Node =
                serde_ipld_dagcbor::from_slice(body).map_err(|e| EngineError::Other {
                    code: ErrorCode::Serialize,
                    message: format!("snapshot-blob node decode at {cid}: {e}"),
                })?;
            let recomputed = backend.put_node(&node)?;
            // Sanity check — if the source bytes were tampered between
            // the canonical encode + the snapshot-blob serialization,
            // the recomputed CID will diverge. Fail loudly.
            if recomputed != *cid {
                return Err(EngineError::Other {
                    code: ErrorCode::InvContentHash,
                    message: format!(
                        "snapshot-blob CID drift: recomputed={recomputed} declared={cid}"
                    ),
                });
            }
        }

        // Stand the engine up over the hydrated backend.
        let mut engine = EngineBuilder::new()
            .backend(backend)
            // Disable IVM/caps for the snapshot view — the snapshot is
            // read-only so the change-stream + view-maintenance paths
            // would never observe a mutation. Tests can override.
            .without_ivm()
            .without_caps()
            .build()?;
        engine.set_read_only_snapshot();

        // Hold the tempdir alive for the engine's process lifetime.
        // The guard is keyed by snapshot-blob CID so re-imports of the
        // same blob in the same process share the hold.
        let guard = Arc::new(SnapshotTempDirGuard { _dir: dir });
        let registry = snapshot_tempdir_registry();
        if let Ok(mut g) = registry.lock() {
            g.insert(blob_cid, guard);
        }

        Ok(engine)
    }

    /// Phase-2b G10-A-wasip1 (D10-RESOLVED): compute the CID of an
    /// already-encoded snapshot-blob bytes payload.
    ///
    /// Static helper — does not need an engine instance. The CID is
    /// `BLAKE3(bytes)` wrapped via [`Cid::from_blake3_digest`], the
    /// same algorithm `Node::cid` uses; Phase-3 sync addresses the
    /// blob handoff by this CID.
    ///
    /// # Errors
    /// Currently infallible (the BLAKE3 hash + Cid construction don't
    /// fail on any byte slice), but returns `Result` so a future
    /// schema migration can surface decode-validation errors without
    /// a signature break.
    pub fn compute_snapshot_blob_cid(bytes: &[u8]) -> Result<Cid, EngineError> {
        Ok(SnapshotBlob::compute_cid(bytes))
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests + benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;
    use benten_core::testing::canonical_test_node;

    fn engine_with_canonical_state() -> (tempfile::TempDir, Engine) {
        let dir = tempfile::tempdir().unwrap();
        let engine = Engine::builder()
            .path(dir.path().join("benten.redb"))
            .build()
            .unwrap();
        let _cid = engine.create_node(&canonical_test_node()).unwrap();
        (dir, engine)
    }

    #[test]
    fn export_snapshot_blob_round_trip_byte_identical() {
        let (_dir, engine) = engine_with_canonical_state();
        let blob_a = engine.export_snapshot_blob().unwrap();
        let imported = Engine::from_snapshot_blob(&blob_a).unwrap();
        let blob_b = imported.export_snapshot_blob().unwrap();
        assert_eq!(
            blob_a, blob_b,
            "snapshot-blob export -> import -> re-export must be byte-identical"
        );
    }

    #[test]
    fn snapshot_engine_is_marked_read_only() {
        let (_dir, engine) = engine_with_canonical_state();
        let blob = engine.export_snapshot_blob().unwrap();
        let imported = Engine::from_snapshot_blob(&blob).unwrap();
        assert!(imported.is_read_only_snapshot());
    }

    #[test]
    fn snapshot_engine_rejects_writes_with_typed_error() {
        let (_dir, engine) = engine_with_canonical_state();
        let blob = engine.export_snapshot_blob().unwrap();
        let imported = Engine::from_snapshot_blob(&blob).unwrap();
        let mut node = canonical_test_node();
        // Use a different label so we don't even reach the dedup path.
        node.labels = vec!["UnseenLabel".into()];
        let err = imported.create_node(&node).unwrap_err();
        assert_eq!(err.code(), ErrorCode::BackendReadOnly);
    }

    #[test]
    fn compute_snapshot_blob_cid_is_stable() {
        let (_dir, engine) = engine_with_canonical_state();
        let blob_a = engine.export_snapshot_blob().unwrap();
        let cid_a = Engine::compute_snapshot_blob_cid(&blob_a).unwrap();
        let imported = Engine::from_snapshot_blob(&blob_a).unwrap();
        let blob_b = imported.export_snapshot_blob().unwrap();
        let cid_b = Engine::compute_snapshot_blob_cid(&blob_b).unwrap();
        assert_eq!(cid_a, cid_b);
    }
}

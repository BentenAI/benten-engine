//! Phase-2b G10-A-wasip1 snapshot-blob handoff API on [`Engine`]
//! (D10-RESOLVED) — Phase-3 G13-D wave-3 direct-wire (no tempdir).
//!
//! ## What this is
//!
//! Three engine-level methods that pair with the
//! [`benten_graph::SnapshotBlobBackend`] read-only backend:
//!
//! - [`Engine::export_snapshot_blob`] — walk the engine's storage and
//!   encode a canonical DAG-CBOR `SnapshotBlob` for handoff.
//! - [`Engine::from_snapshot_blob`] — decode a snapshot-blob and
//!   construct a read-only engine over an in-memory hydration backend;
//!   mutation methods surface `E_BACKEND_READ_ONLY`.
//! - [`Engine::compute_snapshot_blob_cid`] — static helper hashing an
//!   already-encoded blob.
//!
//! ## Why an in-memory hydration backend (G13-D wave-3 posture)
//!
//! Phase-2b shipped a tempdir-resident hydration shape: `from_snapshot_blob`
//! created a `tempfile::tempdir()` redb file and replayed the blob's nodes
//! into it. G13-D drops the tempdir — the hydration target is
//! [`RedbBackend::open_in_memory`] so the function never touches the
//! filesystem. The engine still consumes [`RedbBackend`] (the existing
//! resolved-alias `Engine = EngineGeneric<RedbBackend>` shape) so every
//! engine method (CRUD, dispatched-handler `call`, IVM views, change
//! subscribers) continues to work uniformly against the snapshot view —
//! the read-only contract fires at the user-facing engine surface
//! (`engine_crud.rs`, `primitive_host.rs::check_not_read_only_snapshot`)
//! exactly as it did pre-G13-D.
//!
//! ### What G13-D does NOT do (BELONGS-NAMED-NOW carry)
//!
//! The full direct-wire to `EngineGeneric<SnapshotBlobBackend>` —
//! making the engine consume the snapshot blob *as the backend* without
//! the in-memory redb hop — requires lifting every `impl Engine`
//! method (≈ 10 modules: `engine_crud.rs`, `engine_modules.rs`,
//! `engine_subscribe.rs`, `engine_views.rs`, `engine_caps.rs`,
//! `engine_sandbox.rs`, `engine_stream.rs`, `engine_wait.rs`,
//! `engine_diagnostics.rs`, `primitive_host.rs::PrimitiveHost`) into
//! `impl<B: GraphBackend> EngineGeneric<B>` form. That structural lift is
//! out of G13-D's scope-real-15 budget (~100-200 LOC) and is carried as
//! `docs/future/phase-3-backlog.md §1.2-followup` per HARD RULE
//! BELONGS-NAMED-NOW — the destination row receives the entry in the
//! same PR as this commit.
//!
//! G13-D delivers what fits in scope-real-15:
//!
//! 1. [`SnapshotBlobBackend`] is now a first-class
//!    [`benten_graph::GraphBackend`] (per the umbrella trait, with
//!    [`benten_graph::NodeStore`] + [`benten_graph::EdgeStore`] + the
//!    snapshot/transaction/subscriber/put-with-context surface). Tests
//!    pinned at `crates/benten-graph/tests/snapshot_blob_backend.rs`.
//! 2. `from_snapshot_blob` no longer creates a tempdir — the in-memory
//!    redb path replaces the filesystem hop. Pinned at
//!    `crates/benten-engine/tests/snapshot_no_tempdir.rs`.
//!
//! Native-target only — see `lib.rs` for the wasm32 cfg-gate rationale.

use std::collections::BTreeMap;

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

// G13-D wave-3: the Phase-2b `SnapshotTempDirGuard` + process-wide
// `SNAPSHOT_TEMP_DIRS` registry are retired. The hydration target is
// `RedbBackend::open_in_memory()` — there is no on-disk tempdir to keep
// alive, so the keep-alive registry has nothing to guard.

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

    /// G13-D wave-3 (D10-RESOLVED + direct-wire): construct a read-only
    /// engine view over a snapshot-blob handoff payload — without
    /// touching the filesystem.
    ///
    /// The bytes are decoded as a canonical DAG-CBOR [`SnapshotBlob`].
    /// The contents are hydrated into an in-memory [`RedbBackend`]
    /// (no tempdir, no on-disk path) and the engine's
    /// [`Engine::is_read_only_snapshot`] flag is set so user-facing
    /// mutations surface [`ErrorCode::BackendReadOnly`].
    ///
    /// ## G13-D delta vs Phase-2b
    ///
    /// Phase-2b spilled the hydration into a `tempfile::tempdir()`
    /// redb file kept alive via a process-wide guard registry. G13-D
    /// drops both: the hydration target is [`RedbBackend::open_in_memory`]
    /// and there is no filesystem touch to keep alive. Pinned at
    /// `crates/benten-engine/tests/snapshot_no_tempdir.rs::from_snapshot_blob_no_tempdir_in_path`.
    ///
    /// The full direct-wire (`EngineGeneric<SnapshotBlobBackend>` with
    /// no in-memory redb hop) requires lifting every `impl Engine`
    /// method into `impl<B: GraphBackend> EngineGeneric<B>` form.
    /// That structural lift is out of G13-D's scope-real-15 budget and
    /// is carried as a follow-up in
    /// `docs/future/phase-3-backlog.md §1.2-followup`.
    ///
    /// # Errors
    /// - [`EngineError::Other`] (`E_SERIALIZE`) on snapshot-blob decode
    ///   failure or schema-version mismatch.
    /// - [`EngineError::Graph`] on in-memory redb construction failure
    ///   (extremely unlikely — only happens on allocator failure
    ///   inside the redb cache).
    /// - [`EngineError::Other`] (`E_INV_CONTENT_HASH`) if a hydrated
    ///   Node's recomputed CID does not match the declared key, which
    ///   indicates tampering between the source-side canonical encode
    ///   and the destination-side decode.
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

        // Prove the [`SnapshotBlobBackend`] will actually accept the
        // bytes — a separate check from the engine-side hydration so
        // the (G13-D) direct-backed `SnapshotBlobBackend` is exercised
        // along the same input-validation path the in-memory
        // hydration consumes. Defends against schema drift between
        // the read-only [`SnapshotBlobBackend`] surface and the
        // engine-side decode.
        let _ = benten_graph::SnapshotBlobBackend::from_bytes(bytes).map_err(
            |e: SnapshotBlobError| EngineError::Other {
                code: e.code(),
                message: format!("{e}"),
            },
        )?;

        // G13-D: hydrate into an in-memory redb backend instead of a
        // tempdir-resident on-disk file. `RedbBackend::open_in_memory`
        // forces durability to `Async` (there is no disk to fsync to)
        // which is appropriate for a read-only snapshot view.
        let backend = RedbBackend::open_in_memory()?;

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

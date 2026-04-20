//! # benten-graph
//!
//! Storage layer for the Benten graph engine. Defines the [`KVBackend`] trait
//! and a [`RedbBackend`] implementation over [`redb`] v4.
//!
//! The trait boundary is deliberate: a future WASM target will fetch content-
//! addressed bytes from peers (via `iroh` or HTTP) with an in-memory cache,
//! and the evaluator should not notice the difference. Defining `KVBackend`
//! in Phase 1 preserves that option.
//!
//! ## Module layout
//!
//! - [`backend`] — the [`KVBackend`] trait, [`ScanResult`], [`BatchOp`],
//!   [`DurabilityMode`].
//! - [`store`] — [`NodeStore`] / [`EdgeStore`] traits plus the
//!   [`ChangeSubscriber`] trait and [`ChangeEvent`] schema. Each backend
//!   implements `NodeStore` / `EdgeStore` directly (no blanket impl — the
//!   index-maintenance contract is per-backend).
//! - [`redb_backend`] — the concrete [`RedbBackend`], its `KVBackend` /
//!   `NodeStore` / `EdgeStore` impls, and the index maintenance.
//! - this module — [`GraphError`] and the Phase-1 stubs (`Transaction`,
//!   `WriteContext`, `SnapshotHandle`) owned by G3 / G6.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "Phase 1 stubs cleared as G2-B/G3/G5/G6 land")]

pub use benten_core::ErrorCode;
use benten_core::{Cid, CoreError, Node};

pub mod backend;
pub(crate) mod indexes;
pub mod redb_backend;
pub mod store;
pub mod transaction;

pub use backend::{BatchOp, DurabilityMode, KVBackend, ScanIter, ScanResult};
pub use redb_backend::RedbBackend;
pub use store::{ChangeEvent, ChangeKind, ChangeSubscriber, EdgeStore, NodeStore};
pub use transaction::{PendingOp, Transaction};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the storage layer.
///
/// R1 triage `P1.graph.error-polymorphism` (G2-A) moved backend errors behind
/// an associated [`KVBackend::Error`] type. `GraphError` remains the concrete
/// error for `RedbBackend` and the type into which `CoreError` (serialization,
/// CID parsing) flows; other backends are free to pick their own.
///
/// r6-err-3 added `RedbSource(#[from] redb::Error)` so the six redb sub-type
/// `From` impls funnel the original error through `redb::Error` with
/// `std::error::Error::source()` preserved. The string-payload `Redb(String)`
/// variant is retained for test-fixture injection (see
/// `tests/failure_injection_rollback.rs`) and for internal
/// "missing transaction handle" bookkeeping.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    /// Propagated from `benten-core` (CID construction, canonical
    /// serialization, DAG-CBOR decode via `CoreError::Serialize`).
    #[error("core: {0}")]
    Core(#[from] CoreError),

    /// redb I/O or transactional failure with the original `redb::Error`
    /// preserved behind `#[source]` so `std::error::Error::source()` walks
    /// the chain. The six redb sub-error types (`DatabaseError`,
    /// `TransactionError`, `TableError`, `StorageError`, `CommitError`)
    /// each have a native `From<X> for redb::Error`; our `From` impls
    /// funnel through that so the origin kind is preserved.
    #[error("redb: {0}")]
    RedbSource(#[from] redb::Error),

    /// redb I/O or transactional failure, string-payload form. Retained
    /// for test-fixture injection (e.g.
    /// `GraphError::Redb("injected failure".into())`) and for internal
    /// "post-commit handle missing" bookkeeping inside the transaction
    /// primitive. Production conversion sites should use [`GraphError::RedbSource`]
    /// instead so the `std::error::Error::source` chain is preserved.
    #[error("redb: {0}")]
    Redb(String),

    /// DAG-CBOR decode of a stored Node failed. Indicates on-disk corruption
    /// or a format drift. The [`NodeStore`] / [`EdgeStore`] blanket impls
    /// route decode errors through [`CoreError::Serialize`] → `Core` instead;
    /// this variant is retained for any direct-decode call path (notably the
    /// retained inherent `RedbBackend::get_node` helper).
    #[error("decode: {0}")]
    Decode(String),

    /// `open_existing` was called on a path where no database file exists.
    ///
    /// The Display form shows only the basename (e.g. `benten.redb`) so the
    /// rendered message — which flows through napi into JS `Error.message`
    /// — does not leak the absolute filesystem path (r6-err-7: avoids
    /// leaking the caller's home-directory / username). The full `PathBuf`
    /// remains on the struct field for programmatic introspection and
    /// Debug rendering.
    #[error("backend not found: {}", redact_path_for_display(path))]
    BackendNotFound {
        /// Path supplied to the failed `open_existing` call.
        path: std::path::PathBuf,
    },

    /// A write was attempted on a system-zone label (label starting with
    /// `"system:"`) without the privileged flag set. Phase 1 SC1 stopgap.
    #[error("system-zone write not permitted from user path: {label}")]
    SystemZoneWrite {
        /// The `system:` label the user-zone path tried to write.
        label: String,
    },

    /// A nested transaction was rejected. Phase 1 G3 stub.
    #[error("nested transactions are not supported")]
    NestedTransactionNotSupported {},

    /// The transaction's closure returned `Err`, so the write batch was
    /// rolled back.
    #[error("transaction aborted: {reason}")]
    TxAborted {
        /// Human-readable reason the closure returned `Err`.
        reason: String,
    },
}

impl GraphError {
    /// Map a `GraphError` to its stable ERROR-CATALOG code.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            GraphError::Core(e) => e.code(),
            // `RedbSource` preserves the full `std::error::Error::source`
            // chain (r6-err-3); the catalog code is still `E_GRAPH_INTERNAL`
            // because the underlying redb error kind is opaque to
            // cross-language consumers. The string-payload `Redb` and
            // `Decode` variants carry the same catalog code for parity.
            GraphError::RedbSource(_) | GraphError::Redb(_) | GraphError::Decode(_) => {
                ErrorCode::GraphInternal
            }
            GraphError::BackendNotFound { .. } => ErrorCode::BackendNotFound,
            GraphError::SystemZoneWrite { .. } => ErrorCode::SystemZoneWrite,
            GraphError::NestedTransactionNotSupported {} => {
                ErrorCode::NestedTransactionNotSupported
            }
            GraphError::TxAborted { .. } => ErrorCode::TxAborted,
        }
    }
}

/// Render a `Path` for the Display of [`GraphError::BackendNotFound`] with
/// only its basename + a placeholder prefix so the rendered message does
/// not leak the absolute filesystem path through to user-facing error
/// strings. The full path is still available on the struct variant for
/// programmatic use and for `Debug` rendering.
fn redact_path_for_display(path: &std::path::Path) -> String {
    match path.file_name() {
        Some(name) => format!("<redacted>/{}", name.to_string_lossy()),
        None => "<redacted>".to_string(),
    }
}

// r6-err-3: preserve `std::error::Error::source()` on redb failures.
// Each redb sub-error type has a native `From<X> for redb::Error` in the
// redb crate, so we funnel through `redb::Error` and store it under
// `RedbSource` with `#[source]` preservation via `thiserror`'s `#[from]`.
impl From<redb::DatabaseError> for GraphError {
    fn from(e: redb::DatabaseError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::TransactionError> for GraphError {
    fn from(e: redb::TransactionError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::TableError> for GraphError {
    fn from(e: redb::TableError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::StorageError> for GraphError {
    fn from(e: redb::StorageError) -> Self {
        GraphError::RedbSource(e.into())
    }
}
impl From<redb::CommitError> for GraphError {
    fn from(e: redb::CommitError) -> Self {
        GraphError::RedbSource(e.into())
    }
}

// ---------------------------------------------------------------------------
// RedbBackend
// ---------------------------------------------------------------------------
//
// The concrete `RedbBackend` struct, its `KVBackend` impl, the three
// construction entry points (`open` / `open_existing` / `open_or_create`),
// and the label + property-value index plumbing all live in
// [`redb_backend`]. `pub use redb_backend::RedbBackend` re-exports it at
// crate root so existing call sites (and the integration tests) don't need
// to know about the module split.

// ---------------------------------------------------------------------------
// Phase 1 stubs — expanded in G3 / G6
// ---------------------------------------------------------------------------

/// A MVCC snapshot handle returned by [`RedbBackend::snapshot`]. Reads
/// through this handle observe the database state at the instant the
/// snapshot was opened; concurrent writes to the backend are invisible until
/// the handle is dropped.
///
/// G3-A lands a partial shape: [`SnapshotHandle::get_node`] is implemented
/// (thin wrapper over a `redb::ReadTransaction` held across the handle's
/// lifetime). [`SnapshotHandle::scan_label`] stays a G6 stub — it depends
/// on the label-index scan plumbing that G6 owns.
///
/// Implements `Drop` so explicit `drop(handle)` in tests is the idiomatic
/// way to release the snapshot's read-transaction lifetime.
pub struct SnapshotHandle {
    /// redb ReadTransaction captured at snapshot-open time. redb's read
    /// transactions are lightweight (no writer lock held) and observe the
    /// committed state at the instant `begin_read()` returned.
    pub(crate) read_txn: Option<redb::ReadTransaction>,
}

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        // Dropping the `ReadTransaction` releases the snapshot naturally.
        self.read_txn.take();
    }
}

impl SnapshotHandle {
    /// Retrieve a Node by CID from the snapshot view. Reads through the
    /// handle observe the point-in-time state captured when
    /// [`RedbBackend::snapshot`] was called; concurrent writes are
    /// invisible until the handle is dropped and a fresh snapshot is
    /// opened.
    ///
    /// # Errors
    /// - [`GraphError::Redb`] on any redb I/O failure.
    /// - [`GraphError::Decode`] if a stored Node fails to decode.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        use redb::ReadableTable;
        let Some(read_txn) = self.read_txn.as_ref() else {
            return Ok(None);
        };
        let table = read_txn.open_table(redb_backend::NODES_TABLE)?;
        let key = store::node_key(cid);
        let Some(v) = table.get(key.as_slice())? else {
            return Ok(None);
        };
        let node: Node = serde_ipld_dagcbor::from_slice(&v.value())
            .map_err(|e| GraphError::Decode(format!("snapshot get_node decode: {e}")))?;
        Ok(Some(node))
    }

    /// Scan all nodes with a given label from the snapshot view.
    ///
    /// Uses the label-index multimap table opened against the snapshot's
    /// read-transaction so results reflect the point-in-time state captured
    /// when [`RedbBackend::snapshot`] was called; concurrent writes are
    /// invisible to this scan.
    ///
    /// # Errors
    ///
    /// - [`GraphError::Redb`] on any redb I/O failure.
    /// - [`GraphError::Core`] if an index entry fails to decode.
    pub fn scan_label(&self, label: &str) -> Result<Vec<Cid>, GraphError> {
        if label.is_empty() {
            return Ok(Vec::new());
        }
        let Some(read_txn) = self.read_txn.as_ref() else {
            return Ok(Vec::new());
        };
        let table = read_txn.open_multimap_table(crate::indexes::LABEL_INDEX_TABLE)?;
        let values = table.get(label.as_bytes())?;
        let mut out = Vec::new();
        for v in values {
            let v = v?;
            let cid = crate::indexes::cid_from_index_bytes(v.value())?;
            out.push(cid);
        }
        Ok(out)
    }
}

// ChangeReceiver intentionally does NOT live in benten-graph.
//
// Per the implementation plan (R1 architect addendum, line ~605), the
// channel concretion — tokio-broadcast on native, synchronous
// `Vec<Box<dyn ChangeSubscriber>>` fan-out on WASM — lives in
// `benten-engine::change`. The graph crate exposes only the
// [`ChangeSubscriber`] callback trait ([`store::ChangeSubscriber`]) so it
// carries no async-runtime dependency. Backends register subscribers via
// `RedbBackend::register_subscriber(Arc<dyn ChangeSubscriber>)`; the
// transaction primitive (G3) fans change events out to registered
// subscribers synchronously after a successful commit.

/// Metadata passed to the capability pre-write hook.
///
/// `is_privileged = true` marks an engine-API-only path (grant_capability,
/// create_view, revoke_capability), bypassing the system-zone label ban.
///
/// **Phase 1 G3-A / SC1 stub.**
#[derive(Debug, Clone)]
pub struct WriteContext {
    /// The Node's primary label — used for the system-zone prefix check.
    pub label: String,
    /// Marks an engine-API-only path. User code cannot reach this without
    /// going through one of the engine's privileged methods.
    pub is_privileged: bool,
}

impl Default for WriteContext {
    fn default() -> Self {
        Self {
            label: String::new(),
            is_privileged: false,
        }
    }
}

impl WriteContext {
    /// Construct a non-privileged write context for a given label. This is
    /// the constructor user-authored code paths use.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            is_privileged: false,
        }
    }

    /// Construct a WriteContext flagged as privileged (engine-API-only
    /// path). This is the only constructor that bypasses the SC1
    /// system-zone ban. User code cannot call this without going through
    /// `Engine::grant_capability`, `Engine::create_view`, or
    /// `Engine::revoke_capability`.
    #[must_use]
    pub fn privileged_for_engine_api() -> Self {
        Self {
            label: String::new(),
            is_privileged: true,
        }
    }

    /// Called by the transaction primitive to enforce the SC1 stopgap.
    /// Rejects writes to any label starting with `"system:"` unless
    /// `is_privileged == true`. Returns the `label` string in the error so
    /// diagnostics can point at the exact reserved label the write
    /// attempted.
    ///
    /// # Errors
    /// [`GraphError::SystemZoneWrite`] on an unprivileged system-zone
    /// label.
    pub fn enforce_system_zone(&self) -> Result<(), GraphError> {
        if !self.is_privileged && self.label.starts_with("system:") {
            return Err(GraphError::SystemZoneWrite {
                label: self.label.clone(),
            });
        }
        Ok(())
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

    fn temp_backend() -> (RedbBackend, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("benten.redb");
        let backend = RedbBackend::open(&path).unwrap();
        (backend, dir)
    }

    #[test]
    fn put_then_get_roundtrip() {
        let (backend, _dir) = temp_backend();
        let node = canonical_test_node();
        let cid = backend.put_node(&node).unwrap();

        let fetched = backend.get_node(&cid).unwrap().expect("node must exist");
        assert_eq!(fetched, node);

        // Re-hashing the fetched node reproduces the CID — proves end-to-end
        // content-addressing through the storage layer.
        assert_eq!(fetched.cid().unwrap(), cid);
    }

    #[test]
    fn get_missing_returns_none() {
        let (backend, _dir) = temp_backend();
        let cid = canonical_test_node().cid().unwrap();
        assert!(backend.get_node(&cid).unwrap().is_none());
    }

    #[test]
    fn delete_is_idempotent() {
        let (backend, _dir) = temp_backend();
        let node = canonical_test_node();
        let cid = backend.put_node(&node).unwrap();
        // Delete via the Node-level API (uses the `n:` key schema).
        backend.delete_node(&cid).unwrap();
        backend.delete_node(&cid).unwrap(); // second delete must not panic
        assert!(backend.get_node(&cid).unwrap().is_none());
    }

    #[test]
    fn batch_put_is_atomic() {
        let (backend, _dir) = temp_backend();
        let pairs = vec![
            (b"k1".to_vec(), b"v1".to_vec()),
            (b"k2".to_vec(), b"v2".to_vec()),
        ];
        backend.put_batch(&pairs).unwrap();
        assert_eq!(backend.get(b"k1").unwrap().as_deref(), Some(b"v1".as_ref()));
        assert_eq!(backend.get(b"k2").unwrap().as_deref(), Some(b"v2".as_ref()));
    }

    #[test]
    fn scan_empty_prefix_returns_everything() {
        let (backend, _dir) = temp_backend();
        let pairs = vec![
            (b"alpha".to_vec(), b"1".to_vec()),
            (b"beta".to_vec(), b"2".to_vec()),
            (b"gamma".to_vec(), b"3".to_vec()),
        ];
        backend.put_batch(&pairs).unwrap();

        let hits = backend.scan(&[]).unwrap();
        assert_eq!(hits.len(), 3, "empty prefix must match every key");

        // Confirm redb returns results in sorted key order so callers can
        // rely on it for deterministic downstream processing (content
        // listings, IVM bootstrap).
        let mut keys: Vec<&[u8]> = hits.iter().map(|(k, _)| k.as_slice()).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted);
        keys.sort();
        assert_eq!(
            keys,
            [b"alpha".as_ref(), b"beta".as_ref(), b"gamma".as_ref()]
        );
    }

    #[test]
    fn scan_zero_hit_prefix_returns_empty() {
        let (backend, _dir) = temp_backend();
        backend
            .put_batch(&[(b"alpha".to_vec(), b"1".to_vec())])
            .unwrap();

        // A prefix that sorts after every stored key (and cannot be a
        // prefix of any stored key) must return an empty result, not error.
        let hits = backend.scan(b"zzz").unwrap();
        assert!(hits.is_empty());

        // A prefix on an empty store must also return empty.
        let (empty_backend, _empty_dir) = temp_backend();
        let hits = empty_backend.scan(b"anything").unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn scan_prefix_bounds_the_range() {
        // Regression test for the earlier O(n) implementation that iterated
        // the full table regardless of prefix.
        let (backend, _dir) = temp_backend();
        let pairs = vec![
            (b"post:1".to_vec(), b"p1".to_vec()),
            (b"post:2".to_vec(), b"p2".to_vec()),
            (b"user:1".to_vec(), b"u1".to_vec()),
            (b"user:2".to_vec(), b"u2".to_vec()),
            (b"zzz".to_vec(), b"z".to_vec()),
        ];
        backend.put_batch(&pairs).unwrap();

        let posts = backend.scan(b"post:").unwrap();
        assert_eq!(posts.len(), 2);
        assert!(posts.iter().all(|(k, _)| k.starts_with(b"post:")));

        let users = backend.scan(b"user:").unwrap();
        assert_eq!(users.len(), 2);
        assert!(users.iter().all(|(k, _)| k.starts_with(b"user:")));
    }

    #[test]
    fn scan_all_0xff_prefix_is_open_ended() {
        let (backend, _dir) = temp_backend();
        backend.put(&[0xff, 0xff, 0xff], b"sentinel").unwrap();
        backend.put(b"unrelated", b"nope").unwrap();

        let hits = backend.scan(&[0xff, 0xff]).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits.as_slice()[0].0, vec![0xff, 0xff, 0xff]);
    }

    #[test]
    fn batch_put_empty_slice_is_a_noop() {
        let (backend, _dir) = temp_backend();
        backend.put_batch(&[]).unwrap();
        assert!(backend.scan(&[]).unwrap().is_empty());
    }

    // `next_prefix_increments_and_trims` — moved to `redb_backend.rs` in G2-B
    // alongside the helper it exercises.
}

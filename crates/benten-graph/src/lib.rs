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
//! - [`store`] — [`NodeStore`] / [`EdgeStore`] blanket impls over any
//!   `KVBackend`, plus the [`ChangeSubscriber`] trait and [`ChangeEvent`]
//!   schema.
//! - this module — the concrete [`RedbBackend`], [`GraphError`], and the
//!   Phase-1 stubs (`Transaction`, `WriteContext`, `SnapshotHandle`,
//!   `ChangeReceiver`) owned by G2-B / G3 / G5 / G6.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "Phase 1 stubs cleared as G2-B/G3/G5/G6 land")]

use std::path::Path;

pub use benten_core::ErrorCode;
use benten_core::{Cid, CoreError, Edge, Node};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

pub mod backend;
pub mod store;

pub use backend::{BatchOp, DurabilityMode, KVBackend, ScanResult};
pub use store::{ChangeEvent, ChangeKind, ChangeSubscriber, EdgeStore, NodeStore};

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
/// Phase 1 follow-up tracked in G2-B: the `Redb(String)` variant still
/// stringifies `redb::*` errors and loses `std::error::Error::source`.
/// G2-B refines this to preserve the chain via `#[from] + #[source]`.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    /// Propagated from `benten-core` (CID construction, canonical
    /// serialization, DAG-CBOR decode via `CoreError::Serialize`).
    #[error("core: {0}")]
    Core(#[from] CoreError),

    /// redb I/O or transactional failure. G2-B will preserve the source
    /// chain; spike-era String coercion documented above.
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
    /// Phase 1 G2-B stub.
    #[error("backend not found at path")]
    BackendNotFound {
        /// Path supplied to the failed `open_existing` call (Phase 1 stub
        /// — the PathBuf plumbing lands in G2-B; today this is a
        /// placeholder).
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
            GraphError::Redb(_) | GraphError::Decode(_) => {
                ErrorCode::Unknown(String::from("graph_internal"))
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

impl From<redb::Error> for GraphError {
    fn from(e: redb::Error) -> Self {
        GraphError::Redb(e.to_string())
    }
}
impl From<redb::DatabaseError> for GraphError {
    fn from(e: redb::DatabaseError) -> Self {
        GraphError::Redb(e.to_string())
    }
}
impl From<redb::TransactionError> for GraphError {
    fn from(e: redb::TransactionError) -> Self {
        GraphError::Redb(e.to_string())
    }
}
impl From<redb::TableError> for GraphError {
    fn from(e: redb::TableError) -> Self {
        GraphError::Redb(e.to_string())
    }
}
impl From<redb::StorageError> for GraphError {
    fn from(e: redb::StorageError) -> Self {
        GraphError::Redb(e.to_string())
    }
}
impl From<redb::CommitError> for GraphError {
    fn from(e: redb::CommitError) -> Self {
        GraphError::Redb(e.to_string())
    }
}

// ---------------------------------------------------------------------------
// redb backend
// ---------------------------------------------------------------------------

/// Single table that stores every opaque (key, value) pair. The graph-level
/// [`NodeStore`] and [`EdgeStore`] blanket impls layer a key-schema (`n:`,
/// `e:`, `es:`, `et:`) on top of this table.
const NODES_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("benten_nodes");

/// Lexicographic successor of `prefix` — the smallest byte string strictly
/// greater than every string that begins with `prefix`. Used to turn a
/// prefix scan into a bounded range scan.
///
/// Returns `None` when `prefix` is all-`0xff` (no successor exists in the
/// byte-string ordering), signalling that the caller should do an
/// unbounded `prefix..` scan instead.
fn next_prefix(prefix: &[u8]) -> Option<Vec<u8>> {
    let mut out = prefix.to_vec();
    while let Some(last) = out.last_mut() {
        if *last < 0xff {
            *last += 1;
            return Some(out);
        }
        out.pop();
    }
    None
}

/// A [`KVBackend`] implementation backed by a local redb v4 database file.
///
/// redb provides serializable isolation (single writer, multiple readers) and
/// durable commits via a two-phase commit with checksummed pages. We rely on
/// those guarantees rather than rolling our own WAL.
///
/// # Concurrency
///
/// `RedbBackend` is not `Clone`. To share a single backend across threads,
/// wrap it in an `Arc`: `let backend = Arc::new(RedbBackend::open(path)?)`.
/// redb's own API is `&self`, so multiple readers and a single writer can
/// proceed concurrently through the shared `Arc`.
///
/// # Path handling
///
/// `RedbBackend::open` does not canonicalize or validate the database path.
/// Callers that receive paths from an untrusted source (capability-delegated
/// subgraphs, multi-tenant configurations) are responsible for path
/// sanitization before invoking `open`. A future `benten-engine` wrapper will
/// constrain paths to a configured data directory.
pub struct RedbBackend {
    db: Database,
}

impl core::fmt::Debug for RedbBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RedbBackend").finish_non_exhaustive()
    }
}

impl RedbBackend {
    /// Open or create a redb database at `path`. The parent directory must
    /// already exist (redb does not `mkdir -p`).
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] if redb cannot open or create the file,
    /// or if the initial table creation transaction fails.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        let db = Database::create(path.as_ref())?;
        // Make sure the nodes table exists so subsequent reads don't fail on
        // a cold database. Creating an existing table is a no-op.
        let write_txn = db.begin_write()?;
        {
            let _ = write_txn.open_table(NODES_TABLE)?;
        }
        write_txn.commit()?;
        Ok(Self { db })
    }

    /// Inherent `put_node` — kept so existing call sites that don't `use
    /// NodeStore` still compile. Delegates to the [`NodeStore`] blanket
    /// impl so the key schema (`n:CID`) is identical.
    ///
    /// # Errors
    /// Propagates the [`NodeStore`] blanket-impl error shape
    /// ([`GraphError`]).
    pub fn put_node(&self, node: &Node) -> Result<Cid, GraphError> {
        <Self as NodeStore>::put_node(self, node)
    }

    /// Inherent `get_node` — delegates to [`NodeStore`] blanket impl. See
    /// [`Self::put_node`] for the rationale.
    ///
    /// # Errors
    /// Propagates the [`NodeStore`] blanket-impl error shape.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        <Self as NodeStore>::get_node(self, cid)
    }

    /// Inherent `delete_node` — delegates to [`NodeStore`] blanket impl.
    ///
    /// # Errors
    /// Propagates the [`NodeStore`] blanket-impl error shape.
    pub fn delete_node(&self, cid: &Cid) -> Result<(), GraphError> {
        <Self as NodeStore>::delete_node(self, cid)
    }

    /// Inherent `put_edge` — delegates to [`EdgeStore`] blanket impl.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] blanket-impl error shape.
    pub fn put_edge(&self, edge: &Edge) -> Result<Cid, GraphError> {
        <Self as EdgeStore>::put_edge(self, edge)
    }

    /// Inherent `get_edge` — delegates to [`EdgeStore`] blanket impl.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] blanket-impl error shape.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, GraphError> {
        <Self as EdgeStore>::get_edge(self, cid)
    }

    /// Inherent `edges_from` — delegates to [`EdgeStore`] blanket impl.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] blanket-impl error shape.
    pub fn edges_from(&self, cid: &Cid) -> Result<Vec<Edge>, GraphError> {
        <Self as EdgeStore>::edges_from(self, cid)
    }

    /// Inherent `edges_to` — delegates to [`EdgeStore`] blanket impl.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] blanket-impl error shape.
    pub fn edges_to(&self, cid: &Cid) -> Result<Vec<Edge>, GraphError> {
        <Self as EdgeStore>::edges_to(self, cid)
    }

    /// Store a Node under a caller-supplied [`WriteContext`]. The context
    /// controls system-zone prefix enforcement (SC1 stopgap for Invariant 11).
    ///
    /// **Phase 1 SC1 stub.** R5 G3/G7 wires the actual enforcement; the
    /// surface is here so tests can pin the contract before the
    /// implementation lands.
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn put_node_with_context(
        &self,
        _node: &Node,
        _ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        todo!("RedbBackend::put_node_with_context — SC1 (Phase 1)")
    }
}

impl KVBackend for RedbBackend {
    type Error = GraphError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(NODES_TABLE)?;
        Ok(table.get(key)?.map(|v| v.value().to_vec()))
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), GraphError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            table.remove(key)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(NODES_TABLE)?;
        let mut out: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();

        // For a non-empty prefix we use redb's ordered range to bound the scan
        // to keys in [prefix, next_prefix). This turns an O(n) full-table walk
        // into O(matches + log n). `next_prefix` is the lexicographic successor
        // of `prefix` obtained by incrementing the last non-0xff byte and
        // dropping the tail; if `prefix` is all 0xff (no successor exists),
        // the upper bound is open-ended and the scan continues to the end.
        if prefix.is_empty() {
            for item in table.iter()? {
                let (k, v) = item?;
                out.push((k.value().to_vec(), v.value().to_vec()));
            }
        } else {
            let next = next_prefix(prefix);
            let iter = match next.as_deref() {
                Some(upper) => table.range::<&[u8]>(prefix..upper)?,
                None => table.range::<&[u8]>(prefix..)?,
            };
            for item in iter {
                let (k, v) = item?;
                out.push((k.value().to_vec(), v.value().to_vec()));
            }
        }

        Ok(ScanResult::from_vec(out))
    }

    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), GraphError> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            for (k, v) in pairs {
                table.insert(k.as_slice(), v.as_slice())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Phase 1 stubs — expanded in G2-B / G3 / G5 / G6
// ---------------------------------------------------------------------------

impl RedbBackend {
    /// Open a redb database that must already exist at `path`. Distinct from
    /// [`RedbBackend::open`] which creates on miss.
    ///
    /// **Phase 1 G2-B stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn open_existing(_path: impl AsRef<Path>) -> Result<Self, GraphError> {
        todo!("RedbBackend::open_existing — G2-B (Phase 1)")
    }

    /// Open-or-create semantics (equivalent to [`RedbBackend::open`] today,
    /// kept as an explicit name for the G2-B split).
    ///
    /// # Errors
    /// Propagates [`RedbBackend::open`] errors.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        RedbBackend::open(path)
    }

    /// Transaction primitive — a closure over a write transaction handle.
    /// Atomic: all writes inside the closure commit together, or none do.
    ///
    /// **Phase 1 G3-A stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn transaction<F, R>(&self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        todo!("RedbBackend::transaction — G3-A (Phase 1)")
    }

    /// Transaction variant used by the commit-denial edge-case test. The
    /// closure runs to completion; a deny-on-commit hook fires before the
    /// redb commit actually persists. **Phase 1 G3-A stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn transaction_with_deny_on_commit<F, R>(&self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        todo!("RedbBackend::transaction_with_deny_on_commit — G3-A (Phase 1)")
    }

    /// Subscribe to the post-commit change stream.
    ///
    /// **Phase 1 G3-A stub.**
    pub fn subscribe(&self) -> ChangeReceiver {
        todo!("RedbBackend::subscribe — G3-A (Phase 1)")
    }

    /// Open a MVCC snapshot handle. The returned handle observes the
    /// database state at open-time; writes committed to the backend after
    /// the snapshot opens are NOT visible through it until it is dropped
    /// and a fresh snapshot is acquired.
    ///
    /// **Phase 1 G6 stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn snapshot(&self) -> Result<SnapshotHandle, GraphError> {
        todo!("RedbBackend::snapshot — G6 (Phase 1)")
    }

    /// Label-index lookup. **G5 stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn get_by_label(&self, _label: &str) -> Result<Vec<Cid>, GraphError> {
        todo!("RedbBackend::get_by_label — G5 (Phase 1)")
    }

    /// Property-value index lookup. **G5 stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn get_by_property(
        &self,
        _label: &str,
        _prop: &str,
        _value: &benten_core::Value,
    ) -> Result<Vec<Cid>, GraphError> {
        todo!("RedbBackend::get_by_property — G5 (Phase 1)")
    }
}

/// A write transaction handle, passed into the `transaction` closure. All
/// writes are atomic at commit.
///
/// **Phase 1 G3-A stub.**
pub struct Transaction<'a> {
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl Transaction<'_> {
    /// Put a Node inside the transaction.
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn put_node(&mut self, _node: &Node) -> Result<Cid, GraphError> {
        todo!("Transaction::put_node — G3-A (Phase 1)")
    }

    /// Put an Edge inside the transaction.
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn put_edge(&mut self, _edge: &Edge) -> Result<Cid, GraphError> {
        todo!("Transaction::put_edge — G3-A (Phase 1)")
    }

    /// Open a nested transaction. Phase 1 always rejects with
    /// [`GraphError::NestedTransactionNotSupported`].
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn transaction<F, R>(&mut self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        todo!("Transaction::transaction (nested) — G3-A (Phase 1)")
    }
}

/// A MVCC snapshot handle returned by [`RedbBackend::snapshot`]. Reads
/// through this handle observe the database state at the instant the
/// snapshot was opened; concurrent writes to the backend are invisible until
/// the handle is dropped. **Phase 1 G6 stub.**
///
/// Implements `Drop` so explicit `drop(handle)` in tests is the idiomatic
/// way to release the snapshot's read-transaction lifetime.
pub struct SnapshotHandle;

impl Drop for SnapshotHandle {
    fn drop(&mut self) {
        // G6 wires the underlying redb ReadTransaction release here.
    }
}

impl SnapshotHandle {
    /// Retrieve a Node by CID from the snapshot view. **Phase 1 G6 stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn get_node(&self, _cid: &Cid) -> Result<Option<Node>, GraphError> {
        todo!("SnapshotHandle::get_node — G6 (Phase 1)")
    }

    /// Scan all nodes with a given label from the snapshot view.
    /// **Phase 1 G6 stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn scan_label(&self, _label: &str) -> Result<Vec<Cid>, GraphError> {
        todo!("SnapshotHandle::scan_label — G6 (Phase 1)")
    }
}

/// Return handle from [`RedbBackend::subscribe`]. **Phase 1 G3-A stub.**
pub struct ChangeReceiver;

impl ChangeReceiver {
    /// Receive the next change event (blocking).
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn recv(&self) -> Result<ChangeEvent, GraphError> {
        todo!("ChangeReceiver::recv — G3-A (Phase 1)")
    }
}

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
    /// `is_privileged == true`.
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn enforce_system_zone(&self) -> Result<(), GraphError> {
        todo!("WriteContext::enforce_system_zone — SC1 (Phase 1)")
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
        assert_eq!(hits[0].0, vec![0xff, 0xff, 0xff]);
    }

    #[test]
    fn batch_put_empty_slice_is_a_noop() {
        let (backend, _dir) = temp_backend();
        backend.put_batch(&[]).unwrap();
        assert!(backend.scan(&[]).unwrap().is_empty());
    }

    #[test]
    fn next_prefix_increments_and_trims() {
        assert_eq!(next_prefix(b"a"), Some(b"b".to_vec()));
        assert_eq!(next_prefix(b"az"), Some(b"a{".to_vec())); // b'z' + 1 = b'{'
        assert_eq!(next_prefix(&[0xff]), None, "all-0xff has no successor");
        assert_eq!(
            next_prefix(&[0x01, 0xff, 0xff]),
            Some(vec![0x02]),
            "trailing 0xff bytes are dropped and the last non-0xff increments"
        );
        assert_eq!(next_prefix(&[]), None, "empty prefix has no successor");
    }
}

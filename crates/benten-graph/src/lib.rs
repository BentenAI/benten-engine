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
//! This spike implements the minimum surface: `get`, `put`, `delete`, a
//! prefix `scan`, and an atomic `put_batch`. A real engine will grow the
//! trait (range scans, transaction handles, change notification streams) as
//! Phase 1 proper proceeds.

#![forbid(unsafe_code)]
#![allow(clippy::todo, reason = "R3 red-phase stubs; R5 removes todos")]

use std::path::Path;

pub use benten_core::ErrorCode;
use benten_core::{Cid, CoreError, Edge, Node};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the storage layer.
///
/// Phase 1 follow-up — `GraphError` currently names `Redb` directly and erases
/// the underlying `redb::Error` chain via `to_string()`. Both decisions are
/// spike-stage compromises flagged by the `code-reviewer` critic:
///
/// - The `Redb` variant forces any non-redb `KVBackend` impl (in-memory mock,
///   iroh-fetch, WASM peer-fetch) to stringify its errors into a variant that
///   lies about where they came from. Phase 1 proper will rename to
///   `Backend(String)` or, better, make `KVBackend` carry an associated
///   `type Error` so each backend picks its own error type.
/// - Stringifying `redb::*` errors loses `std::error::Error::source`, so
///   callers cannot distinguish I/O failure from corruption from capacity.
///   Phase 1 proper will preserve the chain via `#[from]` + `#[source]` or a
///   boxed source field.
///
/// Tracked in SPIKE-phase-1-stack-RESULTS.md Next Actions; see
/// `.addl/spike/code-reviewer-benten-graph.json` findings for the full
/// rationale.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    /// Propagated from `benten-core` (CID construction, canonical serialization).
    #[error("core: {0}")]
    Core(#[from] CoreError),

    /// redb I/O or transactional failure. Wrapped into a string because
    /// `redb`'s error enum is not `Clone` and its variants are internal
    /// details the caller does not need to switch on in the spike. See the
    /// type-level doc above for the Phase 1 refactor plan.
    #[error("redb: {0}")]
    Redb(String),

    /// DAG-CBOR decode of a stored Node failed. Indicates on-disk corruption
    /// or a format drift.
    #[error("decode: {0}")]
    Decode(String),

    /// `open_existing` was called on a path where no database file exists.
    /// Phase 1 G2-B stub.
    #[error("backend not found at path")]
    BackendNotFound {
        /// Path supplied to the failed `open_existing` call (Phase 1 stub
        /// — the PathBuf plumbing lands in R5; today this is a placeholder).
        path: std::path::PathBuf,
    },

    /// A write was attempted on a system-zone label (label starting with
    /// `"system:"`) without the privileged flag set. Phase 1 SC1 stopgap.
    #[error("system-zone write not permitted from user path: {label}")]
    SystemZoneWrite { label: String },

    /// A nested transaction was rejected. Phase 1 G3 stub.
    #[error("nested transactions are not supported")]
    NestedTransactionNotSupported {},

    /// The transaction's closure returned `Err`, so the write batch was rolled back.
    #[error("transaction aborted: {reason}")]
    TxAborted { reason: String },
}

impl GraphError {
    /// Map a `GraphError` to its stable ERROR-CATALOG code. **Phase 1 stub —
    /// R5 refines the mapping for `Core` / `Decode` / `Redb`.**
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
// KVBackend trait
// ---------------------------------------------------------------------------

/// Alias for the vector of `(key, value)` pairs returned by a prefix scan.
///
/// Phase 1 follow-up — returning the full result as a `Vec` is a spike-stage
/// footgun: it forecloses pagination, streaming, and early termination, and
/// forces any future network-fetch backend to download the full result before
/// yielding anything. Phase 1 proper will reshape `scan` to return an iterator
/// (likely `Box<dyn Iterator<Item = Result<(Vec<u8>, Vec<u8>), GraphError>>>`
/// or a custom `Scan` type). Tracked in SPIKE-phase-1-stack-RESULTS.md Next
/// Actions and `.addl/spike/code-reviewer-benten-graph.json`.
pub type ScanResult = Vec<(Vec<u8>, Vec<u8>)>;

/// Minimal key/value backend trait for the Benten graph.
///
/// Values are opaque byte blobs from the trait's perspective; the graph layer
/// above is responsible for (de)serializing Nodes. Keys are also opaque bytes
/// so the graph layer can choose its own key schema (the spike uses CIDs
/// directly as keys).
///
/// `put_batch` must be atomic: either all pairs are committed or none are.
/// This is the primitive the transaction primitive (`begin`/`commit`/`rollback`)
/// will be built on in `benten-eval`.
///
/// # Spike-stage shape
///
/// This trait is deliberately minimal to prove out the abstraction. Two
/// Phase 1 follow-ups already documented above:
///
/// - Error typing (see [`GraphError`] doc): likely move to an associated
///   `type Error` so backends don't lie through a redb-named variant.
/// - Scan shape (see [`ScanResult`] doc): return an iterator, not a `Vec`.
pub trait KVBackend {
    /// Backend-specific error type. `RedbBackend` sets this to [`GraphError`];
    /// alternative backends (in-memory mock, WASM peer-fetch, iroh-fetch) each
    /// choose their own so they aren't forced to lie through a redb-named
    /// variant. See the `backend_error_polymorphism` edge-case test for the
    /// property this enables.
    type Error;

    /// Fetch the value stored under `key`. Returns `Ok(None)` on a clean miss.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error>;

    /// Insert or overwrite the value at `key`.
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), Self::Error>;

    /// Delete the value at `key`. Idempotent: returns `Ok(())` even if the key
    /// was absent.
    fn delete(&self, key: &[u8]) -> Result<(), Self::Error>;

    /// Return every (key, value) pair whose key starts with `prefix`.
    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, Self::Error>;

    /// Commit multiple puts atomically. Either every pair lands or none do.
    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), Self::Error>;
}

// ---------------------------------------------------------------------------
// redb backend
// ---------------------------------------------------------------------------

/// Single table that stores every Node keyed by its CID bytes.
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

    // ------------------------------------------------------------------
    // Node-level convenience API (built on top of the KV trait). This is
    // what `benten-engine` calls through.
    // ------------------------------------------------------------------

    /// Store a Node under its CID. Returns the CID for caller convenience.
    ///
    /// # Errors
    /// Returns [`GraphError::Core`] if the Node cannot be DAG-CBOR encoded,
    /// or [`GraphError::Redb`] on storage errors.
    pub fn put_node(&self, node: &Node) -> Result<Cid, GraphError> {
        let cid = node.cid()?;
        let bytes = node.canonical_bytes()?;
        self.put(cid.as_bytes(), &bytes)?;
        Ok(cid)
    }

    /// Store a Node under a caller-supplied `WriteContext`. The context
    /// controls system-zone prefix enforcement (SC1 stopgap for Invariant 11).
    ///
    /// **Phase 1 SC1 stub.** R5 wires the actual enforcement; the surface is
    /// here so tests can pin the contract before the implementation lands.
    pub fn put_node_with_context(
        &self,
        _node: &Node,
        _ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        todo!("RedbBackend::put_node_with_context — SC1 (Phase 1)")
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] on storage errors or
    /// [`GraphError::Decode`] if the stored bytes cannot be parsed as a Node.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        let Some(bytes) = self.get(cid.as_bytes())? else {
            return Ok(None);
        };
        let node: Node = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(|e| GraphError::Decode(e.to_string()))?;
        Ok(Some(node))
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
        let mut out = Vec::new();

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

        Ok(out)
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
// Phase 1 stubs — expanded in G2/G3/G5
// ---------------------------------------------------------------------------

impl RedbBackend {
    /// Open a redb database that must already exist at `path`. Distinct from
    /// [`RedbBackend::open`] which creates on miss.
    ///
    /// **Phase 1 G2-B stub.**
    pub fn open_existing(_path: impl AsRef<Path>) -> Result<Self, GraphError> {
        todo!("RedbBackend::open_existing — G2-B (Phase 1)")
    }

    /// Open-or-create semantics (equivalent to [`RedbBackend::open`] today,
    /// kept as an explicit name for the G2-B split).
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        RedbBackend::open(path)
    }

    /// Transaction primitive — a closure over a write transaction handle.
    /// Atomic: all writes inside the closure commit together, or none do.
    ///
    /// **Phase 1 G3-A stub.**
    pub fn transaction<F, R>(&self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        todo!("RedbBackend::transaction — G3-A (Phase 1)")
    }

    /// Transaction variant used by the commit-denial edge-case test. The
    /// closure runs to completion; a deny-on-commit hook fires before the
    /// redb commit actually persists. **Phase 1 G3-A stub.**
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

    /// Enqueue a Node put into a label-index update. **G5 stub.**
    pub fn get_by_label(&self, _label: &str) -> Result<Vec<Cid>, GraphError> {
        todo!("RedbBackend::get_by_label — G5 (Phase 1)")
    }

    /// Property-value index lookup. **G5 stub.**
    pub fn get_by_property(
        &self,
        _label: &str,
        _prop: &str,
        _value: &benten_core::Value,
    ) -> Result<Vec<Cid>, GraphError> {
        todo!("RedbBackend::get_by_property — G5 (Phase 1)")
    }

    /// Put an Edge (separate from put_node). **G4 stub.**
    pub fn put_edge(&self, _edge: &Edge) -> Result<Cid, GraphError> {
        todo!("RedbBackend::put_edge — G4 (Phase 1)")
    }

    /// Get an Edge by CID. **G4 stub.**
    pub fn get_edge(&self, _cid: &Cid) -> Result<Option<Edge>, GraphError> {
        todo!("RedbBackend::get_edge — G4 (Phase 1)")
    }

    /// All edges whose `source == cid`. **G4 stub.**
    pub fn edges_from(&self, _cid: &Cid) -> Result<Vec<Edge>, GraphError> {
        todo!("RedbBackend::edges_from — G4 (Phase 1)")
    }

    /// All edges whose `target == cid`. **G4 stub.**
    pub fn edges_to(&self, _cid: &Cid) -> Result<Vec<Edge>, GraphError> {
        todo!("RedbBackend::edges_to — G4 (Phase 1)")
    }
}

/// Durability knob for writes.
///
/// **Phase 1 G2-B stub** — semantics finalized in Phase 1 proper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurabilityMode {
    /// fsync before commit returns. Safest, slowest.
    Immediate,
    /// Batch commits, single fsync per batch window.
    Group,
    /// Commit returns before fsync.
    Async,
}

/// A write transaction handle, passed into the `transaction` closure. All
/// writes are atomic at commit.
///
/// **Phase 1 G3-A stub.**
pub struct Transaction<'a> {
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> Transaction<'a> {
    /// Put a Node inside the transaction.
    pub fn put_node(&mut self, _node: &Node) -> Result<Cid, GraphError> {
        todo!("Transaction::put_node — G3-A (Phase 1)")
    }

    /// Put an Edge inside the transaction.
    pub fn put_edge(&mut self, _edge: &Edge) -> Result<Cid, GraphError> {
        todo!("Transaction::put_edge — G3-A (Phase 1)")
    }

    /// Open a nested transaction. Phase 1 always rejects with
    /// [`GraphError::NestedTransactionNotSupported`].
    pub fn transaction<F, R>(&mut self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        todo!("Transaction::transaction (nested) — G3-A (Phase 1)")
    }
}

/// Category of change emitted on the change stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    Created,
    Updated,
    Deleted,
}

/// A post-commit change event. Emitted for every write once the redb commit
/// completes. Consumed by IVM subscribers.
///
/// **Phase 1 G3-A stub.**
#[derive(Debug, Clone)]
pub struct ChangeEvent {
    pub cid: Cid,
    pub label: String,
    pub kind: ChangeKind,
    pub tx_id: u64,
    pub actor_cid: Option<Cid>,
    pub handler_cid: Option<Cid>,
    pub capability_grant_cid: Option<Cid>,
}

impl ChangeEvent {
    /// Stable string form of the event kind, used by integration tests.
    #[must_use]
    pub fn kind_str(&self) -> &'static str {
        match self.kind {
            ChangeKind::Created => "Created",
            ChangeKind::Updated => "Updated",
            ChangeKind::Deleted => "Deleted",
        }
    }
}

/// Abstract subscriber shape for change events. Decouples `benten-graph` from
/// any specific async runtime (R1 architect major #1: no tokio in graph).
///
/// **Phase 1 G3-A stub.**
pub trait ChangeSubscriber: Send + Sync {
    fn on_change(&self, event: &ChangeEvent);
}

/// Return handle from [`RedbBackend::subscribe`]. **Phase 1 G3-A stub.**
pub struct ChangeReceiver;

impl ChangeReceiver {
    /// Receive the next change event (blocking).
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
    pub label: String,
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
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            is_privileged: false,
        }
    }

    /// Construct a WriteContext flagged as privileged (engine-API-only path).
    /// This is the only constructor that bypasses the SC1 system-zone ban.
    /// User code cannot call this without going through `Engine::grant_capability`,
    /// `Engine::create_view`, or `Engine::revoke_capability`.
    #[must_use]
    pub fn privileged_for_engine_api() -> Self {
        Self {
            label: String::new(),
            is_privileged: true,
        }
    }

    /// Called by the transaction primitive to enforce the SC1 stopgap. Rejects
    /// writes to any label starting with `"system:"` unless
    /// `is_privileged == true`.
    pub fn enforce_system_zone(&self) -> Result<(), GraphError> {
        todo!("WriteContext::enforce_system_zone — SC1 (Phase 1)")
    }
}

/// Blanket Node store API — available for any [`KVBackend`] via a key schema
/// rule (CID prefix, schema-versioned). **Phase 1 G4 stub.**
pub trait NodeStore {
    fn put_node(&self, node: &Node) -> Result<Cid, GraphError>;
    fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError>;
}

/// Blanket Edge store API. **Phase 1 G4 stub.**
pub trait EdgeStore {
    fn put_edge(&self, edge: &Edge) -> Result<Cid, GraphError>;
    fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, GraphError>;
    fn edges_from(&self, cid: &Cid) -> Result<Vec<Edge>, GraphError>;
    fn edges_to(&self, cid: &Cid) -> Result<Vec<Edge>, GraphError>;
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
        backend.delete(cid.as_bytes()).unwrap();
        backend.delete(cid.as_bytes()).unwrap(); // second delete must not panic
        assert!(backend.get_node(&cid).unwrap().is_none());
    }

    #[test]
    fn scan_with_prefix() {
        let (backend, _dir) = temp_backend();
        backend.put_node(&canonical_test_node()).unwrap();
        // Every Benten CID starts with the version byte 0x01.
        let hits = backend.scan(&[0x01]).unwrap();
        assert_eq!(hits.len(), 1);
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

        // Confirm redb returns results in sorted key order so callers can rely
        // on it for deterministic downstream processing (content listings, IVM
        // bootstrap).
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

        // A prefix that sorts after every stored key (and cannot be a prefix
        // of any stored key) must return an empty vec, not an error.
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
        // the full table regardless of prefix. Populate three disjoint prefix
        // groups and confirm each scan returns only its group.
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
        // Edge case: a prefix of all-0xff has no lexicographic successor, so
        // the scan falls back to an unbounded `prefix..` range. Verify no
        // panic and that a matching key is still found.
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
        // Store remains empty.
        assert!(backend.scan(&[]).unwrap().is_empty());
    }

    #[test]
    fn next_prefix_increments_and_trims() {
        // Direct unit tests for the range-scan helper.
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

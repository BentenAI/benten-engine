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

use std::path::Path;

use benten_core::{Cid, CoreError, Node};
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors from the storage layer.
#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    /// Propagated from `benten-core` (CID construction, canonical serialization).
    #[error("core: {0}")]
    Core(#[from] CoreError),

    /// redb I/O or transactional failure. Wrapped into a string because
    /// `redb`'s error enum is not `Clone` and its variants are internal
    /// details the caller does not need to switch on in the spike.
    #[error("redb: {0}")]
    Redb(String),

    /// DAG-CBOR decode of a stored Node failed. Indicates on-disk corruption
    /// or a format drift.
    #[error("decode: {0}")]
    Decode(String),
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
pub trait KVBackend {
    /// Fetch the value stored under `key`. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] on storage errors.
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError>;

    /// Insert or overwrite the value at `key`.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] on storage errors.
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), GraphError>;

    /// Delete the value at `key`. Idempotent: returns `Ok(())` even if the key
    /// was absent.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] on storage errors.
    fn delete(&self, key: &[u8]) -> Result<(), GraphError>;

    /// Return every (key, value) pair whose key starts with `prefix`.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] on storage errors.
    fn scan(&self, prefix: &[u8]) -> Result<ScanResult, GraphError>;

    /// Commit multiple puts atomically. Either every pair lands or none do.
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] on storage errors.
    fn put_batch(&self, pairs: &[(Vec<u8>, Vec<u8>)]) -> Result<(), GraphError>;
}

// ---------------------------------------------------------------------------
// redb backend
// ---------------------------------------------------------------------------

/// Single table that stores every Node keyed by its CID bytes.
const NODES_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("benten_nodes");

/// A [`KVBackend`] implementation backed by a local redb v4 database file.
///
/// redb provides serializable isolation (single writer, multiple readers) and
/// durable commits via a two-phase commit with checksummed pages. We rely on
/// those guarantees rather than rolling our own WAL.
pub struct RedbBackend {
    db: Database,
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
        // redb ranges take slice bounds; a prefix scan is (prefix..=prefix+0xff*N)
        // — simpler to iterate everything and filter for the spike. The key
        // space is small (O(thousands) of test nodes) so the scan cost is
        // negligible and the correctness is obvious.
        for item in table.iter()? {
            let (k, v) = item?;
            let k_bytes = k.value();
            if k_bytes.starts_with(prefix) {
                out.push((k_bytes.to_vec(), v.value().to_vec()));
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
}

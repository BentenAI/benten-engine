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

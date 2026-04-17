//! Concrete [`RedbBackend`] — a [`KVBackend`] implementation over redb v4.
//!
//! Extracted from `lib.rs` as part of G2-B, alongside:
//! - the explicit `open_existing` / `open_or_create` split
//!   (R1 triage `P1.graph.open-vs-create`);
//! - the [`DurabilityMode`] wiring (R1 triage `P1.graph.durability`);
//! - the label and property-value indexes (crate-private `indexes` module,
//!   R1 triage `P1.graph.indexes-on-write`).
//!
//! The module owns the redb table definitions and all of the redb-specific
//! plumbing. The `KVBackend` trait it implements lives in [`crate::backend`],
//! and the higher-level `NodeStore` / `EdgeStore` traits it implements live
//! in [`crate::store`]. Inherent methods on [`RedbBackend`] (`put_node`,
//! `delete_node`, …) are the single source of truth for the index contract:
//! they maintain the label and property-value indexes as part of the same
//! write transaction, so the indexes are always in sync with the node store.

use std::path::Path;

use benten_core::{Cid, Edge, Node, Value};
use redb::{
    Database, Durability, MultimapTableDefinition, ReadableDatabase, ReadableMultimapTable,
    ReadableTable, TableDefinition,
};

use crate::backend::{DurabilityMode, KVBackend, ScanResult};
use crate::indexes::{
    LABEL_INDEX_TABLE, PROP_INDEX_TABLE, cid_from_index_bytes, property_index_key,
    value_index_bytes,
};
use crate::store::{
    ChangeSubscriber, EDGE_SRC_PREFIX, EDGE_TGT_PREFIX, EdgeStore, NodeStore, decode_err, edge_key,
    edge_src_index_key, edge_src_index_prefix, edge_tgt_index_key, edge_tgt_index_prefix, node_key,
};
use crate::{GraphError, WriteContext};

/// Primary key/value table storing every `(key, value)` pair. The Node and
/// Edge stores layer the `n:CID`, `e:CID`, `es:SRC|EDGE`, `et:TGT|EDGE` key
/// schema on top of this table.
pub(crate) const NODES_TABLE: TableDefinition<&[u8], &[u8]> = TableDefinition::new("benten_nodes");

/// Lexicographic successor of `prefix` — the smallest byte string strictly
/// greater than every string that begins with `prefix`. Used to turn a
/// prefix scan into a bounded range scan.
///
/// Returns `None` when `prefix` is all-`0xff` (no successor exists in the
/// byte-string ordering), signalling that the caller should do an
/// unbounded `prefix..` scan instead.
pub(crate) fn next_prefix(prefix: &[u8]) -> Option<Vec<u8>> {
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

/// Map a [`DurabilityMode`] onto redb's own `Durability` enum.
///
/// redb v4 currently exposes `Durability::Immediate` (fsync-on-commit) and
/// `Durability::None` (in-memory, lost on crash) only; the intermediate
/// `Group` mode the Benten trait exposes has no direct redb equivalent yet,
/// so it collapses to `Immediate` until redb grows batched-fsync support.
/// This conservative mapping preserves durability at the cost of the
/// throughput win; Phase 2 can revisit without breaking the public enum.
fn to_redb_durability(mode: DurabilityMode) -> Durability {
    match mode {
        DurabilityMode::Immediate | DurabilityMode::Group => Durability::Immediate,
        DurabilityMode::Async => Durability::None,
    }
}

/// Emit a one-shot warning when a caller requests `DurabilityMode::Group` so
/// benchmarks and production tuning don't silently compare Group to
/// Immediate and conclude grouped-fsync "doesn't help" — it simply isn't
/// wired yet. Fires at most once per process.
///
/// Written to stderr directly (no `tracing` dep on this crate); `clippy::
/// print_stderr` is allowed for this one callsite with an explicit reason.
#[allow(
    clippy::print_stderr,
    reason = "one-shot operator-visible warning about a Phase-1 API gap; \
              benten-graph has no tracing dep"
)]
fn warn_if_group_durability_collapsed(mode: DurabilityMode) {
    use std::sync::Once;
    static WARNED: Once = Once::new();
    if matches!(mode, DurabilityMode::Group) {
        WARNED.call_once(|| {
            eprintln!(
                "benten-graph: DurabilityMode::Group collapses to Immediate in \
                 Phase 1 — redb v4 does not yet expose grouped-commit. \
                 Benchmarks comparing Group vs. Immediate will see no delta."
            );
        });
    }
}

/// A [`KVBackend`] implementation backed by a local redb v4 database file.
///
/// redb provides serializable isolation (single writer, multiple readers)
/// and durable commits via a two-phase commit with checksummed pages.
///
/// # Construction
///
/// Three entry points, each with an explicit contract:
///
/// | Constructor | Existing file | Missing file |
/// |---|---|---|
/// | [`RedbBackend::open_existing`] | opens | errors with [`GraphError::BackendNotFound`] |
/// | [`RedbBackend::open_or_create`] | opens | creates |
/// | [`RedbBackend::open`] | opens | creates (kept for backward-compatibility with the spike; new code should pick `open_existing` or `open_or_create` explicitly) |
///
/// `open_existing` is the safer default — it refuses to silently materialize a
/// fresh database under a typoed path (R1 triage `P1.graph.open-vs-create`).
///
/// # Durability
///
/// Both constructors take the default [`DurabilityMode::Immediate`]. The
/// [`RedbBackend::open_existing_with_durability`] and
/// [`RedbBackend::open_or_create_with_durability`] variants let callers pick
/// a looser mode when correctness under a crash is not load-bearing (bench
/// harness, ephemeral test fixture).
///
/// # Concurrency
///
/// `RedbBackend` is not `Clone`. To share a single backend across threads,
/// wrap it in an `Arc`: `let backend = Arc::new(RedbBackend::open_or_create(path)?)`.
/// redb's own API is `&self`, so multiple readers and a single writer can
/// proceed concurrently through the shared `Arc`.
///
/// # Path handling
///
/// The constructors do not canonicalize or validate the database path.
/// Callers receiving paths from untrusted sources (capability-delegated
/// subgraphs, multi-tenant configurations) must sanitize before invoking.
pub struct RedbBackend {
    db: Database,
    durability: Durability,
}

impl core::fmt::Debug for RedbBackend {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RedbBackend").finish_non_exhaustive()
    }
}

impl RedbBackend {
    // ---- Construction -----------------------------------------------------

    /// Open a redb database that must already exist at `path`. Fails with
    /// [`GraphError::BackendNotFound`] if the file is missing — this is the
    /// safer default for production code paths that want to refuse to
    /// silently materialize a new database under a typoed path.
    ///
    /// Commits use [`DurabilityMode::Immediate`] (fsync per commit).
    ///
    /// # Errors
    /// - [`GraphError::BackendNotFound`] if `path` does not exist.
    /// - [`GraphError::Redb`] for any other redb open failure (corrupt file,
    ///   incompatible version, I/O error, lock contention).
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::{GraphError, RedbBackend};
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let missing = dir.path().join("does-not-exist.redb");
    /// let err = RedbBackend::open_existing(&missing).unwrap_err();
    /// assert!(matches!(err, GraphError::BackendNotFound { .. }));
    /// ```
    pub fn open_existing(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Self::open_existing_with_durability(path, DurabilityMode::default())
    }

    /// Open-existing with an explicit [`DurabilityMode`].
    ///
    /// # Errors
    /// Same as [`Self::open_existing`].
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::{DurabilityMode, RedbBackend};
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let path = dir.path().join("db.redb");
    /// // Materialize the file first so `open_existing*` has something to open.
    /// let _first = RedbBackend::open_or_create(&path).unwrap();
    /// drop(_first);
    ///
    /// let _reopened = RedbBackend::open_existing_with_durability(
    ///     &path,
    ///     DurabilityMode::Immediate,
    /// )
    /// .unwrap();
    /// ```
    pub fn open_existing_with_durability(
        path: impl AsRef<Path>,
        durability: DurabilityMode,
    ) -> Result<Self, GraphError> {
        // Note: the `path.exists()` check below races with external
        // filesystem mutations (TOCTOU). In Phase 1 the value is a clean
        // `GraphError::BackendNotFound` instead of an opaque
        // "unable to allocate page" leak through `GraphError::Redb` —
        // acceptable for single-user local stores under redb's exclusive
        // lock. Phase 3 P2P workloads may revisit.
        warn_if_group_durability_collapsed(durability);
        let path = path.as_ref();
        if !path.exists() {
            return Err(GraphError::BackendNotFound {
                path: path.to_path_buf(),
            });
        }
        let db = Database::open(path)?;
        let backend = Self {
            db,
            durability: to_redb_durability(durability),
        };
        backend.ensure_tables()?;
        Ok(backend)
    }

    /// Open the redb database at `path`, creating it if it doesn't already
    /// exist. Idempotent on an existing file.
    ///
    /// Commits use [`DurabilityMode::Immediate`] (fsync per commit).
    ///
    /// # Errors
    /// Returns [`GraphError::Redb`] if redb cannot open or create the file,
    /// or if the initial table creation transaction fails.
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let path = dir.path().join("fresh.redb");
    /// let _backend = RedbBackend::open_or_create(&path).unwrap();
    /// assert!(path.exists());
    /// ```
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Self::open_or_create_with_durability(path, DurabilityMode::default())
    }

    /// Open-or-create with an explicit [`DurabilityMode`].
    ///
    /// # Errors
    /// Same as [`Self::open_or_create`].
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::{DurabilityMode, RedbBackend};
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let path = dir.path().join("bench.redb");
    /// // `Async` durability — commit returns before fsync. Test/bench only.
    /// let _backend = RedbBackend::open_or_create_with_durability(
    ///     &path,
    ///     DurabilityMode::Async,
    /// )
    /// .unwrap();
    /// ```
    pub fn open_or_create_with_durability(
        path: impl AsRef<Path>,
        durability: DurabilityMode,
    ) -> Result<Self, GraphError> {
        warn_if_group_durability_collapsed(durability);
        let db = Database::create(path.as_ref())?;
        let backend = Self {
            db,
            durability: to_redb_durability(durability),
        };
        backend.ensure_tables()?;
        Ok(backend)
    }

    /// Backward-compatible alias for [`Self::open_or_create`]. New code should
    /// pick the explicit variant so the create-on-miss semantics are visible
    /// at the call site.
    ///
    /// # Errors
    /// See [`Self::open_or_create`].
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let _backend = RedbBackend::open(dir.path().join("db.redb")).unwrap();
    /// ```
    pub fn open(path: impl AsRef<Path>) -> Result<Self, GraphError> {
        Self::open_or_create(path)
    }

    /// Materialize every table we need so cold-database reads don't fail.
    /// Creating an existing table is a redb no-op.
    fn ensure_tables(&self) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let _ = write_txn.open_table(NODES_TABLE)?;
            let _ = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
            let _ = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    /// Begin a write transaction with this backend's configured
    /// durability. Centralizing the durability wiring here means every
    /// mutating path picks up a durability change automatically.
    fn begin_write_txn(&self) -> Result<redb::WriteTransaction, GraphError> {
        let mut txn = self.db.begin_write()?;
        txn.set_durability(self.durability)
            .map_err(|e| GraphError::Redb(e.to_string()))?;
        Ok(txn)
    }

    // ---- Inherent node/edge delegates ------------------------------------

    /// Store a Node under its CID, and maintain the label and property-value
    /// indexes in the same write transaction.
    ///
    /// Inserts one multimap entry per `(node, label)` pair into the
    /// crate-private label index, and one per `(node, label, prop_name)`
    /// triple into the crate-private property-value index. All writes —
    /// body plus every index entry — commit atomically.
    ///
    /// # Errors
    /// - [`GraphError::Core`] if the Node cannot be DAG-CBOR encoded or its
    ///   CID cannot be computed.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_core::{Node, Value};
    /// use benten_graph::RedbBackend;
    /// use std::collections::BTreeMap;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let mut props = BTreeMap::new();
    /// props.insert("title".to_string(), Value::text("hello"));
    /// let cid = b.put_node(&Node::new(vec!["Post".to_string()], props)).unwrap();
    /// assert!(b.get_by_label("Post").unwrap().contains(&cid));
    /// ```
    pub fn put_node(&self, node: &Node) -> Result<Cid, GraphError> {
        let cid = node.cid()?;
        let bytes = node.canonical_bytes()?;
        let n_key = node_key(&cid);

        let write_txn = self.begin_write_txn()?;
        {
            let mut nodes = write_txn.open_table(NODES_TABLE)?;
            nodes.insert(n_key.as_slice(), bytes.as_slice())?;
        }
        {
            let mut label_idx = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
            for label in &node.labels {
                label_idx.insert(label.as_bytes(), cid.as_bytes().as_slice())?;
            }
        }
        {
            let mut prop_idx = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
            for label in &node.labels {
                for (prop_name, value) in &node.properties {
                    let vbytes = value_index_bytes(value)?;
                    let key = property_index_key(label, prop_name, &vbytes);
                    prop_idx.insert(key.as_slice(), cid.as_bytes().as_slice())?;
                }
            }
        }
        write_txn.commit()?;
        Ok(cid)
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Propagates the [`NodeStore`] error shape.
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use benten_core::testing::canonical_test_node;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let node = canonical_test_node();
    /// let cid = b.put_node(&node).unwrap();
    /// assert_eq!(b.get_node(&cid).unwrap().unwrap(), node);
    /// ```
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, GraphError> {
        let Some(bytes) = self.get(&node_key(cid))? else {
            return Ok(None);
        };
        let node: Node = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(GraphError::from)?;
        Ok(Some(node))
    }

    /// Delete a Node by CID, and remove it from the label and property-value
    /// indexes in the same write transaction. Idempotent — deleting an absent
    /// CID is not an error.
    ///
    /// # Errors
    /// - [`GraphError::Core`] if a stored Node cannot be decoded back to
    ///   compute its index keys.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_graph::RedbBackend;
    /// use benten_core::testing::canonical_test_node;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let node = canonical_test_node();
    /// let cid = b.put_node(&node).unwrap();
    /// b.delete_node(&cid).unwrap();
    /// b.delete_node(&cid).unwrap(); // idempotent
    /// assert!(b.get_node(&cid).unwrap().is_none());
    /// ```
    pub fn delete_node(&self, cid: &Cid) -> Result<(), GraphError> {
        // SAFETY-REASONING: reading the existing Node outside the delete's
        // write transaction is safe under the content-addressed invariant.
        // A concurrent `put_node(same CID)` writes identical body bytes and
        // identical index keys (labels + DAG-CBOR-encoded values are a
        // pure function of the CID), so our read-view index-key set cannot
        // diverge from the current state — the removal targets the same
        // keys either way, and redb multimap `remove` is idempotent. This
        // invariant breaks for Phase-2 mutable identities (Anchor.CURRENT
        // pointer, named roots); re-evaluate when those land.
        let existing = self.get_node(cid)?;
        let n_key = node_key(cid);

        let write_txn = self.begin_write_txn()?;
        {
            let mut nodes = write_txn.open_table(NODES_TABLE)?;
            nodes.remove(n_key.as_slice())?;
        }
        if let Some(node) = existing {
            {
                let mut label_idx = write_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
                for label in &node.labels {
                    label_idx.remove(label.as_bytes(), cid.as_bytes().as_slice())?;
                }
            }
            {
                let mut prop_idx = write_txn.open_multimap_table(PROP_INDEX_TABLE)?;
                for label in &node.labels {
                    for (prop_name, value) in &node.properties {
                        let vbytes = value_index_bytes(value)?;
                        let key = property_index_key(label, prop_name, &vbytes);
                        prop_idx.remove(key.as_slice(), cid.as_bytes().as_slice())?;
                    }
                }
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    // ---- Edge CRUD -------------------------------------------------------

    /// Store an Edge and its source/target indexes. Returns the Edge CID.
    ///
    /// # Errors
    /// - [`GraphError::Core`] if the Edge cannot be DAG-CBOR encoded.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    pub fn put_edge(&self, edge: &Edge) -> Result<Cid, GraphError> {
        let cid = edge.cid()?;
        let bytes = edge.canonical_bytes()?;
        // Body first, then indexes. The body/index pair is idempotent
        // (re-putting the same edge writes identical bytes to the same
        // keys), so ordering under the non-transactional path is not
        // load-bearing at Phase 1. G3 wraps these in a single redb txn.
        self.put(&edge_key(&cid), &bytes)?;
        self.put(&edge_src_index_key(&edge.source, &cid), &[])?;
        self.put(&edge_tgt_index_key(&edge.target, &cid), &[])?;
        Ok(cid)
    }

    /// Retrieve an Edge by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, GraphError> {
        let Some(bytes) = self.get(&edge_key(cid))? else {
            return Ok(None);
        };
        let edge: Edge = serde_ipld_dagcbor::from_slice(&bytes)
            .map_err(decode_err)
            .map_err(GraphError::from)?;
        Ok(Some(edge))
    }

    /// Delete an Edge and its source/target indexes. Idempotent.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn delete_edge(&self, cid: &Cid) -> Result<(), GraphError> {
        if let Some(edge) = self.get_edge(cid)? {
            self.delete(&edge_src_index_key(&edge.source, cid))?;
            self.delete(&edge_tgt_index_key(&edge.target, cid))?;
        }
        self.delete(&edge_key(cid))
    }

    /// All edges whose `source == cid`.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, GraphError> {
        let hits = self.scan(&edge_src_index_prefix(source))?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            let Some(edge_cid_bytes) = k.get(EDGE_SRC_PREFIX.len() + source.as_bytes().len()..)
            else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(GraphError::from)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }

    /// All edges whose `target == cid`.
    ///
    /// # Errors
    /// Propagates the [`EdgeStore`] error shape.
    pub fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, GraphError> {
        let hits = self.scan(&edge_tgt_index_prefix(target))?;
        let mut out = Vec::with_capacity(hits.len());
        for (k, _v) in hits.iter() {
            let Some(edge_cid_bytes) = k.get(EDGE_TGT_PREFIX.len() + target.as_bytes().len()..)
            else {
                continue;
            };
            let edge_cid = Cid::from_bytes(edge_cid_bytes).map_err(GraphError::from)?;
            if let Some(edge) = self.get_edge(&edge_cid)? {
                out.push(edge);
            }
        }
        Ok(out)
    }

    // ---- Indexes ---------------------------------------------------------

    /// Every Node CID stored under `label`. Empty [`Vec`] on a miss.
    ///
    /// Case-sensitive: the stored label must match byte-for-byte. Empty input
    /// returns an empty result (label lookups never match the empty label).
    ///
    /// # Errors
    /// - [`GraphError::Redb`] on a read failure.
    /// - [`GraphError::Core`] if an index entry's bytes don't round-trip
    ///   through [`Cid::from_bytes`] (indicates on-disk corruption).
    ///
    /// # Examples
    /// ```rust
    /// use benten_core::{Node, Value};
    /// use benten_graph::RedbBackend;
    /// use std::collections::BTreeMap;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let mut props = BTreeMap::new();
    /// props.insert("title".to_string(), Value::text("hi"));
    /// let cid = b.put_node(&Node::new(vec!["Post".to_string()], props)).unwrap();
    /// assert_eq!(b.get_by_label("Post").unwrap(), vec![cid]);
    /// assert!(b.get_by_label("Missing").unwrap().is_empty());
    /// ```
    pub fn get_by_label(&self, label: &str) -> Result<Vec<Cid>, GraphError> {
        if label.is_empty() {
            return Ok(Vec::new());
        }
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_multimap_table(LABEL_INDEX_TABLE)?;
        let values = table.get(label.as_bytes())?;
        let mut out = Vec::new();
        for v in values {
            let v = v?;
            let cid = cid_from_index_bytes(v.value())?;
            out.push(cid);
        }
        Ok(out)
    }

    /// Every Node CID stored under `label` whose property `prop_name` equals
    /// `value` (exact byte-level match after DAG-CBOR encoding).
    ///
    /// Returns an empty vector on any kind of miss — unknown label, unknown
    /// property, value mismatch, value *type* mismatch (`Int(10)` vs
    /// `Text("10")`).
    ///
    /// # Errors
    /// - [`GraphError::Core`] if the supplied `value` cannot be encoded, or
    ///   if an index entry fails to decode back to a CID.
    /// - [`GraphError::Redb`] on a read failure.
    ///
    /// # Examples
    /// ```rust
    /// use benten_core::{Node, Value};
    /// use benten_graph::RedbBackend;
    /// use std::collections::BTreeMap;
    /// use tempfile::tempdir;
    ///
    /// let dir = tempdir().unwrap();
    /// let b = RedbBackend::open_or_create(dir.path().join("db.redb")).unwrap();
    /// let mut props = BTreeMap::new();
    /// props.insert("views".to_string(), Value::Int(10));
    /// let cid = b.put_node(&Node::new(vec!["Post".to_string()], props)).unwrap();
    /// assert_eq!(
    ///     b.get_by_property("Post", "views", &Value::Int(10)).unwrap(),
    ///     vec![cid],
    /// );
    /// assert!(
    ///     b.get_by_property("Post", "views", &Value::Int(11))
    ///         .unwrap()
    ///         .is_empty()
    /// );
    /// ```
    pub fn get_by_property(
        &self,
        label: &str,
        prop_name: &str,
        value: &Value,
    ) -> Result<Vec<Cid>, GraphError> {
        let vbytes = value_index_bytes(value).map_err(GraphError::from)?;
        let key = property_index_key(label, prop_name, &vbytes);
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_multimap_table(PROP_INDEX_TABLE)?;
        let values = table.get(key.as_slice())?;
        let mut out = Vec::new();
        for v in values {
            let v = v?;
            let cid = cid_from_index_bytes(v.value())?;
            out.push(cid);
        }
        Ok(out)
    }

    // ---- Stubs owned by later groups (G3 / G6 / SC1) ----------------------

    /// Store a Node under a caller-supplied [`WriteContext`]. G3/G6 own the
    /// full semantics; this stub is invoked by the SC1 system-zone tests.
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

    /// Transaction primitive — a closure over a write transaction handle.
    /// Atomic: all writes inside the closure commit together, or none do.
    ///
    /// **Phase 1 G3-A stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn transaction<F, R>(&self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut crate::Transaction<'_>) -> Result<R, GraphError>,
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
        F: FnOnce(&mut crate::Transaction<'_>) -> Result<R, GraphError>,
    {
        todo!("RedbBackend::transaction_with_deny_on_commit — G3-A (Phase 1)")
    }

    /// Register a change subscriber. The transaction primitive (G3) fans
    /// change events out synchronously to every registered subscriber after
    /// a successful commit. The subscriber is stored as an `Arc<dyn
    /// ChangeSubscriber>` so heterogeneous IVM views can coexist.
    ///
    /// Per the plan's R1 architect ratification (§line-605), the pull-shaped
    /// channel concretion — tokio-broadcast on native, synchronous
    /// `Vec<Box<dyn ChangeSubscriber>>` fan-out on WASM — lives in
    /// [`benten-engine::change`](https://docs.rs/benten-engine), NOT here.
    /// `benten-graph` stays runtime-agnostic.
    ///
    /// # Errors
    /// Stub — currently `todo!()`. G3 wires the subscriber list into the
    /// transaction primitive's post-commit callback fan-out.
    pub fn register_subscriber(
        &self,
        _subscriber: std::sync::Arc<dyn ChangeSubscriber>,
    ) -> Result<(), GraphError> {
        todo!("RedbBackend::register_subscriber — G3-A (Phase 1)")
    }

    /// Open a MVCC snapshot handle. **Phase 1 G6 stub.**
    ///
    /// # Errors
    /// Stub — currently `todo!()`.
    pub fn snapshot(&self) -> Result<crate::SnapshotHandle, GraphError> {
        todo!("RedbBackend::snapshot — G6 (Phase 1)")
    }
}

// ---------------------------------------------------------------------------
// KVBackend impl
// ---------------------------------------------------------------------------

impl KVBackend for RedbBackend {
    type Error = GraphError;

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, GraphError> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(NODES_TABLE)?;
        Ok(table.get(key)?.map(|v| v.value().to_vec()))
    }

    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
        {
            let mut table = write_txn.open_table(NODES_TABLE)?;
            table.insert(key, value)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn delete(&self, key: &[u8]) -> Result<(), GraphError> {
        let write_txn = self.begin_write_txn()?;
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

        // For a non-empty prefix we bound the scan to keys in
        // `[prefix, next_prefix)`. `next_prefix` is the lexicographic successor
        // of `prefix` obtained by incrementing the last non-0xff byte; if
        // `prefix` is all 0xff the upper bound is open-ended.
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
        let write_txn = self.begin_write_txn()?;
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
// NodeStore / EdgeStore — concrete impls for RedbBackend
// ---------------------------------------------------------------------------
//
// The blanket `impl<T: KVBackend>` was removed (g2-cr-1) to close a latent
// footgun where generic trait dispatch silently skipped index maintenance.
// RedbBackend now implements NodeStore / EdgeStore directly; the impls
// forward to the inherent methods above, which are the single source of
// truth for the index contract.

impl NodeStore for RedbBackend {
    type Error = GraphError;

    fn put_node(&self, node: &Node) -> Result<Cid, Self::Error> {
        RedbBackend::put_node(self, node)
    }

    fn get_node(&self, cid: &Cid) -> Result<Option<Node>, Self::Error> {
        RedbBackend::get_node(self, cid)
    }

    fn delete_node(&self, cid: &Cid) -> Result<(), Self::Error> {
        RedbBackend::delete_node(self, cid)
    }
}

impl EdgeStore for RedbBackend {
    type Error = GraphError;

    fn put_edge(&self, edge: &Edge) -> Result<Cid, Self::Error> {
        RedbBackend::put_edge(self, edge)
    }

    fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, Self::Error> {
        RedbBackend::get_edge(self, cid)
    }

    fn delete_edge(&self, cid: &Cid) -> Result<(), Self::Error> {
        RedbBackend::delete_edge(self, cid)
    }

    fn edges_from(&self, source: &Cid) -> Result<Vec<Edge>, Self::Error> {
        RedbBackend::edges_from(self, source)
    }

    fn edges_to(&self, target: &Cid) -> Result<Vec<Edge>, Self::Error> {
        RedbBackend::edges_to(self, target)
    }
}

// ---------------------------------------------------------------------------
// Tests for module-private helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests and benches may use unwrap/expect per workspace policy"
)]
mod tests {
    use super::*;

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

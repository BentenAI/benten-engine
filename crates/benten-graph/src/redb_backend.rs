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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

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
use crate::transaction::{TxGuard, fan_out};
use crate::{GraphError, Transaction, WriteContext};

/// Shared system-zone label-prefix check used by every write entry point on
/// [`RedbBackend`] — both the `WriteContext`-aware paths and the inherent
/// `put_node` / `put_edge` that the [`NodeStore`] / [`EdgeStore`] trait
/// delegates route through. An unprivileged write whose Node has any label
/// starting with `"system:"` returns `E_SYSTEM_ZONE_WRITE`.
///
/// Extracted at the G3 mini-review fix-pass (chaos-engineer g3-ce-1). Before
/// this helper existed, the inherent `RedbBackend::put_node` bypassed the
/// guard entirely — a binding caller or trait-dispatching generic code could
/// forge a `system:CapabilityGrant` via the plain `put_node` path while the
/// `put_node_with_context` path correctly rejected.
fn guard_system_zone_node(node: &Node, is_privileged: bool) -> Result<(), GraphError> {
    if is_privileged {
        return Ok(());
    }
    for label in &node.labels {
        if label.starts_with("system:") {
            return Err(GraphError::SystemZoneWrite {
                label: label.clone(),
            });
        }
    }
    Ok(())
}

/// Edge counterpart of [`guard_system_zone_node`]. R1 SC1 named only Node
/// labels explicitly, but edges with `"system:"`-prefixed labels are the
/// obvious smuggling vector (an edge `system:Grant` from an attacker's
/// principal to a privileged capability), so the prefix reservation
/// extends to edge labels as well.
fn guard_system_zone_edge(edge: &Edge, is_privileged: bool) -> Result<(), GraphError> {
    if !is_privileged && edge.label.starts_with("system:") {
        return Err(GraphError::SystemZoneWrite {
            label: edge.label.clone(),
        });
    }
    Ok(())
}

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
    /// Registered change-event subscribers. Behind a `Mutex<Vec<...>>` so
    /// `register_subscriber` and the post-commit fan-out can share one
    /// list without forcing callers to hold an `Arc<RedbBackend>`.
    subscribers: Arc<Mutex<Vec<Arc<dyn ChangeSubscriber>>>>,
    /// In-transaction flag. Set via [`TxGuard`] at the start of a
    /// closure-based transaction, cleared on drop. Prevents nested
    /// `backend.transaction(|_| backend.transaction(...))` calls from
    /// deadlocking on redb's single-writer lock; the second
    /// `RedbBackend::transaction` sees `true` and returns
    /// [`GraphError::NestedTransactionNotSupported`] without ever asking
    /// redb to open a second write txn.
    ///
    /// TODO(phase-2): the flag is per-`Arc<RedbBackend>`; two distinct Arc
    /// handles opened on the same redb file do not coordinate and fall
    /// through to redb's single-writer lock (which blocks rather than
    /// deadlocking). Mini-review g3-ce-7 proposes keying the flag on the
    /// canonical DB path via a process-wide static. Phase 1 treats the
    /// single-handle invariant as documented.
    tx_flag: Arc<Mutex<bool>>,
    /// Monotonically increasing transaction id stamped onto
    /// [`crate::ChangeEvent::tx_id`]. Starts at 1 so that tests can reserve
    /// 0 as "no event". Atomic because the backend may be shared across
    /// threads behind an `Arc`.
    ///
    /// TODO(phase-2): `tx_id` is process-lifetime-only; reopening the
    /// backend restarts the counter at 1. An IVM persistence layer that
    /// uses `tx_id` as a durable high-water-mark would see a monotonicity
    /// violation across restart. Mini-review g3-ce-8 proposes persisting
    /// the counter into a dedicated redb table; Phase 1 documents the
    /// limitation (IVM views rebuild from scratch on restart in Phase 1).
    next_tx_id: Arc<AtomicU64>,
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
            subscribers: Arc::new(Mutex::new(Vec::new())),
            tx_flag: Arc::new(Mutex::new(false)),
            next_tx_id: Arc::new(AtomicU64::new(1)),
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
            subscribers: Arc::new(Mutex::new(Vec::new())),
            tx_flag: Arc::new(Mutex::new(false)),
            next_tx_id: Arc::new(AtomicU64::new(1)),
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
        // Fail-closed on the inherent path: the `NodeStore::put_node` trait
        // delegate and any direct user/binding call route here, so the
        // system-zone guard MUST fire before any redb write. Engine-internal
        // privileged paths go through `put_node_with_context` with a
        // privileged `WriteContext`.
        guard_system_zone_node(node, /* is_privileged= */ false)?;
        self.put_node_unchecked(node)
    }

    /// Internal helper: the indexed put without the system-zone guard.
    /// Callers (the guarded `put_node`, the context-aware
    /// `put_node_with_context`) enforce the guard before calling; this body
    /// runs the redb write and index maintenance under a single commit.
    fn put_node_unchecked(&self, node: &Node) -> Result<Cid, GraphError> {
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
    /// Fail-closed system-zone guard: an edge whose label begins with
    /// `"system:"` is rejected on the user path with `E_SYSTEM_ZONE_WRITE`
    /// (R1 SC1 extension to edges; mini-review g3-ce-2). Engine-internal
    /// privileged paths go through [`Self::put_edge_with_context`] with a
    /// privileged `WriteContext`.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone edge.
    /// - [`GraphError::Core`] if the Edge cannot be DAG-CBOR encoded.
    /// - [`GraphError::Redb`] on any underlying redb failure.
    pub fn put_edge(&self, edge: &Edge) -> Result<Cid, GraphError> {
        guard_system_zone_edge(edge, /* is_privileged= */ false)?;
        self.put_edge_unchecked(edge)
    }

    /// Put an Edge under a caller-supplied [`WriteContext`]. Mirrors
    /// [`Self::put_node_with_context`] — privileged contexts bypass the
    /// `"system:"` label guard; unprivileged contexts enforce it.
    ///
    /// Phase 1 exposes this primarily for symmetry with `put_node_with_context`
    /// and for G7 engine-internal code that needs to write system-zone
    /// edges (grant-backed capability edges).
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone edge.
    /// - Every error [`Self::put_edge`] can surface.
    pub fn put_edge_with_context(
        &self,
        edge: &Edge,
        ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        guard_system_zone_edge(edge, ctx.is_privileged)?;
        self.put_edge_unchecked(edge)
    }

    /// Internal helper — the edge write and index maintenance without the
    /// system-zone guard. Used by `put_edge` (guarded) and
    /// `put_edge_with_context` (context-driven guard).
    fn put_edge_unchecked(&self, edge: &Edge) -> Result<Cid, GraphError> {
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

    // ---- G3-A transaction + change-stream surface ------------------------

    /// Store a Node under a caller-supplied [`WriteContext`]. The R1 SC1
    /// system-zone stopgap: an unprivileged context (`is_privileged ==
    /// false`) rejects any Node whose label list contains a `"system:"`-
    /// prefixed label. A privileged context (set only by the engine-API
    /// paths `grant_capability` / `create_view` / `revoke_capability`) may
    /// write system-zone labels.
    ///
    /// On success this delegates to the inherent [`RedbBackend::put_node`]
    /// — the system-zone guard is the only thing this method adds; label
    /// and property-index maintenance and the actual redb write happen
    /// identically to the direct-path call.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone
    ///   label.
    /// - Every error [`RedbBackend::put_node`] can surface.
    pub fn put_node_with_context(
        &self,
        node: &Node,
        ctx: &WriteContext,
    ) -> Result<Cid, GraphError> {
        guard_system_zone_node(node, ctx.is_privileged)?;
        // Route around the inherent put_node's guard (already enforced above)
        // so a privileged context can land a system-zone write.
        self.put_node_unchecked(node)
    }

    /// Transaction primitive — a closure over a write transaction handle.
    /// Atomic: all writes inside the closure commit together, or none do.
    ///
    /// Execution shape:
    /// 1. Acquire the in-transaction guard. A concurrent or nested
    ///    `.transaction()` call short-circuits here with
    ///    [`GraphError::NestedTransactionNotSupported`] without ever
    ///    touching redb's single-writer lock.
    /// 2. Begin a redb write transaction at the configured durability.
    /// 3. Run the closure against a [`Transaction`] wrapper. Writes go
    ///    straight to the inner redb txn AND accumulate in a pending-ops
    ///    list used for post-commit change-event fan-out.
    /// 4. On closure `Ok`: commit the redb txn, then fan
    ///    [`crate::ChangeEvent`]s to every registered subscriber. Events
    ///    are only emitted after commit succeeds — a commit-time I/O
    ///    failure swallows the batch.
    /// 5. On closure `Err`: drop the txn (redb aborts automatically),
    ///    return [`GraphError::TxAborted`] wrapping the inner reason.
    /// 6. On closure panic: the txn drops cleanly, the guard releases via
    ///    RAII, and the panic propagates to the caller.
    ///
    /// # Errors
    /// - [`GraphError::NestedTransactionNotSupported`] on a nested or
    ///   concurrent call.
    /// - [`GraphError::Redb`] on a redb commit failure.
    /// - [`GraphError::TxAborted`] wrapping the closure's `Err`.
    pub fn transaction<F, R>(&self, f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        let _guard = TxGuard::try_acquire(Arc::clone(&self.tx_flag))?;
        let write_txn = self.begin_write_txn()?;
        // `begin_write_txn` already sets durability on the inner txn, so
        // `Transaction::new` sees a fresh WriteTransaction with the
        // backend's configured durability already in place. Transaction::new
        // re-applies it defensively (cheap; idempotent).
        let mut tx = Transaction::new(write_txn, self.durability, /* privileged */ false)?;

        match f(&mut tx) {
            Ok(value) => {
                let pending = tx.commit()?;
                if !pending.is_empty() {
                    let tx_id = self.next_tx_id.fetch_add(1, Ordering::SeqCst);
                    // Skip the clone entirely when no subscribers are
                    // registered (thinness path — every commit skips a
                    // vec-clone when IVM isn't wired). Chaos-engineer
                    // g3-ce-10 reservation: a subscriber registered between
                    // the commit and the snapshot below observes the just-
                    // committed event; one registered afterwards does not.
                    let subs = {
                        let guard = self.subscribers.lock().unwrap_or_else(|e| e.into_inner());
                        if guard.is_empty() {
                            Vec::new()
                        } else {
                            guard.clone()
                        }
                    };
                    if !subs.is_empty() {
                        fan_out(&subs, &pending, tx_id);
                    }
                }
                Ok(value)
            }
            Err(inner) => {
                // `tx` drops here without commit — redb aborts automatically.
                drop(tx);
                Err(GraphError::TxAborted {
                    reason: inner.to_string(),
                })
            }
        }
    }

    /// Transaction variant that ALWAYS denies at commit, used by the commit-
    /// denial edge-case test (`failure_injection_rollback.rs::
    /// tx_commit_cap_failure_surfaces_partial_trace_with_aborted_step`). The
    /// closure runs to completion; immediately before the redb commit fires,
    /// a synthetic "capability denied" hook rejects the batch. This models
    /// the behavior the engine orchestrator will produce when the real
    /// `CapabilityPolicy::check_write` returns `Err` at the commit
    /// boundary.
    ///
    /// Name history: this method is a dedicated test hook rather than a
    /// configurable predicate — a future caller that needs a configurable
    /// `deny_at_commit` should land a new method rather than layering config
    /// onto this one. Renaming to make the intent obvious is tracked as an
    /// R4b docket item (mini-review g3-cr-9).
    ///
    /// Phase 1 keeps this as a dedicated test hook rather than forcing a
    /// public `CapabilityPolicy` dep into `benten-graph` — the engine
    /// orchestrator (`benten-engine`) is the sole policy-aware caller.
    ///
    /// # Errors
    /// - [`GraphError::TxAborted`] with a `reason` naming "capability" on
    ///   simulated commit-time denial.
    /// - [`GraphError::TxAborted`] with the closure's inner reason if the
    ///   closure itself returned `Err`.
    /// - [`GraphError::NestedTransactionNotSupported`] on a nested call.
    pub fn transaction_with_deny_on_commit<F, R>(&self, f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        let _guard = TxGuard::try_acquire(Arc::clone(&self.tx_flag))?;
        let write_txn = self.begin_write_txn()?;
        let mut tx = Transaction::new(write_txn, self.durability, /* privileged */ false)?;
        let _closure_value = f(&mut tx).map_err(|inner| GraphError::TxAborted {
            reason: inner.to_string(),
        })?;
        // Simulated deny-at-commit hook: always refuses. The redb txn drops
        // without commit, so no writes persist.
        drop(tx);
        Err(GraphError::TxAborted {
            reason: "capability denied at commit (test hook)".to_string(),
        })
    }

    /// Register a change subscriber. The transaction primitive fans change
    /// events out synchronously to every registered subscriber after a
    /// successful commit. The subscriber is stored as an `Arc<dyn
    /// ChangeSubscriber>` so heterogeneous IVM views can coexist.
    ///
    /// Per the plan's R1 architect ratification (§line-605), the pull-shaped
    /// channel concretion — tokio-broadcast on native, synchronous
    /// `Vec<Box<dyn ChangeSubscriber>>` fan-out on WASM — lives in
    /// [`benten-engine::change`](https://docs.rs/benten-engine), not here.
    /// `benten-graph` stays runtime-agnostic.
    ///
    /// # Ordering contract (mini-review g3-ce-10)
    ///
    /// A subscriber registered **strictly before** a commit's post-commit
    /// subscribers snapshot observes that commit's event batch. A subscriber
    /// registered **after** the snapshot does not. The snapshot is taken
    /// inside the transaction method after the redb commit returns success.
    /// An IVM view that snapshot-reads the graph to bootstrap should register
    /// first and read second to avoid double-applying events in the race
    /// window.
    ///
    /// # Subscriber lifecycle
    ///
    /// Phase 1 has no deregister path — subscribers live for the backend's
    /// lifetime. Dropping the `RedbBackend` (or the last `Arc`) releases the
    /// subscriber list. Phase 2 will land a `Subscription` handle with
    /// drop-deregister semantics (tracked as a G5 follow-up per mini-review
    /// g3-cr-15).
    ///
    /// # Errors
    /// Returns `Ok(())` unconditionally in Phase 1. The fallible signature
    /// is preserved for forward-compat with Phase 3 WASM backends that may
    /// reject subscribers whose fan-out shape is incompatible with the
    /// peer-fetch runtime.
    pub fn register_subscriber(
        &self,
        subscriber: Arc<dyn ChangeSubscriber>,
    ) -> Result<(), GraphError> {
        let mut guard = self.subscribers.lock().unwrap_or_else(|e| e.into_inner());
        guard.push(subscriber);
        Ok(())
    }

    /// Count of currently-registered change subscribers. Used by thinness
    /// tests that assert the subscriber list stays empty when IVM is
    /// disabled.
    #[must_use]
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.lock().map(|g| g.len()).unwrap_or(0)
    }

    /// Open a MVCC snapshot handle. The handle captures redb's read-txn at
    /// the call instant; subsequent writes to the backend are invisible to
    /// the snapshot until it is dropped and a fresh one is opened.
    ///
    /// # Errors
    /// [`GraphError::Redb`] if redb refuses to open a read transaction
    /// (an I/O failure or a severely corrupt file).
    pub fn snapshot(&self) -> Result<crate::SnapshotHandle, GraphError> {
        let read_txn = self.db.begin_read()?;
        Ok(crate::SnapshotHandle {
            read_txn: Some(read_txn),
        })
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

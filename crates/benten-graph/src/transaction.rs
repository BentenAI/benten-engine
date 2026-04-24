//! Closure-based [`Transaction`] primitive over a single redb
//! [`redb::WriteTransaction`].
//!
//! Semantics (G3-A, R1 triage "transaction primitive" row):
//!
//! - Callers enter via [`RedbBackend::transaction`](crate::RedbBackend::transaction).
//!   The closure receives `&mut Transaction`.
//! - [`Transaction::put_node`] / [`Transaction::put_edge`] enqueue the
//!   operation in the pending-ops list AND write-through to the redb
//!   `WriteTransaction` so later reads inside the same batch see their own
//!   writes (future G6 snapshot-reads within txn use this).
//! - [`Transaction::transaction`] (nested) is always rejected with
//!   [`GraphError::NestedTransactionNotSupported`]. This is the Phase-1
//!   named compromise (plan §3, R1 triage); Phase 2 may lift.
//! - On closure `Ok`: the backend applies the capability hook (if any),
//!   commits the redb transaction, and fans [`ChangeEvent`]
//!   emissions out to registered subscribers *after* commit succeeds. If
//!   commit fails, no events are emitted.
//! - On closure `Err`: the transaction is aborted via the `Drop` path —
//!   redb drops the `WriteTransaction` without committing, so no writes
//!   land. The error is mapped to [`GraphError::TxAborted`] at the outer
//!   `backend.transaction(...)` call.
//! - Closure panic: the `Drop` impl aborts cleanly (redb's
//!   `WriteTransaction` aborts on drop-without-commit) and the panic
//!   propagates to the caller. The backend's in-tx flag is released via
//!   an internal RAII guard so subsequent calls to `.transaction()`
//!   don't see a stuck lock.
//!
//! # Privileged vs unprivileged transactions (G7 reservation)
//!
//! Phase-1 transactions opened via `RedbBackend::transaction` are always
//! unprivileged — the [`WriteAuthority`] on [`Transaction`] is
//! `WriteAuthority::User` at the backend entry points, and the system-zone
//! label guard consults it via `Transaction::is_privileged`. Engine-
//! internal multi-write operations that need to write `"system:"`-prefixed
//! nodes or edges atomically (grant_capability / create_view /
//! revoke_capability, G7) will land a dedicated privileged entry point
//! that sets `WriteAuthority::EnginePrivileged`; user-authored closures
//! never gain a path to flip the authority.
//!
//! # Subscriber ordering and dead-letter observability (R4b follow-ups)
//!
//! `fan_out` snapshots the subscribers list after the redb commit but
//! before invoking any callback. A subscriber registered strictly before
//! the commit's tx-id fetch observes the event; one registered afterwards
//! does not. Panics inside a subscriber are caught and discarded; Phase 1
//! has no dead-letter counter so repeated panics are invisible to operators
//! — adding a counter (TODO(phase-2-dead-letter) tracked in the
//! chaos-engineer mini-review finding g3-ce-5) is pending the `tracing`
//! dep landing on this crate.

use std::sync::{Arc, Mutex};

use crate::mutex_ext::MutexExt;

use benten_core::{Cid, Edge, Node};
use redb::{Durability, ReadableMultimapTable, ReadableTable};

use crate::GraphError;
use crate::WriteAuthority;
use crate::indexes::{LABEL_INDEX_TABLE, PROP_INDEX_TABLE, property_index_key, value_index_bytes};
use crate::redb_backend::{NODES_TABLE, next_prefix};
use crate::store::{
    ChangeEvent, ChangeKind, ChangeSubscriber, EDGE_SRC_PREFIX, EDGE_TGT_PREFIX, edge_key,
    edge_src_index_key, edge_src_index_prefix, edge_tgt_index_key, edge_tgt_index_prefix, node_key,
};

/// A pending write inside the transaction's batch. Mirrors `benten_caps::PendingOp`
/// but sits in `benten-graph` so the transaction module doesn't drag a
/// capability dependency in. The two shapes are translated by the engine-
/// layer bridge (`benten-engine::transaction`) that owns the capability
/// hook firing.
#[derive(Debug, Clone)]
pub enum PendingOp {
    /// Node put. Carries the full [`Node`] so the emitted [`ChangeEvent`]
    /// surfaces property data (required by the IVM views that key on
    /// `createdAt`, `grantee`, `subscribes_to`, etc.). `labels` is derived
    /// from `node.labels`.
    ///
    /// Attribution triple (`actor_cid`, `handler_cid`, `capability_grant_cid`)
    /// is carried through to the emitted `ChangeEvent` so audit consumers
    /// can trace every write back to the actor + handler that issued it.
    /// Populated by the engine's `PrimitiveHost::put_node` replay path;
    /// direct `Transaction::put_node` callers leave all three `None`.
    PutNode {
        /// The CID of the put node.
        cid: Cid,
        /// The Node being written. `labels` and all properties are
        /// reachable through `node.labels` / `node.properties`.
        node: Node,
        /// Actor CID — the principal that initiated the write.
        actor_cid: Option<Cid>,
        /// Handler CID — the subgraph whose WRITE primitive produced this.
        handler_cid: Option<Cid>,
        /// Capability-grant CID — the grant authorizing the write.
        capability_grant_cid: Option<Cid>,
    },
    /// Edge put. Carries source, target, and label so edge-driven IVM views
    /// (governance inheritance, version current) see endpoints directly on
    /// the emitted [`ChangeEvent`].
    PutEdge {
        /// The CID of the put edge.
        cid: Cid,
        /// Edge source endpoint.
        source: Cid,
        /// Edge target endpoint.
        target: Cid,
        /// Edge label (wrapped single-element when translated to a
        /// [`ChangeEvent`] so the same `labels` contract holds node- and
        /// edge-side).
        label: String,
    },
    /// Node delete by CID. Carries the deleted Node (captured by
    /// read-before-delete inside the same redb txn) so label-filtered
    /// subscribers see deletes and property-driven IVM views can unseat
    /// their derived state. `node` is `None` on an idempotent-delete miss.
    DeleteNode {
        /// The target Node CID.
        cid: Cid,
        /// Labels of the Node that was deleted, or empty when the delete
        /// targeted an already-absent CID (idempotent miss).
        labels: Vec<String>,
        /// The pre-delete Node, if any. Carried so subscribers can inspect
        /// the Node's property body at the moment of deletion.
        node: Option<Node>,
    },
    /// Edge delete by CID. Carries source/target/label captured via
    /// read-before-delete for the same reason `DeleteNode` carries the
    /// Node. Fields are `None` on an idempotent-delete miss.
    DeleteEdge {
        /// The target Edge CID.
        cid: Cid,
        /// Label of the deleted Edge, or `None` when the delete targeted an
        /// already-absent CID (idempotent miss).
        label: Option<String>,
        /// Source endpoint, captured via read-before-delete.
        source: Option<Cid>,
        /// Target endpoint, captured via read-before-delete.
        target: Option<Cid>,
    },
}

impl PendingOp {
    /// Translate this op into a [`ChangeEvent`] stamped with `tx_id`. Deletes
    /// emit [`ChangeKind::Deleted`]; puts emit [`ChangeKind::Created`] (the
    /// Phase-1 content-addressed invariant — every logical "update" is a
    /// new CID, hence always a `Created` event).
    ///
    /// # Phase-2 reservation
    ///
    /// `ChangeKind::Updated` is reserved for the Phase-2 anchor CURRENT
    /// pointer path and similar non-content-addressed identities. A Phase-2
    /// implementer adding a `PendingOp::UpdateAnchor` variant must emit
    /// `ChangeKind::Updated` rather than collapsing into `Created` — IVM
    /// views downstream treat the two as semantically distinct.
    pub(crate) fn to_change_event(&self, tx_id: u64) -> ChangeEvent {
        match self {
            PendingOp::PutNode {
                cid,
                node,
                actor_cid,
                handler_cid,
                capability_grant_cid,
            } => ChangeEvent {
                cid: *cid,
                labels: node.labels.clone(),
                kind: ChangeKind::Created,
                tx_id,
                actor_cid: *actor_cid,
                handler_cid: *handler_cid,
                capability_grant_cid: *capability_grant_cid,
                node: Some(node.clone()),
                edge_endpoints: None,
            },
            PendingOp::PutEdge {
                cid,
                source,
                target,
                label,
            } => ChangeEvent {
                cid: *cid,
                labels: vec![label.clone()],
                kind: ChangeKind::EdgeCreated,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
                node: None,
                edge_endpoints: Some((*source, *target, label.clone())),
            },
            PendingOp::DeleteNode { cid, labels, node } => ChangeEvent {
                cid: *cid,
                labels: labels.clone(),
                kind: ChangeKind::Deleted,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
                node: node.clone(),
                edge_endpoints: None,
            },
            PendingOp::DeleteEdge {
                cid,
                label,
                source,
                target,
            } => ChangeEvent {
                cid: *cid,
                labels: label.clone().into_iter().collect(),
                kind: ChangeKind::EdgeDeleted,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
                node: None,
                edge_endpoints: match (source, target, label) {
                    (Some(s), Some(t), Some(l)) => Some((*s, *t, l.clone())),
                    _ => None,
                },
            },
        }
    }
}

/// RAII guard that clears the backend's in-transaction flag on drop, even
/// if the closure panicked. Paired with the `Mutex<bool>` inside
/// `RedbBackend` so nested / concurrent attempts at `.transaction()` fail
/// fast with [`GraphError::NestedTransactionNotSupported`] instead of
/// deadlocking inside redb's single-writer lock.
pub(crate) struct TxGuard {
    flag: Arc<Mutex<bool>>,
}

impl TxGuard {
    pub(crate) fn try_acquire(flag: Arc<Mutex<bool>>) -> Result<Self, GraphError> {
        let mut guard = flag.lock_recover();
        if *guard {
            return Err(GraphError::NestedTransactionNotSupported {});
        }
        *guard = true;
        drop(guard);
        Ok(Self { flag })
    }
}

impl Drop for TxGuard {
    fn drop(&mut self) {
        let mut guard = self.flag.lock_recover();
        *guard = false;
    }
}

/// A write transaction handle, passed into the `transaction` closure. All
/// writes are atomic at commit.
///
/// The inner `redb::WriteTransaction` is `Option`-wrapped so the commit path
/// can `take()` it and consume it (redb's `commit()` takes `self` by value).
/// A successful commit leaves `inner: None`; a dropped/aborted transaction
/// leaves `inner: Some(_)` which the redb `Drop` implementation aborts.
pub struct Transaction<'a> {
    pub(crate) inner: Option<redb::WriteTransaction>,
    pub(crate) pending: Vec<PendingOp>,
    /// Phantom tying the transaction to the backend's lifetime.
    _phantom: core::marker::PhantomData<&'a ()>,
    /// Phase 2a G2-A / G11-A: authority under which the transaction runs.
    /// Drives two decisions: (a) the redb durability tier selected at txn
    /// construction via [`Self::durability_for_authority`]; (b) the
    /// system-zone label guard in [`Self::put_node`] / [`Self::put_edge`]
    /// (engine-privileged / sync-replica contexts bypass the `"system:"`
    /// prefix guard; user contexts enforce it).
    ///
    /// Today every caller enters via [`crate::RedbBackend::transaction`]
    /// with the default [`WriteAuthority::User`]; the privileged-entry-point
    /// that flips this to [`WriteAuthority::EnginePrivileged`] is G7's
    /// grant-capability / create-view machinery.
    pub(crate) authority: WriteAuthority,
}

impl<'a> Transaction<'a> {
    /// Construct a `Transaction` pinned to an explicit [`WriteAuthority`] and
    /// the redb durability derived from it by the caller. Callers at
    /// `RedbBackend::transaction` use [`Self::durability_for_authority`] to
    /// select the durability tier, then pass it here so the inner redb txn
    /// and the authority field stay in sync.
    pub(crate) fn new_with_authority(
        inner: redb::WriteTransaction,
        durability: Durability,
        authority: WriteAuthority,
    ) -> Result<Self, GraphError> {
        let mut inner = inner;
        inner
            .set_durability(durability)
            .map_err(|e| GraphError::Redb(e.to_string()))?;
        Ok(Self {
            inner: Some(inner),
            pending: Vec::new(),
            _phantom: core::marker::PhantomData,
            authority,
        })
    }

    /// True when the transaction runs under a privileged [`WriteAuthority`]
    /// (EnginePrivileged or SyncReplica). Used by the system-zone label
    /// guard on Node / Edge puts. Derived from [`Self::authority`] rather
    /// than stored separately so the two can never drift.
    #[must_use]
    pub(crate) fn is_privileged(&self) -> bool {
        !matches!(self.authority, WriteAuthority::User)
    }

    /// Map a [`WriteAuthority`] onto the redb durability tier G2-A
    /// specifies: `EnginePrivileged → Immediate`, `User → configured`,
    /// `SyncReplica → None`. The `configured` argument is the backend's
    /// configured redb durability.
    ///
    /// Consumed by the G7 privileged-entry-point work and by G5-A's
    /// `transaction_with_authority` variant when those land; kept on this
    /// module so the per-write-class tier contract lives next to the
    /// `Transaction` that enforces it.
    #[must_use]
    pub(crate) fn durability_for_authority(
        authority: &WriteAuthority,
        configured: Durability,
    ) -> Durability {
        match authority {
            WriteAuthority::EnginePrivileged => Durability::Immediate,
            WriteAuthority::User => configured,
            WriteAuthority::SyncReplica { .. } => Durability::None,
        }
    }

    /// Put a Node inside the transaction. Maintains the label and
    /// property-value indexes in the same write transaction.
    ///
    /// Also enforces the R1 SC1 system-zone stopgap: an unprivileged
    /// transaction rejects any Node whose label list contains a
    /// `"system:"`-prefixed label.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone write.
    /// - [`GraphError::Core`] on Node serialization / CID failure.
    /// - [`GraphError::Redb`] on any redb I/O failure.
    pub fn put_node(&mut self, node: &Node) -> Result<Cid, GraphError> {
        self.put_node_with_attribution(node, None, None, None, None)
    }

    /// Put a Node, stamping the emitted [`ChangeEvent`] with the supplied
    /// attribution triple and (optionally) a pre-computed CID.
    ///
    /// The CID argument exists so replay paths that already hashed the Node
    /// (e.g. `PrimitiveHost::put_node`'s projected-CID contract) can skip the
    /// double hash. If `None`, the CID is computed here.
    ///
    /// The three attribution fields surface on the emitted `ChangeEvent` and
    /// on downstream `PendingOp` records. See [`PendingOp::PutNode`].
    ///
    /// # Errors
    /// Same as [`Self::put_node`].
    pub fn put_node_with_attribution(
        &mut self,
        node: &Node,
        precomputed_cid: Option<Cid>,
        actor_cid: Option<Cid>,
        handler_cid: Option<Cid>,
        capability_grant_cid: Option<Cid>,
    ) -> Result<Cid, GraphError> {
        if !self.is_privileged() {
            for label in &node.labels {
                if label.starts_with("system:") {
                    return Err(GraphError::SystemZoneWrite {
                        label: label.clone(),
                    });
                }
            }
        }
        // r6-perf-3: the caller (PrimitiveHost replay) already ran
        // `node.cid()` when buffering the op, so accept a precomputed CID
        // and skip the redundant BLAKE3+DAG-CBOR hash here.
        let cid = match precomputed_cid {
            Some(c) => c,
            None => node.cid()?,
        };
        let bytes = node.canonical_bytes()?;
        let n_key = node_key(&cid);
        let txn = self
            .inner
            .as_mut()
            .ok_or_else(|| GraphError::Redb("put_node after commit/abort".into()))?;
        {
            let mut nodes = txn.open_table(NODES_TABLE)?;
            nodes.insert(n_key.as_slice(), bytes.as_slice())?;
        }
        {
            let mut label_idx = txn.open_multimap_table(LABEL_INDEX_TABLE)?;
            for label in &node.labels {
                label_idx.insert(label.as_bytes(), cid.as_bytes().as_slice())?;
            }
        }
        {
            let mut prop_idx = txn.open_multimap_table(PROP_INDEX_TABLE)?;
            for label in &node.labels {
                for (prop_name, value) in &node.properties {
                    let vbytes = value_index_bytes(value)?;
                    let key = property_index_key(label, prop_name, &vbytes);
                    prop_idx.insert(key.as_slice(), cid.as_bytes().as_slice())?;
                }
            }
        }
        self.pending.push(PendingOp::PutNode {
            cid,
            node: node.clone(),
            actor_cid,
            handler_cid,
            capability_grant_cid,
        });
        Ok(cid)
    }

    /// Put an Edge inside the transaction. Also writes the source and target
    /// index entries so `edges_from` / `edges_to` see the edge after commit.
    ///
    /// Enforces the R1 SC1 system-zone stopgap for edges: an unprivileged
    /// transaction rejects any Edge whose label begins with `"system:"`.
    /// Edges in the system-zone label namespace are the obvious smuggling
    /// vector for capability-grant forgery (an edge labeled `"system:Grant"`
    /// connecting an attacker's principal to a privileged capability), so
    /// the stopgap extends to them even though the R1 SC1 text named only
    /// Node labels explicitly.
    ///
    /// # Errors
    /// - [`GraphError::SystemZoneWrite`] on an unprivileged system-zone edge.
    /// - Encoding or redb failures.
    pub fn put_edge(&mut self, edge: &Edge) -> Result<Cid, GraphError> {
        if !self.is_privileged() && edge.label.starts_with("system:") {
            return Err(GraphError::SystemZoneWrite {
                label: edge.label.clone(),
            });
        }
        let cid = edge.cid()?;
        let bytes = edge.canonical_bytes()?;
        let txn = self
            .inner
            .as_mut()
            .ok_or_else(|| GraphError::Redb("put_edge after commit/abort".into()))?;
        {
            let mut table = txn.open_table(NODES_TABLE)?;
            table.insert(edge_key(&cid).as_slice(), bytes.as_slice())?;
            table.insert(edge_src_index_key(&edge.source, &cid).as_slice(), &[][..])?;
            table.insert(edge_tgt_index_key(&edge.target, &cid).as_slice(), &[][..])?;
        }
        self.pending.push(PendingOp::PutEdge {
            cid,
            source: edge.source,
            target: edge.target,
            label: edge.label.clone(),
        });
        Ok(cid)
    }

    /// Delete a Node by CID, and remove it from the label and property-value
    /// indexes in the same write transaction. Idempotent — deleting an absent
    /// CID is not an error.
    ///
    /// Captures the Node's labels via a read-before-delete inside the same
    /// redb txn so the emitted `ChangeEvent` carries the full label set.
    /// Label-filtered subscribers (IVM views, CDC consumers) therefore see
    /// delete events on the same routing API as puts.
    ///
    /// ## Cascade edge delete (r6b-ivm-1)
    ///
    /// Every Edge whose `source` or `target` is `cid` is deleted in the
    /// same transaction **before** the Node body itself, and each cascaded
    /// edge removal emits a [`PendingOp::DeleteEdge`] so downstream IVM
    /// views see a `ChangeKind::EdgeDeleted` event per dangling edge. The
    /// prior implementation dropped the Node alone, leaving the edges
    /// pointing at a non-existent CID — the prototype bug the R5 MUST
    /// clause explicitly forbids regressing, and the one that corrupts the
    /// governance-inheritance / version-chain views on cascade.
    ///
    /// Edges whose source AND target are both `cid` (self-loops) are
    /// deleted exactly once — a `BTreeSet` dedupes the union of the two
    /// index scans.
    ///
    /// # Errors
    /// Propagates decode or redb failures. Previously the decode error was
    /// silently `.ok()`-dropped; that concealed on-disk corruption (a
    /// partially-readable body would get removed from the `n:` table but
    /// leave dangling label / property index entries behind). The decode
    /// failure now propagates so callers can surface it.
    pub fn delete_node(&mut self, cid: &Cid) -> Result<Vec<String>, GraphError> {
        // Read the existing body (if any) inside the same txn so the index
        // removals target the right keys. The content-addressed invariant
        // makes this race-free: a concurrent put of the same CID writes
        // identical label/property bytes, so the index-key set can't drift.
        //
        // Returns the captured labels so the engine-layer caller can thread
        // them into `benten_caps::PendingOp::DeleteNode` for capability-
        // policy scope derivation (r6-sec-8). An idempotent-miss delete
        // returns an empty vec.

        // r6b-ivm-1 — cascade edge delete. Collect every Edge CID whose
        // source or target is this Node, then call `delete_edge` on each
        // (within the same txn) so the index + body removals stay atomic
        // with the Node delete. Must run BEFORE the Node body removal so a
        // concurrent read inside the same txn can still decode the Node if
        // needed (edge delete doesn't touch the Node body, but the ordering
        // matches the "logical delete order" a cascade implies).
        let cascaded_edge_cids = self.collect_edges_referencing(cid)?;
        for edge_cid in &cascaded_edge_cids {
            // `delete_edge` is idempotent; if two scans both picked it up
            // (self-loop on the same node → appears in both es: and et:
            // ranges) the BTreeSet above already deduped, but even a stray
            // double-call wouldn't error.
            self.delete_edge(edge_cid)?;
        }

        let n_key = node_key(cid);
        let txn = self
            .inner
            .as_mut()
            .ok_or_else(|| GraphError::Redb("delete_node after commit/abort".into()))?;
        let existing: Option<Node> = {
            let table = txn.open_table(NODES_TABLE)?;
            match table.get(n_key.as_slice())? {
                Some(v) => {
                    let bytes: &[u8] = v.value();
                    Some(
                        serde_ipld_dagcbor::from_slice::<Node>(bytes)
                            .map_err(|e| GraphError::Decode(format!("delete_node decode: {e}")))?,
                    )
                }
                None => None,
            }
        };
        {
            let mut nodes = txn.open_table(NODES_TABLE)?;
            nodes.remove(n_key.as_slice())?;
        }
        let labels: Vec<String> = existing
            .as_ref()
            .map(|n| n.labels.clone())
            .unwrap_or_default();
        if let Some(node) = existing.as_ref() {
            {
                let mut label_idx = txn.open_multimap_table(LABEL_INDEX_TABLE)?;
                for label in &node.labels {
                    label_idx.remove(label.as_bytes(), cid.as_bytes().as_slice())?;
                }
            }
            {
                let mut prop_idx = txn.open_multimap_table(PROP_INDEX_TABLE)?;
                for label in &node.labels {
                    for (prop_name, value) in &node.properties {
                        let vbytes = value_index_bytes(value)?;
                        let key = property_index_key(label, prop_name, &vbytes);
                        prop_idx.remove(key.as_slice(), cid.as_bytes().as_slice())?;
                    }
                }
            }
        }
        self.pending.push(PendingOp::DeleteNode {
            cid: *cid,
            labels: labels.clone(),
            node: existing,
        });
        Ok(labels)
    }

    /// Collect every Edge CID that references `node_cid` as source or
    /// target, scanning the `es:` and `et:` prefix indexes inside the
    /// current write transaction. Order is ascending by CID (BTreeSet
    /// iteration order); deduplicated across the two scans so a self-loop
    /// only surfaces once.
    ///
    /// Uses the same `NODES_TABLE` the edge put/delete paths use for index
    /// storage — a single range scan per prefix.
    fn collect_edges_referencing(
        &mut self,
        node_cid: &Cid,
    ) -> Result<std::collections::BTreeSet<Cid>, GraphError> {
        let src_prefix = edge_src_index_prefix(node_cid);
        let tgt_prefix = edge_tgt_index_prefix(node_cid);
        let txn = self.inner.as_mut().ok_or_else(|| {
            GraphError::Redb("collect_edges_referencing after commit/abort".into())
        })?;
        let mut out: std::collections::BTreeSet<Cid> = std::collections::BTreeSet::new();
        let table = txn.open_table(NODES_TABLE)?;

        // Scan the `es:` index for edges whose source is node_cid.
        scan_edge_index(
            &table,
            &src_prefix,
            EDGE_SRC_PREFIX.len(),
            node_cid,
            &mut out,
        )?;
        // Scan the `et:` index for edges whose target is node_cid.
        scan_edge_index(
            &table,
            &tgt_prefix,
            EDGE_TGT_PREFIX.len(),
            node_cid,
            &mut out,
        )?;
        Ok(out)
    }

    /// Delete an Edge by CID. Idempotent. Captures the Edge's label
    /// via read-before-delete so the emitted `ChangeEvent` carries the
    /// routing info.
    ///
    /// Returns the captured label (or `None` when the delete targeted an
    /// already-absent CID) so the engine-layer caller can thread it into
    /// `benten_caps::PendingOp::DeleteEdge` for capability-policy scope
    /// derivation (r6-sec-8).
    ///
    /// # Errors
    /// Propagates decode or redb failures.
    pub fn delete_edge(&mut self, cid: &Cid) -> Result<Option<String>, GraphError> {
        let txn = self
            .inner
            .as_mut()
            .ok_or_else(|| GraphError::Redb("delete_edge after commit/abort".into()))?;
        // Read the edge to compute its index keys. Decode errors now
        // propagate rather than silently `.ok()`-dropping — matches the
        // delete_node rationale above.
        let edge: Option<Edge> = {
            let table = txn.open_table(NODES_TABLE)?;
            match table.get(edge_key(cid).as_slice())? {
                Some(v) => {
                    let bytes: &[u8] = v.value();
                    Some(
                        serde_ipld_dagcbor::from_slice::<Edge>(bytes)
                            .map_err(|e| GraphError::Decode(format!("delete_edge decode: {e}")))?,
                    )
                }
                None => None,
            }
        };
        let label = edge.as_ref().map(|e| e.label.clone());
        let source = edge.as_ref().map(|e| e.source);
        let target = edge.as_ref().map(|e| e.target);
        {
            let mut table = txn.open_table(NODES_TABLE)?;
            if let Some(e) = &edge {
                table.remove(edge_src_index_key(&e.source, cid).as_slice())?;
                table.remove(edge_tgt_index_key(&e.target, cid).as_slice())?;
            }
            table.remove(edge_key(cid).as_slice())?;
        }
        self.pending.push(PendingOp::DeleteEdge {
            cid: *cid,
            label: label.clone(),
            source,
            target,
        });
        Ok(label)
    }

    /// Open a nested transaction. Phase 1 always rejects with
    /// [`GraphError::NestedTransactionNotSupported`].
    ///
    /// # Errors
    /// Always returns [`GraphError::NestedTransactionNotSupported`].
    pub fn transaction<F, R>(&mut self, _f: F) -> Result<R, GraphError>
    where
        F: FnOnce(&mut Transaction<'_>) -> Result<R, GraphError>,
    {
        Err(GraphError::NestedTransactionNotSupported {})
    }

    /// Consume the transaction and commit the underlying redb write txn.
    pub(crate) fn commit(mut self) -> Result<Vec<PendingOp>, GraphError> {
        if let Some(inner) = self.inner.take() {
            inner.commit()?;
        }
        Ok(std::mem::take(&mut self.pending))
    }
}

/// Bounded range-scan over the `NODES_TABLE` for an `es:`/`et:` index
/// prefix, extracting the edge CID suffix from each matching key and
/// inserting it into `out`. Used by cascade-delete to discover every edge
/// whose endpoint is the Node being deleted.
///
/// `prefix_len` is the length of the index-prefix header (`es:` / `et:`
/// bytes) PLUS the source/target CID the prefix encodes — everything after
/// that offset is the edge's own CID. A malformed entry (key shorter than
/// the expected prefix, or an un-parseable CID suffix) is skipped silently;
/// on-disk corruption at the index level manifests as a missed cascade
/// rather than an aborted delete, which is the same degradation model the
/// rest of the scan paths use.
fn scan_edge_index(
    table: &redb::Table<'_, &[u8], &[u8]>,
    prefix: &[u8],
    prefix_len: usize,
    endpoint_cid: &Cid,
    out: &mut std::collections::BTreeSet<Cid>,
) -> Result<(), GraphError> {
    let full_header_len = prefix_len + endpoint_cid.as_bytes().len();
    let next = next_prefix(prefix);
    let iter = match next.as_deref() {
        Some(upper) => table.range::<&[u8]>(prefix..upper)?,
        None => table.range::<&[u8]>(prefix..)?,
    };
    for item in iter {
        let (k, _v) = item?;
        let key = k.value();
        let Some(edge_cid_bytes) = key.get(full_header_len..) else {
            continue;
        };
        let Ok(edge_cid) = Cid::from_bytes(edge_cid_bytes) else {
            continue;
        };
        out.insert(edge_cid);
    }
    Ok(())
}

/// Fan a change-event batch out to every registered subscriber. Panics in
/// subscriber callbacks are caught and discarded so a single misbehaving
/// subscriber cannot poison the commit path. Emission happens after the
/// redb commit has already returned success.
///
/// TODO(phase-2-dead-letter): a permanently-broken subscriber drifts invisibly today —
/// Phase 2 will land a dead-letter counter alongside a `tracing::warn!`
/// once the tracing dep arrives on this crate (mini-review g3-ce-5).
#[allow(
    clippy::print_stderr,
    reason = "operator-visible warning on subscriber panic; benten-graph has no tracing dep"
)]
pub(crate) fn fan_out(subscribers: &[Arc<dyn ChangeSubscriber>], ops: &[PendingOp], tx_id: u64) {
    // r6-perf-4: build each ChangeEvent once, not once per subscriber.
    // Previously this nested `for sub in subs { for op in ops }` loop
    // rebuilt the event S times (O(N·M) event constructions); invert to
    // O(M) construction + O(N·M) dispatch.
    let events: Vec<ChangeEvent> = ops.iter().map(|op| op.to_change_event(tx_id)).collect();
    for sub in subscribers {
        for event in &events {
            let sub_clone = Arc::clone(sub);
            let event_clone = event.clone();
            // Ignore panics from individual subscribers — they must not
            // take down the commit thread. Phase 2 will replace the stderr
            // breadcrumb with tracing::warn! when tracing lands.
            if let Err(_panic_payload) =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                    sub_clone.on_change(&event_clone);
                }))
            {
                eprintln!(
                    "benten-graph: change subscriber panicked while processing \
                     tx_id={tx_id}; event discarded (Phase-2 will add a \
                     dead-letter counter)"
                );
            }
        }
    }
}

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

use std::sync::{Arc, Mutex};

use benten_core::{Cid, Edge, Node};
use redb::{Durability, ReadableMultimapTable, ReadableTable};

use crate::GraphError;
use crate::indexes::{LABEL_INDEX_TABLE, PROP_INDEX_TABLE, property_index_key, value_index_bytes};
use crate::redb_backend::NODES_TABLE;
use crate::store::{
    ChangeEvent, ChangeKind, ChangeSubscriber, EDGE_SRC_PREFIX, EDGE_TGT_PREFIX, edge_key,
    edge_src_index_key, edge_tgt_index_key, node_key,
};

/// A pending write inside the transaction's batch. Mirrors `benten_caps::PendingOp`
/// but sits in `benten-graph` so the transaction module doesn't drag a
/// capability dependency in. The two shapes are translated by the engine-
/// layer bridge (`benten-engine::transaction`) that owns the capability
/// hook firing.
#[derive(Debug, Clone)]
pub enum PendingOp {
    /// Node put.
    PutNode {
        /// The CID of the put node.
        cid: Cid,
        /// Full label set on the node.
        labels: Vec<String>,
    },
    /// Edge put.
    PutEdge {
        /// The CID of the put edge.
        cid: Cid,
        /// Edge label.
        label: String,
    },
    /// Node delete by CID.
    DeleteNode {
        /// The target Node CID.
        cid: Cid,
    },
    /// Edge delete by CID.
    DeleteEdge {
        /// The target Edge CID.
        cid: Cid,
    },
}

impl PendingOp {
    /// Translate this op into a [`ChangeEvent`] stamped with `tx_id`. Deletes
    /// emit [`ChangeKind::Deleted`]; puts emit [`ChangeKind::Created`] (the
    /// Phase-1 content-addressed invariant — every logical "update" is a
    /// new CID, hence always a `Created` event).
    pub(crate) fn to_change_event(&self, tx_id: u64) -> ChangeEvent {
        match self {
            PendingOp::PutNode { cid, labels } => ChangeEvent {
                cid: cid.clone(),
                label: labels.first().cloned().unwrap_or_default(),
                kind: ChangeKind::Created,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
            },
            PendingOp::PutEdge { cid, label } => ChangeEvent {
                cid: cid.clone(),
                label: label.clone(),
                kind: ChangeKind::Created,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
            },
            PendingOp::DeleteNode { cid } | PendingOp::DeleteEdge { cid } => ChangeEvent {
                cid: cid.clone(),
                label: String::new(),
                kind: ChangeKind::Deleted,
                tx_id,
                actor_cid: None,
                handler_cid: None,
                capability_grant_cid: None,
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
        let mut guard = flag.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut guard = self.flag.lock().unwrap_or_else(|e| e.into_inner());
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
    /// True when the calling backend is flagged as privileged (engine-API
    /// path only). Controls the system-zone label check on Node puts.
    pub(crate) is_privileged: bool,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(
        inner: redb::WriteTransaction,
        durability: Durability,
        is_privileged: bool,
    ) -> Result<Self, GraphError> {
        let mut inner = inner;
        inner
            .set_durability(durability)
            .map_err(|e| GraphError::Redb(e.to_string()))?;
        Ok(Self {
            inner: Some(inner),
            pending: Vec::new(),
            _phantom: core::marker::PhantomData,
            is_privileged,
        })
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
        if !self.is_privileged {
            for label in &node.labels {
                if label.starts_with("system:") {
                    return Err(GraphError::SystemZoneWrite {
                        label: label.clone(),
                    });
                }
            }
        }
        let cid = node.cid()?;
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
            cid: cid.clone(),
            labels: node.labels.clone(),
        });
        Ok(cid)
    }

    /// Put an Edge inside the transaction. Also writes the source and target
    /// index entries so `edges_from` / `edges_to` see the edge after commit.
    ///
    /// # Errors
    /// Propagates encoding or redb failures.
    pub fn put_edge(&mut self, edge: &Edge) -> Result<Cid, GraphError> {
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
            cid: cid.clone(),
            label: edge.label.clone(),
        });
        Ok(cid)
    }

    /// Delete a Node by CID, and remove it from the label and property-value
    /// indexes in the same write transaction. Idempotent — deleting an absent
    /// CID is not an error.
    ///
    /// # Errors
    /// Propagates decode or redb failures.
    pub fn delete_node(&mut self, cid: &Cid) -> Result<(), GraphError> {
        // Read the existing body (if any) inside the same txn so the index
        // removals target the right keys. The content-addressed invariant
        // makes this race-free: a concurrent put of the same CID writes
        // identical label/property bytes, so the index-key set can't drift.
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
                    serde_ipld_dagcbor::from_slice::<Node>(bytes)
                        .map_err(|e| GraphError::Decode(format!("delete_node decode: {e}")))
                        .ok()
                }
                None => None,
            }
        };
        {
            let mut nodes = txn.open_table(NODES_TABLE)?;
            nodes.remove(n_key.as_slice())?;
        }
        if let Some(node) = existing {
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
        self.pending
            .push(PendingOp::DeleteNode { cid: cid.clone() });
        Ok(())
    }

    /// Delete an Edge by CID. Idempotent.
    ///
    /// # Errors
    /// Propagates decode or redb failures.
    pub fn delete_edge(&mut self, cid: &Cid) -> Result<(), GraphError> {
        let txn = self
            .inner
            .as_mut()
            .ok_or_else(|| GraphError::Redb("delete_edge after commit/abort".into()))?;
        // Read the edge to compute its index keys.
        let edge: Option<Edge> = {
            let table = txn.open_table(NODES_TABLE)?;
            match table.get(edge_key(cid).as_slice())? {
                Some(v) => {
                    let bytes: &[u8] = v.value();
                    serde_ipld_dagcbor::from_slice::<Edge>(bytes)
                        .map_err(|e| GraphError::Decode(format!("delete_edge decode: {e}")))
                        .ok()
                }
                None => None,
            }
        };
        {
            let mut table = txn.open_table(NODES_TABLE)?;
            if let Some(e) = &edge {
                table.remove(edge_src_index_key(&e.source, cid).as_slice())?;
                table.remove(edge_tgt_index_key(&e.target, cid).as_slice())?;
            }
            table.remove(edge_key(cid).as_slice())?;
        }
        self.pending
            .push(PendingOp::DeleteEdge { cid: cid.clone() });
        // Suppress unused-prefix warnings — used above via helpers.
        let _ = EDGE_SRC_PREFIX;
        let _ = EDGE_TGT_PREFIX;
        Ok(())
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

/// Fan a change-event batch out to every registered subscriber. Panics in
/// subscriber callbacks are caught and discarded so a single misbehaving
/// subscriber cannot poison the commit path. Emission happens after the
/// redb commit has already returned success.
pub(crate) fn fan_out(subscribers: &[Arc<dyn ChangeSubscriber>], ops: &[PendingOp], tx_id: u64) {
    for sub in subscribers {
        for op in ops {
            let event = op.to_change_event(tx_id);
            let sub_clone = Arc::clone(sub);
            // Ignore panics from individual subscribers — they must not
            // take down the commit thread. Phase 2 revisits when the
            // tracing dep lands on this crate.
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
                sub_clone.on_change(&event);
            }));
        }
    }
}

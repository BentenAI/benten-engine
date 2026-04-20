//! Engine-level transaction handle (`EngineTransaction`) passed into
//! `Engine::transaction(|tx| ...)`.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). Wraps a lower-level
//! `benten_graph::Transaction` plus a side-channel collector for
//! `benten_caps::PendingOp`s the engine layer feeds into the capability hook
//! at commit time.

use benten_core::{Cid, Node};
use benten_graph::{GraphError, MutexExt};

use crate::error::EngineError;
use crate::outcome::NestedTx;

// ---------------------------------------------------------------------------
// Parallel transaction types
// ---------------------------------------------------------------------------
//
// TODO(phase-2-unify-tx-types): the engine currently exposes
// `EngineTransaction` (this module) while `benten_graph::Transaction` remains
// the lower-level redb-bound handle the closure actually drives. The two
// shapes coexist because `EngineTransaction` also feeds the capability hook
// via `ops_collector` — a side-channel the pure-graph `Transaction` doesn't
// need. Arch-7 flagged the redundancy for Phase-2 unification; the decision
// there will either promote `ops_collector` into `benten_graph::Transaction`
// or push the capability binding up into the engine closure wrapper. Either
// way is additive; the Phase-1 shape stays as-is for R5's frozen ABI.

/// Engine-level transaction handle (passed into `Engine::transaction`).
///
/// Wraps a lower-level `benten_graph::Transaction` plus a side-channel
/// collector for `benten_caps::PendingOp`s the engine layer feeds into the
/// capability hook at commit time.
pub struct EngineTransaction<'tx, 'coll> {
    pub(crate) inner: &'tx mut (dyn GraphTxLike + 'tx),
    pub(crate) ops_collector: &'coll std::sync::Mutex<Vec<benten_caps::PendingOp>>,
}

/// Object-safe shim over [`benten_graph::Transaction`] that elides the
/// lifetime parameter.
pub(crate) trait GraphTxLike {
    fn put_node(&mut self, node: &Node) -> Result<Cid, GraphError>;
    fn put_node_with_attribution(
        &mut self,
        node: &Node,
        precomputed_cid: Option<Cid>,
        actor_cid: Option<Cid>,
        handler_cid: Option<Cid>,
        capability_grant_cid: Option<Cid>,
    ) -> Result<Cid, GraphError>;
    /// Delete a Node and return its captured labels so the engine-layer
    /// transaction handle can thread them into
    /// `benten_caps::PendingOp::DeleteNode` for capability-policy scope
    /// derivation (r6-sec-8). An idempotent-miss delete returns an empty
    /// vec.
    fn delete_node(&mut self, cid: &Cid) -> Result<Vec<String>, GraphError>;
}

impl GraphTxLike for benten_graph::Transaction<'_> {
    fn put_node(&mut self, node: &Node) -> Result<Cid, GraphError> {
        benten_graph::Transaction::put_node(self, node)
    }

    fn put_node_with_attribution(
        &mut self,
        node: &Node,
        precomputed_cid: Option<Cid>,
        actor_cid: Option<Cid>,
        handler_cid: Option<Cid>,
        capability_grant_cid: Option<Cid>,
    ) -> Result<Cid, GraphError> {
        benten_graph::Transaction::put_node_with_attribution(
            self,
            node,
            precomputed_cid,
            actor_cid,
            handler_cid,
            capability_grant_cid,
        )
    }

    fn delete_node(&mut self, cid: &Cid) -> Result<Vec<String>, GraphError> {
        benten_graph::Transaction::delete_node(self, cid)
    }
}

impl std::fmt::Debug for EngineTransaction<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineTransaction").finish_non_exhaustive()
    }
}

impl EngineTransaction<'_, '_> {
    /// Alias for [`Self::put_node`].
    pub fn create_node(&mut self, node: &Node) -> Result<Cid, EngineError> {
        self.put_node(node)
    }

    /// Put a Node inside the transaction.
    pub fn put_node(&mut self, node: &Node) -> Result<Cid, EngineError> {
        let cid = self.inner.put_node(node).map_err(EngineError::Graph)?;
        self.ops_collector
            .lock_recover()
            .push(benten_caps::PendingOp::PutNode {
                cid: cid.clone(),
                labels: node.labels.clone(),
            });
        Ok(cid)
    }

    /// Put a Node with attribution and an optional pre-computed CID.
    ///
    /// Replay path used by `Engine::dispatch_call` — the caller already ran
    /// `node.cid()` when buffering the op, so the precomputed CID skips the
    /// double-hash (r6-perf-3). Attribution flows through to the emitted
    /// `ChangeEvent` (r6-sec-3).
    pub(crate) fn put_node_with_attribution(
        &mut self,
        node: &Node,
        precomputed_cid: Option<Cid>,
        actor_cid: Option<Cid>,
        handler_cid: Option<Cid>,
        capability_grant_cid: Option<Cid>,
    ) -> Result<Cid, EngineError> {
        let cid = self
            .inner
            .put_node_with_attribution(
                node,
                precomputed_cid,
                actor_cid,
                handler_cid,
                capability_grant_cid,
            )
            .map_err(EngineError::Graph)?;
        self.ops_collector
            .lock_recover()
            .push(benten_caps::PendingOp::PutNode {
                cid: cid.clone(),
                labels: node.labels.clone(),
            });
        Ok(cid)
    }

    /// Delete a Node by CID inside the transaction.
    ///
    /// Threads the Node's labels (captured via read-before-delete inside
    /// the lower-level redb transaction) into
    /// `benten_caps::PendingOp::DeleteNode` so the capability policy can
    /// derive the `store:<label>:write` scope for the delete. An
    /// idempotent-miss delete yields an empty `labels` vec, which the
    /// policy treats as a no-op. See r6-sec-8.
    pub fn delete_node(&mut self, cid: &Cid) -> Result<(), EngineError> {
        let labels = self.inner.delete_node(cid).map_err(EngineError::Graph)?;
        self.ops_collector
            .lock_recover()
            .push(benten_caps::PendingOp::DeleteNode {
                cid: cid.clone(),
                labels,
            });
        Ok(())
    }

    /// Open a nested transaction. Phase 1 always rejects.
    pub fn begin_nested(&mut self) -> Result<NestedTx, EngineError> {
        Err(EngineError::NestedTransactionNotSupported)
    }
}

//! Node + Edge CRUD surface for [`crate::engine::Engine`].
//!
//! Split from `engine.rs` for file-size hygiene (R6 Wave 2 follow-up). Houses
//! `create_node` / `get_node` / `update_node` / `delete_node` and the Edge
//! equivalents, plus `edges_from` / `edges_to` and the private
//! `read_denied_for_cid` helper that implements the Option-C denial posture
//! from named compromise #2. Every method is a plain `impl Engine` item —
//! Rust resolves it through the same inherent-impl set as if it still lived
//! in `engine.rs`.

use benten_caps::CapError;
use benten_core::{Cid, Edge, Node};
use benten_graph::GraphError;

use crate::engine::Engine;
use crate::error::EngineError;

impl Engine {
    // -------- CRUD surface (Node + Edge) --------

    /// Hash `node` (CIDv1 over labels + properties only), store it, and return
    /// its CID. Idempotent.
    ///
    /// The unprivileged user-API path — system-zone labels (labels starting
    /// with `"system:"`) are rejected with `E_SYSTEM_ZONE_WRITE`. Engine-
    /// internal paths (grant/revoke/create_view) bypass the check via a
    /// privileged `WriteContext`.
    ///
    /// Runs inside a transaction so ChangeEvents fan out to registered
    /// subscribers (IVM, change-stream probes) at commit.
    pub fn create_node(&self, node: &Node) -> Result<Cid, EngineError> {
        // Short-circuit the system-zone guard so the typed SystemZoneWrite
        // error surfaces directly — running inside the transaction closure
        // would rewrap it as TxAborted.
        for label in &node.labels {
            if label.starts_with("system:") {
                return Err(EngineError::Graph(GraphError::SystemZoneWrite {
                    label: label.clone(),
                }));
            }
        }
        Ok(self.backend.transaction(|tx| tx.put_node(node))?)
    }

    /// Retrieve a Node by CID. Returns `Ok(None)` on a clean miss.
    ///
    /// # Named compromise #2 (Option C, 5d-J workstream 1)
    ///
    /// When a capability policy is configured and `policy.check_read`
    /// rejects the read, the return collapses to `Ok(None)` — symmetric
    /// with a genuine backend miss. An unauthorised caller cannot
    /// distinguish denial from not-found via this API. To introspect
    /// the difference (e.g. for operator diagnostics), use
    /// [`Engine::diagnose_read`], which is gated on a separate
    /// `debug:read` capability.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, EngineError> {
        let node = self.backend.get_node(cid)?;
        let Some(node) = node else {
            return Ok(None);
        };
        // Gate on the primary label. A Node with no labels collapses to
        // an empty-label ReadContext; GrantBackedPolicy permits the
        // empty-label path (introspection reads) so this stays
        // backwards-compatible for hand-constructed Nodes.
        let label = node.labels.first().cloned().unwrap_or_default();
        if let Some(policy) = self.policy.as_deref() {
            let ctx = benten_caps::ReadContext {
                label,
                target_cid: Some(cid.clone()),
                ..Default::default()
            };
            if let Err(CapError::DeniedRead { .. }) = policy.check_read(&ctx) {
                return Ok(None);
            }
        }
        Ok(Some(node))
    }

    /// Update an existing Node. The old CID entry is deleted and the new node
    /// is stored under its own content-addressed CID. Returns the new CID.
    pub fn update_node(&self, old_cid: &Cid, new_node: &Node) -> Result<Cid, EngineError> {
        self.backend.transaction(|tx| {
            tx.delete_node(old_cid)?;
            tx.put_node(new_node)
        })?;
        Ok(new_node.cid()?)
    }

    /// Delete a Node by CID.
    pub fn delete_node(&self, cid: &Cid) -> Result<(), EngineError> {
        self.backend.transaction(|tx| tx.delete_node(cid))?;
        Ok(())
    }

    /// Create an Edge between two Nodes with the given label, returning the
    /// Edge's content-addressed CID.
    pub fn create_edge(&self, source: &Cid, target: &Cid, label: &str) -> Result<Cid, EngineError> {
        let edge = Edge::new(source.clone(), target.clone(), label.to_string(), None);
        Ok(self.backend.put_edge(&edge)?)
    }

    /// Retrieve an Edge by CID. Returns `Ok(None)` on a clean miss.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, EngineError> {
        Ok(self.backend.get_edge(cid)?)
    }

    /// Delete an Edge by CID.
    pub fn delete_edge(&self, cid: &Cid) -> Result<(), EngineError> {
        self.backend.transaction(|tx| tx.delete_edge(cid))?;
        Ok(())
    }

    /// Return every Edge whose `source == cid`.
    ///
    /// Option C applies: when the policy's `check_read` denies a read on
    /// the source Node, the returned Vec is empty (symmetric with a
    /// source CID that has no outgoing edges). See [`Engine::get_node`]
    /// for the full semantics.
    pub fn edges_from(&self, cid: &Cid) -> Result<Vec<Edge>, EngineError> {
        if self.read_denied_for_cid(cid)? {
            return Ok(Vec::new());
        }
        Ok(self.backend.edges_from(cid)?)
    }

    /// Return every Edge whose `target == cid`.
    ///
    /// Option C applies: see [`Engine::edges_from`].
    pub fn edges_to(&self, cid: &Cid) -> Result<Vec<Edge>, EngineError> {
        if self.read_denied_for_cid(cid)? {
            return Ok(Vec::new());
        }
        Ok(self.backend.edges_to(cid)?)
    }

    /// Internal helper: does `policy.check_read` deny a read against the
    /// Node stored at `cid`? Looks up the Node's primary label and runs
    /// it through the policy. Returns `Ok(false)` when the backend has
    /// no Node at `cid` (no leakage signal — we fall through to the
    /// normal empty-list / None path).
    fn read_denied_for_cid(&self, cid: &Cid) -> Result<bool, EngineError> {
        let Some(policy) = self.policy.as_deref() else {
            return Ok(false);
        };
        let Some(node) = self.backend.get_node(cid)? else {
            return Ok(false);
        };
        let label = node.labels.first().cloned().unwrap_or_default();
        let ctx = benten_caps::ReadContext {
            label,
            target_cid: Some(cid.clone()),
            ..Default::default()
        };
        Ok(matches!(
            policy.check_read(&ctx),
            Err(CapError::DeniedRead { .. })
        ))
    }
}

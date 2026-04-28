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
use benten_errors::ErrorCode;

use crate::engine::Engine;
use crate::error::EngineError;

/// Helper: surface `E_BACKEND_READ_ONLY` for a given operation when the
/// engine was constructed via [`Engine::from_snapshot_blob`].
fn backend_read_only(operation: &'static str) -> EngineError {
    EngineError::Other {
        code: ErrorCode::BackendReadOnly,
        message: format!("backend is read-only: {operation} rejected (snapshot-blob engine)"),
    }
}

impl Engine {
    // -------- CRUD surface (Node + Edge) --------

    /// Hash `node` (CIDv1 over labels + properties only), store it, and return
    /// its CID. Idempotent.
    ///
    /// The unprivileged user-API path — system-zone labels (labels whose
    /// prefix appears in
    /// [`crate::system_zones::SYSTEM_ZONE_PREFIXES`]) are rejected with
    /// `E_INV_SYSTEM_ZONE` (Phase 2a G5-B-i Inv-11). The Phase-1
    /// `E_SYSTEM_ZONE_WRITE` host-layer stopgap is retired at the
    /// user-facing surface; the identical check in the storage layer
    /// (`benten-graph/src/redb_backend.rs::guard_system_zone_node`) is
    /// retained as defence-in-depth per plan §9.10.
    ///
    /// Engine-internal paths (grant/revoke/create_view) bypass Inv-11 via
    /// a privileged `WriteContext`.
    ///
    /// Runs inside a transaction so ChangeEvents fan out to registered
    /// subscribers (IVM, change-stream probes) at commit.
    pub fn create_node(&self, node: &Node) -> Result<Cid, EngineError> {
        // G10-A-wasip1 (D10-RESOLVED): snapshot-blob engines are
        // read-only — surface E_BACKEND_READ_ONLY rather than corrupting
        // the snapshot's canonical-bytes invariant.
        if self.is_read_only_snapshot() {
            return Err(backend_read_only("create_node"));
        }
        // Phase-2a Inv-11 user-facing check. Short-circuits the guard so
        // the typed `E_INV_SYSTEM_ZONE` code surfaces directly — running
        // inside the transaction closure would rewrap the storage-layer
        // `E_SYSTEM_ZONE_WRITE` (defence-in-depth) as `TxAborted`.
        for label in &node.labels {
            if crate::primitive_host::is_system_zone_label(label) {
                return Err(EngineError::Other {
                    code: ErrorCode::InvSystemZone,
                    message: format!(
                        "Inv-11: system-zone label `{label}` not writable via user API"
                    ),
                });
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
    ///
    /// # Phase-2a Inv-11 runtime probe (G5-B-i)
    ///
    /// When the resolved Node's primary label falls inside a
    /// Phase-2a system-zone prefix
    /// ([`crate::system_zones::SYSTEM_ZONE_PREFIXES`]), the read
    /// collapses to `Ok(None)` independently of the configured
    /// capability policy. Inv-11 is an engine-side invariant stricter
    /// than the pluggable cap policy — a user-facing `get_node(grant_cid)`
    /// MUST NOT return the privileged content even under the
    /// `NoAuthBackend` default. Engine-privileged code paths that need
    /// to inspect system-zone Nodes reach through
    /// `self.backend.get_node(cid)` directly.
    pub fn get_node(&self, cid: &Cid) -> Result<Option<Node>, EngineError> {
        let node = self.backend.get_node(cid)?;
        let Some(node) = node else {
            return Ok(None);
        };
        // Phase-2a Inv-11 runtime probe (code-as-graph Major #1): probe
        // the RESOLVED Node's first label — NOT a passing `Value` payload
        // — against the engine-side system-zone prefix list. Applied
        // before the cap-policy gate so the policy's verdict cannot
        // override Inv-11.
        let label = node.labels.first().cloned().unwrap_or_default();
        if crate::primitive_host::is_system_zone_label(&label) {
            return Ok(None);
        }
        // Gate on the primary label. A Node with no labels collapses to
        // an empty-label ReadContext; GrantBackedPolicy permits the
        // empty-label path (introspection reads) so this stays
        // backwards-compatible for hand-constructed Nodes.
        if let Some(policy) = self.policy.as_deref() {
            let ctx = benten_caps::ReadContext {
                label,
                target_cid: Some(*cid),
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
        if self.is_read_only_snapshot() {
            return Err(backend_read_only("update_node"));
        }
        self.backend.transaction(|tx| {
            tx.delete_node(old_cid)?;
            tx.put_node(new_node)
        })?;
        Ok(new_node.cid()?)
    }

    /// Delete a Node by CID.
    pub fn delete_node(&self, cid: &Cid) -> Result<(), EngineError> {
        if self.is_read_only_snapshot() {
            return Err(backend_read_only("delete_node"));
        }
        self.backend.transaction(|tx| tx.delete_node(cid))?;
        Ok(())
    }

    /// Create an Edge between two Nodes with the given label, returning the
    /// Edge's content-addressed CID.
    pub fn create_edge(&self, source: &Cid, target: &Cid, label: &str) -> Result<Cid, EngineError> {
        if self.is_read_only_snapshot() {
            return Err(backend_read_only("create_edge"));
        }
        let edge = Edge::new(*source, *target, label.to_string(), None);
        Ok(self.backend.put_edge(&edge)?)
    }

    /// Retrieve an Edge by CID. Returns `Ok(None)` on a clean miss.
    pub fn get_edge(&self, cid: &Cid) -> Result<Option<Edge>, EngineError> {
        Ok(self.backend.get_edge(cid)?)
    }

    /// Delete an Edge by CID.
    pub fn delete_edge(&self, cid: &Cid) -> Result<(), EngineError> {
        if self.is_read_only_snapshot() {
            return Err(backend_read_only("delete_edge"));
        }
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
            target_cid: Some(*cid),
            ..Default::default()
        };
        Ok(matches!(
            policy.check_read(&ctx),
            Err(CapError::DeniedRead { .. })
        ))
    }
}

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
    /// # #593 — engine-internal read = read-as-the-engine-user-root principal (NOT an auth bypass)
    ///
    /// Per the post-Phase-4-Foundation trust-model reframe
    /// (`DECISION-RECORD-trust-model-reframe.md` §4, RATIFIED) and
    /// CLAUDE.md baked-in commitment #18: **there is no such thing as an
    /// un-principal'd access.** This method is the engine-internal
    /// un-attributed read pathway, and "un-attributed" here means
    /// *attributed to the engine's own user-root principal* — the trust
    /// anchor — NOT the absence of a principal. Engine-internal callers
    /// (IVM incremental recompute, Atrium sync materialization, view
    /// recompute, audit) are acting AS the engine's user-root; that is a
    /// legitimate principal authorised by construction (it cannot be
    /// attenuated below root because it *is* root). META #593 frames the
    /// `get_node` / `read_node_as` split as a parallel "auth-bypass"
    /// pathway; under the unified model the correct reading is the one
    /// stated here: `get_node` is read-as-user-root, `read_node_as` is
    /// read-as-an-attenuated-principal.
    ///
    /// **The contract this implies (containment, not a per-call check):**
    /// the `read_node_as(principal, cid)` surface
    /// ([`Engine::read_node_as`]) is the ONLY pathway any
    /// external / untrusted / plugin / non-engine-root caller may use to
    /// read a Node — it threads `actor_cid: Some(principal)` so the
    /// cap policy attenuates below root. Adding a per-call permission
    /// check to *this* method would be wrong (it would regress hot
    /// paths and is semantically incorrect — the engine-internal
    /// principal IS authorised). The security property is upheld by a
    /// **containment proof**: no external/plugin call path reaches
    /// `get_node` (or the raw backend read it wraps) without going
    /// through a principal-gated seam. That containment is asserted by
    /// `tests/engine_internal_get_node_is_read_as_user_root_containment.rs`
    /// (a would-FAIL guard if a new external un-attributed caller — e.g.
    /// a napi re-export of the raw backend read — were introduced).
    /// Note this method *already* applies the Inv-11 system-zone probe
    /// and the configured `policy.check_read` gate with
    /// `actor_cid: None`; the genuinely un-gated read is the raw
    /// `self.backend.get_node(cid)`, consumed only by engine internals.
    ///
    /// The visibility of this method (`pub` vs `pub(crate)`) is a
    /// v1-API-stabilization decision tracked at
    /// `docs/future/phase-4-backlog.md §4.43` (Phase-4-Meta) and is
    /// intentionally **not** changed by the #593 re-scope — P6 is a
    /// semantic-documentation + containment-assertion change only.
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
            // Phase-3 G16-B-prime fp (consumer-audit closure of cor-1 /
            // cap-g16bp-3): thread the engine's configured device-DID-
            // attestation CID into get_node's primary read-gate
            // ReadContext so heterogeneous policies dispatch per-device
            // per D-PHASE-3-25. Default-None for legacy / non-attested
            // engines.
            let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
            let ctx = benten_caps::ReadContext {
                label,
                target_cid: Some(*cid),
                device_cid,
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
        // Phase-3 G16-B-prime fp (consumer-audit closure of cor-1 /
        // cap-g16bp-3): thread device-DID-attestation CID into
        // edge-read symmetric denial-probe ReadContext.
        let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
        let ctx = benten_caps::ReadContext {
            label,
            target_cid: Some(*cid),
            device_cid,
            ..Default::default()
        };
        Ok(matches!(
            policy.check_read(&ctx),
            Err(CapError::DeniedRead { .. })
        ))
    }
}

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
        // Refinement-audit-2026-05 D1 #1189 (Qual-1 #695 + Safe-1 #534 /
        // META #593): the user-facing default read = the canonical
        // `read_node_inner` seam with no attributed principal. Inv-11
        // probe + Option-C `DeniedRead` collapse + fail-CLOSED on every
        // other `CapError` denial variant all live there now. A Node
        // with no labels collapses to an empty-label `ReadContext`;
        // `GrantBackedPolicy` permits the empty-label introspection path
        // so this stays backwards-compatible for hand-constructed Nodes.
        self.read_node_inner(cid, None)
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
    /// Node stored at `cid`? Returns `Ok(false)` when the backend has
    /// no Node at `cid` (no leakage signal — we fall through to the
    /// normal empty-list / None path).
    ///
    /// Refinement-audit-2026-05 D1 #1189 (Pattern-F Bundle 1; closes
    /// Qual-1 #695 + Safe-1 #534 / META #593): this is now a thin
    /// composition over the canonical [`Self::read_node_inner`] seam —
    /// the prior hand-rolled `ReadContext` + `matches!(.., DeniedRead)`
    /// body was one of three near-duplicate sites that silently
    /// permitted reads on every non-`DeniedRead` `CapError` (Revoked /
    /// RevokedMidEval / future `#[non_exhaustive]` variants). The
    /// canonical path fails CLOSED. A non-`DeniedRead` denial now
    /// propagates as `Err(EngineError::Cap(..))` from `read_node_inner`
    /// rather than being collapsed to a permitted read; the `?` here
    /// surfaces it to the edge-read caller (correct — a revoked actor
    /// must not silently observe an empty edge list as if permitted).
    fn read_denied_for_cid(&self, cid: &Cid) -> Result<bool, EngineError> {
        if self.policy.as_deref().is_none() {
            return Ok(false);
        }
        // The Node-resolution + Inv-11 probe + Option-C cap collapse all
        // live in the canonical helper. `read_node_inner` returns
        // `Ok(None)` for a clean backend miss, an Inv-11 system-zone
        // reject, OR an Option-C `DeniedRead` collapse — all three are
        // "denied / absent" for edge-read symmetry purposes. A clean
        // backend miss must NOT report "denied" (no leakage signal), so
        // we additionally confirm the Node actually exists.
        let denied_or_absent = self.read_node_inner(cid, None)?.is_none();
        Ok(denied_or_absent && self.backend.get_node(cid)?.is_some())
    }

    /// Canonical fail-CLOSED capability read-gate decision.
    ///
    /// Refinement-audit-2026-05 D1 #1189 (Safe-1 #534 / META #593
    /// class-B-β auth-bypass closure-pin). The pre-fix read paths all
    /// pattern-matched ONLY `Err(CapError::DeniedRead { .. })` and let
    /// every other `check_read` `Err` fall through to a permitted read.
    /// `CapError` is `#[non_exhaustive]` with multiple denial variants
    /// (`Denied`, `Revoked`, `RevokedMidEval`, `NotImplemented`, the
    /// Phase-3 UCAN variants, plus any future addition) — so the old
    /// shape silently permitted access whenever a policy denied via any
    /// non-`DeniedRead` variant.
    ///
    /// This single canonical arm fails CLOSED:
    /// - `Ok(())` → [`ReadGate::Permitted`]
    /// - `Err(CapError::DeniedRead { .. })` → [`ReadGate::DeniedReadCollapse`]
    ///   — the Option-C posture per Phase-1 named compromise #2
    ///   (CID-existence-leak defense: a denied read is indistinguishable
    ///   from a clean miss, so callers collapse to `None` / empty).
    /// - `Err(other)` → propagated as `Err(EngineError::Cap(other))`.
    ///   Revocation, mid-eval revocation, and any future denial variant
    ///   surface a typed error rather than a silent permit.
    pub(crate) fn check_read_gate(
        &self,
        ctx: &benten_caps::ReadContext,
    ) -> Result<ReadGate, EngineError> {
        let Some(policy) = self.policy.as_deref() else {
            return Ok(ReadGate::Permitted);
        };
        match policy.check_read(ctx) {
            Ok(()) => Ok(ReadGate::Permitted),
            Err(CapError::DeniedRead { .. }) => Ok(ReadGate::DeniedReadCollapse),
            Err(other) => Err(EngineError::Cap(other)),
        }
    }

    /// Canonical Node read pathway: backend resolve → Inv-11 runtime
    /// probe → Option-C capability gate → optional principal threading.
    ///
    /// Refinement-audit-2026-05 D1 #1189 (Qual-1 #695 closure-pin). This
    /// collapses the three near-duplicate read bodies (`get_node`,
    /// `read_node_as`, `read_denied_for_cid`) into one canonical seam so
    /// the Safe-1 #534 fail-CLOSED fix lands ONCE rather than thrice,
    /// and so the `self.device_cid()` accessor is the single
    /// device-CID access pattern (closing the prior drift where
    /// `read_node_as` used `self.device_cid()` while `get_node` reached
    /// `*MutexExt::lock_recover(&self.inner.device_cid)` raw — the two
    /// are semantically identical today; the accessor is the canonical
    /// surface if it ever gains caching / refresh / observability).
    ///
    /// Returns `Ok(None)` for: a clean backend miss, an Inv-11
    /// system-zone reject (Inv-11 cannot be overridden by the policy —
    /// probed before the cap gate), OR an Option-C `DeniedRead`
    /// collapse. Returns `Err(EngineError::Cap(..))` for any non-
    /// `DeniedRead` denial (fail CLOSED). `principal` is `None` for the
    /// user-facing default read and `Some(cid)` for the Class-B-β
    /// attributed read (`read_node_as`, CLAUDE.md baked-in #18).
    pub(crate) fn read_node_inner(
        &self,
        cid: &Cid,
        principal: Option<Cid>,
    ) -> Result<Option<Node>, EngineError> {
        let Some(node) = self.backend.get_node(cid)? else {
            return Ok(None);
        };
        // Phase-2a Inv-11 runtime probe (code-as-graph Major #1): probe
        // the RESOLVED Node's first label against the engine-side
        // system-zone prefix list, BEFORE the cap-policy gate so the
        // policy's verdict cannot override Inv-11.
        let label = node.labels.first().cloned().unwrap_or_default();
        if crate::primitive_host::is_system_zone_label(&label) {
            return Ok(None);
        }
        // Phase-3 G16-B-prime fp (cor-1 / cap-g16bp-3): thread the
        // engine's configured device-DID-attestation CID (D-PHASE-3-25
        // heterogeneous-policy per-device dispatch) and, for the
        // attributed-read path, the caller's principal CID.
        let ctx = benten_caps::ReadContext {
            label,
            target_cid: Some(*cid),
            actor_cid: principal,
            device_cid: self.device_cid(),
            ..Default::default()
        };
        match self.check_read_gate(&ctx)? {
            ReadGate::Permitted => Ok(Some(node)),
            ReadGate::DeniedReadCollapse => Ok(None),
        }
    }
}

/// Outcome of the canonical fail-CLOSED capability read-gate
/// ([`Engine::check_read_gate`]). Non-`DeniedRead` denials never reach
/// this enum — they propagate as `Err(EngineError::Cap(..))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReadGate {
    /// Policy permitted the read (or no policy is configured).
    Permitted,
    /// Policy denied via `CapError::DeniedRead` — Option-C posture per
    /// named compromise #2: caller collapses to `None` / empty so a
    /// denied read is indistinguishable from a clean miss.
    DeniedReadCollapse,
}

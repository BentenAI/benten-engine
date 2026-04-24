//! The [`CapabilityPolicy`] pre-write hook trait + [`WriteContext`] +
//! [`ReadContext`].
//!
//! The policy fires at commit time (not per-WRITE) so a multi-write
//! transaction is permitted or denied atomically. See
//! `tests/check_write_called_at_commit.rs` for the contract; the actual wiring
//! into the transaction primitive lands in G3.

use benten_core::Cid;

use crate::DEFAULT_BATCH_BOUNDARY;
use crate::error::CapError;

/// Re-export of [`benten_core::WriteAuthority`]. Single canonical type
/// across benten-core, benten-graph, and benten-caps.
pub use benten_core::WriteAuthority;

/// Context handed to the capability policy at commit time.
///
/// The field set is a union of the axes the R1 triage + R2 landscape named
/// across the various test writers. Fields are deliberately public so
/// downstream policy implementors can match on them without going through
/// accessors — the trait is meant to be easy to implement from a single
/// match expression.
///
/// `TODO(phase-3)`: `actor_hint` is a `String` placeholder for the eventual
/// DID / VC identity. Phase 3 `benten-id` replaces it with a typed principal.
///
/// A pending write enqueued inside the transaction primitive's batch.
/// G3-A landed the [`WriteContext::pending_ops`] surface (R4 pass-2 residual
/// g4-uc-5) so commit-time policies can reason about the whole batch — not
/// just the "primary" op reflected in the convenience fields.
///
/// The enum is deliberately lean — policies only need the label and CID of
/// each op to route denials. Richer shapes (full Node body, property diffs)
/// are a Phase-2 concern and would require `benten-caps` to take a direct
/// dep on `benten-graph` (a layering break).
#[derive(Debug, Clone)]
pub enum PendingOp {
    /// A Node write. `labels` is the full label set of the Node being put.
    PutNode {
        /// The content-addressed CID of the Node after encoding.
        cid: Cid,
        /// Every label the Node carries.
        labels: Vec<String>,
    },
    /// An Edge write. `label` is the Edge's single label.
    PutEdge {
        /// The content-addressed CID of the Edge after encoding.
        cid: Cid,
        /// The Edge's label.
        label: String,
    },
    /// A Node deletion by CID.
    ///
    /// `labels` is the label set captured at delete time via read-before-
    /// delete (see `benten_graph::Transaction::delete_node`). The engine
    /// threads the captured labels into this variant so the capability
    /// policy can derive the same `store:<label>:write` scope it uses for
    /// the PutNode side. An empty `labels` means the delete targeted an
    /// already-absent CID (idempotent miss); the policy treats that as a
    /// no-op scope with no grant required. See r6-sec-8.
    DeleteNode {
        /// The target Node CID.
        cid: Cid,
        /// Labels of the Node being deleted (captured via read-before-
        /// delete). Empty on idempotent miss.
        labels: Vec<String>,
    },
    /// An Edge deletion by CID.
    ///
    /// `label` is the Edge's single label captured at delete time.
    /// `None` means the delete targeted an already-absent CID (idempotent
    /// miss); the policy treats that as a no-op scope. See r6-sec-8.
    DeleteEdge {
        /// The target Edge CID.
        cid: Cid,
        /// Label of the Edge being deleted (captured via read-before-
        /// delete). `None` on idempotent miss.
        label: Option<String>,
    },
}

/// Context passed to [`CapabilityPolicy::check_write`].
///
/// Carries the pending-ops batch, the actor identity (Phase-3), and a
/// privileged-flag for engine-internal writes. Backends inspect these to
/// decide whether to authorize the transaction.
#[derive(Debug, Clone, Default)]
pub struct WriteContext {
    /// Label of the Node about to be written. For multi-op batches this
    /// carries the primary label of the first op (convenience field;
    /// structured routing should use [`WriteContext::pending_ops`]).
    pub label: String,
    /// Actor CID identity (Phase 3). `None` in Phase 1; reserved so the
    /// struct shape is stable across phases.
    pub actor_cid: Option<Cid>,
    /// The capability scope the operation targets
    /// (e.g. `"store:post:write"`).
    pub scope: String,
    /// True if the caller is engine-privileged (system-zone writes arrive via
    /// the engine API only; user subgraphs never set this).
    pub is_privileged: bool,
    /// Non-Cid actor hint (a string identifier) used in test fixtures and
    /// Phase-1 in-process policies.
    pub actor_hint: Option<String>,
    /// Full pending-writes batch the transaction will commit atomically.
    /// Empty for check paths outside a transaction; G3-A populates this
    /// from the transaction primitive's pending-ops list at commit time.
    ///
    /// Closes R4 pass-2 residual `g4-uc-5`: policies can now inspect the
    /// full batch rather than just the primary op reflected by `label`.
    pub pending_ops: Vec<PendingOp>,
    /// Phase 2a G2-B / ucca-9 / arch-r1-2: authority under which the write
    /// runs. Defaults to [`WriteAuthority::User`].
    pub authority: WriteAuthority,
}

impl WriteContext {
    /// Construct a lightweight synthetic context for unit tests. Fields are
    /// stable-but-synthetic placeholders so the unit-test surface does not
    /// depend on the evaluator being wired in.
    #[must_use]
    pub fn synthetic_for_test() -> Self {
        Self {
            label: "synthetic".into(),
            actor_cid: None,
            scope: "synthetic:write".into(),
            is_privileged: false,
            actor_hint: Some("synthetic-actor".into()),
            pending_ops: Vec::new(),
            authority: WriteAuthority::User,
        }
    }

    /// Builder: set the [`WriteAuthority`] on a context. Phase 2a G2-B.
    #[must_use]
    pub fn with_authority(mut self, authority: WriteAuthority) -> Self {
        if matches!(authority, WriteAuthority::EnginePrivileged) {
            self.is_privileged = true;
        }
        self.authority = authority;
        self
    }
}

/// Context handed to the capability policy at read time.
///
/// Phase 1 ships the shape so named compromise #2 has a concrete anchor on
/// [`CapabilityPolicy::check_read`]; the default policy permits every read.
/// Phase 3 `benten-id` swaps in a typed principal and wires real read-grant
/// enforcement.
///
/// See `docs/ERROR-CATALOG.md` for [`crate::CapError::DeniedRead`].
#[derive(Debug, Clone, Default)]
pub struct ReadContext {
    /// Label of the Node (or view / anchor) the caller is trying to read.
    pub label: String,
    /// CID of the target entity when known (None if the caller is reading
    /// by label / query).
    pub target_cid: Option<Cid>,
    /// Non-Cid actor hint used by Phase-1 in-process test policies.
    pub actor_hint: Option<String>,
    /// Actor CID identity (Phase 3). Reserved.
    pub actor_cid: Option<Cid>,
}

impl ReadContext {
    /// Construct a lightweight synthetic context for unit tests.
    #[must_use]
    pub fn synthetic_for_test() -> Self {
        Self {
            label: "synthetic".into(),
            target_cid: None,
            actor_hint: Some("synthetic-actor".into()),
            actor_cid: None,
        }
    }

    /// Construct a `ReadContext` for a CID-only read (no label in scope).
    ///
    /// G11-A EVAL wave-1 (G4-A nit): the "empty-label means CID-only"
    /// convention previously lived as an unwritten rule at
    /// `benten-engine/src/primitive_host.rs:146` and a few engine
    /// call-sites that constructed `ReadContext { label: String::new(),
    /// target_cid: Some(cid), ..Default::default() }` inline. A typed
    /// constructor makes the convention explicit and gives
    /// `CapabilityPolicy::check_read` implementations a single pattern
    /// to match on.
    #[must_use]
    pub fn by_cid_only(cid: Cid) -> Self {
        Self {
            label: String::new(),
            target_cid: Some(cid),
            actor_hint: None,
            actor_cid: None,
        }
    }
}

/// The capability pre-write hook trait.
///
/// Called by the transaction primitive at commit time — not per-WRITE. A
/// multi-write subgraph is either permitted atomically or denied atomically.
///
/// Object-safe: integration tests routinely box this behind `dyn
/// CapabilityPolicy`. Keep any future extensions to this trait object-safe
/// (no `where Self: Sized` defaults that take `self` by value, no generic
/// methods without `where Self: Sized`).
pub trait CapabilityPolicy: Send + Sync {
    /// Permit or deny the pending write batch.
    ///
    /// # Errors
    ///
    /// Implementations return [`CapError`] for any denial or backend failure.
    /// The default [`crate::NoAuthBackend`] always returns `Ok(())`.
    fn check_write(&self, ctx: &WriteContext) -> Result<(), CapError>;

    /// Permit or deny an incoming read.
    ///
    /// # Named compromise #2 — `E_CAP_DENIED_READ` leaks existence
    ///
    /// A backend that chooses to DENY a read returns
    /// [`CapError::DeniedRead`], which surfaces "this CID exists but you
    /// cannot see it" — leaking existence to an unauthorized caller. Phase 3
    /// `benten-id` revisits once the identity surface lands and silent-`None`
    /// (indistinguishable from not-found) becomes safe to attribute.
    ///
    /// Phase 1 default: permit every read. Embedded and local-only
    /// deployments pay nothing; capability-scoped reads are an opt-in
    /// backend concern.
    ///
    /// # Errors
    ///
    /// Return [`CapError::DeniedRead`] to deny. Other variants
    /// ([`CapError::Revoked`], [`CapError::NotImplemented`]) route through
    /// `ON_ERROR` per the evaluator contract in
    /// `tests/ucan_stub_messages.rs`.
    fn check_read(&self, _ctx: &ReadContext) -> Result<(), CapError> {
        Ok(())
    }

    /// Maximum number of ITERATE loop bodies that may execute between
    /// capability-snapshot refreshes, as recommended by this policy.
    ///
    /// The default is [`DEFAULT_BATCH_BOUNDARY`] — the Phase 1 named
    /// compromise #1 boundary. A revocation-sensitive backend (Phase 3
    /// UCAN with a short TTL; a testing backend that wants to force a
    /// refresh every iteration) can override to tighten the bound.
    ///
    /// # Phase-1 wiring caveat
    ///
    /// The evaluator reads its batch cadence from
    /// `benten_eval::PrimitiveHost::iterate_batch_boundary`, not from the
    /// configured `CapabilityPolicy`. In Phase 1 the default engine
    /// `PrimitiveHost` implementation does NOT delegate to the policy's
    /// override, so customising this value on a bespoke
    /// `CapabilityPolicy` will not affect the evaluator's actual refresh
    /// cadence. This method is therefore a policy-level constant in
    /// Phase 1, consulted by capability-aware tooling that inspects the
    /// policy directly.
    ///
    /// TODO(phase-2-iterate-boundary-delegation): wire the engine's
    /// `PrimitiveHost::iterate_batch_boundary` to delegate to the
    /// configured `CapabilityPolicy::iterate_batch_boundary` so the
    /// policy override becomes load-bearing end-to-end.
    ///
    /// Lowering this bound increases capability-check load; raising it
    /// widens the TOCTOU window. Keep in lockstep with the named compromise
    /// prose in `.addl/phase-1/r1-triage.md` if the default is ever
    /// adjusted.
    // TODO(phase-2-wallclock-toctou): honor wall-clock bound in addition to iteration count per
    // R4b compromise #1 tightening (auditor finding g4-p2-uc-2). A
    // TRANSFORM-heavy or CALL-heavy handler at 1 iter/10sec pushes past 10
    // minutes between refreshes under iteration-count alone; the first real
    // capability backend MUST additionally enforce a wall-clock ceiling
    // (min(iteration_count, wall_clock_seconds), default ≤300s).
    fn iterate_batch_boundary(&self) -> usize {
        DEFAULT_BATCH_BOUNDARY
    }

    /// Phase 2a G9-A / P1 / §9.13 refresh-point-5: maximum wall-clock
    /// duration between capability-grant revalidations during a long-running
    /// ITERATE or CALL. Default 300s per the dual-source resolution; the
    /// evaluator's monotonic source drives the cadence, the HLC rides
    /// alongside for federation correlation.
    ///
    /// TODO(phase-2a-G9-A): wire into the evaluator's refresh path so the
    /// override becomes load-bearing end-to-end.
    fn wallclock_refresh_ceiling(&self) -> core::time::Duration {
        core::time::Duration::from_secs(300)
    }
}

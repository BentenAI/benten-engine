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
/// `TODO(G3)`: a `Vec<PendingOp>` of pending writes is deliberately NOT
/// present today — G3 will add the pending-writes list so a policy can make
/// per-write decisions at commit time. The auditor (g4-uc-5) flagged the
/// missing surface; the shape is a G3 concern because only G3 knows the
/// transaction primitive's batch structure. Pre-landing the field without
/// the wiring would freeze a shape G3 hasn't yet designed.
#[derive(Debug, Clone, Default)]
pub struct WriteContext {
    /// Label of the Node about to be written.
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
        }
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
    /// capability-snapshot refreshes.
    ///
    /// The default is [`DEFAULT_BATCH_BOUNDARY`] — the Phase 1 named
    /// compromise #1 boundary. A revocation-sensitive backend (Phase 3 UCAN
    /// with a short TTL; a testing backend that wants to force a refresh
    /// every iteration) can override to tighten the bound. The evaluator
    /// (G6) reads this value via `CapabilityPolicy::iterate_batch_boundary`
    /// and re-reads caps every N iterations.
    ///
    /// Lowering this bound increases capability-check load; raising it
    /// widens the TOCTOU window. Keep in lockstep with the named compromise
    /// prose in `.addl/phase-1/r1-triage.md` if the default is ever
    /// adjusted.
    fn iterate_batch_boundary(&self) -> usize {
        DEFAULT_BATCH_BOUNDARY
    }
}

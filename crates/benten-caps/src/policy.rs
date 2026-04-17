//! The [`CapabilityPolicy`] pre-write hook trait + [`WriteContext`].
//!
//! The policy fires at commit time (not per-WRITE) so a multi-write
//! transaction is permitted or denied atomically. See
//! `tests/check_write_called_at_commit.rs` for the contract; the actual wiring
//! into the transaction primitive lands in G3.

use benten_core::Cid;

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
#[derive(Debug, Clone, Default)]
pub struct WriteContext {
    /// Primary label of the Node about to be written (short-hand for the
    /// common-case single-label Node).
    pub label: String,
    /// Actor CID identity (Phase 3). `None` in Phase 1; reserved so the
    /// struct shape is stable across phases.
    pub actor_cid: Option<Cid>,
    /// The capability scope the operation targets
    /// (e.g. `"store:post:write"`).
    pub scope: String,
    /// Fully-resolved target label — redundant with `label`, kept for the
    /// proptest surface that names it explicitly.
    pub target_label: String,
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
            target_label: "synthetic".into(),
            is_privileged: false,
            actor_hint: Some("synthetic-actor".into()),
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
}

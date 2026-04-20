//! `PrimitiveHost` — the trait the `Evaluator` calls through to reach the
//! backend, the capability policy, the IVM subscriber, and sibling handlers.
//!
//! The evaluator itself is ignorant of `benten-graph`, `benten-caps`, and
//! `benten-ivm`. Every backend-facing operation a primitive executor performs
//! (READ / WRITE / CALL / EMIT / ITERATE batch-boundary capability re-checks)
//! routes through a `&dyn PrimitiveHost`. `benten-engine` implements this
//! trait so the evaluator can dispatch every handler uniformly.
//!
//! Closes Compromise #8: `Engine::call` no longer short-circuits the
//! evaluator for CRUD handlers; the whole dispatch walks the registered
//! Subgraph through `Evaluator::run(..., &dyn PrimitiveHost)`.

use benten_core::{Cid, Edge, Node, Value};

use crate::EvalError;

#[cfg(doc)]
use crate::Evaluator;

/// View-query shape used by `PrimitiveHost::read_view`.
///
/// Defined locally in `benten-eval` so the evaluator does not depend on
/// `benten-ivm` (thinness discipline — the evaluator is ignorant of the
/// IVM subsystem). `benten-engine`'s `impl PrimitiveHost for Engine` maps
/// this shape onto the matching `benten_ivm::ViewQuery` fields.
#[derive(Debug, Clone, Default)]
pub struct ViewQuery {
    /// Label filter.
    pub label: Option<String>,
    /// Page size.
    pub limit: Option<usize>,
    /// Page offset.
    pub offset: Option<usize>,
    /// Version-chain anchor id.
    pub anchor_id: Option<u64>,
    /// Entity CID filter.
    pub entity_cid: Option<Cid>,
    /// Event-name filter.
    pub event_name: Option<String>,
}

/// The surface the evaluator uses to reach host-managed state. Implementors
/// map each call onto their backend, capability policy, IVM subscriber, and
/// sibling-handler dispatch path.
///
/// This trait is object-safe by construction: every method uses concrete
/// types or `&dyn`-compatible shapes, and the trait has no generic methods.
/// `Send + Sync` supertraits let the host be stored behind an `Arc` or
/// referenced from parallel `ITERATE` bodies in Phase 2.
pub trait PrimitiveHost: Send + Sync {
    /// READ primitive: fetch a Node by CID. `Ok(None)` on a clean miss.
    ///
    /// # Errors
    /// Surfaces [`EvalError::Backend`] (or a more specific variant) when the
    /// host's backend rejects the read.
    fn read_node(&self, cid: &Cid) -> Result<Option<Node>, EvalError>;

    /// READ primitive (by-label branch): every CID whose Node carries the
    /// given label.
    ///
    /// # Errors
    /// See [`PrimitiveHost::read_node`].
    fn get_by_label(&self, label: &str) -> Result<Vec<Cid>, EvalError>;

    /// READ primitive (by-property branch): every CID whose Node carries the
    /// given `(label, prop, value)` tuple.
    ///
    /// # Errors
    /// See [`PrimitiveHost::read_node`].
    fn get_by_property(
        &self,
        label: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Cid>, EvalError>;

    /// WRITE primitive (create / update): persist a Node and return its CID.
    ///
    /// # Errors
    /// Surfaces [`EvalError::Backend`] on backend failure; capability denial
    /// is surfaced via [`EvalError::Capability`].
    fn put_node(&self, node: &Node) -> Result<Cid, EvalError>;

    /// WRITE primitive (edge create): persist an Edge and return its CID.
    ///
    /// # Errors
    /// See [`PrimitiveHost::put_node`].
    fn put_edge(&self, edge: &Edge) -> Result<Cid, EvalError>;

    /// WRITE primitive (delete): remove a Node by CID.
    ///
    /// # Errors
    /// See [`PrimitiveHost::put_node`].
    fn delete_node(&self, cid: &Cid) -> Result<(), EvalError>;

    /// WRITE primitive (delete): remove an Edge by CID.
    ///
    /// # Errors
    /// See [`PrimitiveHost::put_node`].
    fn delete_edge(&self, cid: &Cid) -> Result<(), EvalError>;

    /// CALL primitive: dispatch a sibling handler by id. The host is
    /// responsible for depth tracking (Invariant 8) and attenuation checking.
    ///
    /// Return: the callee's terminal output `Value`.
    ///
    /// # Errors
    /// Surfaces [`EvalError::Backend`] for engine-level failures, plus
    /// [`EvalError::Capability`] for attenuation rejections.
    fn call_handler(&self, handler_id: &str, op: &str, input: Node) -> Result<Value, EvalError>;

    /// EMIT primitive: fire-and-forget change notification. Intentionally
    /// returns `()` — EMIT never fails the evaluator.
    fn emit_event(&self, name: &str, payload: Value);

    /// Capability re-check hook used by ITERATE and CALL.
    ///
    /// Called by ITERATE at iteration 0 and every
    /// [`PrimitiveHost::iterate_batch_boundary`] iterations (named
    /// compromise #1), and by CALL at entry before sibling-handler
    /// dispatch. These are the TOCTOU-refresh points the evaluator
    /// owns.
    ///
    /// READ / WRITE primitive enforcement does NOT route through this
    /// method — their capability gate runs at the transaction-commit
    /// boundary inside `benten-engine`, which calls
    /// `CapabilityPolicy::check_write` / `check_read` against the full
    /// pending-ops batch. This method only exists for the evaluator-
    /// scoped refresh points where there is no transaction to anchor
    /// the check on.
    ///
    /// # Errors
    /// Returns [`EvalError::Capability`] when the configured policy denies.
    fn check_capability(&self, required: &str, target: Option<&Cid>) -> Result<(), EvalError>;

    /// Read an IVM view through the subscriber.
    ///
    /// # Errors
    /// Surfaces [`EvalError::Backend`] on subscriber failure (unknown view,
    /// stale read in strict mode, etc.).
    fn read_view(&self, view_id: &str, query: &ViewQuery) -> Result<Value, EvalError>;

    /// Capability-re-check cadence for ITERATE bodies (Phase-1 named
    /// compromise #1). Defaults to 100; engines may override.
    fn iterate_batch_boundary(&self) -> usize {
        100
    }

    /// Capability check for a READ primitive against `target_cid` carrying
    /// `label`. 5d-J workstream 1 (Option C) — the engine-layer public
    /// read surface (`get_node`, `read_view`, `edges_from`, `edges_to`)
    /// consults this method; a `CapError::DeniedRead` is mapped to
    /// `Ok(None)` at the public boundary so an unauthorised reader
    /// cannot distinguish a denied CID from a missing one. Diagnostic
    /// insight lives behind the engine's own `diagnose_read` method,
    /// gated on the `debug:read` capability.
    ///
    /// Default: permit — matches NoAuth posture and every existing Phase-1
    /// host test that does not wire a read-gating policy.
    ///
    /// # Errors
    /// Returns [`EvalError::Capability`] when the configured policy denies.
    fn check_read_capability(
        &self,
        _label: &str,
        _target_cid: Option<&Cid>,
    ) -> Result<(), EvalError> {
        Ok(())
    }
}

/// A no-op [`PrimitiveHost`] for unit tests and benchmarks that exercise
/// the evaluator's per-primitive shape without a real backend. Reads miss,
/// captures are ignored, and anything that would require state surfaces
/// [`EvalError::Backend`] so a unit test that *shouldn't* touch the
/// backend is loud rather than silently succeeding.
pub struct NullHost;

impl PrimitiveHost for NullHost {
    fn read_node(&self, _cid: &Cid) -> Result<Option<Node>, EvalError> {
        Ok(None)
    }
    fn get_by_label(&self, _label: &str) -> Result<Vec<Cid>, EvalError> {
        Ok(Vec::new())
    }
    fn get_by_property(
        &self,
        _label: &str,
        _prop: &str,
        _value: &Value,
    ) -> Result<Vec<Cid>, EvalError> {
        Ok(Vec::new())
    }
    fn put_node(&self, _node: &Node) -> Result<Cid, EvalError> {
        Err(EvalError::Backend("NullHost: put_node unsupported".into()))
    }
    fn put_edge(&self, _edge: &Edge) -> Result<Cid, EvalError> {
        Err(EvalError::Backend("NullHost: put_edge unsupported".into()))
    }
    fn delete_node(&self, _cid: &Cid) -> Result<(), EvalError> {
        Ok(())
    }
    fn delete_edge(&self, _cid: &Cid) -> Result<(), EvalError> {
        Ok(())
    }
    fn call_handler(&self, _handler_id: &str, _op: &str, _input: Node) -> Result<Value, EvalError> {
        Err(EvalError::Backend(
            "NullHost: call_handler unsupported".into(),
        ))
    }
    fn emit_event(&self, _name: &str, _payload: Value) {}
    fn check_capability(&self, _required: &str, _target: Option<&Cid>) -> Result<(), EvalError> {
        Ok(())
    }
    fn read_view(&self, _view_id: &str, _query: &ViewQuery) -> Result<Value, EvalError> {
        Err(EvalError::Backend("NullHost: read_view unsupported".into()))
    }
}

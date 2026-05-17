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
///
/// # SemVer contract (v1-API-stabilization, refinement-audit #1008)
///
/// `PrimitiveHost` is a **CLAUDE.md #19 engine-extension trait point**:
/// out-of-tree engine extensions (alternate hosts, custom backends) are
/// expected to `impl PrimitiveHost`. It is therefore **deliberately NOT
/// sealed and NOT `#[non_exhaustive]`** — those mechanisms would block the
/// out-of-tree implementations CLAUDE.md #19 explicitly sanctions, and
/// `#[non_exhaustive]` is not even applicable to a `trait` (it applies to
/// structs/enums only). The SemVer discipline that #1008 asks for is
/// instead expressed as an **explicit written contract**, mirroring the
/// `BlobBackend` "additive-default posture locked as the v1 commitment"
/// pattern (`docs/future/phase-4-backlog.md §4.62`):
///
/// - Adding a **required** method is a **breaking change** (every external
///   impl must be updated) — permitted only at a major version bump.
/// - Adding a method **with a default body** is additive/non-breaking and
///   is the preferred evolution path (see `iterate_batch_boundary` /
///   `check_read_capability` / `suspension_store` precedents below).
/// - Changing an existing method signature is breaking.
///
/// New host capabilities should be introduced as defaulted methods so the
/// trait stays SemVer-stable for the out-of-tree extension surface.
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

    /// Phase-2b Wave-8i: hand the dispatcher the suspension store the
    /// host wants WAIT to persist envelopes + metadata into.
    ///
    /// The engine impl returns the engine's durable redb-backed store
    /// (`Engine::suspension_store()`). The default — used by `NullHost`
    /// + every unit-test PrimitiveHost — falls back to the process-
    /// default singleton, matching the Phase-2a `EvalContext` fallback
    /// for the same reason: WAIT unit tests don't wire an engine.
    fn suspension_store(&self) -> std::sync::Arc<dyn crate::suspension_store::SuspensionStore> {
        crate::suspension_store::default_process_store()
    }

    /// Phase-2b Wave-8i: hand the dispatcher the host's current
    /// monotonic-clock reading in milliseconds (or `None` if no clock
    /// is wired). WAIT records this as `suspend_elapsed_ms` so the
    /// resume-time deadline check has a stable start reference.
    ///
    /// Default: `None` (no clock — matches `EvalContext::elapsed_ms`
    /// returning `None` when no `TimeSource` is injected).
    fn elapsed_ms(&self) -> Option<u64> {
        None
    }

    /// Phase-2b Wave-8i fix-pass (w8i-wait-cag-01): hand the WAIT
    /// dispatcher the principal CID the current dispatch was invoked
    /// under, so the suspension envelope's
    /// `resumption_principal_cid` carries the caller-named principal
    /// rather than a signal-derived placeholder.
    ///
    /// The mini-review found that the Wave-8i regular-walk path
    /// silently dropped `call_as_with_suspension`'s `principal` arg;
    /// the resulting envelope was keyed on `BLAKE3(signal_name)`, so
    /// `resume_from_bytes_as(_, _, &caller_cid)` fired
    /// `E_RESUME_ACTOR_MISMATCH` against any non-trivial principal
    /// for real WAIT handlers. Threading this accessor through the
    /// trait restores the principal-binding contract for the
    /// regular-walk path without forcing every `PrimitiveHost`
    /// implementation to re-plumb its own per-call principal model.
    ///
    /// Default: `None` (no principal bound — matches the
    /// pre-fix-pass behaviour for `NullHost` + every test
    /// `PrimitiveHost`).
    fn suspending_principal(&self) -> Option<benten_core::Cid> {
        None
    }

    /// SANDBOX primitive dispatch (Phase 2b Wave-8b).
    ///
    /// The evaluator routes SANDBOX OperationNode dispatch through this
    /// method. The host implementation is responsible for:
    ///   1. Reading the SANDBOX node's `module` property (CID of the
    ///      WebAssembly module bytes) and fetching the bytes from the
    ///      engine's KV backend.
    ///   2. Resolving the manifest ref (named or inline) from the node's
    ///      properties.
    ///   3. Constructing the [`crate::primitives::sandbox::SandboxConfig`]
    ///      from the engine policy + the node's per-handler overrides
    ///      (D6 + D24 widening).
    ///   4. Looking up the dispatching grant's capability set.
    ///   5. Invoking [`crate::primitives::sandbox::execute`] with the
    ///      assembled inputs.
    ///   6. Mapping the [`crate::primitives::sandbox::SandboxResult`]
    ///      back to a [`crate::StepResult`].
    ///
    /// The default impl returns [`EvalError::PrimitiveNotImplemented`]
    /// so existing `NullHost`-backed unit tests continue to behave as
    /// before. The engine implementation in `benten-engine` overrides
    /// this method to route to the executor (paired wave-8c work).
    ///
    /// # Errors
    /// Returns [`EvalError::PrimitiveNotImplemented`] in the default
    /// (NullHost / Phase-1) impl; engine impl returns the executor's
    /// typed errors.
    fn execute_sandbox(&self, _op: &crate::OperationNode) -> Result<crate::StepResult, EvalError> {
        Err(EvalError::PrimitiveNotImplemented(
            crate::PrimitiveKind::Sandbox,
        ))
    }

    /// Phase-3 G21-T1: typed-CALL engine-side dispatch hook.
    ///
    /// The CALL primitive routes here when its `target` (handler_id)
    /// starts with [`crate::typed_call::TYPED_CALL_PREFIX`]
    /// (`"engine:typed:"`). The host implementation is responsible
    /// for:
    ///   1. Capability gating — calling
    ///      [`PrimitiveHost::check_capability`] with
    ///      [`crate::typed_call::TypedCallOp::required_cap`] before
    ///      invoking the underlying op (a denied call has zero
    ///      observable side effect).
    ///   2. Dispatching the named op against the underlying
    ///      `benten-id` / `benten-core` API (`benten-eval` cannot
    ///      depend on those crates per arch-r1-10, so the actual op
    ///      runs here).
    ///   3. Mapping op-internal typed errors (e.g. `KeypairError` /
    ///      `UcanError` / `VcError`) to
    ///      [`EvalError::TypedCallDispatchError`].
    ///
    /// The default impl returns
    /// [`EvalError::TypedCallDispatchError`] with reason
    /// `"typed-CALL dispatch not implemented on this host"` so
    /// existing `NullHost`-backed unit tests continue to behave as
    /// before. The engine implementation in `benten-engine`
    /// overrides this method to route to the real op handlers.
    ///
    /// # Errors
    /// Returns [`EvalError::TypedCallCapDenied`] when the per-op
    /// cap-check denies; [`EvalError::TypedCallDispatchError`] on
    /// op-internal failure.
    fn dispatch_typed_call(
        &self,
        op: crate::typed_call::TypedCallOp,
        _input: &Value,
    ) -> Result<Value, EvalError> {
        Err(EvalError::TypedCallDispatchError {
            op_name: op.name(),
            reason: "typed-CALL dispatch not implemented on this host".to_string(),
        })
    }

    /// Phase-3 G19-E: TRANSFORM AST cache lookup hook.
    ///
    /// Returns a parsed [`crate::expr::Expr`] for the TRANSFORM operation
    /// node identified by `node_id` within the currently-dispatching
    /// handler. The host resolves `node_id` against its current
    /// `active_call` frame's `handler_cid` so the lookup is keyed on the
    /// `(handler_cid, node_id)` pair the cache stores at registration
    /// time.
    ///
    /// `None` falls back to the per-call parse path inside
    /// [`crate::primitives::transform::execute`]. The Phase-1 default
    /// returns `None` so `NullHost` + every test `PrimitiveHost` keeps the
    /// pre-G19-E re-parse-every-call behaviour. The
    /// `benten-engine` impl overrides this to consult the engine's
    /// `AstCache` populated at `register_subgraph` /
    /// `register_subgraph_replace` time. Per phase-2-backlog §9.2 closure.
    fn cached_transform_ast(&self, _node_id: &str) -> Option<std::sync::Arc<crate::expr::Expr>> {
        None
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

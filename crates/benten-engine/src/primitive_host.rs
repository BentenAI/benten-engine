//! `impl PrimitiveHost for Engine` + the buffered-replay machinery.
//!
//! Extracted from `lib.rs` by R6 Wave 2 (R-major-01). The module owns the
//! call-frame bookkeeping (`ActiveCall` / `PendingHostOp` — both crate-
//! private) and the boundary-trait impl that lets `benten_eval::Evaluator`
//! drive the engine's backend writes.
//!
//! # Two-phase write (arch-2)
//!
//! The `impl PrimitiveHost for Engine` implementation is deliberately
//! *buffered*: every `put_node` / `delete_node` / `put_edge` / `delete_edge`
//! pushed by the evaluator lands in the active-call frame's `pending_ops`
//! list. After the evaluator walk terminates, `dispatch_call_inner` opens a
//! single transaction and replays the buffered ops atomically. Benefits:
//!
//! 1. **Single commit boundary.** The capability hook fires once per
//!    `Engine::call` against a fully-assembled `WriteContext`, not once per
//!    primitive. That matches the Phase-1 R1 triage decision (named
//!    compromise #5) that primitive-level cap checks are Phase-2 scope.
//! 2. **Rollback-on-error parity.** If the evaluator returns `Err` mid-walk
//!    (or if `test_inject_failure` trips), the `pending_ops` list is
//!    dropped without a commit — the backend never sees the partial
//!    writes, and the `Outcome` routes through `ON_ERROR / E_TX_ABORTED`.
//! 3. **Attribution fidelity.** Each buffered `PendingHostOp::PutNode`
//!    captures the `(actor_cid, handler_cid, capability_grant_cid)`
//!    triple from the `ActiveCall` frame at *buffer time*; by replay time
//!    the frame has already popped, but the emitted `ChangeEvent` still
//!    carries the originating audit context (r6-sec-3).
//!
//! The tradeoff is that individual primitives never see commit failures —
//! an evaluator `TRANSFORM` that wants to observe the effect of an earlier
//! `WRITE` sees the *projected* CID (computed at buffer time via `node.cid()`)
//! rather than the post-commit on-disk CID. Phase 1's test harnesses are
//! fine with that because content-addressed hashing is deterministic: the
//! projected CID matches the eventual committed CID byte-for-byte.

use benten_caps::CapError;
use benten_core::{Cid, Edge, Node, Value};
use benten_errors::ErrorCode;
use benten_eval::{HostError, PrimitiveHost};
use benten_graph::{GraphError, MutexExt};

use crate::engine::{Engine, is_known_view_id};
use crate::error::EngineError;
use crate::outcome::Outcome;
use crate::system_zones::SYSTEM_ZONE_PREFIXES;

/// Phase 2a G9-A-cont: wall-clock refresh ceiling per §9.13 refresh
/// point #3. An ITERATE / CALL loop that elapses this much monotonic
/// wall-time since its last cap-policy re-check MUST force another
/// `check_write` at the next batch boundary, regardless of wall-clock
/// drift. Matches the plan's 300-second cadence; exported publicly so
/// tests can reference the exact constant when constructing a frozen-
/// clock fixture.
pub const WALLCLOCK_REFRESH_CEILING: std::time::Duration = std::time::Duration::from_secs(300);

// ---------------------------------------------------------------------------
// Phase 2a G5-B-i: Inv-11 runtime probe
// ---------------------------------------------------------------------------

/// Phase-2a Inv-11 runtime probe: `true` when `label` is inside the
/// `system:*` reserved zone.
///
/// Every `system:*`-prefixed label is privileged — the broad check
/// matches the Phase-1 storage-layer stopgap
/// (`benten_graph::guard_system_zone_node`) so the registration-time
/// walker, the runtime probe, and the graph storage guard share one
/// deniable-set classification
/// (`both_paths_agree_on_deniable_set`).
///
/// [`SYSTEM_ZONE_PREFIXES`] remains documented as the list of concrete
/// system zones the engine itself writes, consumed by the
/// `inv_11_system_zone_drift_test` CI guard; the classification used
/// here is intentionally broader so an unknown-but-still-`system:`-
/// prefixed label still routes through Inv-11.
#[must_use]
pub(crate) fn is_system_zone_label(label: &str) -> bool {
    label.starts_with("system:")
}

/// Inv-11 runtime probe: resolve `cid` through the backend's label-only
/// fast path and return `true` when the stored Node carries a system-zone
/// label. A missing CID returns `false` (no node → no disclosure).
fn resolved_cid_in_system_zone(engine: &Engine, cid: &Cid) -> bool {
    match engine.backend().get_node_label_only(cid) {
        Ok(Some(label)) => is_system_zone_label(&label),
        Ok(None) | Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// ActiveCall + PendingHostOp
// ---------------------------------------------------------------------------

/// Per-call metadata tracked so [`PrimitiveHost`] methods can access the
/// in-flight actor / op without additional argument threading.
#[derive(Debug)]
pub(crate) struct ActiveCall {
    /// Handler id that initiated the call. Retained so Phase-2 capability
    /// binding can scope the cap-grant lookup to the specific handler.
    #[allow(
        dead_code,
        reason = "retained for Phase-2 capability-binding (R-minor-09)"
    )]
    pub(crate) handler_id: String,
    /// Op name (`"create"`, `"list"`, `"update"`, `"delete"`, …). Retained
    /// so Phase-2 per-op capability enforcement has the discriminator.
    #[allow(
        dead_code,
        reason = "retained for Phase-2 per-op capability enforcement (R-minor-09)"
    )]
    pub(crate) op: String,
    pub(crate) actor: Option<Cid>,
    /// Content-addressed identifier of the handler subgraph that issued
    /// the in-flight call. Captured alongside `handler_id` so the
    /// PrimitiveHost write path can stamp emitted ChangeEvents with
    /// `handler_cid` for audit attribution (r6-sec-3).
    pub(crate) handler_cid: Option<Cid>,
    /// Buffered write operations, replayed as a single transaction after the
    /// Evaluator completes. Populated by `impl PrimitiveHost::put_node` /
    /// `delete_node` / `put_edge` / `delete_edge`.
    pub(crate) pending_ops: Vec<PendingHostOp>,
    /// Whether a host-side `test_inject_failure` signalled a rollback.
    pub(crate) inject_failure: bool,
    /// Phase 2a G9-A-cont: monotonic elapsed at the last wall-clock
    /// refresh. `None` at dispatch start — the first batch-boundary
    /// `check_capability` populates it and every subsequent boundary
    /// compares against it to decide whether to force a re-check per
    /// §9.13 refresh point #3.
    pub(crate) last_refresh: Option<std::time::Duration>,
    /// Phase 2a G9-A-cont: per-call iteration counter, incremented on
    /// every `check_capability` call. Used by the
    /// `schedule_revocation_at_iteration` test harness so the refresh
    /// path can observe a scheduled revocation target.
    pub(crate) iteration: u64,
}

/// A deferred host-side write op, replayed inside `dispatch_call`'s
/// transaction after the evaluator walk completes.
///
/// `PutNode` carries the per-op attribution triple so the replayed
/// `ChangeEvent` can surface the audit trail (r6-sec-3). The triple is
/// captured from the `ActiveCall` frame at buffer time — by replay time
/// the frame has already popped.
#[derive(Debug, Clone)]
pub(crate) enum PendingHostOp {
    PutNode {
        node: Node,
        projected_cid: Cid,
        actor_cid: Option<Cid>,
        handler_cid: Option<Cid>,
        capability_grant_cid: Option<Cid>,
    },
    DeleteNode {
        cid: Cid,
    },
}

// ---------------------------------------------------------------------------
// Typed read-context plumbing (G11-A Wave-2a carry — EVAL Wave-1 M2)
// ---------------------------------------------------------------------------

impl Engine {
    /// Route a typed [`benten_caps::ReadContext`] through the configured
    /// capability policy's `check_read` hook.
    ///
    /// Engine-side Option C flanking sites (`read_node`, `get_by_label`,
    /// `get_by_property`, `read_view`) construct the context directly via
    /// [`benten_caps::ReadContext::by_cid_only`] or
    /// [`benten_caps::ReadContext::by_label_only`] so the read-shape
    /// intent is typed rather than encoded as an unwritten "empty label"
    /// convention on the trait method's `(label, Option<&Cid>)` pair.
    ///
    /// # Errors
    ///
    /// Returns [`benten_eval::EvalError::Capability`] when the configured
    /// policy denies. No policy → `Ok(())`.
    pub(crate) fn check_read_ctx(
        &self,
        ctx: &benten_caps::ReadContext,
    ) -> Result<(), benten_eval::EvalError> {
        if let Some(policy) = self.policy()
            && let Err(c) = policy.check_read(ctx)
        {
            return Err(benten_eval::EvalError::Capability(c));
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// impl PrimitiveHost for Engine
// ---------------------------------------------------------------------------

impl PrimitiveHost for Engine {
    // r6-err-2: typed conversions (via `?`) preserve the origin error's
    // stable catalog code through the EvalError → EngineError → napi → TS
    // pipeline. Prior to this, GraphError was stringified into
    // `EvalError::Backend(String)` and the catalog code collapsed to
    // `E_EVAL_BACKEND` at the boundary.
    fn read_node(&self, cid: &Cid) -> Result<Option<Node>, benten_eval::EvalError> {
        // Phase-2a Inv-11 runtime probe (G5-B-i / Code-as-graph Major #1):
        // a TRANSFORM-computed CID whose resolved Node carries a
        // `system:*` label MUST NOT leak to user code, regardless of what
        // the configured capability policy permits. Inv-11 is an engine-
        // side invariant stricter than the pluggable cap policy. We
        // collapse to `Ok(None)` (symmetric with a clean miss) rather
        // than surface a typed error so the adversary cannot distinguish
        // "system-zone CID" from "no such CID" via the evaluator-visible
        // return shape. Probe the RESOLVED Node's label via the backend's
        // `get_node_label_only` fast path — per Major #1 the check is on
        // the resolved label, not any passing `Value` payload.
        if resolved_cid_in_system_zone(self, cid) {
            return Ok(None);
        }
        // Option C flanking (sec-r1-5 / atk-5): consult the read-gate on
        // every content-returning method — the by-CID branch collapses a
        // denial to `Ok(None)` so the caller cannot distinguish a denied
        // CID from a genuine miss. Mirrors the shape of `Engine::get_node`
        // (the public API wrapper); the evaluator-visible path needs the
        // same collapse so a TRANSFORM-driven handler cannot flank.
        if let Err(benten_eval::EvalError::Capability(_)) =
            self.check_read_ctx(&benten_caps::ReadContext::by_cid_only(*cid))
        {
            return Ok(None);
        }
        self.backend().get_node(cid).map_err(graph_err_to_eval)
    }

    fn get_by_label(&self, label: &str) -> Result<Vec<Cid>, benten_eval::EvalError> {
        // Phase-2a Inv-11 runtime probe: a user subgraph asking for every
        // CID carrying a `system:*` label is a direct enumeration attack
        // on the capability grant zone. Collapse to an empty list —
        // symmetric with "no matching Nodes" — before the backend is
        // consulted so the index probe itself is not a side-channel.
        if is_system_zone_label(label) {
            return Ok(Vec::new());
        }
        // Option C flanking (sec-r1-5 / atk-5): consult the read-gate
        // before the backend probe. A denial collapses to an empty list —
        // symmetric with "no matching Nodes" — so a TRANSFORM-driven
        // handler that flanks through this accessor cannot distinguish a
        // denied label from an empty one. See `docs/SECURITY-POSTURE.md`
        // Compromise #2 (Option C) for the posture contract.
        if let Err(benten_eval::EvalError::Capability(_)) =
            self.check_read_ctx(&benten_caps::ReadContext::by_label_only(label))
        {
            return Ok(Vec::new());
        }
        self.backend()
            .get_by_label(label)
            .map_err(graph_err_to_eval)
    }

    fn get_by_property(
        &self,
        label: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Cid>, benten_eval::EvalError> {
        // Phase-2a Inv-11 runtime probe: symmetric with `get_by_label`
        // above — a `system:*`-filtered property query enumerates the
        // system zone. Collapse before the backend probe.
        if is_system_zone_label(label) {
            return Ok(Vec::new());
        }
        // Option C flanking (sec-r1-5 / atk-5): symmetric with
        // `get_by_label` — cap-denied reads collapse to an empty result
        // rather than leak existence via a populated list.
        if let Err(benten_eval::EvalError::Capability(_)) =
            self.check_read_ctx(&benten_caps::ReadContext::by_label_only(label))
        {
            return Ok(Vec::new());
        }
        self.backend()
            .get_by_property(label, prop, value)
            .map_err(graph_err_to_eval)
    }

    fn put_node(&self, node: &Node) -> Result<Cid, benten_eval::EvalError> {
        // Phase-2a Inv-11 runtime probe (G5-B-i mini-review C1): a handler
        // WRITE whose Node carries a `system:*` label MUST fire
        // `E_INV_SYSTEM_ZONE` at the evaluator-visible boundary, NOT the
        // Phase-1 storage-layer stopgap `E_SYSTEM_ZONE_WRITE`. Mirrors the
        // user-facing check in `engine_crud::create_node`: the same broad
        // `is_system_zone_label` probe fires before any PendingHostOp is
        // buffered so the replay path never sees the violating op. The
        // storage-layer `guard_system_zone_node` stays wired as
        // defence-in-depth per plan §9.10.
        for label in &node.labels {
            if is_system_zone_label(label) {
                return Err(benten_eval::EvalError::Invariant(
                    benten_eval::InvariantViolation::SystemZone,
                ));
            }
        }
        // Project the Node's CID up front so the evaluator's StepResult can
        // echo it back immediately; the real backend write happens after
        // the evaluator walk completes, inside a single transaction.
        let projected = node.cid()?;
        let mut guard = self.active_call().lock_recover();
        if let Some(frame) = guard.last_mut() {
            // r6-sec-3 attribution: capture actor/handler so the replay
            // path can stamp each emitted ChangeEvent with the originating
            // audit context. capability_grant_cid is intentionally None
            // under NoAuthBackend (no grant entity); populated Phase 3.
            //
            // If the caller did not supply an explicit actor, synthesize a
            // stable pseudo-actor CID from the NoAuth label so audit
            // consumers can distinguish "no one supplied an actor" from
            // "the write wasn't attributed at all". The seed is fixed so
            // every noauth call produces the same CID process-wide.
            let actor_cid = frame.actor.or_else(|| Some(noauth_pseudo_actor_cid()));
            let handler_cid = frame.handler_cid;
            frame.pending_ops.push(PendingHostOp::PutNode {
                node: node.clone(),
                projected_cid: projected,
                actor_cid,
                handler_cid,
                capability_grant_cid: None,
            });
            Ok(projected)
        } else {
            // Outside a dispatch_call — fall through to a direct backend
            // transaction. Preserves behavior for any Phase-1 code paths
            // that call impl PrimitiveHost::put_node without a containing
            // dispatch.
            drop(guard);
            self.backend().put_node(node).map_err(graph_err_to_eval)
        }
    }

    fn put_edge(&self, _edge: &Edge) -> Result<Cid, benten_eval::EvalError> {
        // r6b-ce-2: the prior buffering-then-silent-no-op shape contradicted
        // the PrimitiveHost buffer+replay ALL-or-NONE atomicity claim — the
        // replay arm in `dispatch_call_inner` dropped edge ops on the floor.
        // Phase-1 has no evaluator path that reaches this method, so failing
        // loud here (rather than silently succeeding then losing the write)
        // prevents a Phase-2 contributor from wiring a primitive that
        // depends on an edge write that never lands.
        //
        // Maps to `E_NOT_IMPLEMENTED` at the catalog; the full edge-ops
        // replay wires with the dedicated EngineTransaction edge API in
        // Phase 2.
        Err(benten_eval::EvalError::Unsupported {
            operation: "put_edge".to_string(),
        })
    }

    fn delete_node(&self, cid: &Cid) -> Result<(), benten_eval::EvalError> {
        let mut guard = self.active_call().lock_recover();
        if let Some(frame) = guard.last_mut() {
            frame
                .pending_ops
                .push(PendingHostOp::DeleteNode { cid: *cid });
            Ok(())
        } else {
            drop(guard);
            self.backend().delete_node(cid).map_err(graph_err_to_eval)?;
            Ok(())
        }
    }

    fn delete_edge(&self, _cid: &Cid) -> Result<(), benten_eval::EvalError> {
        // r6b-ce-2: symmetric fail-loud with `put_edge`. See that method
        // for the full rationale.
        Err(benten_eval::EvalError::Unsupported {
            operation: "delete_edge".to_string(),
        })
    }

    fn call_handler(
        &self,
        handler_id: &str,
        op: &str,
        input: Node,
    ) -> Result<Value, benten_eval::EvalError> {
        match self.dispatch_call(handler_id, op, input, None) {
            Ok(outcome) => {
                // Translate the outcome shape into a best-effort Value for the
                // caller. Callees that RESPOND a Map payload surface it
                // directly; other shapes surface an empty Map.
                if let Some(list) = outcome.list {
                    Ok(Value::List(
                        list.into_iter().map(|n| Value::Map(n.properties)).collect(),
                    ))
                } else if let Some(cid) = outcome.created_cid {
                    Ok(Value::Text(cid.to_base32()))
                } else {
                    Ok(Value::Null)
                }
            }
            // r6b-err-1: typed pass-through preserves the origin catalog
            // code across the EngineError → EvalError boundary. Prior to
            // this, every non-Cap error collapsed into
            // `EvalError::Backend(format!("{e:?}"))` — the stable
            // catalog code (`E_UNKNOWN_VIEW`, `E_IVM_VIEW_STALE`,
            // `E_SUBSYSTEM_DISABLED`, `E_GRAPH_INTERNAL`, …) was lost at
            // the boundary and the TS wrapper saw an opaque
            // `E_EVAL_BACKEND` with a debug-formatted message.
            Err(e) => Err(engine_error_to_eval_error(e)),
        }
    }

    fn emit_event(&self, _name: &str, _payload: Value) {
        // Phase-1 EMIT is a no-op at the host level — the change-broadcast
        // fan-out is already wired to storage WRITEs; standalone EMIT
        // primitives without a backing store mutation don't carry a
        // ChangeEvent payload shape yet. Reserved for Phase-2.
    }

    fn check_read_capability(
        &self,
        label: &str,
        target_cid: Option<&Cid>,
    ) -> Result<(), benten_eval::EvalError> {
        // G11-A Wave-2a carry (EVAL Wave-1 M2): the trait method retains
        // its `(label, Option<&Cid>)` signature for boundary stability,
        // but routes through the typed `check_read_ctx` helper so the
        // "empty label means CID-only" convention becomes explicit via
        // `ReadContext::by_cid_only` / `ReadContext::by_label_only`.
        // Engine-side sites call `check_read_ctx` directly with a typed
        // context; external PrimitiveHost callers hitting this method
        // land on the branch that matches the `(label, target_cid)`
        // pair.
        let ctx = match (label.is_empty(), target_cid) {
            (true, Some(cid)) => benten_caps::ReadContext::by_cid_only(*cid),
            (_, None) => benten_caps::ReadContext::by_label_only(label),
            (false, Some(cid)) => benten_caps::ReadContext {
                label: label.to_string(),
                target_cid: Some(*cid),
                ..Default::default()
            },
        };
        self.check_read_ctx(&ctx)
    }

    fn check_capability(
        &self,
        required: &str,
        _target: Option<&Cid>,
    ) -> Result<(), benten_eval::EvalError> {
        // Phase 2a G9-A-cont refresh-point-3 (§9.13): drive the wall-clock
        // TOCTOU refresh cadence off the configured *monotonic* source,
        // NOT off the HLC / wall-clock source. `iterate_batch_boundary`
        // brings the evaluator here every N iterations; every entry
        // bumps the per-call `iteration` counter and every entry past
        // WALLCLOCK_REFRESH_CEILING of monotonic elapsed forces a
        // policy re-check.
        //
        // A scheduled revocation (`Engine::schedule_revocation_at_
        // iteration(grant, n)`) surfaces here by making the
        // `check_write` hook deny once `iteration > n`. That keeps the
        // in-process test harness honest without wiring an auxiliary
        // queue into the cap-policy layer.
        let iteration_now;
        let revocation_due;
        {
            let mut guard = self.active_call().lock_recover();
            if let Some(frame) = guard.last_mut() {
                frame.iteration = frame.iteration.saturating_add(1);
                iteration_now = frame.iteration;

                // Monotonic refresh probe. A `MockMonotonicSource` returns
                // caller-controlled elapsed; `InstantMonotonicSource`
                // returns true-monotonic. Either way, wall-clock drift
                // cannot make the cadence skip.
                let elapsed = self.monotonic_source.elapsed_since_start();
                let due = match frame.last_refresh {
                    None => true, // first boundary always fires
                    Some(last) => elapsed.saturating_sub(last) >= WALLCLOCK_REFRESH_CEILING,
                };
                if due {
                    frame.last_refresh = Some(elapsed);
                }

                // Consult the scheduled-revocation map: has any grant hit
                // its target iteration?
                let revoke_guard = benten_graph::MutexExt::lock_recover(&self.revoke_at_iteration);
                revocation_due = revoke_guard.values().any(|&target| iteration_now > target);
            } else {
                iteration_now = 0;
                revocation_due = false;
            }
        }

        if revocation_due {
            // Synthesize a `RevokedMidEval` cap error so the evaluator's
            // routing arm surfaces `E_CAP_REVOKED_MID_EVAL`. Matches
            // `benten_caps::CapError::code()` row for this variant.
            let _ = required; // referenced above for the WriteContext path
            return Err(benten_eval::EvalError::Capability(
                benten_caps::CapError::RevokedMidEval,
            ));
        }

        if let Some(policy) = self.policy() {
            let ctx = benten_caps::WriteContext {
                label: required.to_string(),
                ..Default::default()
            };
            if let Err(c) = policy.check_write(&ctx) {
                return Err(benten_eval::EvalError::Capability(c));
            }
        }
        let _ = iteration_now;
        Ok(())
    }

    fn read_view(
        &self,
        view_id: &str,
        query: &benten_eval::ViewQuery,
    ) -> Result<Value, benten_eval::EvalError> {
        // Phase-2a Inv-11 runtime probe: a user subgraph reading a
        // `system:ivm:*` view id or a `system:*` query label enumerates
        // engine-privileged projections. Collapse to an empty list
        // before the IVM subscriber is consulted so the view-id
        // registry is not itself a side-channel.
        if is_system_zone_label(view_id) || query.label.as_deref().is_some_and(is_system_zone_label)
        {
            return Ok(Value::List(Vec::new()));
        }
        // Option C flanking (sec-r1-5 / atk-5) — coarse-grained per
        // named Compromise #N+2 (IVM views are coarse-grained read-gated
        // in Phase 2a; per-row gating is Phase 3). The cap gate keys off
        // the query's label filter when one is present, falling back to
        // the view_id as a scope identifier. A denial collapses the
        // whole view to an empty list rather than leaking existence.
        let label = query.label.as_deref().unwrap_or(view_id);
        if let Err(benten_eval::EvalError::Capability(_)) =
            self.check_read_ctx(&benten_caps::ReadContext::by_label_only(label))
        {
            return Ok(Value::List(Vec::new()));
        }
        match Engine::read_view(self, view_id) {
            Ok(outcome) => {
                if let Some(list) = outcome.list {
                    Ok(Value::List(
                        list.into_iter().map(|n| Value::Map(n.properties)).collect(),
                    ))
                } else {
                    Ok(Value::Null)
                }
            }
            // r6b-err-1: typed pass-through — see `call_handler` above.
            Err(e) => Err(engine_error_to_eval_error(e)),
        }
    }

    /// Phase 2a G9-A / P2: delegate the ITERATE batch-boundary cadence to
    /// the configured capability policy (§9.13 refresh-point-3 + plan §3 G9).
    ///
    /// The default `PrimitiveHost` impl returns the hard-coded Phase-1
    /// constant (100); this override makes the policy's
    /// `iterate_batch_boundary` method load-bearing end-to-end so a
    /// revocation-sensitive backend (Phase-3 UCAN with a short TTL) can
    /// tighten the bound. When no policy is configured we keep the Phase-1
    /// default via `benten_caps::DEFAULT_BATCH_BOUNDARY`.
    ///
    /// Routing goes through
    /// [`benten_caps::evaluator_delegation::iterate_batch_boundary_for`] so
    /// the shared helper's test harness (see
    /// `crates/benten-caps/tests/wallclock_delegation.rs`) and the engine's
    /// production path converge on a single delegation point.
    fn iterate_batch_boundary(&self) -> usize {
        match self.policy() {
            Some(p) => benten_caps::evaluator_delegation::iterate_batch_boundary_for(p),
            None => benten_caps::DEFAULT_BATCH_BOUNDARY,
        }
    }
}

// ---------------------------------------------------------------------------
// EvalError ↔ EngineError conversion + attribution helpers
// ---------------------------------------------------------------------------

/// Convert an `EngineError` into the equivalent `EvalError` for return
/// across the `PrimitiveHost` boundary (r6b-err-1). Preserves the origin
/// catalog code by dispatching to typed `EvalError` variants for every
/// `EngineError` shape that has a stable downstream identity; the
/// catch-all `Backend(message)` path now only fires for residual
/// `EngineError::Other` and `NotImplemented` shapes.
fn engine_error_to_eval_error(e: EngineError) -> benten_eval::EvalError {
    match e {
        EngineError::Cap(c) => benten_eval::EvalError::Capability(c),
        // arch-1 dep-break (G1-B / phil-r1-2 / plan §9.10 + §9.14): the former
        // `EvalError::Graph(GraphError)` round-trip is replaced with a
        // HostError envelope. `benten-eval` no longer depends on
        // `benten-graph`, so the mapping from GraphError → evaluator-visible
        // error happens HERE, at the `PrimitiveHost` boundary. The catalog
        // code on the `HostError` mirrors the `GraphError`'s `code()` so
        // `EvalError::code()` still returns the same stable discriminant
        // callers saw pre-G1-B (no TS-wire regression).
        EngineError::Graph(g) => benten_eval::EvalError::Host(graph_error_to_host_error(g)),
        EngineError::Core(c) => benten_eval::EvalError::Core(c),
        EngineError::UnknownView { view_id } => benten_eval::EvalError::UnknownView(view_id),
        EngineError::IvmViewStale { view_id } => benten_eval::EvalError::IvmViewStale(view_id),
        EngineError::SubsystemDisabled { subsystem } => {
            benten_eval::EvalError::SubsystemDisabled(subsystem.to_string())
        }
        // Remaining shapes (Invariant, DuplicateHandler, NestedTransaction,
        // Other, NotImplemented, the two builder-guard errors) have no
        // typed EvalError representation; fall back to the debug-formatted
        // Backend channel. The stable catalog code is still recoverable
        // via the `{error_code()}` accessor on the resurrected EngineError
        // above the TS boundary, and the residual shapes are
        // engine-orchestrator concerns rather than evaluator-visible
        // states, so the drop is acceptable.
        other => benten_eval::EvalError::Backend(format!("{other:?}")),
    }
}

/// Convenience: wrap a `GraphError` as an `EvalError::Host(HostError)` so
/// `impl PrimitiveHost for Engine` methods can funnel backend-side
/// rejections through `.map_err(graph_err_to_eval)` without restating the
/// HostError construction (G1-B / arch-1 dep-break).
fn graph_err_to_eval(g: GraphError) -> benten_eval::EvalError {
    benten_eval::EvalError::Host(graph_error_to_host_error(g))
}

/// Map a `GraphError` to a `HostError` for routing across the
/// `PrimitiveHost` boundary (G1-B / arch-1 dep-break). Preserves the
/// origin stable catalog code on `HostError.code` so `EvalError::code()`
/// returns the same discriminant the pre-G1-B `EvalError::Graph(g).code()`
/// path surfaced. The `GraphError` itself becomes the opaque
/// `Box<dyn StdError>` source — it never crosses back onto a wire because
/// `HostError::to_wire_bytes` excludes `source` per sec-r1-6 / atk-6.
fn graph_error_to_host_error(g: GraphError) -> HostError {
    let code = g.code();
    // Render a user-safe context from the Display form. Display on
    // `GraphError::BackendNotFound` is already redacted to a basename
    // (see `redact_path_for_display` in benten-graph), so this string is
    // safe to surface to callers. We do NOT route a context for
    // RedbSource / Redb / Decode because their Display forms can embed
    // redb internal identifiers; the opaque source chain is the
    // programmatic path instead.
    let context = match &g {
        GraphError::BackendNotFound { .. }
        | GraphError::SystemZoneWrite { .. }
        | GraphError::NestedTransactionNotSupported {}
        | GraphError::TxAborted { .. } => Some(g.to_string()),
        // CoreError Display is audit-safe: it renders serialize/decode
        // error messages that never embed CID bytes (the CID bytes only
        // appear inside Display of backend-storage-layer errors like
        // `GraphError::Decode`, which wraps a redb encode step; those
        // stay opaque below). Surfacing CoreError's Display restores the
        // pre-G1-B "graph: core: <msg>" readability for downstream
        // match-on-source callers. Fix: G1-B mini-review M1.
        GraphError::Core(inner) => Some(format!("core: {inner}")),
        // RedbSource / Redb / Decode stay opaque: their Display can embed
        // redb internal identifiers (page numbers, raw byte offsets) that
        // a wire observer could correlate with disk layout. The
        // programmatic path is the opaque `source` chain, not `context`.
        GraphError::RedbSource(_) | GraphError::Redb(_) | GraphError::Decode(_) => None,
        // GraphError is #[non_exhaustive]; a future variant falls through
        // as "no context" so a Phase-2 addition never silently leaks a
        // raw Debug payload through the envelope.
        _ => None,
    };
    HostError {
        code,
        source: Box::new(g),
        context,
    }
}

/// Map a `HostError` surfaced from the evaluator back into an
/// `EngineError` for the transaction closure's return type (G1-B).
/// The `HostError.source` is attempted-downcast back to `GraphError` so
/// the engine side preserves the typed variant for call sites that still
/// match on `EngineError::Graph`. When the downcast fails (source was a
/// non-graph error, e.g. after a future Phase-2b sandbox-host wires a
/// wasmtime-side error into HostError), we fall through to
/// `EngineError::Other` keyed on the stable catalog code.
fn host_error_to_engine_error(h: HostError) -> EngineError {
    let code = h.code.clone();
    // Fallback to the full HostError Display form (which includes the
    // `host error (CODE): context` shape) rather than the bare code
    // string — preserves Display consistency for callers that render the
    // EngineError to a log line. Fix: G1-B mini-review M2.
    let message = h.context.clone().unwrap_or_else(|| h.to_string());
    // Try to recover the original `GraphError` — the common case in
    // Phase-1 / 2a where the eval-side saw a HostError we ourselves
    // minted from a GraphError three lines upstream. `Box<dyn Error + Send
    // + Sync>` supports `downcast` via `std::error::Error::is` +
    // `Box::<dyn Any>::downcast`; we use the former for safety.
    match h.source.downcast::<GraphError>() {
        Ok(g) => EngineError::Graph(*g),
        Err(_) => EngineError::Other { code, message },
    }
}

/// Signature-level arch-1 gate compile check (plan §9.14). Confirms the
/// `HostError` envelope is the *only* shape that carries backend-side
/// failure across the `PrimitiveHost` boundary — if a future edit adds
/// a `benten_graph::*` path to a `PrimitiveHost` trait-method signature
/// or to an `EvalError` variant, CI + the signature-level unit tests
/// (`arch_1_no_graph_types_in_primitive_host.rs`, the YAML gate) fire.
///
/// Implemented as a `const fn` pointer alias so deleting it fails the
/// compile instead of silently losing the guarantee (G1-B mini-review
/// N2). The alias takes the canonical coercion shape — a
/// `fn(HostError) -> benten_eval::EvalError` — and constructs it from
/// `EvalError::Host` at const-eval time. A refactor that renames the
/// variant or changes the shape breaks the alias.
#[allow(
    dead_code,
    reason = "arch-1 anchor: proves HostError is the sole backend-error surface; see plan §9.14"
)]
const _ARCH_1_HOST_ERROR_IS_THE_BOUNDARY: fn(HostError) -> benten_eval::EvalError =
    benten_eval::EvalError::Host;

/// Convert an `EvalError` back into an `EngineError` for the transaction
/// closure's return type.
///
/// r6b-err-3: the `EvalError::Backend` arm now routes to the same stable
/// string (`E_EVAL_BACKEND`) that `EvalError::code()` emits for the same
/// variant. Prior to this, the engine-side conversion spelled it
/// `E_BACKEND` while the eval-side `.code()` spelled it `E_EVAL_BACKEND`;
/// two catalog strings for one conceptual state meant a TS caller doing a
/// `switch (err.code)` branch saw the code flip depending on which side
/// of the boundary the error was observed on.
pub(crate) fn eval_error_to_engine_error(e: benten_eval::EvalError) -> EngineError {
    match e {
        benten_eval::EvalError::Capability(c) => EngineError::Cap(c),
        // arch-1 dep-break (G1-B): the Phase-1 `EvalError::Graph(GraphError)`
        // round-trip is replaced by `EvalError::Host(HostError)`. The inverse
        // mapping downcasts `HostError.source` back to `GraphError` when
        // possible, preserving the pre-G1-B `EngineError::Graph(g)` shape
        // for callers that still match on it. See
        // `host_error_to_engine_error` for the recovery logic.
        benten_eval::EvalError::Host(h) => host_error_to_engine_error(h),
        benten_eval::EvalError::Core(c) => EngineError::Core(c),
        // r6b-err-1: typed round-trip. An `EvalError::UnknownView` that
        // came from an engine-side rejection (via `engine_error_to_eval_error`)
        // must land back on the same engine-side variant, preserving the
        // stable catalog code across the round-trip. Similarly for
        // `IvmViewStale` and `SubsystemDisabled`.
        //
        // `SubsystemDisabled` round-trips through a `String` because
        // `EngineError::SubsystemDisabled` carries a `&'static str`; the
        // outer boundary spelling picks "ivm" or "capabilities" per the
        // set of constants the engine uses. Phase-1 hits exactly those two
        // strings; Phase-2 can intern to a typed enum if the set grows.
        benten_eval::EvalError::UnknownView(view_id) => EngineError::UnknownView { view_id },
        benten_eval::EvalError::IvmViewStale(view_id) => EngineError::IvmViewStale { view_id },
        benten_eval::EvalError::SubsystemDisabled(subsystem) => EngineError::Other {
            code: benten_errors::ErrorCode::SubsystemDisabled,
            message: format!("subsystem disabled: {subsystem}"),
        },
        benten_eval::EvalError::Backend(m) => EngineError::Other {
            // Single source of truth — mirrors
            // `EvalError::Backend.code()` in benten-eval (r6b-err-3).
            code: benten_errors::ErrorCode::Unknown("E_EVAL_BACKEND".into()),
            message: m,
        },
        other => EngineError::Other {
            code: other.code(),
            message: format!("{other:?}"),
        },
    }
}

/// Return the stable pseudo-actor CID used when NoAuthBackend issues a write
/// without a caller-supplied actor. Derived from a fixed seed so every
/// noauth write process-wide attributes to the same CID — audit consumers
/// can then tell "noauth" writes apart from cross-principal writes without
/// needing the capability policy to carry identity state.
pub(crate) fn noauth_pseudo_actor_cid() -> Cid {
    // Fixed 32-byte BLAKE3 digest of the UTF-8 bytes of "noauth-pseudo-actor-v1".
    // Computed at runtime (no `const` path for blake3) → stable across releases.
    let digest: [u8; 32] = *blake3::hash(b"noauth-pseudo-actor-v1").as_bytes();
    Cid::from_blake3_digest(digest)
}

// ---------------------------------------------------------------------------
// Outcome-mapping helpers
// ---------------------------------------------------------------------------

/// Map the evaluator's terminal (`edge`, `output`) pair into the engine's
/// user-facing `Outcome` shape. `list_hint`, when set, directs the mapper
/// to materialize `outcome.list` by consulting View 3 (content_listing)
/// when the IVM subscriber has a view registered for that label; otherwise
/// falls back to a direct label-index walk. `created_cid_hint` is the CID
/// returned by the transaction replay of host-side WRITEs.
pub(crate) fn outcome_from_terminal_with_cid(
    engine: &Engine,
    edge: &str,
    _output: Value,
    list_hint: Option<String>,
    created_cid_hint: Option<Cid>,
) -> Outcome {
    // RESPOND's terminal edge is `"terminal"`; WRITE / READ terminate on
    // `"ok"`. Both map to the user-facing `"OK"` edge. Typed error edges
    // round-trip verbatim.
    let (normalized_edge, error_code) = match edge {
        "terminal" | "ok" => ("OK".to_string(), None),
        "ON_NOT_FOUND" => ("ON_NOT_FOUND".to_string(), Some("E_NOT_FOUND".to_string())),
        "ON_DENIED" => (
            "ON_DENIED".to_string(),
            Some("E_CAP_DENIED_READ".to_string()),
        ),
        "ON_CONFLICT" => (
            "ON_CONFLICT".to_string(),
            Some("E_WRITE_CONFLICT".to_string()),
        ),
        "ON_LIMIT" => ("ON_LIMIT".to_string(), Some("E_INPUT_LIMIT".to_string())),
        "ON_ERROR" => ("ON_ERROR".to_string(), Some("E_UNKNOWN".to_string())),
        other => (other.to_string(), None),
    };

    let created_cid = created_cid_hint;

    // List hint: resolve the list.
    // - `"get:<label>:<base32>"` — single-Node resolution via label scan.
    // - any other `<label>` — plural listing. Prefer View 3 (content_listing)
    //   when the IVM subscriber has a view registered for that label; fall
    //   back to the backend label index (`without_ivm` engines + views that
    //   haven't been created yet).
    let list = if let Some(hint) = list_hint.as_deref() {
        if let Some(rest) = hint.strip_prefix("get:") {
            let mut out = Vec::new();
            if let Some((scan_label, b32)) = rest.split_once(':')
                && let Ok(cids) = engine.backend().get_by_label(scan_label)
                && let Some(cid) = cids.into_iter().find(|c| c.to_base32() == b32)
                && let Ok(Some(node)) = engine.backend().get_node(&cid)
            {
                out.push(node);
            }
            Some(out)
        } else {
            Some(resolve_list_via_view_or_backend(engine, hint))
        }
    } else {
        None
    };

    let successful_write_count = u32::from(created_cid.is_some());
    Outcome {
        edge: Some(normalized_edge),
        error_code,
        error_message: None,
        created_cid,
        list,
        completed_iterations: None,
        successful_write_count,
    }
}

/// Route a `<label>` listing through View 3 (`content_listing:<label>`)
/// when IVM is enabled and a view is registered for that label; falls back
/// to the backend label index otherwise (defense-in-depth for `without_ivm`
/// engines and for views that haven't been `create_view`-registered yet).
///
/// The View 3 path returns CIDs in the view's native sort order
/// (`createdAt` ascending with disambiguator for ties); the fallback path
/// reads the label index and sorts in-memory by the same `createdAt`
/// property so the observable ordering matches across the two paths.
pub(crate) fn resolve_list_via_view_or_backend(engine: &Engine, label: &str) -> Vec<Node> {
    // Try View 3 first. View ids registered by the engine's `create_view`
    // use `"content_listing"` for the default "post" view (auto-registered
    // at assembly) and `"content_listing_<label>"` for any per-label view
    // registered via `register_crud`. Probe both shapes.
    if let Some(subscriber) = engine.ivm() {
        let view_id_candidates = [
            format!("content_listing_{label}"),
            "content_listing".to_string(),
        ];
        for view_id in &view_id_candidates {
            let query = benten_ivm::ViewQuery {
                label: Some(label.to_string()),
                limit: None,
                offset: None,
                ..Default::default()
            };
            match subscriber.read_view(view_id, &query) {
                Some(Ok(benten_ivm::ViewResult::Cids(cids))) if !cids.is_empty() => {
                    let mut out = Vec::new();
                    for cid in cids {
                        if let Ok(Some(node)) = engine.backend().get_node(&cid) {
                            out.push(node);
                        }
                    }
                    // View 3 sorts by createdAt; preserve that order. If any
                    // Node is missing (post-delete concurrency), it's just
                    // elided — the view is the source of truth for order.
                    return out;
                }
                // Fall through to the next candidate / backend fallback if
                // the view returned empty, errored (stale), or isn't
                // registered under that id.
                Some(Ok(_) | Err(_)) | None => {}
            }
        }
    }

    // Backend label-index fallback.
    let mut items: Vec<(i64, Node)> = Vec::new();
    if let Ok(cids) = engine.backend().get_by_label(label) {
        for cid in cids {
            if let Ok(Some(node)) = engine.backend().get_node(&cid) {
                let ts = match node.properties.get("createdAt") {
                    Some(Value::Int(i)) => *i,
                    #[allow(
                        clippy::cast_possible_truncation,
                        reason = "millisecond-epoch timestamps fit in i64"
                    )]
                    Some(Value::Float(f)) => *f as i64,
                    _ => 0,
                };
                items.push((ts, node));
            }
        }
    }
    items.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cid().ok().cmp(&b.1.cid().ok()))
    });
    items.into_iter().map(|(_, n)| n).collect()
}

// Capability conversion — used by dispatch to route ON_ERROR vs ON_DENIED.
pub(crate) fn cap_error_to_outcome(cap: &CapError) -> Outcome {
    // NotImplemented routes through ON_ERROR (operator config pointer) —
    // everything else is a denial through ON_DENIED. Conflating the two
    // makes Phase 3 operators audit their grants when the real problem is
    // backend selection. See r6-sec-4.
    let edge = match cap {
        CapError::NotImplemented { .. } => "ON_ERROR",
        _ => "ON_DENIED",
    };
    Outcome {
        edge: Some(edge.into()),
        error_code: Some(cap.code().as_str().to_string()),
        error_message: Some(cap.to_string()),
        ..Outcome::default()
    }
}

/// Map a graph `SystemZoneWrite` rejection to its user-facing `Outcome`.
pub(crate) fn system_zone_to_outcome() -> Outcome {
    Outcome {
        edge: Some("ON_ERROR".into()),
        error_code: Some("E_SYSTEM_ZONE_WRITE".into()),
        error_message: Some("system zone write rejected".into()),
        ..Outcome::default()
    }
}

/// Phase 2a G5-B-i mini-review C1: map an evaluator-raised Inv-11
/// (`EvalError::Invariant(SystemZone)`) to its user-facing `Outcome`.
///
/// Symmetric with [`system_zone_to_outcome`] (the Phase-1 storage-layer
/// stopgap shape) but fires the Phase-2a user-surface code
/// `E_INV_SYSTEM_ZONE` — matching `Engine::create_node`'s routing. This
/// is the shape `dispatch_call_inner` surfaces when
/// `impl PrimitiveHost::put_node` short-circuits a system-zone handler
/// WRITE before the `PendingHostOp` is buffered. The storage-layer stopgap
/// `system_zone_to_outcome` is unreachable through the evaluator path
/// under Phase 2a; it stays wired as defence-in-depth for direct
/// backend-level writes (exercised in `crates/benten-graph/tests/`).
pub(crate) fn inv_system_zone_to_outcome() -> Outcome {
    Outcome {
        edge: Some("ON_ERROR".into()),
        error_code: Some("E_INV_SYSTEM_ZONE".into()),
        error_message: Some("system-zone label not writable via user subgraph".into()),
        ..Outcome::default()
    }
}

/// Map an `inject_failure` evaluator result to its rollback `Outcome`.
pub(crate) fn tx_aborted_outcome() -> Outcome {
    Outcome {
        edge: Some("ON_ERROR".into()),
        error_code: Some("E_TX_ABORTED".into()),
        error_message: Some("transaction aborted due to injected failure".into()),
        ..Outcome::default()
    }
}

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
pub const WALLCLOCK_REFRESH_CEILING: std::time::Duration = std::time::Duration::from_mins(5);

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
    /// R6FP-Group-1 (r6-cr-1 / r6-mpc-4 / r6-wsa-1) — cumulative
    /// SANDBOX nest count along the active call chain. The value is
    /// `0` at the top-level handler entry; the engine's
    /// [`Engine::execute_sandbox`] override constructs the dispatching
    /// [`benten_eval::AttributionFrame`] with
    /// `sandbox_depth: parent.sandbox_depth + 1` so the eval-side
    /// runtime arm in `benten_eval::sandbox::execute` observes the
    /// correct nest depth and fires `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED`
    /// once the chain exceeds [`benten_eval::SandboxConfig::max_nest_depth`].
    /// Pre-R6FP-G1 the field was missing and the override hardcoded
    /// `sandbox_depth: 1` literally — D20 runtime arm dormant.
    pub(crate) sandbox_depth: u8,
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

    /// R6-R3 r6-r3-arch-1: shared D10 read-only-snapshot enforcement helper.
    ///
    /// `Engine::from_snapshot_blob` constructs an engine whose backend is
    /// CID-pinned + structurally immutable. Every WRITE PrimitiveHost arm
    /// (`put_node`, `delete_node`; `put_edge` / `delete_edge` when those
    /// wire in Phase 3 — see docs/future/phase-3-backlog.md §1.1) MUST
    /// surface `E_BACKEND_READ_ONLY` when invoked
    /// against such an engine — both via the direct `engine_crud::*` API
    /// and via the dispatch-through-handler path that
    /// `engine.call(handler, ':...', ...)` exercises. The check fires
    /// BEFORE any `PendingHostOp` is buffered so the replay path never
    /// sees the violating op.
    ///
    /// `op_name` is included in the io::Error message body so logs name
    /// the specific WRITE arm that was rejected (e.g.
    /// `"backend is read-only: put_node rejected (snapshot-blob engine)"`).
    pub(crate) fn check_not_read_only_snapshot(
        &self,
        op_name: &'static str,
    ) -> Result<(), benten_eval::EvalError> {
        if self.is_read_only_snapshot() {
            return Err(benten_eval::EvalError::Host(benten_eval::HostError {
                code: benten_errors::ErrorCode::BackendReadOnly,
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    format!("backend is read-only: {op_name} rejected (snapshot-blob engine)"),
                )),
                context: Some(
                    "snapshot-blob engine constructed via Engine::from_snapshot_blob is \
                     user-write-immutable per D10 read-only contract"
                        .to_string(),
                ),
            }));
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
        // D10 read-only-snapshot enforcement: when the engine was
        // constructed via `Engine::from_snapshot_blob`, user-facing
        // WRITE primitives MUST surface `E_BACKEND_READ_ONLY` so a
        // handler dispatched via `engine.call(...)` cannot corrupt the
        // snapshot's canonical-bytes invariant. Mirrors the check in
        // `engine_crud::create_node` for the direct-call path; this
        // closes the gap on the dispatch-through-handler path that the
        // `snapshot_blob_round_trip.test.ts::"rejects writes (read-only
        // contract)"` pin exercises. The check fires BEFORE the
        // PendingHostOp is buffered so the replay path never sees the
        // violating write.
        //
        // R6-R3 r6-r3-arch-1: extracted to `check_not_read_only_snapshot`
        // so `delete_node` (and future `put_edge` / `delete_edge` when
        // they wire) enforce the contract symmetrically. Pre-extraction
        // PR #68 wired the put-direction only; `delete_node` silently
        // permitted deletes from a snapshot-blob engine via dispatched
        // handlers.
        self.check_not_read_only_snapshot("put_node")?;
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
        // R6-R3 r6-r3-arch-1 (MAJOR): D10 read-only-snapshot enforcement
        // mirrors `put_node`. Pre-fix PR #68 wired the put-direction
        // only — a handler dispatched via `engine.call(handler, ':delete',
        // ...)` against an Engine constructed via
        // `Engine::from_snapshot_blob` SILENTLY DELETED Nodes, bypassing
        // the read-only-contract guarantee. The check fires BEFORE the
        // PendingHostOp::DeleteNode is buffered so the replay path never
        // sees the violating delete. Symmetric to `put_node` via the
        // shared `check_not_read_only_snapshot` helper.
        self.check_not_read_only_snapshot("delete_node")?;
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

    fn emit_event(&self, name: &str, payload: Value) {
        // Wave-8h audit-gap fix — publish the EMIT through the engine's
        // dedicated EMIT broadcast channel so a handler with a
        // standalone EMIT primitive (no backing WRITE) produces an
        // observable event. The audit at
        // `.addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json`
        // surfaced that the prior no-op silently dropped the payload.
        //
        // The EMIT channel is structurally separate from
        // `ChangeBroadcast` — see `crate::emit_broadcast` module docs
        // for why we don't extend `benten_graph::ChangeEvent` with an
        // emit variant. Subscribers attach via
        // [`Engine::subscribe_emit_events`].
        self.inner
            .emit_broadcast
            .publish(&crate::emit_broadcast::EmitEvent {
                channel: name.to_string(),
                payload,
            });
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
        // Phase-3 G16-B-prime fp (consumer-audit closure of cor-1 /
        // cap-g16bp-3): thread the engine's configured device-DID-
        // attestation CID into the dual-shape (label+target_cid)
        // ReadContext branch so user-subgraph reads dispatch per-device
        // per D-PHASE-3-25. The two by_*_only constructor branches
        // populate device_cid via ReadContext::Default which is None;
        // typed callers can opt-in via the dual-shape branch.
        let ctx = match (label.is_empty(), target_cid) {
            (true, Some(cid)) => benten_caps::ReadContext::by_cid_only(*cid),
            (_, None) => benten_caps::ReadContext::by_label_only(label),
            (false, Some(cid)) => {
                let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
                benten_caps::ReadContext {
                    label: label.to_string(),
                    target_cid: Some(*cid),
                    device_cid,
                    ..Default::default()
                }
            }
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
            // Phase-3 G16-B-prime (§6.12 item 3): thread the engine's
            // configured device-DID-attestation CID into the
            // primitive-host check_capability path so heterogeneous
            // policies can dispatch per-device per D-PHASE-3-25.
            // `None` for legacy / non-attested engines.
            let device_cid = *benten_graph::MutexExt::lock_recover(&self.inner.device_cid);
            let ctx = benten_caps::CapWriteContext {
                label: required.to_string(),
                device_cid,
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

    /// Phase-2b Wave-8i: hand the dispatcher the engine's durable
    /// redb-backed [`SuspensionStore`](benten_eval::SuspensionStore) so
    /// WAIT primitives reached during a regular `Engine::call` walk
    /// persist envelopes + metadata into the same store
    /// `Engine::call_with_suspension` + `Engine::resume_with_meta`
    /// consult. Without this override, the dispatcher would fall back
    /// to the trait default (process-default singleton), which would
    /// silently lose envelopes across an engine drop and break
    /// cross-process resume.
    fn suspension_store(&self) -> std::sync::Arc<dyn benten_eval::SuspensionStore> {
        Engine::suspension_store(self)
    }

    /// Phase-2b Wave-8i fix-pass (w8i-wait-cag-02): hand the WAIT
    /// dispatcher the engine's monotonic-clock reading in milliseconds
    /// so `WaitMetadata.suspend_elapsed_ms` records a real start
    /// reference. The trait default returns `None`; without this
    /// override, the production `engine.call()` WAIT path stamped
    /// `suspend_elapsed_ms = None` and `resume_with_meta`'s deadline
    /// check `if let (Some(timeout), Some(start), Some(now)) = ...`
    /// silently never fired, disabling resume-time deadline
    /// enforcement on the regular-walk path.
    fn elapsed_ms(&self) -> Option<u64> {
        // `Duration::as_millis()` returns u128; saturate to u64 max for
        // the (theoretically reachable, practically never) >584-million-
        // year process uptime case rather than truncating with `as u64`
        // (which would silently wrap and trigger `cast_possible_truncation`
        // under 1.95 clippy).
        Some(
            u64::try_from(self.monotonic_source.elapsed_since_start().as_millis())
                .unwrap_or(u64::MAX),
        )
    }

    /// Phase-2b Wave-8i fix-pass (w8i-wait-cag-01): hand the WAIT
    /// dispatcher the principal CID the current dispatch is running
    /// under, drawn from the top of the engine's `active_call` stack.
    /// `dispatch_call(_, _, _, Some(principal))` pushes the caller's
    /// principal onto the stack; the trait override here surfaces it
    /// to `wait::evaluate_op` so the envelope's
    /// `resumption_principal_cid` is the caller-named CID rather than
    /// `BLAKE3(signal_name)`.
    ///
    /// The active-call stack is the same surface used by attribution
    /// frame construction (`engine.rs::dispatch_call_inner` line
    /// `actor_cid` lookup), so this accessor inherits the stack
    /// discipline that already exists for actor attribution.
    fn suspending_principal(&self) -> Option<benten_core::Cid> {
        let guard = benten_graph::MutexExt::lock_recover(&self.active_call);
        guard.last().and_then(|f| f.actor)
    }

    /// Phase-3 G19-E (wave-7b): TRANSFORM AST cache lookup.
    ///
    /// Closes `docs/future/phase-2-backlog.md` §9.2 by serving the
    /// pre-parsed [`benten_eval::expr::Expr`] for the supplied
    /// `node_id` from the engine's `crate::ast_cache::AstCache`. The
    /// cache is keyed on `(handler_cid, node_id)`; we resolve
    /// `handler_cid` from the top of the engine's `active_call` stack
    /// (the same surface attribution + `suspending_principal` use). On
    /// miss — including the absent-handler-frame case (defensive; should
    /// not happen on a real dispatch) and the
    /// `testing_force_reregister_with_different_cid` scenario (the test
    /// hook flips `handler_cid` without re-populating the cache) — we
    /// return `None` so the TRANSFORM executor falls through to the
    /// per-call parse path.
    fn cached_transform_ast(
        &self,
        node_id: &str,
    ) -> Option<std::sync::Arc<benten_eval::expr::Expr>> {
        let handler_cid = {
            let guard = benten_graph::MutexExt::lock_recover(&self.active_call);
            guard.last().and_then(|f| f.handler_cid)
        };
        let handler_cid = handler_cid?;
        self.inner.ast_cache.lookup(&handler_cid, node_id)
    }

    /// Phase 2b Wave-8b — engine-side SANDBOX dispatch.
    ///
    /// **wsa-w8b-1 fix-pass:** the trait-default body returns
    /// `EvalError::PrimitiveNotImplemented` so `NullHost`-driven unit
    /// tests keep behaving as they did pre-Wave-8b. The engine
    /// implementation MUST override the default to actually invoke
    /// `benten_eval::sandbox::execute(...)`. Without this override, the
    /// dispatcher at `crates/benten-eval/src/primitives/mod.rs:96`
    /// (`PrimitiveKind::Sandbox => host.execute_sandbox(op)`) collapses
    /// to the default's `Err(PrimitiveNotImplemented)` and the
    /// production-runtime gate the wave was meant to close —
    /// Compromise #4 "WASM runtime is compile-check only" — stays open
    /// at the engine boundary.
    ///
    /// ## Wiring
    ///
    /// 1. Read the SANDBOX OperationNode's `module` property (Text,
    ///    base32 CID). Decode via [`Cid::from_str`]; absent / malformed
    ///    → typed error.
    /// 2. Look up wasm bytes via
    ///    `Engine::module_bytes_for` (a crate-private accessor;
    ///    bytes are registered through
    ///    [`Engine::register_module_bytes`]). Missing → typed error.
    /// 3. Resolve [`benten_eval::sandbox::ManifestRef`]:
    ///    - `manifest` property (Text) → `Named` lookup against the
    ///      shared [`benten_eval::sandbox::ManifestRegistry`].
    ///    - `caps` property (List of Text) → `Inline` bundle with the
    ///      caps as the required set.
    ///    - Both absent → typed error (no manifest = no cap surface).
    /// 4. Build the [`benten_eval::sandbox::SandboxConfig`] starting
    ///    from `default()` and applying per-handler overrides from
    ///    OperationNode properties: `fuel`, `wallclock_ms`,
    ///    `output_limit` (Int).
    /// 5. Resolve grant caps from the active call's frame. Phase-2b
    ///    NoAuth posture: an empty cap-set when no policy is wired;
    ///    a real grant-cap lookup lands when Phase-3 UCAN ships.
    /// 6. Build the dispatching [`benten_eval::AttributionFrame`] from
    ///    the active-call frame (actor / handler / grant CIDs +
    ///    `sandbox_depth` increment for Inv-4).
    /// 7. Invoke [`benten_eval::sandbox::execute`] with the assembled
    ///    inputs. Map the returned
    ///    [`benten_eval::sandbox::SandboxError`] to
    ///    [`benten_eval::EvalError::Backend`] keyed on the typed
    ///    catalog code via
    ///    [`benten_eval::sandbox::SandboxError::code`] so the
    ///    downstream TS layer sees the stable
    ///    `E_SANDBOX_*` discriminants.
    /// 8. On success, route the executor's
    ///    [`benten_eval::sandbox::SandboxResult::output`] bytes onto a
    ///    [`benten_eval::StepResult`] with `edge_label = "ok"` so the
    ///    walker continues to the SANDBOX node's outgoing edge.
    ///
    /// ## Error mapping
    ///
    /// **Wave-8d-types refactor:** `SandboxError` propagates through
    /// the typed `EvalError::Sandbox` variant (added in
    /// `crates/benten-eval/src/lib.rs`); the wave-8b temporary
    /// `EvalError::Backend(format!("...({code})"))` shape is gone.
    /// The stable `E_SANDBOX_*` catalog code now survives the
    /// `EvalError → EngineError → napi → TS` pipeline cleanly because
    /// `EvalError::Sandbox(s).code()` dispatches to `s.code()` and the
    /// engine-side `eval_error_to_engine_error` catch-all preserves
    /// `other.code()` on `EngineError::Other`.
    ///
    /// Module-lookup failures (no bytes registered for the declared
    /// CID) surface specifically through
    /// `SandboxError::ModuleNotInstalled` — distinct from
    /// `SandboxError::ModuleInvalid` (bytes present but failed
    /// wasmtime structural validation) — and route to
    /// [`ErrorCode::SandboxModuleNotInstalled`] (the operator-actionable
    /// `E_SANDBOX_MODULE_NOT_INSTALLED` discriminant).
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(
        clippy::too_many_lines,
        reason = "The 8-step plan (read property → look up bytes → resolve manifest → \
                  build config → resolve grant caps → build attribution → invoke executor → \
                  map result) is intentionally readable top-to-bottom; splitting it into \
                  helpers would obscure the wave-8b wiring narrative."
    )]
    fn execute_sandbox(
        &self,
        op: &benten_eval::OperationNode,
    ) -> Result<benten_eval::StepResult, benten_eval::EvalError> {
        // 1. Read the `module` property (Text, base32 CID).
        let module_cid_str = match op.properties.get("module") {
            Some(Value::Text(s)) => s.as_str(),
            Some(_) => {
                return Err(benten_eval::EvalError::Backend(
                    "SANDBOX: `module` property must be Text (base32 CID); got non-Text"
                        .to_string(),
                ));
            }
            None => {
                return Err(benten_eval::EvalError::Backend(
                    "SANDBOX: `module` property missing — handler did not declare a module CID"
                        .to_string(),
                ));
            }
        };
        let module_cid = Cid::from_str(module_cid_str).map_err(|e| {
            benten_eval::EvalError::Backend(format!(
                "SANDBOX: `module` property is not a valid base32 CID ({module_cid_str:?}): {e}"
            ))
        })?;

        // 2. Look up wasm bytes from the engine's in-memory module-bytes
        //    registry. Wave-8d-types: surface the typed
        //    `SandboxError::ModuleNotInstalled` variant rather than the
        //    prior placeholder `EvalError::Backend(format!(...))` shape so
        //    `E_SANDBOX_MODULE_NOT_INSTALLED` survives the boundary.
        let module_bytes = self.module_bytes_for(&module_cid).ok_or_else(|| {
            benten_eval::EvalError::Sandbox(benten_eval::sandbox::SandboxError::ModuleNotInstalled(
                module_cid,
            ))
        })?;

        // 3. Resolve the manifest reference. Named takes precedence; the
        //    `caps` inline list is the fallback escape hatch when no
        //    manifest is named.
        let manifest_ref = if let Some(Value::Text(name)) = op.properties.get("manifest") {
            benten_eval::sandbox::ManifestRef::named(name.clone())
        } else if let Some(Value::List(items)) = op.properties.get("caps") {
            let caps: Vec<String> = items
                .iter()
                .filter_map(|v| match v {
                    Value::Text(s) => Some(s.clone()),
                    _ => None,
                })
                .collect();
            benten_eval::sandbox::ManifestRef::Inline(benten_eval::sandbox::CapBundle::new(
                caps, None,
            ))
        } else {
            return Err(benten_eval::EvalError::Backend(
                "SANDBOX: neither `manifest` (named) nor `caps` (inline) property present — \
                 a SANDBOX node must declare its capability surface"
                    .to_string(),
            ));
        };

        // 4. Construct SandboxConfig + apply per-handler property
        //    overrides. The overrides match the property keys
        //    `SandboxNodeDescription` documents (`fuel`, `wallclock_ms`,
        //    `output_limit`).
        let mut config = benten_eval::sandbox::SandboxConfig::default();
        if let Some(Value::Int(fuel)) = op.properties.get("fuel") {
            config.fuel = u64::try_from(*fuel).unwrap_or(config.fuel);
        }
        if let Some(Value::Int(ms)) = op.properties.get("wallclock_ms") {
            // Use the typed setter so values above the D24 ceiling are
            // rejected with the typed `E_SANDBOX_WALLCLOCK_INVALID`
            // error rather than silently clamped.
            config = config
                .with_wallclock_ms(u64::try_from(*ms).unwrap_or(0))
                .map_err(|code| {
                    benten_eval::EvalError::Backend(format!(
                        "SANDBOX: per-handler wallclock_ms invalid ({ms}): {code:?}"
                    ))
                })?;
        }
        if let Some(Value::Int(limit)) = op.properties.get("output_limit") {
            config.output_bytes = u64::try_from(*limit).unwrap_or(config.output_bytes);
        }
        // Phase-3 G17-A2 — per-manifest `random` host-fn budget override
        // (additive optional `host_fns.random.budget_bytes_per_call` on
        // `ModuleManifest`; per r1-wsa-8). Resolution flows through
        // `Engine::random_budget_for_named_manifest` for Named manifests;
        // Inline manifests cannot carry overrides (no parent manifest
        // to attach to) and fall through to the codegen default
        // (4096 bytes/call).
        if let Some(Value::Text(name)) = op.properties.get("manifest") {
            config.random_budget_bytes_per_call =
                self.random_budget_for_named_manifest(name.as_str());
        }

        // 5. Resolve grant caps. Phase-2b NoAuth posture: when no
        //    capability policy is wired, an empty grant cap-set means
        //    the bundle's caps must already be a subset of the empty
        //    set — which would deny every cap declaration. To preserve
        //    the existing NoAuth narrative ("every commit permitted"),
        //    we surface the manifest's declared caps AS the grant
        //    cap-set under NoAuth. Phase-3 wires the real grant lookup
        //    via the dispatching frame's `capability_grant_cid`.
        //
        //    The cap-intersection inside `sandbox::execute` then sees
        //    grant ⊇ manifest, so D7 init-snapshot intersection
        //    succeeds. A test that wants to drive a denied-cap path
        //    constructs an inline manifest claiming a cap the test's
        //    grant set does NOT carry; that path lives in
        //    `crates/benten-eval/tests/sandbox_escape_attempts_denied.rs`
        //    today and exercises the executor directly without the
        //    engine wrapper.
        // Wave-8h audit-gap fix: hydrate the registry from the engine's
        // installed-modules active set so `manifest: '<installed-name>'`
        // resolves through the production SANDBOX path. Prior to wave-8h
        // every callsite constructed `ManifestRegistry::new()` (codegen
        // defaults only), so install_module persisted state that
        // execute_sandbox never consulted.
        let grant_caps = match self.policy() {
            // NoAuth path — synthesise grant ⊇ manifest. See comment
            // above. This collapses to a trivially-permissive grant
            // surface, matching the rest of the Phase-2b NoAuth posture.
            None => {
                let registry = self.manifest_registry();
                match manifest_ref.resolve(&registry) {
                    Ok(bundle) => bundle.caps.clone(),
                    Err(_) => Vec::new(),
                }
            }
            // Phase-3 hook — real cap lookup against the dispatching
            // grant's `capability_grant_cid` lands here. For Phase-2b
            // we use the same NoAuth-equivalent path until Phase-3
            // UCAN wires the real grant store.
            Some(_) => {
                let registry = self.manifest_registry();
                match manifest_ref.resolve(&registry) {
                    Ok(bundle) => bundle.caps.clone(),
                    Err(_) => Vec::new(),
                }
            }
        };

        // 6. Build the dispatching AttributionFrame. D20-RESOLVED:
        //    `sandbox_depth` increments at every SANDBOX entry.
        //
        //    R6FP-Group-1 (r6-cr-1 / r6-mpc-4 / r6-wsa-1): thread
        //    `parent.sandbox_depth + 1` from the active-call frame so
        //    the eval-side runtime arm in `benten_eval::sandbox::execute`
        //    observes the correct cumulative depth. Pre-R6FP-G1 this
        //    was hardcoded to literal `1` at both branches and the
        //    runtime arm could never fire (depth never exceeded
        //    `max_nest_depth`). The 3-lens convergent finding (R6
        //    code-reviewer + metadata-producer-vs-consumer + wasmtime-
        //    sandbox-auditor) named this as the load-bearing fix that
        //    closes the Inv-4 / D20 dormant-arm gap.
        //
        //    Bump the ActiveCall's sandbox_depth on every SANDBOX
        //    entry so a chain handler1→CALL→handler2→CALL→...→handlerN
        //    where each handler runs a SANDBOX can deepen: the outer
        //    SANDBOX's bump is observed by the subsequent CALL push
        //    (which inherits parent.sandbox_depth via the
        //    `parent_sandbox_depth = guard.last().map_or(0, |f| f.sandbox_depth)`
        //    read in `crates/benten-engine/src/engine.rs::dispatch_call_inner`
        //    immediately before the new ActiveCall push),
        //    so each nested handler's SANDBOX sees a higher depth than
        //    its predecessor. This is the chain shape the runtime arm
        //    is designed to defend (Inv-4 / D20 read-side enforcement).
        let attribution = {
            let mut guard = self.active_call().lock_recover();
            let zero = Cid::from_blake3_digest([0u8; 32]);
            let frame_snapshot = guard.last().map(|frame| {
                (
                    frame.actor.unwrap_or_else(noauth_pseudo_actor_cid),
                    frame.handler_cid.unwrap_or(zero),
                )
            });
            // Persist the bump on the parent ActiveCall so any
            // subsequent CALL primitive in this handler pushes a child
            // frame inheriting the bumped depth. This is the
            // load-bearing semantic for the Inv-4 runtime arm: the
            // depth grows along the (handler→CALL→inner)-with-SANDBOX
            // chain even though SANDBOX guest code itself can't drive
            // a nested dispatch (D19 blocks that path).
            let nested_depth = if let Some(frame) = guard.last_mut() {
                frame.sandbox_depth = frame.sandbox_depth.saturating_add(1);
                frame.sandbox_depth
            } else {
                1
            };
            match frame_snapshot {
                Some((actor, handler)) => benten_eval::AttributionFrame {
                    actor_cid: actor,
                    handler_cid: handler,
                    capability_grant_cid: noauth_zero_grant_cid(),
                    sandbox_depth: nested_depth,
                    ..Default::default()
                },
                None => benten_eval::AttributionFrame {
                    actor_cid: noauth_pseudo_actor_cid(),
                    handler_cid: zero,
                    capability_grant_cid: noauth_zero_grant_cid(),
                    sandbox_depth: nested_depth,
                    ..Default::default()
                },
            }
        };

        // 7. Invoke the executor. Wave-8h audit-gap fix: hydrate the
        //    registry from installed_modules so Named-manifest dispatch
        //    consults install_module persisted state (the prior
        //    `ManifestRegistry::new()` carried only codegen defaults).
        //
        //    Phase-3 wave-5c §6.1-followup task #5 — construct the live
        //    cap-recheck callback. The callback observes the engine's
        //    revoked-actors set (cloned `Arc<Mutex<HashSet<Cid>>>`) on
        //    every invocation: if the dispatching actor has been
        //    revoked since SANDBOX entry, EVERY cap-string returns
        //    `false` (the actor has no caps). Otherwise, falls back to
        //    the grant_caps snapshot (Phase-2b NoAuth posture). Closes
        //    ESC-9 r1-wsa-3 MAJOR end-to-end. The cadence is fires-on-
        //    every-host-fn-invocation (cadence (a) per r4-r1-wsa-4).
        let registry = self.manifest_registry();
        let live_cap_check: Option<benten_eval::sandbox::LiveCapCheck> = {
            let revoked = self.inner.revoked_actors_arc();
            let actor = attribution.actor_cid;
            let snapshot: Vec<String> = grant_caps.clone();
            let callback: benten_eval::sandbox::LiveCapCheck =
                std::sync::Arc::new(move |cap: &str| -> bool {
                    let revoked_set = revoked
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    if revoked_set.contains(&actor) {
                        // Dispatching actor revoked mid-call: every cap
                        // returns false (host-fn denies). The next host-fn
                        // invocation observes the revocation; see ESC-9
                        // closure narrative in `docs/SECURITY-POSTURE.md`.
                        return false;
                    }
                    drop(revoked_set);
                    snapshot.iter().any(|c| c == cap)
                });
            Some(callback)
        };
        // Phase-3 G19-C2 wave-7 (§7.1 SANDBOX execution metrics
        // propagation): capture the resolved per-call limits + the
        // dispatching handler-id BEFORE the executor call so we can
        // populate the engine-side high-water tracker on success.
        // Manifest-id captured as the named-manifest string (None for
        // the inline `caps` escape hatch).
        let metrics_handler_id: Option<String> = self
            .active_call()
            .lock_recover()
            .last()
            .map(|frame| frame.handler_id.clone());
        let metrics_manifest_id: Option<String> =
            if let Some(Value::Text(name)) = op.properties.get("manifest") {
                Some(name.clone())
            } else {
                None
            };
        let metrics_fuel = config.fuel;
        let metrics_wallclock_ms = config.wallclock_ms;
        let metrics_output_limit_bytes = config.output_bytes;
        let metrics_invocation_start = std::time::Instant::now();

        let result = benten_eval::sandbox::execute_with_live_cap_check(
            &module_bytes,
            manifest_ref,
            &registry,
            config,
            &grant_caps,
            &attribution,
            live_cap_check,
        );

        // 8. Map the result. Wave-8d-types: SandboxError propagates
        //    through the typed `EvalError::Sandbox` variant (via the
        //    `#[from]` impl), preserving the stable `E_SANDBOX_*`
        //    catalog code for the downstream TS layer.
        match result {
            Ok(sandbox_result) => {
                // Phase-3 G19-C2 wave-7 (§7.1): record per-invocation
                // measurements + bump the per-handler high-water mark
                // so `Engine::describe_sandbox_node_for_handler` returns
                // real metrics.
                if let Some(handler_id) = metrics_handler_id {
                    let elapsed_ms = u64::try_from(metrics_invocation_start.elapsed().as_millis())
                        .unwrap_or(u64::MAX);
                    self.inner.record_sandbox_metric(
                        &handler_id,
                        crate::engine::SandboxNodeMetrics {
                            module_cid: Some(module_cid),
                            manifest_id: metrics_manifest_id,
                            fuel: metrics_fuel,
                            wallclock_ms: metrics_wallclock_ms,
                            output_limit_bytes: metrics_output_limit_bytes,
                            fuel_consumed_high_water: Some(sandbox_result.fuel_consumed),
                            output_consumed_high_water: Some(sandbox_result.output_consumed),
                            last_invocation_ms: Some(elapsed_ms),
                        },
                    );
                }
                Ok(benten_eval::StepResult {
                    next: None,
                    edge_label: "ok".to_string(),
                    output: Value::Bytes(sandbox_result.output),
                })
            }
            Err(sandbox_err) => Err(benten_eval::EvalError::Sandbox(sandbox_err)),
        }
    }

    /// wasm32-target stub: `benten_eval::sandbox` is cfg-gated off on
    /// `target_arch = "wasm32"` (wasmtime doesn't compile to wasm32),
    /// so the production override is unreachable. Surface the typed
    /// E_SANDBOX_UNAVAILABLE_ON_WASM-class error per wsa-14 / Compromise
    /// #N+9 (browser-target SANDBOX disabled) so the DSL composition
    /// flow reports the actionable error at execution time rather than
    /// fielding a missing-symbol link error at module load.
    ///
    /// **Wave-8d-types refactor:** the typed
    /// `EvalError::SubsystemDisabled` variant carries the
    /// [`ErrorCode::SubsystemDisabled`] catalog code through the
    /// `EvalError → EngineError → napi → TS` pipeline (replaces the
    /// prior `EvalError::Backend(format!(...))` placeholder shape).
    /// The dedicated `ErrorCode::SandboxUnavailableOnWasm` variant
    /// remains 8c-cont scope (it would let the wasm32 stub fire its
    /// own discriminant); for now the actionable wsa-14 text rides on
    /// the `SubsystemDisabled` envelope and remains pinned by
    /// `tests/sandbox_unavailable_on_wasm_error_message_exact_text_pin.rs`.
    #[cfg(target_arch = "wasm32")]
    fn execute_sandbox(
        &self,
        _op: &benten_eval::OperationNode,
    ) -> Result<benten_eval::StepResult, benten_eval::EvalError> {
        Err(benten_eval::EvalError::SubsystemDisabled(
            "SANDBOX is unavailable on wasm32-unknown-unknown — \
             the browser engine ships without wasmtime. \
             E_SANDBOX_UNAVAILABLE_ON_WASM (wsa-14)."
                .to_string(),
        ))
    }

    /// Phase-3 G21-T1 — engine-side typed-CALL dispatch.
    ///
    /// Wires the 10 typed-CALL ops to their underlying implementations
    /// in `benten-id` (Ed25519 / DID / UCAN / VC) + `benten-core`
    /// (BLAKE3 / multibase). Per CLAUDE.md baked-in commitment #16:
    /// SANDBOX is for compute that does NOT fit other primitives —
    /// crypto ops fit CALL because they're input → typed result, no
    /// side effects on engine state. The 12-primitive commitment (#1)
    /// holds; typed-CALL is dispatched THROUGH the existing CALL
    /// primitive when its `target` starts with `engine:typed:`.
    ///
    /// The cap-check has already fired in the eval-side
    /// `execute_typed_call` (via `host.check_capability`); this method
    /// is invoked AFTER the gate clears. A clean negative result (e.g.
    /// `Ed25519Verify` returns `valid: false`) is NOT a dispatch error
    /// — it's a structured `{ valid: false }` Map. Only well-formed
    /// op-internal failures (malformed key bytes / corrupted UCAN
    /// envelope / unsupported DID method) surface as
    /// `EvalError::TypedCallDispatchError`.
    #[cfg(not(target_arch = "wasm32"))]
    fn dispatch_typed_call(
        &self,
        op: benten_eval::TypedCallOp,
        input: &Value,
    ) -> Result<Value, benten_eval::EvalError> {
        crate::typed_call_dispatch::dispatch(op, input)
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
        // Wave-8d-types: typed `EvalError::Sandbox` round-trips with the
        // stable `E_SANDBOX_*` catalog code on `EngineError::Other.code`
        // PLUS the actionable Display rendering on `.message` (so
        // operator-facing log lines + the Wave-8b acceptance test's
        // substring assertion both see the user-friendly text rather
        // than the bare Debug payload).
        #[cfg(not(target_arch = "wasm32"))]
        benten_eval::EvalError::Sandbox(s) => EngineError::Other {
            code: s.code(),
            message: format!("{s}"),
        },
        // Phase-2b Wave-8i: WAIT-suspended is a control-flow signal —
        // route to the typed `EngineError::WaitSuspended` variant so
        // callers can pattern-match the carried `SuspendedHandle`
        // without parsing message strings.
        benten_eval::EvalError::WaitSuspended { handle } => EngineError::WaitSuspended { handle },
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

/// Return the placeholder grant CID used while the capability backend is
/// `NoAuthBackend` (Phase 1/2a). Every NoAuth write/dispatch attributes to
/// the same all-zero `Cid` because the backend issues no real grant entities;
/// once Phase-3 wires UCAN, the call sites that currently invoke
/// `noauth_zero_grant_cid()` flip to the real grant CID their backend stamps.
///
/// R6 round-2 C2-R2-4: prior to centralisation the zero-grant placeholder
/// was open-coded as `Cid::from_blake3_digest([0u8; 32])` at multiple
/// dispatch + WRITE sites. Folding it into a named helper next to
/// `noauth_pseudo_actor_cid` gives Phase-3 wiring a single grep target
/// when the time comes to swap the placeholder out for a real grant CID.
pub(crate) fn noauth_zero_grant_cid() -> Cid {
    Cid::from_blake3_digest([0u8; 32])
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

/// G12-A: trace-mode error outcome for an Inv-8 cumulative-step budget
/// exhaustion. Lets `engine.trace(...)` return the captured trace (which
/// carries the terminal `TraceStep::BudgetExhausted` row) instead of
/// dropping it on an `Err` propagation. The non-trace `engine.call` path
/// still surfaces the typed `EngineError::Invariant(IterateBudget)`.
pub(crate) fn inv_iterate_budget_to_outcome() -> Outcome {
    Outcome {
        edge: Some("ON_ERROR".into()),
        error_code: Some("E_INV_ITERATE_BUDGET".into()),
        error_message: Some(
            "iteration step budget exhausted; trace carries terminal BudgetExhausted row".into(),
        ),
        ..Outcome::default()
    }
}

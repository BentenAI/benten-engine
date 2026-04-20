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

use std::sync::Arc;

use benten_caps::CapError;
use benten_core::{Cid, Edge, Node, Value};
use benten_eval::PrimitiveHost;
use benten_graph::{ChangeEvent, GraphError, MutexExt};

use crate::engine::{Engine, is_known_view_id};
use crate::error::EngineError;
use crate::outcome::Outcome;

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
    PutEdge {
        edge: Edge,
        projected_cid: Cid,
    },
    DeleteEdge {
        cid: Cid,
    },
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
        Ok(self.backend().get_node(cid)?)
    }

    fn get_by_label(&self, label: &str) -> Result<Vec<Cid>, benten_eval::EvalError> {
        Ok(self.backend().get_by_label(label)?)
    }

    fn get_by_property(
        &self,
        label: &str,
        prop: &str,
        value: &Value,
    ) -> Result<Vec<Cid>, benten_eval::EvalError> {
        Ok(self.backend().get_by_property(label, prop, value)?)
    }

    fn put_node(&self, node: &Node) -> Result<Cid, benten_eval::EvalError> {
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
            let actor_cid = frame
                .actor
                .clone()
                .or_else(|| Some(noauth_pseudo_actor_cid()));
            let handler_cid = frame.handler_cid.clone();
            frame.pending_ops.push(PendingHostOp::PutNode {
                node: node.clone(),
                projected_cid: projected.clone(),
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
            Ok(self.backend().put_node(node)?)
        }
    }

    fn put_edge(&self, edge: &Edge) -> Result<Cid, benten_eval::EvalError> {
        let projected = edge.cid()?;
        let mut guard = self.active_call().lock_recover();
        if let Some(frame) = guard.last_mut() {
            frame.pending_ops.push(PendingHostOp::PutEdge {
                edge: edge.clone(),
                projected_cid: projected.clone(),
            });
            Ok(projected)
        } else {
            drop(guard);
            Ok(self.backend().put_edge(edge)?)
        }
    }

    fn delete_node(&self, cid: &Cid) -> Result<(), benten_eval::EvalError> {
        let mut guard = self.active_call().lock_recover();
        if let Some(frame) = guard.last_mut() {
            frame
                .pending_ops
                .push(PendingHostOp::DeleteNode { cid: cid.clone() });
            Ok(())
        } else {
            drop(guard);
            self.backend().delete_node(cid)?;
            Ok(())
        }
    }

    fn delete_edge(&self, cid: &Cid) -> Result<(), benten_eval::EvalError> {
        let mut guard = self.active_call().lock_recover();
        if let Some(frame) = guard.last_mut() {
            frame
                .pending_ops
                .push(PendingHostOp::DeleteEdge { cid: cid.clone() });
            Ok(())
        } else {
            drop(guard);
            self.backend().delete_edge(cid)?;
            Ok(())
        }
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
            Err(EngineError::Cap(c)) => Err(benten_eval::EvalError::Capability(c)),
            Err(e) => Err(benten_eval::EvalError::Backend(format!("{e:?}"))),
        }
    }

    fn emit_event(&self, _name: &str, _payload: Value) {
        // Phase-1 EMIT is a no-op at the host level — the change-broadcast
        // fan-out is already wired to storage WRITEs; standalone EMIT
        // primitives without a backing store mutation don't carry a
        // ChangeEvent payload shape yet. Reserved for Phase-2.
    }

    fn check_capability(
        &self,
        required: &str,
        _target: Option<&Cid>,
    ) -> Result<(), benten_eval::EvalError> {
        // Phase-1: capability gating runs at tx-commit via the policy's
        // check_write hook. A per-primitive check is a no-op here; once
        // per-primitive `requires:` enforcement lands (Phase-2), this
        // threads through the configured policy.
        if let Some(policy) = self.policy() {
            // Pass a shape the policy can inspect; we only populate the
            // `label` slot with the requested scope so a policy that keys
            // off write-labels sees it.
            let ctx = benten_caps::WriteContext {
                label: required.to_string(),
                ..Default::default()
            };
            if let Err(c) = policy.check_write(&ctx) {
                return Err(benten_eval::EvalError::Capability(c));
            }
        }
        Ok(())
    }

    fn read_view(
        &self,
        view_id: &str,
        _query: &benten_eval::ViewQuery,
    ) -> Result<Value, benten_eval::EvalError> {
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
            Err(e) => Err(benten_eval::EvalError::Backend(format!("{e:?}"))),
        }
    }
}

// ---------------------------------------------------------------------------
// EvalError ↔ EngineError conversion + attribution helpers
// ---------------------------------------------------------------------------

/// Convert an `EvalError` back into an `EngineError` for the transaction
/// closure's return type.
pub(crate) fn eval_error_to_engine_error(e: benten_eval::EvalError) -> EngineError {
    match e {
        benten_eval::EvalError::Capability(c) => EngineError::Cap(c),
        benten_eval::EvalError::Graph(g) => EngineError::Graph(g),
        benten_eval::EvalError::Core(c) => EngineError::Core(c),
        benten_eval::EvalError::Backend(m) => EngineError::Other {
            code: benten_core::ErrorCode::Unknown("E_BACKEND".into()),
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
            if let Some((scan_label, b32)) = rest.split_once(':') {
                if let Ok(cids) = engine.backend().get_by_label(scan_label) {
                    if let Some(cid) = cids.into_iter().find(|c| c.to_base32() == b32) {
                        if let Ok(Some(node)) = engine.backend().get_node(&cid) {
                            out.push(node);
                        }
                    }
                }
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

/// Map an `inject_failure` evaluator result to its rollback `Outcome`.
pub(crate) fn tx_aborted_outcome() -> Outcome {
    Outcome {
        edge: Some("ON_ERROR".into()),
        error_code: Some("E_TX_ABORTED".into()),
        error_message: Some("transaction aborted due to injected failure".into()),
        ..Outcome::default()
    }
}

// Touch unused imports to keep them available when downstream expansion needs
// them (GraphError + ChangeEvent + Arc are all used by the engine.rs consumer
// of this module; keeping them imported here mirrors the original layout).
#[allow(dead_code)]
fn _keep_imports(_g: Option<GraphError>, _c: Option<ChangeEvent>, _a: Option<Arc<()>>) {}

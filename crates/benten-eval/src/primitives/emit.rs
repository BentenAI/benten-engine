//! EMIT primitive executor.
//!
//! EMIT is a fire-and-forget change notification: the primitive schedules a
//! message onto the engine's change broadcast and immediately continues on
//! the `"ok"` evaluator edge. It does not block, does not wait for
//! acknowledgement, and does not surface subscriber failures to the caller.
//!
//! Per ENGINE-SPEC §3.9, EMIT is classified non-deterministic (see
//! [`PrimitiveKind::is_deterministic`](crate::PrimitiveKind::is_deterministic))
//! because it couples the handler to observer side effects that the engine
//! cannot replay. Its determinism classification matters for invariant 9 —
//! EMIT cannot appear inside a `deterministic`-declared subgraph.
//!
//! The Phase-1 executor is property-driven: `channel` and `payload`
//! operation-node properties describe the intended message, and the
//! executor returns `Value::Null` on the output edge so the evaluator
//! doesn't thread a value forward. The real `ChangeBroadcast` wiring lands
//! alongside the engine handle in G7; until then this executor honours the
//! fire-and-forget edge contract without touching the broadcast.
//!
//! EMIT's typed error-edge set (`ON_ERROR`) is advertised by
//! [`PrimitiveKind::error_edges`](crate::PrimitiveKind::error_edges) for
//! validator use, but the Phase-1 executor never routes there: a failed
//! broadcast deliver is swallowed by design (fire-and-forget).

use benten_core::Value;

use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

/// Execute an EMIT primitive.
///
/// Routes the emit through [`PrimitiveHost::emit_event`] so the engine's
/// change-broadcast can fan it out. Returns `"ok"` unconditionally; EMIT is
/// fire-and-forget and never surfaces subscriber failures.
///
/// # Errors
///
/// EMIT does not currently surface any error variants; the function
/// signature preserves the dispatcher shape used by the other executors.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    if let Some(Value::Text(channel)) = op.properties.get("channel") {
        let payload = op.properties.get("payload").cloned().unwrap_or(Value::Null);
        host.emit_event(channel, payload);
    }
    Ok(StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::Null,
    })
}

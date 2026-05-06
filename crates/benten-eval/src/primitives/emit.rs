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
/// # G14-D wave-5a — handler-id-router seam (seq-major-8 + stream-r1-2)
///
/// When the OperationNode carries a `handler: Text(handler_id)`
/// property, the EMIT routes THROUGH the named handler subgraph
/// instead of the default fan-out. The host's
/// [`PrimitiveHost::call_handler`] is invoked with the channel as the
/// op name; any side effects (probe writes, RESPOND values) are
/// observably attributable to the named handler. The default
/// fan-out broadcast is SUPPRESSED for this event — the routing
/// decision must produce observably different traces per
/// stream-r1-2 LOAD-BEARING.
///
/// # Errors
///
/// EMIT's default fan-out arm never surfaces error variants. The
/// `Named(handler_id)` arm surfaces [`EvalError::Backend`] when the
/// host's `call_handler` rejects (handler not registered, etc.) but
/// preserves fire-and-forget on success.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
    if let Some(Value::Text(channel)) = op.properties.get("channel") {
        let payload = op.properties.get("payload").cloned().unwrap_or(Value::Null);
        // G14-D wave-5a: handler-id-router seam (seq-major-8 +
        // stream-r1-2). When `handler: Text(id)` is present, route
        // through the named handler subgraph instead of default
        // fan-out — observably different runtime trace.
        if let Some(Value::Text(handler_id)) = op.properties.get("handler") {
            // Construct a synthetic "input" Node carrying the channel
            // name + payload. The named handler reads them as
            // properties on the entrypoint Node.
            use benten_core::Node;
            use std::collections::BTreeMap;
            let mut props: BTreeMap<String, Value> = BTreeMap::new();
            props.insert("channel".into(), Value::Text(channel.clone()));
            props.insert("payload".into(), payload.clone());
            let input = Node::new(vec!["EmitInput".into()], props);
            let _ = host.call_handler(handler_id.as_str(), channel.as_str(), input)?;
            return Ok(StepResult {
                next: None,
                edge_label: "ok".to_string(),
                output: Value::Null,
            });
        }
        host.emit_event(channel, payload);
    }
    Ok(StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::Null,
    })
}

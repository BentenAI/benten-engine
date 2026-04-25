//! Trace projection for TypeScript.
//!
//! Phase 2a G11-A Wave 2b — TraceStep unification: each emitted step is a
//! discriminated union mirroring [`benten_engine::TraceStep`]. The wire
//! shape carries a `type` discriminant so TS callers can `switch` on the
//! variant exhaustively. The four variants:
//!
//! - `{ type: "primitive", nodeCid, durationUs, primitive, nodeId, inputs?,
//!    outputs?, error?, attribution? }`
//! - `{ type: "suspend_boundary", stateCid }`
//! - `{ type: "resume_boundary", stateCid, signalValue }`
//! - `{ type: "budget_exhausted", budgetType, consumed, limit, path }`
//!
//! Top-level shape: `{ steps: [...], result? }`.
//!
//! Pre-Wave-2b shape (`{ nodeCid, durationUs, primitive }` per step) is gone;
//! per CLAUDE.md §5 no compatibility shims, callers consume the new union.

use benten_engine::{Trace, TraceStep};

use crate::node::value_to_json;
use crate::subgraph::outcome_to_json;

#[allow(
    clippy::too_many_lines,
    reason = "single-function dispatch over the four TraceStep variants is the simplest read; splitting per-variant helpers would scatter the discriminant-name string literals across the file."
)]
fn trace_step_to_json(step: &TraceStep) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    match step {
        TraceStep::Step {
            duration_us,
            node_cid,
            primitive,
            node_id,
            inputs,
            outputs,
            error,
            attribution,
        } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("primitive".to_string()),
            );
            obj.insert(
                "nodeCid".to_string(),
                serde_json::Value::String(node_cid.to_base32()),
            );
            obj.insert(
                "durationUs".to_string(),
                serde_json::Value::Number((*duration_us).into()),
            );
            obj.insert(
                "primitive".to_string(),
                serde_json::Value::String(primitive.clone()),
            );
            obj.insert(
                "nodeId".to_string(),
                serde_json::Value::String(node_id.clone()),
            );
            obj.insert("inputs".to_string(), value_to_json(inputs));
            obj.insert("outputs".to_string(), value_to_json(outputs));
            if let Some(code) = error {
                obj.insert(
                    "error".to_string(),
                    serde_json::Value::String(code.as_str().to_string()),
                );
            }
            if let Some(attr) = attribution {
                let mut a = serde_json::Map::new();
                a.insert(
                    "actorCid".to_string(),
                    serde_json::Value::String(attr.actor_cid.to_base32()),
                );
                a.insert(
                    "handlerCid".to_string(),
                    serde_json::Value::String(attr.handler_cid.to_base32()),
                );
                a.insert(
                    "capabilityGrantCid".to_string(),
                    serde_json::Value::String(attr.capability_grant_cid.to_base32()),
                );
                obj.insert("attribution".to_string(), serde_json::Value::Object(a));
            }
        }
        TraceStep::SuspendBoundary { state_cid } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("suspend_boundary".to_string()),
            );
            obj.insert(
                "stateCid".to_string(),
                serde_json::Value::String(state_cid.to_base32()),
            );
        }
        TraceStep::ResumeBoundary {
            state_cid,
            signal_value,
        } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("resume_boundary".to_string()),
            );
            obj.insert(
                "stateCid".to_string(),
                serde_json::Value::String(state_cid.to_base32()),
            );
            obj.insert("signalValue".to_string(), value_to_json(signal_value));
        }
        TraceStep::BudgetExhausted {
            budget_type,
            consumed,
            limit,
            path,
        } => {
            obj.insert(
                "type".to_string(),
                serde_json::Value::String("budget_exhausted".to_string()),
            );
            obj.insert(
                "budgetType".to_string(),
                serde_json::Value::String((*budget_type).to_string()),
            );
            obj.insert(
                "consumed".to_string(),
                serde_json::Value::Number((*consumed).into()),
            );
            obj.insert(
                "limit".to_string(),
                serde_json::Value::Number((*limit).into()),
            );
            obj.insert(
                "path".to_string(),
                serde_json::Value::Array(
                    path.iter()
                        .map(|s| serde_json::Value::String(s.clone()))
                        .collect(),
                ),
            );
        }
    }
    serde_json::Value::Object(obj)
}

pub(crate) fn trace_to_json(trace: &Trace) -> serde_json::Value {
    let steps = trace.steps().iter().map(trace_step_to_json).collect();
    let mut out = serde_json::Map::new();
    out.insert("steps".to_string(), serde_json::Value::Array(steps));
    if let Some(outcome) = trace.outcome() {
        out.insert("result".to_string(), outcome_to_json(outcome));
    }
    serde_json::Value::Object(out)
}

//! Trace projection for TypeScript.
//!
//! Shape: `{ steps: [{ nodeCid: string, durationUs: number, primitive: string }, ...], result? }`.

use benten_engine::Trace;

use crate::subgraph::outcome_to_json;

pub(crate) fn trace_to_json(trace: &Trace) -> serde_json::Value {
    let steps = trace
        .steps()
        .iter()
        .map(|step| {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "nodeCid".to_string(),
                serde_json::Value::String(step.node_cid().to_base32()),
            );
            obj.insert(
                "durationUs".to_string(),
                serde_json::Value::Number(step.duration_us().into()),
            );
            obj.insert(
                "primitive".to_string(),
                serde_json::Value::String(step.primitive().to_string()),
            );
            serde_json::Value::Object(obj)
        })
        .collect();
    let mut out = serde_json::Map::new();
    out.insert("steps".to_string(), serde_json::Value::Array(steps));
    if let Some(outcome) = trace.outcome() {
        out.insert("result".to_string(), outcome_to_json(outcome));
    }
    serde_json::Value::Object(out)
}

//! ITERATE primitive executor.
//!
//! ITERATE walks a bounded input list and invokes a body subgraph for each
//! element. Phase 1's executor is property-driven (matching the other
//! primitives): the real body-subgraph dispatch wires up in G7 when the
//! engine handle is available. The contract this executor locks in is the
//! edge-routing shape the evaluator uses:
//!
//! - `items: List` — the iteration input. When omitted, the executor
//!   treats the input as empty.
//! - `max: Int` — the iteration cap. Required at *registration* time
//!   (enforced by [`SubgraphBuilder`](crate::SubgraphBuilder) — missing
//!   `max` is impossible to express in Phase 1, see
//!   [`ErrorCode::InvIterateMaxMissing`](benten_core::ErrorCode)). The
//!   executor additionally routes `ON_LIMIT` if the actual item count
//!   exceeds `max`.
//! - `parallel: Bool` — when `true`, the evaluator invokes the body on
//!   each element concurrently via `std::thread::scope`. Ordering of the
//!   accumulated result list is preserved.
//! - `batch_size: Int` — chunk size for capability re-checking
//!   (cooperating with `benten_caps::DEFAULT_BATCH_BOUNDARY`).
//!
//! Phase-1 named compromise #1: the capability re-check fires at
//! `DEFAULT_BATCH_BOUNDARY` (100 items) rather than per-iteration. The
//! engine's CapabilityPolicy supplies the batch boundary via
//! [`CapabilityPolicy::iterate_batch_boundary`](benten_caps::CapabilityPolicy::iterate_batch_boundary).
//!
//! The Phase-1 executor does not actually invoke a body subgraph (no engine
//! handle is in scope); instead it returns a sentinel `StepResult` whose
//! `output` summarises the intended iteration count for the evaluator trace.

use benten_core::Value;

use crate::{EvalError, OperationNode, StepResult};

/// Execute an ITERATE primitive.
///
/// # Errors
///
/// Returns `Ok` with the `ON_LIMIT` edge when the input exceeds `max`.
/// Never errors via `Err`; budget failures are routed through the typed
/// error edge so the engine's trace shows the overrun.
pub fn execute(op: &OperationNode) -> Result<StepResult, EvalError> {
    let items_len = match op.properties.get("items") {
        Some(Value::List(l)) => l.len(),
        _ => 0,
    };
    let max = op
        .properties
        .get("max")
        .and_then(|v| match v {
            Value::Int(i) => usize::try_from(*i).ok(),
            _ => None,
        })
        .unwrap_or(usize::MAX);

    if items_len > max {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_LIMIT".to_string(),
            output: Value::Null,
        });
    }

    // Happy-path result: an empty accumulator list. The real engine-backed
    // body invocation (G7) replaces this with the accumulated per-iteration
    // results.
    //
    // TODO(R4b / G7): staged-primitive observability — ensure a chained
    // `iterate(items, body).map(...)` handler produces non-empty output
    // once G7 wires body invocation. Add an integration test that pins
    // accumulated output. Mini-review `g6-cr-9` / `g6-cr-10`.
    Ok(StepResult {
        next: None,
        edge_label: "ok".to_string(),
        output: Value::List(Vec::new()),
    })
}

/// The capability re-check cadence for ITERATE bodies.
///
/// Re-exported from [`benten_caps::DEFAULT_BATCH_BOUNDARY`] so G7's
/// evaluator can pass it to long-running iteration loops; the constant
/// lives there so the capability layer owns the policy. See Phase 1 named
/// compromise #1.
pub const DEFAULT_BATCH_BOUNDARY: usize = benten_caps::DEFAULT_BATCH_BOUNDARY;

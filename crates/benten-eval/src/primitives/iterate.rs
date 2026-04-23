//! ITERATE primitive executor.
//!
//! ITERATE walks a bounded iteration counter and drives the capability
//! re-check cadence (Named Compromise #1). The contract this executor
//! locks in is the edge-routing shape the evaluator uses:
//!
//! - `items: List` — the iteration input. When omitted, the executor
//!   treats the input as empty.
//! - `max: Int` — the iteration cap. Required at *registration* time
//!   (enforced by [`SubgraphBuilder`](crate::SubgraphBuilder) — missing
//!   `max` is impossible to express in Phase 1, see
//!   [`ErrorCode::InvIterateMaxMissing`](benten_errors::ErrorCode)). The
//!   executor additionally routes `ON_LIMIT` if the actual item count
//!   exceeds `max`.
//! - `requires: Text` — the capability scope checked at every batch
//!   boundary (falls back to `"iterate"` when absent).
//!
//! Named Compromise #1 (ITERATE batch-boundary half): the capability
//! re-check fires at `host.iterate_batch_boundary()` (default 100) rather
//! than per-iteration. The executor walks the item count in lockstep and
//! every N iterations calls `host.check_capability` with the scope
//! declared on the operation. The entry refresh (iteration 0) always
//! fires, even when the iteration list is empty. A revocation landing
//! mid-batch is visible at the NEXT batch boundary; the denial routes
//! through the `ON_DENIED` typed edge with the policy's error code in the
//! edge payload. See
//! [`CapabilityPolicy::iterate_batch_boundary`](benten_caps::CapabilityPolicy::iterate_batch_boundary).
//!
//! The Phase-1 executor does not dispatch a body subgraph; iteration is
//! accounted for the cap-refresh cadence only. The returned `output` is
//! an empty accumulator list, so existing tests that pin the happy-path
//! output remain green.
//!
//! # Phase-2 deferred
//!
//! Parallel iteration (`parallel: Bool` via `std::thread::scope`) and
//! user-supplied per-op batch size (`batch_size: Int`) are NOT part of
//! the Phase-1 contract. The Phase-1 executor reads only `items`, `max`,
//! and `requires`; any additional property is ignored. Body-subgraph
//! dispatch with the accumulated-results shape is Phase-2 scope.

use benten_core::Value;

use crate::{EvalError, OperationNode, PrimitiveHost, StepResult};

/// Execute an ITERATE primitive.
///
/// Takes `&dyn PrimitiveHost` so the capability re-check cadence
/// (Named Compromise #1) is host-driven —
/// [`PrimitiveHost::iterate_batch_boundary`] supplies the batch size and
/// [`PrimitiveHost::check_capability`] performs the refresh.
///
/// # Errors
///
/// Returns `Ok` with the `ON_LIMIT` edge when the input exceeds `max`,
/// and `Ok` with `ON_DENIED` when a batch-boundary cap-refresh surfaces
/// a denial. Never errors via `Err`; budget and denial failures are
/// routed through typed error edges so the engine's trace shows them.
pub fn execute(op: &OperationNode, host: &dyn PrimitiveHost) -> Result<StepResult, EvalError> {
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

    // Phase 2a G4-A: consult the shared evaluator/budget helper so the
    // per-iteration runtime budget check routes through one entry point
    // (cr-r1-3). `check_per_iteration_budget` is a pure function over
    // `(consumed, limit)` — the evaluator holds the running counters on
    // its stack; here we pre-check the static `items_len <= max` bound
    // via the same helper so a regression in either side is caught by
    // the helper's unit tests rather than by diverging call-site logic.
    if let Err(_e) =
        crate::evaluator::budget::check_per_iteration_budget(items_len as u64, max as u64)
    {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_LIMIT".to_string(),
            output: Value::Null,
        });
    }

    // Named Compromise #1 (ITERATE batch-boundary half). Walk the item
    // count in fixed-size batches; at every boundary (inclusive of the
    // first iteration — Phase-1 posture re-reads caps at batch 0 so a
    // grant revoked *between* handler registration and ITERATE entry is
    // observed before the first loop body). A denial routes ON_DENIED
    // with the policy's code in the payload.
    let required_scope: String = match op.properties.get("requires") {
        Some(Value::Text(s)) => s.clone(),
        _ => "iterate".to_string(),
    };
    let boundary = host.iterate_batch_boundary().max(1);

    // Entry refresh — zero-th boundary — always fires, even when the
    // iteration list is empty. Matches the ITERATE compromise prose:
    // "the evaluator snapshots caps at batch boundaries; the first
    // snapshot is taken at iteration 0."
    if let Err(EvalError::Capability(c)) = host.check_capability(&required_scope, None) {
        return Ok(StepResult {
            next: None,
            edge_label: "ON_DENIED".to_string(),
            output: Value::text(c.to_string()),
        });
    }

    // Per-batch refresh at iterations `boundary`, `2*boundary`, …
    // Bounded by `items_len` so the loop terminates.
    let mut i = boundary;
    while i < items_len {
        if let Err(EvalError::Capability(c)) = host.check_capability(&required_scope, None) {
            return Ok(StepResult {
                next: None,
                edge_label: "ON_DENIED".to_string(),
                output: Value::text(c.to_string()),
            });
        }
        i = i.saturating_add(boundary);
    }

    // Happy-path result: an empty accumulator list. The real engine-backed
    // body invocation (Phase-2) replaces this with the accumulated
    // per-iteration results. The cap-refresh cadence above is the
    // Phase-1 deliverable — body dispatch can layer on top without
    // reshaping the refresh semantics.
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

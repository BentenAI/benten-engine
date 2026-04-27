//! Iterative evaluator (G6-C) — explicit-stack walker over operation subgraphs.
//!
//! The evaluator is deliberately iterative. Recursion is banned because it
//! makes max-depth enforcement (Invariant 2) unreliable: a recursive walker
//! can blow the native stack before the invariant check fires. Instead we
//! keep a `Vec<ExecutionFrame>` and loop, pushing and popping frames
//! explicitly. See R1 triage architect major #2.
//!
//! The primary public entry point on [`Evaluator`] is
//! [`Evaluator::step`](crate::Evaluator::step), defined in the crate root so
//! the per-primitive dispatcher and `ExecutionFrame` type remain colocated
//! with the rest of the crate's public surface. This module provides:
//!
//! - [`RunResult`] — the terminal outcome of walking a whole subgraph.
//! - [`Evaluator::run`] — iterative `step` loop that follows edge labels and
//!   enforces Invariant 8's stopgap iteration budget.
//! - [`Evaluator::run_with_trace`] — the tracing variant that records per-
//!   node microsecond timings for `engine.trace(...)` (G8's DX deliverable).
//!
//! Per Validated Design Decision #4, this walker is NOT Turing-complete —
//! subgraphs are DAGs by invariant 1, iteration is bounded by invariant 8,
//! and the stack depth is bounded by [`Evaluator::max_stack_depth`] (a
//! belt-and-suspenders guard for the structural invariants enforced at
//! registration).

use benten_core::Value;
use std::collections::HashMap;
use std::time::Instant;

use crate::{
    AttributionFrame, ErrorCode, EvalError, Evaluator, OperationNode, PrimitiveHost, StepResult,
    Subgraph, TraceStep,
};

/// G5-B-ii: runtime attribution threading (Inv-14). Stamps an
/// [`crate::AttributionFrame`] onto every emitted [`TraceStep::Step`] row so
/// trace consumers can walk back to the authorising `(actor, handler,
/// grant)` triple.
pub mod attribution;

/// Phase 2a G4-A: shared iteration-budget helper consumed by
/// `primitives/iterate.rs` and `primitives/call.rs` (plus G9-A's wallclock
/// TOCTOU batch-boundary check). Single coordination point per cr-r1-3.
pub mod budget;

/// G5-B-ii / phil-r1-1: pinned empty-extensions `AttributionFrame` fixture
/// CID. Phase-6 additions to the attribution shape MUST be additive — if the
/// pinned CID shifts, the shape changed non-additively and the drift gate
/// (`tests/invariant_14_fixture_cid.rs`) fires.
pub mod attribution_schema_fixture;

/// Terminal outcome of [`Evaluator::run`].
///
/// Carries the final step's edge label, the `$result` value bound in the
/// context when the walker terminated, and the total number of primitives
/// executed. G7 wraps this into `Outcome` for the user-facing Engine API.
#[derive(Debug, Clone)]
pub struct RunResult {
    /// Edge label of the terminating step (usually `"terminal"` for RESPOND,
    /// `"ok"` for a walker that exhausted the subgraph without a RESPOND).
    pub terminal_edge: String,
    /// Final `$result` binding, if any.
    pub output: Value,
    /// Count of primitive evaluations performed (strictly monotonic over the
    /// run). Used by the benchmark suite + trace ordering asserts.
    pub steps_executed: u64,
}

/// Budget cap for [`Evaluator::run`] — covers Invariant 8's Phase-1 stopgap
/// ("cumulative iteration budget"). Defaults to 100 000 primitive
/// evaluations; callers override via [`Evaluator::run_with_budget`].
///
/// The full multiplicative-through-ITERATE form ships in Phase 2 once the
/// iteration-budget plumbing is wired end-to-end.
pub const DEFAULT_ITERATION_BUDGET: u64 = 100_000;

impl Evaluator {
    /// Walk a subgraph iteratively from the entry node until a terminal edge
    /// is reached or the iteration budget is exhausted.
    ///
    /// Uses the crate-level [`Evaluator::step`] for per-primitive dispatch.
    /// The entry node is the first node whose id does not appear as a `to`
    /// endpoint of any edge — matching G7's handler-entry convention.
    ///
    /// # Errors
    ///
    /// Returns whatever [`Evaluator::step`] surfaces, plus
    /// [`EvalError::StackOverflow`] if the walker overruns
    /// [`Evaluator::max_stack_depth`].
    pub fn run(
        &mut self,
        subgraph: &Subgraph,
        input: Value,
        host: &dyn PrimitiveHost,
    ) -> Result<RunResult, EvalError> {
        self.run_with_budget(subgraph, input, host, DEFAULT_ITERATION_BUDGET)
    }

    /// Variant of [`Evaluator::run`] with a caller-supplied iteration budget.
    ///
    /// The budget caps how many primitive evaluations may run before the
    /// walker aborts with the iteration-budget stopgap error. Provided so
    /// hot-path benchmarks and deterministic-replay tests can pin the cap
    /// explicitly.
    ///
    /// # Errors
    ///
    /// See [`Evaluator::run`].
    pub fn run_with_budget(
        &mut self,
        subgraph: &Subgraph,
        input: Value,
        host: &dyn PrimitiveHost,
        budget: u64,
    ) -> Result<RunResult, EvalError> {
        let (result, _trace) = self.run_inner(subgraph, input, host, budget, false, None);
        result
    }

    /// Walk a subgraph and record per-step trace metadata.
    ///
    /// Each returned [`TraceStep`] carries the node id, the microsecond
    /// duration of its primitive execution, and any typed error edge the
    /// step routed through. G8's `engine.trace` wraps this into the
    /// developer-facing trace object.
    ///
    /// Phase 2a G5-B-ii: Inv-14 runtime threading. Callers that have an
    /// `(actor, handler, grant)` triple in scope should prefer
    /// [`Evaluator::run_with_trace_attributed`] so every emitted
    /// [`TraceStep::Step`] carries the originating
    /// [`AttributionFrame`]. This unattributed entry point exists for
    /// in-crate unit tests + the structural-only invariant suites that
    /// drive the evaluator without an engine-side capability surface;
    /// production traces must go through the attributed entry point.
    ///
    /// # Errors
    ///
    /// See [`Evaluator::run`].
    pub fn run_with_trace(
        &mut self,
        subgraph: &Subgraph,
        input: Value,
        host: &dyn PrimitiveHost,
    ) -> Result<(RunResult, Vec<TraceStep>), EvalError> {
        let (result, trace) =
            self.run_inner(subgraph, input, host, DEFAULT_ITERATION_BUDGET, true, None);
        result.map(|r| (r, trace))
    }

    /// G12-A: trace-preserving + budget-parameterized variant of
    /// [`Evaluator::run_with_trace_attributed`] that returns the recorded
    /// trace ALONGSIDE any [`EvalError`] rather than dropping it on the
    /// error path. Engine-side callers in trace mode use this so the
    /// terminal [`TraceStep::BudgetExhausted`] / typed-error rows pushed
    /// by `run_inner` (private; cannot intra-doc-link without
    /// `--document-private-items`) reach the user-facing
    /// `engine.trace(...)` consumer (§9.12 / `lib.rs:1430-1431` TODO
    /// closure). The explicit `budget` lets the engine thread its
    /// `Engine::testing_set_iteration_budget` override through without
    /// adding a second method.
    pub fn run_with_trace_attributed_capturing_with_budget(
        &mut self,
        subgraph: &Subgraph,
        input: Value,
        host: &dyn PrimitiveHost,
        frame: AttributionFrame,
        budget: u64,
    ) -> (Result<RunResult, EvalError>, Vec<TraceStep>) {
        self.run_inner(subgraph, input, host, budget, true, Some(frame))
    }

    /// G5-B-ii / Inv-14: trace variant that stamps the supplied
    /// [`AttributionFrame`] onto every emitted [`TraceStep::Step`] row.
    /// Callers (notably `Engine::dispatch_call` in `benten-engine`)
    /// construct the frame from the in-flight `(actor, handler, grant)`
    /// triple and pass it here so trace consumers can walk back to the
    /// authorising context.
    ///
    /// Phase-2a contract: boundary variants
    /// ([`TraceStep::SuspendBoundary`] / [`TraceStep::ResumeBoundary`] /
    /// [`TraceStep::BudgetExhausted`]) do not yet carry attribution — the
    /// shape-pin in `crates/benten-eval/tests/inv_8_11_13_14_firing.rs` is
    /// the source of truth. Phase-2b broadens the contract per plan §5
    /// "required on every variant" once the boundary-variant shapes are
    /// reopened.
    ///
    /// # Errors
    ///
    /// See [`Evaluator::run`].
    pub fn run_with_trace_attributed(
        &mut self,
        subgraph: &Subgraph,
        input: Value,
        host: &dyn PrimitiveHost,
        frame: AttributionFrame,
    ) -> Result<(RunResult, Vec<TraceStep>), EvalError> {
        let (result, trace) = self.run_inner(
            subgraph,
            input,
            host,
            DEFAULT_ITERATION_BUDGET,
            true,
            Some(frame),
        );
        result.map(|r| (r, trace))
    }

    fn run_inner(
        &mut self,
        subgraph: &Subgraph,
        _input: Value,
        host: &dyn PrimitiveHost,
        budget: u64,
        collect_trace: bool,
        attribution: Option<AttributionFrame>,
    ) -> (Result<RunResult, EvalError>, Vec<TraceStep>) {
        // Build an adjacency map keyed by (from-node-id, edge-label).
        let mut next_by_edge: HashMap<(&str, &str), &str> = HashMap::new();
        let mut has_incoming: HashMap<&str, bool> = HashMap::new();
        for (from, to, label) in &subgraph.edges {
            next_by_edge.insert((from.as_str(), label.as_str()), to.as_str());
            has_incoming.insert(to.as_str(), true);
        }
        let by_id: HashMap<&str, &OperationNode> =
            subgraph.nodes.iter().map(|n| (n.id.as_str(), n)).collect();

        // Entry is the first node without an incoming edge.
        //
        // TODO(R4b / G7): when engine registration lands, validate that
        // exactly one node has no incoming edges so multi-root handlers
        // are rejected at registration instead of silently picking the
        // first-constructed root. Mini-review `g6-cr-10`.
        let Some(entry) = subgraph
            .nodes
            .iter()
            .find(|n| !has_incoming.contains_key(n.id.as_str()))
        else {
            return (
                Ok(RunResult {
                    terminal_edge: "ok".to_string(),
                    output: Value::Null,
                    steps_executed: 0,
                }),
                Vec::new(),
            );
        };

        let mut trace = Vec::new();
        let mut steps = 0_u64;
        let mut cursor: Option<&OperationNode> = Some(entry);
        let mut last = StepResult {
            next: None,
            edge_label: "ok".to_string(),
            output: Value::Null,
        };
        // G12-A: track the operation-node id walk-path so a runtime
        // `TraceStep::BudgetExhausted` emission can name the chain that
        // produced the exhaustion. Visited ids are pushed at execution time
        // (post-success); the about-to-execute cursor id is appended at
        // emission time so the path's terminal element is the node the
        // budget guard short-circuited before stepping.
        let mut walk_path: Vec<String> = Vec::new();

        while let Some(op) = cursor {
            if steps >= budget {
                // Runtime cumulative-step-budget exhaustion — distinct from
                // the registration-time nesting-depth stopgap
                // (`IterateNestDepth`). Mini-review findings g6-cag-1 /
                // g6-opl-6 / g6-cr-2. Maps to `E_INV_ITERATE_BUDGET`.
                //
                // G12-A: §9.12 / `lib.rs:1430-1431` TODO closure — push a
                // terminal `TraceStep::BudgetExhausted` row before the
                // typed error propagates so `engine.trace(...)` consumers
                // observe the exhaustion in-band rather than as an
                // out-of-band error. Phase-2b G7-A wires the analogous
                // `"sandbox_fuel"` budget_type when SANDBOX fuel metering
                // lands; this group covers `"inv_8_iteration"` only.
                if collect_trace {
                    let mut path = walk_path.clone();
                    path.push(op.id.clone());
                    trace.push(TraceStep::BudgetExhausted {
                        budget_type: "inv_8_iteration",
                        consumed: steps,
                        limit: budget,
                        path,
                    });
                }
                return (
                    Err(EvalError::Invariant(
                        crate::InvariantViolation::IterateBudget,
                    )),
                    trace,
                );
            }
            let start = Instant::now();
            let step_res = self.step(op, host);
            let elapsed_us = start.elapsed().as_micros();
            let elapsed = u64::try_from(elapsed_us).unwrap_or(u64::MAX);
            // Per R4 triage: trace steps must have non-zero timing. Saturate
            // at 1µs when `Instant::now()` returns an identical value — on
            // mac+Linux the monotonic clock is typically 1–10ns granular, so
            // this only triggers on zero-work primitives.
            //
            // TODO(phase-2-trace-docs): document in `diag/trace.rs` that trace timing is NOT
            // included in any content-addressed hash; a trace artifact is an
            // observability output, not a deterministic-replay fixture.
            // Mini-review `g6-cr-11` / `g6-cr-12`.
            let elapsed = elapsed.max(1);
            match step_res {
                Ok(r) => {
                    steps += 1;
                    walk_path.push(op.id.clone());
                    if collect_trace {
                        trace.push(TraceStep::Step {
                            node_id: op.id.clone(),
                            duration_us: elapsed,
                            inputs: Value::Null,
                            outputs: r.output.clone(),
                            error: None,
                            attribution: attribution.clone(),
                        });
                    }
                    last = r;
                    if last.edge_label == "terminal" {
                        break;
                    }
                    // Try the specific edge label; if the graph doesn't
                    // declare one, fall back to the generic continuation
                    // edge (`"next"` — what `SubgraphBuilder` emits by
                    // default for non-error transitions). The fallback is
                    // only consulted when the step returned a success-shape
                    // edge (`"ok"`), so typed error edges remain
                    // unambiguous.
                    cursor = next_by_edge
                        .get(&(op.id.as_str(), last.edge_label.as_str()))
                        .copied()
                        .or_else(|| {
                            if last.edge_label == "ok" {
                                next_by_edge.get(&(op.id.as_str(), "next")).copied()
                            } else {
                                None
                            }
                        })
                        .and_then(|id| by_id.get(id).copied());
                }
                Err(e) => {
                    if collect_trace {
                        // G6-A trace-preservation pattern (D1 carry from
                        // G12-A): when STREAM backpressure / closed-by-peer
                        // errors fire, emit a `TraceStep::BudgetExhausted
                        // { budget_type: "stream_backpressure", ... }` row
                        // BEFORE the typed-error Step row so consumers
                        // observe the exhaustion in-band, mirroring the
                        // `inv_8_iteration` flow above. The path's terminal
                        // element is the node whose primitive surfaced the
                        // typed error.
                        let code = e.code();
                        if matches!(
                            code,
                            ErrorCode::StreamBackpressureDropped
                                | ErrorCode::StreamClosedByPeer
                                | ErrorCode::StreamProducerWallclockExceeded
                        ) {
                            let mut path = walk_path.clone();
                            path.push(op.id.clone());
                            trace.push(TraceStep::BudgetExhausted {
                                budget_type: "stream_backpressure",
                                consumed: 0,
                                limit: 0,
                                path,
                            });
                        }
                        trace.push(TraceStep::Step {
                            node_id: op.id.clone(),
                            duration_us: elapsed,
                            inputs: Value::Null,
                            outputs: Value::Null,
                            error: Some(code),
                            attribution: attribution.clone(),
                        });
                    }
                    return (Err(e), trace);
                }
            }
        }

        (
            Ok(RunResult {
                terminal_edge: last.edge_label,
                output: last.output,
                steps_executed: steps,
            }),
            trace,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NullHost, PrimitiveKind, Subgraph};

    #[test]
    fn run_empty_subgraph_yields_ok_terminal() {
        let mut ev = Evaluator::new();
        let sg = Subgraph::new("empty");
        let r = ev
            .run(&sg, Value::Null, &NullHost)
            .expect("empty subgraph is a no-op");
        assert_eq!(r.terminal_edge, "ok");
        assert_eq!(r.steps_executed, 0);
    }

    #[test]
    fn run_single_respond_terminates() {
        let mut ev = Evaluator::new();
        let sg = Subgraph::new("single_respond")
            .with_node(OperationNode::new("r", PrimitiveKind::Respond));
        let r = ev
            .run(&sg, Value::Null, &NullHost)
            .expect("single respond terminates cleanly");
        assert_eq!(r.terminal_edge, "terminal");
        assert_eq!(r.steps_executed, 1);
    }
}

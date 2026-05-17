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
//! - [`Evaluator::run_with`] — the configurable variant ([`RunOptions`])
//!   that records per-node microsecond timings for `engine.trace(...)`
//!   (G8's DX deliverable) when `collect_trace` is set.
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
/// evaluations; callers override via [`RunOptions::budget`].
pub const DEFAULT_ITERATION_BUDGET: u64 = 100_000;

/// Options for [`Evaluator::run_with`] (v1-API-stabilization,
/// refinement-audit #1145 / Qual-2 #763).
///
/// Collapses the former five suffix-stacked entry points
/// (`run_with_budget` / `run_with_trace` / `run_with_trace_attributed` /
/// `run_with_trace_attributed_capturing_with_budget`) — whose longest name
/// read as a wave-decorated changelog rather than a Rust API — into a
/// single builder. Every old method delegated to the private `run_inner`
/// with different default-arg fillers; this surface makes the orthogonal
/// axes explicit:
///
/// - `budget` — Inv-8 cumulative-step cap (default
///   [`DEFAULT_ITERATION_BUDGET`]).
/// - `attribution` — Inv-14 `(actor, handler, grant)` frame stamped onto
///   every emitted [`TraceStep::Step`]; `None` = unattributed (in-crate
///   structural suites only — production traces MUST attribute).
/// - `collect_trace` — record per-step [`TraceStep`] rows.
/// - `capture_on_err` — return the recorded trace ALONGSIDE any
///   [`EvalError`] instead of dropping it on the error path (G12-A: the
///   terminal `TraceStep::BudgetExhausted` row must reach the
///   `engine.trace(...)` consumer). Only meaningful with `collect_trace`.
///
/// Use [`Evaluator::run`] for the all-defaults case; [`Evaluator::run_with`]
/// otherwise.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    budget: Option<u64>,
    attribution: Option<AttributionFrame>,
    collect_trace: bool,
    capture_on_err: bool,
}

impl RunOptions {
    /// A fresh options set: default budget, no attribution, no trace.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the Inv-8 cumulative-step budget.
    #[must_use]
    pub fn budget(mut self, budget: u64) -> Self {
        self.budget = Some(budget);
        self
    }

    /// Stamp the supplied Inv-14 [`AttributionFrame`] onto every emitted
    /// trace step.
    #[must_use]
    pub fn attribution(mut self, frame: AttributionFrame) -> Self {
        self.attribution = Some(frame);
        self
    }

    /// Record per-step [`TraceStep`] rows.
    #[must_use]
    pub fn collect_trace(mut self, yes: bool) -> Self {
        self.collect_trace = yes;
        self
    }

    /// Return the recorded trace alongside any [`EvalError`] rather than
    /// dropping it on the error path (implies the trace is collected).
    #[must_use]
    pub fn capture_on_err(mut self, yes: bool) -> Self {
        self.capture_on_err = yes;
        self
    }
}

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
        let (result, _trace) =
            self.run_inner(subgraph, input, host, DEFAULT_ITERATION_BUDGET, false, None);
        result
    }

    /// Walk a subgraph under explicit [`RunOptions`].
    ///
    /// v1-API-stabilization (refinement-audit #1145 / Qual-2 #763): this
    /// single entry point replaces the former five suffix-stacked methods
    /// (`run_with_budget` / `run_with_trace` / `run_with_trace_attributed`
    /// / `run_with_trace_attributed_capturing_with_budget`). The orthogonal
    /// axes (budget / attribution / trace-collection / capture-on-error)
    /// are now explicit fields on [`RunOptions`] rather than encoded in the
    /// method name.
    ///
    /// Returns `(result, trace)` uniformly:
    /// - `trace` is empty unless [`RunOptions::collect_trace`] was set.
    /// - On the error path the trace is **retained** only if
    ///   [`RunOptions::capture_on_err`] was set (G12-A: the terminal
    ///   `TraceStep::BudgetExhausted` / typed-error rows must reach the
    ///   `engine.trace(...)` consumer); otherwise it is dropped (empty
    ///   `Vec`) to match the prior non-capturing variants' behaviour.
    ///
    /// Inv-14: callers with an `(actor, handler, grant)` triple in scope
    /// MUST pass [`RunOptions::attribution`] so every emitted
    /// [`TraceStep::Step`] carries the originating [`AttributionFrame`];
    /// the unattributed path exists only for in-crate structural-invariant
    /// suites that drive the evaluator without an engine-side capability
    /// surface.
    ///
    /// Phase-2a contract: boundary variants
    /// ([`TraceStep::SuspendBoundary`] / [`TraceStep::ResumeBoundary`] /
    /// [`TraceStep::BudgetExhausted`]) do not carry attribution — the
    /// shape-pin in `crates/benten-eval/tests/inv_8_11_13_14_firing.rs` is
    /// the source of truth.
    pub fn run_with(
        &mut self,
        subgraph: &Subgraph,
        input: Value,
        host: &dyn PrimitiveHost,
        options: RunOptions,
    ) -> (Result<RunResult, EvalError>, Vec<TraceStep>) {
        let budget = options.budget.unwrap_or(DEFAULT_ITERATION_BUDGET);
        let (result, trace) = self.run_inner(
            subgraph,
            input,
            host,
            budget,
            options.collect_trace,
            options.attribution,
        );
        if result.is_err() && !options.capture_on_err {
            (result, Vec::new())
        } else {
            (result, trace)
        }
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
                // G12-A: §9.12 / the `crates/benten-eval/src/lib.rs::TraceStep`
                // Phase-3 TODO closure above the enum — push a
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
            // TODO(phase-4-meta — backlog §4.75; trace-timing-non-determinism doc): document
            // in `diag/trace.rs` that trace timing is NOT included in
            // any content-addressed hash; a trace artifact is an
            // observability output, not a deterministic-replay fixture.
            // Mini-review `g6-cr-11` / `g6-cr-12`. Carried from Phase-2
            // generic marker.
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
            .push_node_raw(OperationNode::new("r", PrimitiveKind::Respond));
        let r = ev
            .run(&sg, Value::Null, &NullHost)
            .expect("single respond terminates cleanly");
        assert_eq!(r.terminal_edge, "terminal");
        assert_eq!(r.steps_executed, 1);
    }
}

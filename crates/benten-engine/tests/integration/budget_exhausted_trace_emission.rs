//! Phase 2a R4b coverage M2 — runtime Inv-8 BudgetExhausted trace emission.
//!
//! R4b CORRECTNESS finding: the `TraceStep::BudgetExhausted` enum variant
//! is constructed only by shape-pin tests
//! (`inv_8_11_13_14_firing.rs::trace_step_budget_exhausted_variant_shape_pin`).
//! No test exercised the variant via REAL Inv-8 firing — i.e. the
//! evaluator running until the cumulative step budget is exhausted, then
//! emitting a `BudgetExhausted` row at the suspension point. **Phase-2b
//! G12-A** wires that emission path.
//!
//! **Closed API.** The evaluator's runtime budget exhaustion path
//! (`benten-eval/src/evaluator.rs::run_inner`) now pushes a terminal
//! [`TraceStep::BudgetExhausted { budget_type, consumed, limit, path }`] row
//! BEFORE returning `EvalError::Invariant(IterateBudget)`, and the
//! engine-side trace path preserves the captured trace through the error
//! branch (mapping the typed error onto an `Outcome::ON_ERROR` shape
//! carrying `E_INV_ITERATE_BUDGET`) so consumers of `engine.trace(...)`
//! observe the exhaustion in-band rather than as an out-of-band `Err`.
//! Closes the §9.12 / `benten-eval/src/lib.rs:1430-1431` TODO.
//!
//! G12-A scope: `budget_type = "inv_8_iteration"`. The analogous
//! `"sandbox_fuel"` budget_type lands when G7-A wires the SANDBOX fuel
//! metering path; that addition is purely additive and won't reshape this
//! test.
//!
//! Wave-3c R4b fix-pass writer (red-phase); Phase-2b G12-A green-phase
//! rewriter — drove the assertion via a 4-EMIT chain under a test-only
//! iteration budget so the trip fires deterministically without inflating
//! the spec into a 100k-node fixture.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::outcome::TraceStep;
use benten_engine::{Engine, SubgraphSpec};

fn fresh_engine() -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    (dir, engine)
}

/// Register a 4-EMIT-node chain, cap the cumulative iteration budget at 2
/// via the `Engine::testing_set_iteration_budget` G12-A test hook, drive
/// `engine.trace`, and assert at least one `TraceStep::BudgetExhausted`
/// row is emitted with realistic field values before the `IterateBudget`
/// invariant violation is folded into an `Outcome::ON_ERROR`.
///
/// Each EMIT primitive returns the `"ok"` evaluator edge unconditionally
/// (`crates/benten-eval/src/primitives/emit.rs`); the
/// `SubgraphBuilder::add_edge` between successive EMITs uses the `"next"`
/// edge label, which `run_inner` consults as the `"ok"` fallback. The
/// walker therefore steps through the chain one EMIT at a time, bumping
/// the cumulative `steps` counter once per primitive, and the Inv-8 guard
/// (`steps >= budget`) trips when the cursor advances onto the third EMIT
/// (`steps == 2`, `budget == 2`).
#[test]
fn budget_exhausted_runtime_trace_emission() {
    let (_dir, engine) = fresh_engine();

    // Build a 4-WRITE chain via the SubgraphSpec builder so the engine
    // stores the spec for downstream dispatch. Phase-2b G12-D widening:
    // `subgraph_for_spec` now walks `spec.primitives` (the unified
    // per-primitive storage), constructing each declared primitive in
    // order, chaining successive nodes via `"next"` edges, and capping
    // with a terminal RESPOND. Each WRITE primitive returns the `"ok"`
    // evaluator edge on success; the walker steps through the chain one
    // WRITE at a time, bumping the cumulative `steps` counter once per
    // primitive, and the Inv-8 guard (`steps >= budget`) trips when the
    // cursor advances onto the third WRITE (`steps == 2`, `budget == 2`).
    //
    // (`SubgraphSpec::iterate` is a Phase-1 no-op and a single
    // `respond` is terminal-on-first-step, so neither is sufficient
    // to trip the cumulative-step guard.)
    let sg = SubgraphSpec::builder()
        .handler_id("budget:exhauster")
        .write(|w| w.label("budget_step_0"))
        .write(|w| w.label("budget_step_1"))
        .write(|w| w.label("budget_step_2"))
        .write(|w| w.label("budget_step_3"))
        .build();

    let handler_id = engine
        .register_subgraph(sg)
        .expect("registration must succeed; 4 WRITE primitives + RESPOND form a valid linear DAG");

    // G12-A test hook: cap cumulative iteration budget at 2 so the
    // walker trips at the third EMIT (steps == 2 == budget). Without
    // this override the engine uses `DEFAULT_ITERATION_BUDGET = 100_000`
    // and the chain would terminate cleanly after 4 steps.
    engine.testing_set_iteration_budget(Some(2));

    let trace = engine
        .trace(&handler_id, "budget:run", Node::empty())
        .expect("trace returns Ok carrying the runtime trace including the terminal BudgetExhausted row");

    // Walk the trace once and capture (budget_type, consumed, limit,
    // path-len) tuples from any BudgetExhausted rows. We borrow the
    // trace's owned step slice so the views' lifetimes stay tied to
    // `trace` (the BudgetExhaustedView accessor borrows from the
    // TraceStep).
    let steps_total = trace.steps().len();
    let mut budget_rows: Vec<(&'static str, u64, u64, usize)> = Vec::new();
    for step in trace.steps() {
        if let Some(view) = step.as_budget_exhausted() {
            budget_rows.push((
                view.budget_type(),
                view.consumed(),
                view.limit(),
                view.path().len(),
            ));
        }
    }

    assert!(
        !budget_rows.is_empty(),
        "runtime Inv-8 exhaustion must emit at least one TraceStep::BudgetExhausted \
         row before short-circuiting (§9.12). Got {steps_total} steps total, none \
         of which carried the variant."
    );

    // Inspect the first emitted budget row: it must name the right budget
    // family, name a `consumed >= limit` relation, and carry a non-empty
    // path of operation-node ids that produced the exhaustion.
    let (budget_type, consumed, limit, path_len) = budget_rows[0];
    assert_eq!(
        budget_type, "inv_8_iteration",
        "Inv-8 cumulative-step budget rows must carry budget_type \
         \"inv_8_iteration\" (the lib.rs:1467 enum-variant doc names this \
         + \"sandbox_fuel\" as the only valid families)"
    );
    assert!(
        consumed >= limit,
        "BudgetExhausted row's consumed ({consumed}) must be >= limit \
         ({limit}) — that's the entire firing condition \
         (`steps >= budget` in run_inner)"
    );
    assert_eq!(
        limit, 2,
        "the test override pinned limit at 2; if this drifts the override \
         plumbing through `Engine::testing_set_iteration_budget` regressed"
    );
    assert!(
        path_len > 0,
        "BudgetExhausted row must name the operation-node id path that \
         produced the exhaustion; got empty path"
    );
}

/// Sanity that the `as_budget_exhausted` accessor returns `None` for
/// non-budget rows. Keeps the test file from silently regressing when
/// the variant disambiguator changes.
#[test]
fn as_budget_exhausted_is_none_for_non_budget_rows() {
    // Construct a Step row directly and verify the accessor returns None.
    let step = TraceStep::Step {
        duration_us: 1,
        node_cid: benten_core::Cid::from_blake3_digest([0u8; 32]),
        primitive: "respond".into(),
        node_id: "n0".into(),
        inputs: benten_core::Value::Null,
        outputs: benten_core::Value::Null,
        error: None,
        attribution: None,
    };
    assert!(
        step.as_budget_exhausted().is_none(),
        "Step rows must not match as_budget_exhausted"
    );
}

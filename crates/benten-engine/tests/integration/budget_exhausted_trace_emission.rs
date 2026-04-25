//! Phase 2a R4b coverage M2 — runtime Inv-8 BudgetExhausted trace emission.
//!
//! R4b CORRECTNESS finding: the `TraceStep::BudgetExhausted` enum variant
//! is constructed only by shape-pin tests
//! (`inv_8_11_13_14_firing.rs::trace_step_budget_exhausted_variant_shape_pin`).
//! No test exercises the variant via REAL Inv-8 firing — i.e. the
//! evaluator running until the cumulative step budget is exhausted, then
//! emitting a `BudgetExhausted` row at the suspension point.
//!
//! **Target API.** The evaluator's runtime budget exhaustion path
//! (`benten-eval/src/evaluator.rs::run_inner` ~line 185) currently
//! short-circuits with `EvalError::Invariant(IterateBudget)` and DROPS the
//! collected trace. The §9.12 contract documented at
//! `benten-eval/src/lib.rs:1430-1431` (TODO(phase-2a-G3-A / G4-A / G5-B))
//! calls for emitting a terminal `TraceStep::BudgetExhausted { budget_type,
//! consumed, limit, path }` row before returning so consumers of `trace`
//! see the exhaustion in-band rather than as an out-of-band error.
//!
//! **This test is RED-PHASE today.** The runtime emission path is not yet
//! wired (the lib.rs:1430 TODO comment is the single source of truth for
//! that gap). The body documents the target shape so the implementation
//! agent landing the emission path has a concrete pin. Until then the test
//! is `#[ignore]`d with a clear pointer.
//!
//! Wave-3c R4b fix-pass writer.

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

/// Register a subgraph that exhausts the Inv-8 cumulative step budget at
/// runtime, drive `engine.trace`, and assert at least one
/// `TraceStep::BudgetExhausted` row is emitted with realistic field values.
///
/// **Phase-2a status: red-phase.** Runtime emission of `BudgetExhausted`
/// is the §9.12 / lib.rs:1430 TODO; until the evaluator pushes a terminal
/// row before returning `EvalError::Invariant(IterateBudget)`, this test
/// will fail. Tracked under plan §G11-A residuals.
#[test]
#[ignore = "phase-2a-pending: runtime BudgetExhausted trace-row emission per benten-eval/src/lib.rs:1430-1431 TODO. Drop #[ignore] once the evaluator pushes the terminal row before short-circuiting on cumulative-budget exhaustion."]
fn budget_exhausted_runtime_trace_emission() {
    let (_dir, engine) = fresh_engine();

    // ITERATE bound is wired through the builder's `iterate(max, body)`
    // signature (Phase-1 thin shape — body closure is not executed yet).
    // The runtime walker still trips the cumulative `steps >= budget`
    // check inside `run_inner` once the per-step accumulator reaches the
    // configured budget — that's what the BudgetExhausted row pins.
    let sg = SubgraphSpec::builder()
        .handler_id("budget:exhauster")
        .iterate(50_000_u32, |body| body)
        .respond()
        .build();
    let handler_id = engine
        .register_subgraph(sg)
        .expect("registration must succeed; multiplicative budget within default 100k cap");

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

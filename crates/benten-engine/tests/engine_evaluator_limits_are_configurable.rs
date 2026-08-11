//! Falsification arms for the three evaluator-limit knobs.
//!
//! # What this file exists to prevent
//!
//! `benten_eval::InvariantConfig` has carried the rustdoc "Configurable
//! invariant thresholds" since Phase 1, and `docs/future/engine-fit-and-gaps.md`
//! §3.7 records an outside evaluation reporting that neither it, nor
//! `Evaluator::max_stack_depth`, nor `RunOptions::budget` had any production
//! writer. That was correct for all three at the `Engine` level: the three
//! registration paths each built `InvariantConfig::default()` inline, the
//! dispatch path built `Evaluator::new()` (frame cap fixed at the literal 64),
//! and the only engine-level budget writer was
//! `Engine::testing_set_iteration_budget`, gated behind the
//! `iteration-budget-test-grade` feature and therefore absent from a default
//! build.
//!
//! That is the FALSE-RECORD shape CLAUDE.md rule 14 names, and per rule 15 the
//! default disposition is to change the CODE. `EngineBuilder::invariant_config`
//! and `EngineBuilder::iteration_budget` are the writers; the runtime frame cap
//! is DERIVED from `InvariantConfig::max_depth` rather than exposed as a fourth
//! knob that could disagree with it.
//!
//! # Each test names the mutation that breaks it
//!
//! A guard nobody has broken is a guard nobody has verified. Every test below
//! carries the exact one-line revert that makes it fail.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, SubgraphSpec};
use benten_eval::{InvariantConfig, OperationNode, PrimitiveKind, Subgraph};

/// A linear `n`-TRANSFORM chain terminated by RESPOND, built through the raw
/// constructors so the eval-side `SubgraphBuilderExt::build_validated` gate
/// (which has its own hardcoded `InvariantConfig::default()`) does not
/// pre-empt the engine-side check under test.
fn chain(handler_id: &str, n: usize) -> Subgraph {
    let mut sg = Subgraph::new(handler_id);
    for i in 0..n {
        sg = sg.push_node_raw(
            OperationNode::new(format!("t{i}"), PrimitiveKind::Transform)
                .with_property("expr", Value::text("1")),
        );
        if i > 0 {
            sg = sg.push_edge_raw(format!("t{}", i - 1), format!("t{i}"), "ok");
        }
    }
    sg = sg.push_node_raw(OperationNode::new("done", PrimitiveKind::Respond));
    sg.push_edge_raw(format!("t{}", n - 1), "done", "ok")
}

/// A 100-WRITE `SubgraphSpec` (the engine caps it with a terminal RESPOND, so
/// 101 nodes on one path). Registered specs are the only shape `Engine::call`
/// can dispatch — a raw `Subgraph` registers but stores no spec, so
/// `dispatch` surfaces `E_DSL_UNREGISTERED_HANDLER`.
fn deep_spec(handler_id: &str) -> SubgraphSpec {
    let mut b = SubgraphSpec::builder().handler_id(handler_id);
    for i in 0..100 {
        let label = format!("deep_step_{i}");
        b = b.write(move |w| w.label(&label));
    }
    b.build()
}

fn engine_with(cfg: Option<InvariantConfig>, budget: Option<u64>) -> (tempfile::TempDir, Engine) {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Engine::builder().path(dir.path().join("benten.redb"));
    if let Some(cfg) = cfg {
        b = b.invariant_config(cfg);
    }
    if let Some(budget) = budget {
        b = b.iteration_budget(budget);
    }
    (dir, b.build().unwrap())
}

// ---------------------------------------------------------------------------
// 1. InvariantConfig — TIGHTENING is honored.
// ---------------------------------------------------------------------------

/// A `max_nodes` lowered below the handler's node count must reject a
/// registration the default config accepts. This is the narrowest possible
/// proof that the engine reads the CONFIGURED bounds and not
/// `InvariantConfig::default()`.
///
/// **Mutation that falsifies:** in `crates/benten-engine/src/engine.rs`, change
/// `let cfg = self.inner.invariant_config.clone();` back to
/// `let cfg = InvariantConfig::default();` in `register_subgraph`. The
/// tightened engine then accepts the 6-node handler and the `expect_err`
/// panics. VERIFIED by performing exactly that revert.
#[test]
fn tightened_max_nodes_rejects_what_the_default_config_accepts() {
    // Control: the default engine accepts a 6-node handler (5 TRANSFORM +
    // RESPOND) — well inside `DEFAULT_MAX_NODES = 4096`.
    let (_d1, default_engine) = engine_with(None, None);
    default_engine
        .register_subgraph(chain("limits:control", 5))
        .expect("6 nodes is far inside the default max_nodes = 4096");

    // Same handler, engine built with max_nodes = 3.
    let (_d2, tight_engine) = engine_with(
        Some(InvariantConfig {
            max_nodes: 3,
            ..InvariantConfig::default()
        }),
        None,
    );
    let err = tight_engine
        .register_subgraph(chain("limits:tightened", 5))
        .expect_err(
            "an engine built with `.invariant_config(max_nodes = 3)` must reject a \
             6-node handler; if this registers, the engine is still reading \
             InvariantConfig::default() and the knob is decorative",
        );
    assert_eq!(
        err.error_code().as_str(),
        "E_INV_TOO_MANY_NODES",
        "the tightened bound must surface the Inv-5 node-count violation"
    );
}

// ---------------------------------------------------------------------------
// 2. InvariantConfig::max_depth — RAISING is honored at BOTH gates.
// ---------------------------------------------------------------------------

/// The load-bearing arm: raising `max_depth` must raise the registration bound
/// AND the runtime frame cap together.
///
/// `Evaluator::step` pushes one frame per non-terminal step and pops only on a
/// `"terminal"` edge, so `max_stack_depth` is operationally "nodes walkable
/// along one path" — the same quantity Inv-2 `max_depth` bounds at
/// registration. Both were the literal `64`, agreeing by coincidence. Raise
/// `max_depth` alone and a 100-node handler registers cleanly, then dies at
/// call time with `EvalError::StackOverflow`. That is why the engine DERIVES
/// the frame cap instead of offering a second knob.
///
/// **Mutation that falsifies:** in `crates/benten-engine/src/engine.rs`, delete
/// the line `evaluator.max_stack_depth = self.inner.invariant_config.max_depth;`
/// in `dispatch_call_with_mode_and_trace`. Registration still succeeds (the
/// registration half is independent), the call then fails at frame 64 and the
/// `is_ok_edge` assertion below fires with the StackOverflow edge. VERIFIED by
/// performing exactly that deletion.
#[test]
fn raised_max_depth_registers_and_walks_a_100_node_chain() {
    // Control: the default engine REJECTS the deep chain at registration
    // (Inv-2, `longest.len() > max_depth = 64`). Without this arm the test
    // below could pass for the wrong reason.
    let (_d0, default_engine) = engine_with(None, None);
    let control = default_engine
        .register_subgraph(deep_spec("limits:deep_control"))
        .expect_err("100 WRITEs + terminal RESPOND exceeds the default max_depth = 64");
    assert_eq!(control.error_code().as_str(), "E_INV_DEPTH_EXCEEDED");

    // Raised engine: registers AND walks to a clean terminal edge.
    let (_d1, deep_engine) = engine_with(
        Some(InvariantConfig {
            max_depth: 256,
            ..InvariantConfig::default()
        }),
        None,
    );
    let handler_id = deep_engine
        .register_subgraph(deep_spec("limits:deep"))
        .expect("max_depth = 256 must admit a 101-node path");

    let outcome = deep_engine
        .call(&handler_id, "limits:run", Node::empty())
        .expect(
            "the walk must complete. An `EvalError::StackOverflow` here means the \
             runtime frame cap is still pinned at the `Evaluator::new()` literal 64 \
             while registration was raised to 256 — exactly the drift the derive \
             exists to make unrepresentable",
        );
    assert!(
        outcome.is_ok_edge(),
        "a 101-node walk under max_depth = 256 must terminate on the ok edge. \
         Got edge {:?} / error {:?}",
        outcome.edge_taken(),
        outcome.error_code(),
    );
}

/// Default-config behaviour is byte-identical to the pre-derive engine: the
/// derived cap resolves to `64 == 64`. This is the regression guard on the
/// derive itself — it must not have MOVED the default bound.
///
/// **Mutation that falsifies:** change `InvariantConfig::default()`'s
/// `max_depth` (or `limits::DEFAULT_MAX_DEPTH`) away from 64.
#[test]
fn default_depth_bound_is_unchanged_at_64() {
    assert_eq!(
        InvariantConfig::default().max_depth,
        64,
        "the derive makes the runtime frame cap follow max_depth; if this \
         default moves, every default-config deployment's runtime walk bound \
         moves with it and that must be a deliberate, reviewed change"
    );
    let (_d, engine) = engine_with(None, None);
    engine
        .register_subgraph(chain("limits:at_bound", 63))
        .expect("a 64-node path is exactly at the default bound and must register");
}

// ---------------------------------------------------------------------------
// 3. RunOptions::budget — the production knob reaches the walker.
// ---------------------------------------------------------------------------

/// `EngineBuilder::iteration_budget` must reach `RunOptions::budget` on the
/// real dispatch path, in a build with NO test features enabled.
///
/// **Mutation that falsifies:** in `crates/benten-engine/src/engine.rs`, change
/// `(*guard).unwrap_or(self.inner.iteration_budget)` back to
/// `(*guard).unwrap_or(benten_eval::evaluator::DEFAULT_ITERATION_BUDGET)`. The
/// 4-WRITE chain then completes under the 100 000 default and the
/// `expect_err` panics. VERIFIED by performing exactly that revert.
#[test]
fn configured_iteration_budget_trips_the_inv_8_guard() {
    let (_d, engine) = engine_with(None, Some(2));
    let sg = SubgraphSpec::builder()
        .handler_id("limits:budget")
        .write(|w| w.label("budget_step_0"))
        .write(|w| w.label("budget_step_1"))
        .write(|w| w.label("budget_step_2"))
        .write(|w| w.label("budget_step_3"))
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();

    let err = engine
        .call(&handler_id, "limits:run", Node::empty())
        .expect_err(
            "a 4-step chain under `.iteration_budget(2)` must exhaust the Inv-8 \
             cumulative-step budget. If this completes cleanly the builder knob \
             never reached RunOptions::budget and the only engine-level writer is \
             still the cfg-gated test hook",
        );
    assert_eq!(err.error_code().as_str(), "E_INV_ITERATE_BUDGET");
}

/// The default resolves to `DEFAULT_ITERATION_BUDGET`, so an unconfigured
/// engine is unchanged.
///
/// **Mutation that falsifies:** make `EngineBuilder::assemble` default
/// `iteration_budget` to something other than `DEFAULT_ITERATION_BUDGET`
/// (e.g. `0`) — the 4-step chain then trips the guard and the assert fires.
#[test]
fn unconfigured_iteration_budget_leaves_the_default_intact() {
    let (_d, engine) = engine_with(None, None);
    let sg = SubgraphSpec::builder()
        .handler_id("limits:budget_default")
        .write(|w| w.label("d0"))
        .write(|w| w.label("d1"))
        .write(|w| w.label("d2"))
        .write(|w| w.label("d3"))
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();
    let outcome = engine
        .call(&handler_id, "limits:run", Node::empty())
        .expect("default budget must not reject a 4-step chain");
    assert_ne!(
        outcome.error_code(),
        Some("E_INV_ITERATE_BUDGET"),
        "an engine that never called `.iteration_budget(..)` must keep \
         DEFAULT_ITERATION_BUDGET = 100_000"
    );
}

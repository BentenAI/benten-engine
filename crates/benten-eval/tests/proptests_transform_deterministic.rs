//! Proptest: TRANSFORM expression evaluation is deterministic (R4 triage M16).
//!
//! A TRANSFORM expression — given the same input Node(s) — must produce the
//! same output Value across invocations, processes, and machines. This is
//! the load-bearing determinism contract that IVM and the content-hash
//! subsystem both depend on.
//!
//! Red-phase: `parse_transform` and `evaluate_transform` are stubs until the
//! T12 grammar + E4 evaluator land in R5. The proptest compiles, fails at
//! runtime, and locks in the contract.
//!
//! R3 writer: `rust-test-writer-proptest`.

#![allow(clippy::unwrap_used)]

use benten_core::Value;
use proptest::prelude::*;

/// Strategy for arbitrary grammar-accepted TRANSFORM expressions. The
/// surface in Phase 1 is arithmetic + built-in calls + object construction;
/// this proptest exercises a minimal slice so it compiles today.
fn any_accepted_expr() -> impl Strategy<Value = String> {
    prop_oneof![
        any::<i32>().prop_map(|n| n.to_string()),
        (any::<i32>(), any::<i32>()).prop_map(|(a, b)| format!("{a} + {b}")),
        (any::<i32>(), any::<i32>()).prop_map(|(a, b)| format!("{a} * {b}")),
    ]
}

proptest! {
    /// R4 triage M16: same expression + same input = same output. Invoke
    /// the evaluator twice and compare results.
    #[test]
    fn prop_transform_expression_deterministic(expr in any_accepted_expr()) {
        let r1 = evaluate_transform(&expr);
        let r2 = evaluate_transform(&expr);
        prop_assert_eq!(
            r1, r2,
            "TRANSFORM `{}` must produce the same output on repeated invocations",
            expr
        );
    }

    /// Companion to the grammar-fuzz harness (R2 row
    /// `prop_transform_grammar_fuzz_accepted_deterministic`, M16-deferred).
    /// For every grammar-accepted expression, repeated evaluation against
    /// the same (empty) input must produce the same `Value`. Named
    /// separately from `prop_transform_expression_deterministic` so the
    /// grammar-fuzz and determinism harnesses stay individually traceable.
    #[test]
    fn prop_transform_grammar_fuzz_accepted_deterministic(
        expr in any_accepted_expr()
    ) {
        let r1 = evaluate_transform(&expr);
        let r2 = evaluate_transform(&expr);
        prop_assert_eq!(
            r1,
            r2,
            "grammar-accepted TRANSFORM `{}` must evaluate deterministically",
            expr
        );
    }
}

/// R5 wiring (G6-B): parse + evaluate a TRANSFORM expression against an
/// empty context. Returns the evaluated [`Value`], or `Value::Null` on
/// parse / runtime failure (the determinism property still holds: a stable
/// failure is still stable).
fn evaluate_transform(expr: &str) -> Value {
    use benten_eval::{Evaluator, NullHost, OperationNode, PrimitiveKind};
    let mut ev = Evaluator::new();
    let op = OperationNode::new("t", PrimitiveKind::Transform)
        .with_property("expr", Value::text(expr))
        .with_property("input", Value::Null);
    match ev.step(&op, &NullHost) {
        Ok(step) => step.output,
        Err(_) => Value::Null,
    }
}

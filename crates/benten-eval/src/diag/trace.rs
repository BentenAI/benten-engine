//! Trace rendering helpers (diag feature).
//!
//! The primary trace collection happens in
//! [`Evaluator::run_with_trace`](crate::Evaluator::run_with_trace); this
//! module provides convenience accessors and a pretty printer for CLI use.
//! G8's `engine.trace()` wraps these in the user-facing API.

use crate::TraceStep;
use std::fmt::Write as _;

/// Render a trace as a human-readable table. One row per step.
///
/// Example output:
///
/// ```text
/// step | node_id    | primitive        | duration_us | error
/// -----+------------+------------------+-------------+------
///    0 | r1         | READ             |          12 | -
///    1 | done       | RESPOND          |           4 | -
/// ```
///
/// Kept intentionally simple — the TS wrapper (G8) formats its own table
/// for browser display.
#[must_use]
pub fn pretty(steps: &[TraceStep]) -> String {
    let mut out = String::with_capacity(steps.len() * 64);
    let _ = writeln!(out, "step | node_id    | duration_us | error");
    let _ = writeln!(out, "-----+------------+-------------+------");
    for (i, s) in steps.iter().enumerate() {
        let node_id = s.node_id().unwrap_or("-").to_string();
        let err_str = s
            .error()
            .map_or("-".to_string(), |e| e.as_str().to_string());
        let _ = writeln!(
            out,
            "{:4} | {:10} | {:11} | {}",
            i,
            truncate(&node_id, 10),
            s.duration_us(),
            err_str,
        );
    }
    out
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use benten_core::Value;

    #[test]
    fn pretty_includes_header_and_row_per_step() {
        let steps = vec![
            TraceStep::Step {
                node_id: "a".into(),
                duration_us: 10,
                inputs: Value::Null,
                outputs: Value::Null,
                error: None,
                attribution: None,
            },
            TraceStep::Step {
                node_id: "b".into(),
                duration_us: 20,
                inputs: Value::Null,
                outputs: Value::Null,
                error: None,
                attribution: None,
            },
        ];
        let out = pretty(&steps);
        assert!(out.contains("step"));
        assert!(out.contains("a"));
        assert!(out.contains("b"));
    }

    #[test]
    fn truncate_respects_max_len() {
        assert_eq!(truncate("hello", 10), "hello");
        assert!(truncate("a_very_long_id", 5).chars().count() == 5);
    }
}

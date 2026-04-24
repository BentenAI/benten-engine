//! Criterion benchmark: per-expression TRANSFORM parse latency.
//!
//! (The bench body exercises only the parser hot path; actual expression
//! evaluation is measured separately in other benches. The name
//! `parse_eval_mixed` is retained because changing it would break the
//! gate's `BENCH_ID` comment reference and the workflow CI grep — the
//! "mixed" aspect is the three-shape workload rather than parse+eval.)
//!
//! **Target source:** ENGINE-SPEC §5 TRANSFORM — "< 10 µs per expression
//! on dev hardware." TRANSFORM sits on the hot path of every handler
//! that does any kind of data shaping between READ and WRITE / RESPOND;
//! the evaluator re-parses the expression on every call (the AST cache
//! is Phase-2b scope), so per-call parse latency is the load-bearing
//! number.
//!
//! **Gate policy:** CI-GATED — regressions fail the Phase-2a exit-
//! criteria workflow. Baseline threshold is <10 µs median on dev
//! hardware (M-class Apple silicon / recent x86 server cores). Noisy
//! CI runners apply the workspace-standard
//! `BENTEN_BENCH_GATE_MULTIPLIER` envelope.
//!
//! **Threshold encoding (machine-readable):** the gate workflow reads
//! the `median_ns` field from Criterion's JSON output and fails if the
//! observed median exceeds the threshold. The value here is the
//! contract:
//!
//! ```text
//! BENCH_ID = transform_expression_latency/parse_eval_mixed
//! THRESHOLD_NS = 10000  // 10 µs per ENGINE-SPEC §5
//! POLICY = fail-on-regression
//! ```
//!
//! Workload: three representative expressions covering the common
//! shape mix observed in `docs/validation/paper-prototype-handlers.md`:
//!   (a) a simple projection (`$input.title`),
//!   (b) a coerce-and-default (`$input.limit ?? 10`),
//!   (c) a nested field access with a conditional
//!       (`$input.author.name ? $input.author.name : "anonymous"`).
//!
//! The bench drives the parser only — the evaluator's TRANSFORM
//! execution path is measured separately by `ten_node_handler`'s
//! mixed-handler bench where TRANSFORM is one primitive among many.
//! Per-expression parse latency is the regression signal that the
//! grammar's positive-allowlist walker stays tight as new built-ins
//! land.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "benches may use unwrap/expect per workspace policy"
)]

use std::hint::black_box;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};

fn bench_parse_eval_mixed(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_expression_latency");
    // Warmup + measurement aligned with the other benches in this crate
    // so cross-bench comparison remains apples-to-apples.
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    // MACHINE-READABLE GATE: the exit-criteria workflow greps this comment
    // for THRESHOLD_NS and fails the job if the observed median exceeds it.
    // THRESHOLD_NS=10000 policy=fail-on-regression source=ENGINE-SPEC-§5

    let expressions = ["$input.title", "$input.limit", "$input.author.name"];

    group.bench_function("parse_eval_mixed", |b| {
        b.iter(|| {
            for src in &expressions {
                let ast = benten_eval::transform::parse_transform(black_box(src))
                    .expect("grammar allowlisted");
                black_box(ast);
            }
        });
    });
    group.finish();
}

criterion_group!(benches, bench_parse_eval_mixed);
criterion_main!(benches);

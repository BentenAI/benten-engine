//! #929 closure: round-trip bench harness for the 5 MINIMAL-FOR-DEVSERVER
//! fixtures. Single criterion bench group; one bench per fixture; baseline
//! captured at landing time so a future grammar extension that introduces
//! accidental quadratic complexity in parser dispatch trips a visible
//! regression vs the baseline.
//!
//! Run via `cargo bench -p benten-dsl-compiler` (the bench profile is
//! `[profile.bench]` inherits from `[profile.release]` workspace defaults
//! — opt-3 + lto=thin — so timings reflect production-like compilation).
//!
//! ## Scope rationale (per the #929 disposition)
//!
//! DSL compilation is bounded one-shot (per-devserver-authoring-action +
//! per-test inline-compile); not a runtime hot path. The MINIMAL-FOR-
//! DEVSERVER scope (5 primitive fixtures × <10 numeric literals each) sits
//! firmly below profiling noise. This harness is therefore a REGRESSION
//! TRIPWIRE, not a perf-optimization target: the value is catching a 10x
//! slowdown if the parser dispatch grows quadratic / a future shape rule
//! introduces unbounded backtracking, not shaving microseconds off the
//! happy path.
//!
//! Fixtures mirror `tests/dsl_compiler_round_trips_5_primitive_fixtures.rs`
//! so coverage parity holds between the test suite + the bench surface.

use benten_dsl_compiler::compile_str;
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

const FIXTURE_READ_RESPOND: &str = "handler 'h' { read('post') -> respond }";

const FIXTURE_WRITE_RESPOND: &str =
    "handler 'h' { write('post', { author: 'alice', score: 42 }) -> respond }";

const FIXTURE_TRANSFORM_RESPOND: &str = "handler 'h' { \
        transform({ multiplier: 1.5, base: 100, label: 'computed' }) -> respond \
    }";

const FIXTURE_BRANCH_RESPOND: &str =
    "handler 'h' { read('post') -> branch($post.score > 50) -> respond }";

const FIXTURE_CALL_RESPOND: &str = "handler 'h' { \
        read('post') -> call('other_handler', { input: $post.id }) -> respond \
    }";

fn bench_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("dsl_round_trip");

    group.bench_function("read_respond", |b| {
        b.iter(|| {
            let compiled = compile_str(black_box(FIXTURE_READ_RESPOND)).expect("must compile");
            black_box(compiled);
        });
    });

    group.bench_function("write_respond", |b| {
        b.iter(|| {
            let compiled = compile_str(black_box(FIXTURE_WRITE_RESPOND)).expect("must compile");
            black_box(compiled);
        });
    });

    group.bench_function("transform_respond", |b| {
        b.iter(|| {
            let compiled = compile_str(black_box(FIXTURE_TRANSFORM_RESPOND)).expect("must compile");
            black_box(compiled);
        });
    });

    group.bench_function("branch_respond", |b| {
        b.iter(|| {
            let compiled = compile_str(black_box(FIXTURE_BRANCH_RESPOND)).expect("must compile");
            black_box(compiled);
        });
    });

    group.bench_function("call_respond", |b| {
        b.iter(|| {
            let compiled = compile_str(black_box(FIXTURE_CALL_RESPOND)).expect("must compile");
            black_box(compiled);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_round_trip);
criterion_main!(benches);

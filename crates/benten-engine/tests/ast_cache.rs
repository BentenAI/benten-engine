//! R3-E RED-PHASE pins for G19-E Subgraph AST cache full wire-up
//! (wave 7b single-agent serial after wave-7; §9.2 / phase-2-backlog).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-E +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-E must-pass column):
//!
//! - `tests/subgraph_ast_cache_full_wire_up` — §9.2; C-9
//! - `tests/subgraph_ast_cache_correctness_under_handler_re_register` — §9.2
//! - `tests/subgraph_ast_cache_per_call_parse_cost_reduction` — §9.2 (perf)
//! - `tests/engine_call_no_residual_todo_marker` — §9.2; C-14
//!
//! ## What G19-E establishes (§9.2 / phase-2-backlog)
//!
//! Per phase-2-backlog §9.2: full wire-up of Subgraph AST cache at
//! `crates/benten-engine/src/engine.rs::call` dispatch site. The cache
//! retires the in-code TODO marker; per-call parse cost drops materially
//! (benchmark gate).
//!
//! Per stream-r1-3 cross-pin: the AST-cache wrapper must NOT resurrect
//! the deceptive-sentinel pattern for STREAM-bearing handlers (the
//! discipline R6FP-G1 r6-stream-3 closed). G19-E + G19-C2 cross-wave
//! coordination required.
//!
//! ## RED-PHASE discipline
//!
//! AST cache wire-up does NOT yet exist. R5 implementer drops `#[ignore]`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-E wave-7b wires Subgraph AST cache at Engine::call dispatch site (§9.2)"]
fn subgraph_ast_cache_full_wire_up() {
    // §9.2 pin. G19-E implementer wires this:
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph(crud("post")).unwrap();
    //
    //   // Trigger N calls — the AST-parse step should fire ONCE
    //   // (cached after first call):
    //   for _ in 0..100 {
    //       engine.call(sg, "post:create", json!({"title": "x"})).unwrap();
    //   }
    //
    //   // OBSERVABLE consequence: ast_cache hit-rate is high (≥ 99/100):
    //   let stats = engine.testing_ast_cache_stats();
    //   assert!(stats.hits >= 99, "AST cache hit-rate too low: {:?}", stats);
    //   assert_eq!(stats.misses, 1,
    //       "AST cache miss count should be 1 (first call); got {}", stats.misses);
    //
    // Defends against the "cache wired but never consulted" failure mode.
    unimplemented!("G19-E wires Subgraph AST cache at Engine::call");
}

#[test]
#[ignore = "RED-PHASE: G19-E wave-7b — AST cache invalidates on handler re-register"]
fn subgraph_ast_cache_correctness_under_handler_re_register() {
    // §9.2 correctness pin. G19-E implementer wires this:
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg_v1 = engine.register_subgraph_named("test", subgraph_v1()).unwrap();
    //   engine.call(sg_v1, "main", json!({})).unwrap();
    //
    //   // Re-register with a DIFFERENT shape — cache MUST invalidate:
    //   let sg_v2 = engine.replace_subgraph_named("test", subgraph_v2_different_shape()).unwrap();
    //   let result = engine.call(sg_v2, "main", json!({})).unwrap();
    //
    //   // OBSERVABLE consequence: result reflects v2 shape (not v1);
    //   // cache did NOT serve a stale parse:
    //   assert_eq!(result.shape, "v2",
    //       "AST cache served stale parse after re-register");
    //
    // Defends against the cache-incorrectness failure mode (stale parse
    // surviving handler replacement).
    unimplemented!("G19-E wires AST cache invalidation on handler re-register");
}

#[test]
#[ignore = "RED-PHASE: G19-E wave-7b — AST cache reduces per-call parse cost (perf bench)"]
fn subgraph_ast_cache_per_call_parse_cost_reduction() {
    // §9.2 perf pin. G19-E implementer wires this against a benchmark
    // (criterion or simple wallclock measurement):
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg = engine.register_subgraph(crud("post-with-many-nodes")).unwrap();
    //
    //   // Warm the cache with one call:
    //   engine.call(sg, "post:create", json!({"title": "warm"})).unwrap();
    //
    //   // Measure 1000 cached calls:
    //   let cached_start = std::time::Instant::now();
    //   for _ in 0..1000 {
    //       engine.call(sg, "post:create", json!({"title": "x"})).unwrap();
    //   }
    //   let cached_avg_us = cached_start.elapsed().as_micros() / 1000;
    //
    //   // Compare against a no-cache baseline (test-only feature flag
    //   // disables cache):
    //   let nocache_engine = benten_engine::testing::open_with_ast_cache_disabled().unwrap();
    //   let nocache_sg = nocache_engine.register_subgraph(crud("post-with-many-nodes")).unwrap();
    //   let nocache_start = std::time::Instant::now();
    //   for _ in 0..100 {
    //       nocache_engine.call(nocache_sg, "post:create", json!({"title": "x"})).unwrap();
    //   }
    //   let nocache_avg_us = nocache_start.elapsed().as_micros() / 100;
    //
    //   // Cached path must be measurably faster (>= 20% reduction):
    //   assert!(cached_avg_us * 100 < nocache_avg_us * 80,
    //       "AST cache provides insufficient speedup: cached={}us nocache={}us",
    //       cached_avg_us, nocache_avg_us);
    //
    // OBSERVABLE consequence: the cache delivers measurable per-call
    // parse cost reduction. Pim-2 §3.6b end-to-end pin — would FAIL if
    // the cache wire-up were vacuous (cache exists but never serves).
    unimplemented!("G19-E wires AST cache perf benchmark with measurable speedup");
}

#[test]
#[ignore = "RED-PHASE: G19-E wave-7b — Engine::call no residual TODO marker for AST cache"]
fn engine_call_no_residual_todo_marker() {
    // §9.2 + C-14 architectural pin. G19-E implementer wires this:
    //
    //   let engine_rs_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("src").join("engine.rs");
    //   let src = std::fs::read_to_string(&engine_rs_path).unwrap();
    //
    //   // Locate the call() function body. The AST-cache TODO marker
    //   // must be GONE post-G19-E:
    //   //   - "TODO(phase-3): AST cache" or similar must NOT appear
    //   //   - "TODO(phase-2-backlog §9.2)" must NOT appear
    //   for marker in &["TODO(phase-3): AST cache", "TODO(phase-2-backlog §9.2)",
    //                    "// TODO: ast cache"] {
    //       assert!(!src.contains(marker),
    //           "engine.rs::call still carries residual TODO marker {:?} \
    //            post-G19-E (§9.2 closure incomplete)", marker);
    //   }
    //
    // OBSERVABLE consequence: the in-code TODO marker is retired in
    // lockstep with the wire-up landing (per HARD RULE rule-12 — every
    // marker has a named destination AND closes when the destination
    // closes; this test fires when G19-E mini-review verifies the
    // marker is gone).
    unimplemented!("G19-E wires post-G19-E engine.rs::call no-TODO-marker pin");
}

#[test]
#[ignore = "RED-PHASE: G19-E wave-7b — AST cache preserves STREAM execute() loud-fail (stream-r1-3)"]
fn subgraph_ast_cache_preserves_stream_execute_loud_fail_for_engine_call_path() {
    // stream-r1-3 LOAD-BEARING pin. G19-E implementer must NOT
    // resurrect the deceptive-sentinel pattern for STREAM-bearing
    // handlers — the AST-cache wrapper at Engine::call dispatch must
    // preserve the loud-fail behavior (E_PRIMITIVE_NOT_IMPLEMENTED) for
    // STREAM-bearing subgraphs that were closed by R6FP-G1 r6-stream-3.
    //
    //   let engine = benten_engine::Engine::open_in_memory().unwrap();
    //   let sg_with_stream = engine.register_subgraph_with_stream_handler().unwrap();
    //
    //   // Engine::call (NOT call_stream) on a STREAM-bearing handler
    //   // must STILL loud-fail post-G19-E:
    //   let result = engine.call(sg_with_stream, "main", json!({}));
    //   assert!(result.is_err(),
    //       "Engine::call on STREAM-bearing subgraph must loud-fail \
    //        post-G19-E AST-cache wrap (the stream-r1-3 invariant)");
    //   assert_eq!(result.err().unwrap().error_code(),
    //       benten_errors::ErrorCode::PrimitiveNotImplemented,
    //       "loud-fail must produce E_PRIMITIVE_NOT_IMPLEMENTED");
    //
    // OBSERVABLE consequence: the discipline R6FP-G1 r6-stream-3
    // explicitly closed survives the AST-cache wave. Defends against
    // the resurrection-of-deceptive-sentinel failure mode named by
    // stream-r1-3.
    unimplemented!("G19-E preserves STREAM execute() loud-fail under AST-cache wrap");
}

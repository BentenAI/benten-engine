#![cfg(feature = "phase_2b_landed")] // R3-consolidation: gate red-phase test against R5-pending APIs (see .addl/phase-2b/r3-consolidation.md §4)
//! R3-A red-phase: ChunkSink conformance proptest (G6-A).
//!
//! Pin source: benten-philosophy phil-r1-4 — all conformant sinks produce
//! identical TraceStep sequence for the same handler input.
//! Per §10 disambiguation: R3-A owns this (closer to streaming than perf).
//! Phase 2b TDD red-phase. Owner: R3-A.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_eval::testing::{
    testing_run_handler_against_sink_a, testing_run_handler_against_sink_b,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig { cases: 10_000, ..ProptestConfig::default() })]

    /// Two conformant sink implementations (e.g. tokio::mpsc-based vs
    /// in-memory-vec-based reference) MUST produce identical TraceStep
    /// sequences for the same handler input. phil-r1-4 conformance.
    #[test]
    #[ignore = "Phase 2b G6-A pending — phil-r1-4 conformance"]
    fn prop_chunk_sink_traces_identical_across_conformant_sinks(
        chunk_count in 1usize..64,
        chunk_size in 1usize..256,
        seed in any::<u64>(),
    ) {
        let trace_a = testing_run_handler_against_sink_a(chunk_count, chunk_size, seed);
        let trace_b = testing_run_handler_against_sink_b(chunk_count, chunk_size, seed);
        prop_assert_eq!(
            trace_a, trace_b,
            "conformant sinks must produce identical evaluator-observable trace"
        );
    }
}

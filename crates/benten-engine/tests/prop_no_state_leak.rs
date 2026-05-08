//! R3-A RED-PHASE proptest pin: engine generic does not leak state
//! across backend substitution (G13-B wave-2; plan §4 seed).
//!
//! Pin source: r2-test-landscape §2.1 G13-B row
//! `prop_engine_generic_no_state_leak_across_backend_substitution`;
//! plan §4 seed.
//!
//! ## Property under test
//!
//! Two `EngineGeneric<RedbBackend>` instances backed by SEPARATE redb
//! databases must not share any state across the boundary. The
//! generic-cascade is parameterized on the backend instance, so
//! running operations on engine A must not observably affect engine B.
//!
//! This is a regression guard against the failure shape where the
//! engine inadvertently uses a `static` cache or process-wide state
//! that survives across backend instances — visible as A's writes
//! showing up in B's reads.

#![allow(clippy::unwrap_used, unreachable_code)]

use proptest::prelude::*;

proptest! {
    // G20-B audit-3-mr-3 (Phase-3 close): bumped 50 → 1000 — state
    // machine property tests typically need 1k+ iterations to expose
    // corner-case leaks across backend substitution.
    #![proptest_config(ProptestConfig::with_cases(1_000))]

    #[test]
    #[ignore = "RED-PHASE: G13-B wave-2 introduces EngineGeneric<B>"]
    fn prop_engine_generic_no_state_leak_across_backend_substitution(
        write_count_a in 0u32..16,
        write_count_b in 0u32..16,
    ) {
        // G13-B implementer wires this:
        //
        //   let dir_a = tempfile::tempdir().unwrap();
        //   let dir_b = tempfile::tempdir().unwrap();
        //   let engine_a = benten_engine::Engine::open(dir_a.path()).unwrap();
        //   let engine_b = benten_engine::Engine::open(dir_b.path()).unwrap();
        //
        //   // Write distinct content into each engine:
        //   for i in 0..write_count_a {
        //       engine_a.put_node_with_label(format!("a{}", i), ...).unwrap();
        //   }
        //   for i in 0..write_count_b {
        //       engine_b.put_node_with_label(format!("b{}", i), ...).unwrap();
        //   }
        //
        //   // Engine A's reads see only A's writes:
        //   let count_a = engine_a.scan_nodes().count();
        //   prop_assert_eq!(count_a, write_count_a as usize);
        //
        //   // Engine B's reads see only B's writes (cross-instance isolation):
        //   let count_b = engine_b.scan_nodes().count();
        //   prop_assert_eq!(count_b, write_count_b as usize);
        //
        //   // Engine A does not observe ANY of B's writes:
        //   for i in 0..write_count_b {
        //       prop_assert!(engine_a.get_node_by_label(&format!("b{}", i)).is_none(),
        //           "engine A leaked state from engine B at label 'b{}'", i);
        //   }
        //
        // OBSERVABLE consequence: 50 cases × up to 32 cross-instance
        // writes = up to 1 600 cross-instance lookups; ZERO leakage.
        // Defends against the "static cache" / "process-wide mutex"
        // regression where the generic-cascade refactor accidentally
        // introduced shared state.
        let _ = (write_count_a, write_count_b);
        unimplemented!("G13-B wires engine instance-isolation proptest");
    }
}

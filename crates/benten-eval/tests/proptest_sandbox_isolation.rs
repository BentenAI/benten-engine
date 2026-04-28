//! Phase 2b R3-B — SANDBOX no-state-persists-across-calls property test
//! (G7-A).
//!
//! Property: for any module + any sequence of calls, the per-call
//! Store + Instance lifecycle (D3-RESOLVED) means no module state
//! survives between calls.
//!
//! Pin source: plan §4.
//! Iterations: 10k (per R2 §3).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10_000))]

    /// `prop_sandbox_no_module_state_persists_across_calls` —
    /// per-call default. Regression guard against future opt-in pool
    /// reintroduction.
    ///
    /// Strategy:
    ///   - Synthesize a module with module-global memory initialized
    ///     to `init_value` and a single export `set_global(v)`.
    ///   - Random call sequence: alternate `set_global(N)` calls
    ///     with `read_global()` calls.
    ///   - Assert: every `read_global()` returns `init_value`
    ///     (NOT the most-recently-set value from a prior call) —
    ///     because each call gets a fresh Instance.
    ///
    /// If a future PR re-introduces opt-in pooling without the
    /// per-call Store + Instance lifecycle being the explicit default,
    /// this test fires.
    #[test]
    #[ignore = "Phase 2b G7-C pending (PR #33 engine integration) — no-state-persists property"]
    fn prop_sandbox_no_module_state_persists_across_calls(
        init_value in any::<i32>(),
        set_values in proptest::collection::vec(any::<i32>(), 1..20),
    ) {
        // R5 G7-A pseudo:
        //   let module_cid = synth_module_with_global(init_value);
        //   for set_v in &set_values {
        //       engine.sandbox_call(module_cid, default_manifest(),
        //                           encode_set(*set_v))?;
        //       let read = engine.sandbox_call(module_cid, default_manifest(),
        //                                     encode_read())?;
        //       prop_assert_eq!(decode_i32(read), init_value);
        //   }
        let _ = (init_value, set_values);
        // R4-FP-A — `prop_assume!(false)` DISCARDS the case (silent
        // vacuous-pass after un-ignore); `prop_assert!(false, ...)`
        // actually fails the case, preserving fail-fast intent
        // (rust-test-reviewer.json tq-2b-3).
        prop_assert!(
            false,
            "Phase 2b G7-C pending (PR #33 engine integration): write no-state-persists-across-calls \
             property body (replace this prop_assert!(false) with the \
             read-after-set assertion described in the file pseudo)."
        );
    }
}

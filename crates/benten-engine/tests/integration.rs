//! Integration test aggregator for the benten-engine crate.
//!
//! Each integration scenario is a module under `tests/integration/`. This
//! wrapper compiles them into a single test binary so the crate-test
//! compilation cost is paid ONCE for the whole set rather than once per
//! scenario. (R4 triage (m21): causality was backwards in the v1 comment —
//! sharing the cost IS the point; paying-per-scenario is what we avoid.)
//!
//! Owned by `qa-expert` per R2 landscape §4.3 + §4.6. Phase 2a R3 added
//! the WAIT-resume gate composite + the four-invariant composite + the
//! arch-1 dep-break verify + the Option C end-to-end gate + the five
//! cross-crate scenarios named in the R1 triage fix-now list.

mod integration {
    // Phase 1 scenarios (kept for regression).
    pub mod cap_toctou;
    pub mod caps_crud;
    pub mod change_stream;
    pub mod compromises_regression;
    pub mod cross_process_graph;
    pub mod exit_criteria_all_six;
    pub mod ivm_propagation;
    pub mod nested_tx;
    pub mod stale_view;
    pub mod system_zone_integration;
    pub mod trace_no_persist;
    pub mod tx_atomicity;
    pub mod version_current;

    // Phase 2a R3 (qa-expert owned — §8.5).
    // ---- exit gates ----
    pub mod arch_1_dep_break_verified; // gate 3 (bundled)
    pub mod inv_8_11_13_14_firing; // gate 2 (headline)
    pub mod option_c_end_to_end; // gate 4 (bundled)
    pub mod wait_resume_determinism; // gate 1 (headline)
    // ---- cross-crate fix-now integrations ----
    pub mod wait_inside_wait_serializes_correctly;
    pub mod wait_signal_shape_optional_typing;
    pub mod wallclock_toctou_revokes_mid_iterate;
    pub mod write_authority_lift;
    // ---- Phase 2a R4 fix-pass additions ----
    pub mod view_stale_count; // cov-6 G11-A witness
    // ---- Phase 2a R4b Wave-3c fix-pass additions ----
    pub mod budget_exhausted_trace_emission; // cov M2 — runtime BudgetExhausted firing
}

//! Integration test aggregator for the benten-engine crate.
//!
//! Each integration scenario is a module under `tests/integration/`. This
//! wrapper compiles them into a single test binary so the crate-test
//! compilation cost is paid ONCE for the whole set rather than once per
//! scenario. (R4 triage (m21): causality was backwards in the v1 comment —
//! sharing the cost IS the point; paying-per-scenario is what we avoid.)
//!
//! Owned by `qa-expert` per R2 landscape §4.3 + §4.6.

mod integration {
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
}

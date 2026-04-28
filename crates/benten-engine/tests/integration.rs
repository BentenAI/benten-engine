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
    // ---- Phase 2b R3 (red-phase) — gated on `phase_2b_landed` feature.
    // CI's required-check fleet does NOT enable this feature, so the
    // submodules whose bodies reach into R5-pending APIs are excluded
    // from the integration-test binary build during R3-consolidation.
    // R5 implementer briefs flip the feature on per-group as their
    // surfaces land. See `.addl/phase-2b/r3-consolidation.md` §4.
    //
    // R3-A (G6-B integration) — STREAM + SUBSCRIBE engine-shape integration.
    // G6-B (mini-review cr-g6b-mr-2 fix-pass): ungated. The module bodies
    // are rewritten to align with G6-B's actual implemented surface; tests
    // that can pass against the G6-B stub are live, tests that require
    // G6-A's executor body are `#[ignore]`d with explicit pending-on
    // markers per the file headers.
    pub mod engine_stream; // G6-B exit-1 + dx-r1-2b STREAM
    pub mod engine_subscribe; // G6-B exit-1 + dx-r1-2b SUBSCRIBE
    pub mod stream_composition; // G6-B plan §4 STREAM integration (all `#[ignore]`d pending G6-A)
    pub mod stream_napi; // G6-B streaming-systems must_pass napi async iter
    pub mod subscribe_emit; // G6-B plan §4 SUBSCRIBE integration (all `#[ignore]`d pending G6-A)

    // R3-B (G7-A integration) — SANDBOX composition:
    #[cfg(feature = "phase_2b_landed")]
    pub mod engine_sandbox; // G7-C plan §3 G7-C
    #[cfg(feature = "phase_2b_landed")]
    pub mod sandbox_in_crud; // G7-A plan §4 SANDBOX integration
    #[cfg(feature = "phase_2b_landed")]
    pub mod stream_into_sandbox; // G7-A wsa-18 + arch-pre-r1-9

    // R3-E (red-phase) — WASM target + SuspensionStore + WAIT TTL:
    #[cfg(feature = "phase_2b_landed")]
    pub mod browser_target_bundle_size; // wasm-r1-7 (≤500KB gz cap)
    #[cfg(feature = "phase_2b_landed")]
    pub mod cross_process_wait_resume; // G12-E + Compromise #9 closure
    // G10-B landed (R5 wave-5 — Phase 2b): un-gated below.
    pub mod install_module_rejects_cid_mismatch; // G10-B + D16 dual-CID error
    // R4-FP B-3 (R3-followup) — G10-B install/uninstall integration suite:
    pub mod module_install_in_memory_only_in_browser; // r1-wasm-target G10-B
    pub mod module_install_uninstall_round_trip; // exit criterion #4 5-row matrix
    pub mod module_uninstall_releases_capabilities; // cap-retraction integration
    #[cfg(feature = "phase_2b_landed")]
    pub mod sandbox_compile_time_disabled_on_wasm32; // sec-pre-r1-05 + wasm-r1-3
    #[cfg(feature = "phase_2b_landed")]
    pub mod snapshot_blob_round_trip; // D10 export/import round-trip
    #[cfg(feature = "phase_2b_landed")]
    pub mod suspension_store_round_trip_subscription_cursor; // G12-E + D5 cursor
    #[cfg(feature = "phase_2b_landed")]
    pub mod suspension_store_round_trip_wait_metadata; // G12-E generalization
    #[cfg(feature = "phase_2b_landed")]
    pub mod wait_ttl_expires_via_suspension_store; // D12 + Q4 (G12-E owns TTL)
    #[cfg(feature = "phase_2b_landed")]
    pub mod wasip1_target_canonical_cid; // wasm-r1-1 dual-target gate
}

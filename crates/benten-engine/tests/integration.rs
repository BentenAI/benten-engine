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
    // ---- Phase 2b Wave-8d-types ----
    // Typed-error refactor acceptance: `EvalError::Sandbox` variant
    // surfaces `E_SANDBOX_MODULE_NOT_INSTALLED` end-to-end.
    pub mod sandbox_module_not_installed_emits_typed_error;
    // ---- Phase 2b R3 (originated red-phase; `phase_2b_landed` gate
    // retired pre-R4b). Submodules below align with the production
    // surfaces shipped at phase-2b-close + Phase-3 wave-5b.
    //
    // R3-A (G6-B integration) — STREAM + SUBSCRIBE engine-shape integration.
    // G6-B (mini-review cr-g6b-mr-2 fix-pass): ungated. The module bodies
    // align with G6-B's actual implemented surface; tests that can pass
    // against the G6-B stub are live, tests that require G6-A's executor
    // body are `#[ignore]`d with explicit pending-on markers per the
    // file headers.
    pub mod engine_stream; // G6-B exit-1 + dx-r1-2b STREAM
    pub mod engine_subscribe; // G6-B exit-1 + dx-r1-2b SUBSCRIBE
    pub mod esc_subscribe_integration; // wave-8c-subscribe-infra ESC-7/-9/-10/-13/-14
    pub mod stream_composition; // G6-B plan §4 STREAM integration (all `#[ignore]`d pending G6-A)
    pub mod stream_napi; // G6-B streaming-systems must_pass napi async iter
    pub mod subscribe_emit; // G6-B plan §4 SUBSCRIBE integration (all `#[ignore]`d pending G6-A)

    // R3-B (G7-A integration) — SANDBOX composition. G20-A1 wave-8a
    // un-gated: bodies un-ignored against the production runtime
    // arms shipped at phase-2b-close + Phase-3 G17-A1 wave-5b.
    pub mod engine_sandbox; // G7-C plan §3 G7-C
    pub mod sandbox_in_crud; // G7-A plan §4 SANDBOX integration
    pub mod stream_into_sandbox; // G7-A wsa-18 + arch-pre-r1-9

    // R3-E — WASM target + SuspensionStore + WAIT TTL: G20-A3 wave-8a
    // ungated; `bindings/napi/dist/browser/` artifact path committed
    // (placeholder seed; CI overwrites with production bundle) so the
    // bundle-size + node-binary exclusion checks compile + run by
    // default. (`phase_2b_landed` gate retired pre-R4b across the
    // integration suite; remaining R3-pending surfaces use direct
    // `#[ignore]` markers instead.)
    pub mod browser_target_bundle_size; // wasm-r1-7 (≤500KB gz cap)
    // Phase-3 G20-A2 wave-8a — `phase_2b_landed` cfg gate retired
    // for cross_process_wait_resume per the file header rationale
    // (G12-E + Compromise #9 closure surfaces all landed at the
    // engine-side WAIT TTL runtime expiry path + GC machinery).
    pub mod cross_process_wait_resume; // G12-E + Compromise #9 closure
    // G10-B landed (R5 wave-5 — Phase 2b): un-gated below.
    pub mod install_module_rejects_cid_mismatch; // G10-B + D16 dual-CID error
    // R4-FP B-3 (R3-followup) — G10-B install/uninstall integration suite:
    pub mod module_install_in_memory_only_in_browser; // r1-wasm-target G10-B
    pub mod module_install_uninstall_round_trip; // exit criterion #4 5-row matrix
    pub mod module_uninstall_releases_capabilities; // cap-retraction integration
    // G20-A1 wave-8a un-gated: testing_make_minimal_sandbox_spec
    // helper + body un-ignored.
    pub mod sandbox_compile_time_disabled_on_wasm32; // sec-pre-r1-05 + wasm-r1-3
    pub mod snapshot_blob_round_trip; // D10 export/import round-trip
    // The four `testing_*` helpers (`testing_make_wait_spec_with_ttl_hours`,
    // `testing_call_to_suspend`, `testing_suspension_store_has_wait`,
    // `testing_advance_wait_clock`) + `Engine::resume_with_meta` +
    // `Engine::testing_advance_wait_clock_by` all landed at G19/G20-A2;
    // the wasm32-wasip1 cross-target canonical-CID test runs against the
    // production runtime arms that shipped at phase-2b-close.
    pub mod suspension_store_round_trip_subscription_cursor; // G12-E + D5 cursor
    pub mod suspension_store_round_trip_wait_metadata; // G12-E generalization
    pub mod wait_ttl_expires_via_suspension_store; // D12 + Q4 (G12-E owns TTL)
    pub mod wasip1_target_canonical_cid; // wasm-r1-1 dual-target gate

    // ---- Phase 2b Wave-8h audit-gap fixes ----
    // .addl/phase-2b/r4b-followup-primitive-executor-docs-vs-code-audit.json
    pub mod emit_event_observable_via_emit_broadcast; // Wave-8h fix #2 — EMIT
    pub mod ivm_strategy_b_uses_algorithm_b_view; // Wave-8h fix #3 — IVM-B
    pub mod sandbox_named_manifest_resolves_via_install_module; // Wave-8h fix #1 — SANDBOX
}

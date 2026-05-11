//! Integration test aggregator for the benten-engine crate.
//!
//! Each integration scenario is a module under `tests/integration/`.
//! This wrapper compiles them into a single test binary so the
//! crate-test compilation cost is paid ONCE for the whole set rather
//! than once per scenario.

mod integration {
    // ---- Core engine regression scenarios ----
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

    // ---- Exit gates ----
    pub mod arch_1_dep_break_verified;
    pub mod inv_8_11_13_14_firing;
    pub mod option_c_end_to_end;
    pub mod wait_resume_determinism;

    // ---- Cross-crate integrations ----
    pub mod budget_exhausted_trace_emission; // runtime BudgetExhausted firing
    pub mod view_stale_count;
    pub mod wait_inside_wait_serializes_correctly;
    pub mod wait_signal_shape_optional_typing;
    pub mod wallclock_toctou_revokes_mid_iterate;
    pub mod write_authority_lift;

    // Typed-error refactor acceptance: `EvalError::Sandbox` variant
    // surfaces `E_SANDBOX_MODULE_NOT_INSTALLED` end-to-end.
    pub mod sandbox_module_not_installed_emits_typed_error;
    // ---- STREAM + SUBSCRIBE engine-shape integration ----
    pub mod engine_stream; // STREAM
    pub mod engine_subscribe; // SUBSCRIBE
    pub mod esc_subscribe_integration; // ESC-7/-9/-10/-13/-14
    pub mod stream_composition; // STREAM composition
    pub mod stream_napi; // streaming-systems napi async iter
    pub mod subscribe_emit; // SUBSCRIBE composition

    // ---- SANDBOX composition (against production runtime arms) ----
    pub mod engine_sandbox;
    pub mod sandbox_in_crud;
    pub mod stream_into_sandbox;

    // ---- WASM target + SuspensionStore + WAIT TTL ----
    // `bindings/napi/dist/browser/` artifact path is committed
    // (placeholder seed; CI overwrites with the production bundle) so
    // the bundle-size + node-binary exclusion checks compile + run by
    // default.
    pub mod browser_target_bundle_size; // ≤500KB gz cap
    pub mod cross_process_wait_resume; // Compromise #9 closure
    pub mod install_module_rejects_cid_mismatch; // D16 dual-CID error
    pub mod module_install_in_memory_only_in_browser;
    pub mod module_install_uninstall_round_trip; // exit criterion #4 5-row matrix
    pub mod module_uninstall_releases_capabilities; // cap-retraction
    pub mod sandbox_compile_time_disabled_on_wasm32;
    pub mod snapshot_blob_round_trip; // D10 export/import round-trip
    pub mod suspension_store_round_trip_subscription_cursor; // D5 cursor
    pub mod suspension_store_round_trip_wait_metadata;
    pub mod wait_ttl_expires_via_suspension_store; // D12 + Q4
    pub mod wasip1_target_canonical_cid; // dual-target gate

    // ---- Primitive-executor audit-gap fixes ----
    pub mod emit_event_observable_via_emit_broadcast; // EMIT
    pub mod ivm_strategy_b_uses_algorithm_b_view; // IVM-B
    pub mod sandbox_named_manifest_resolves_via_install_module; // SANDBOX
}

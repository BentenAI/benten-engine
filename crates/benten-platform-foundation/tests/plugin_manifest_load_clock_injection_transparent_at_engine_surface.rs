//! LOAD-BEARING per plan §3 G24-D row + post-R1-triage Q6 ratification.
//!
//! Plugin authors do NOT thread clock through their plugin code; the
//! engine surface injects clock at manifest-load. Plugins MAY override
//! for tests.
//!
//! Fail-closed if clock not injected (sec-3.5-r1-7):
//! `E_UCAN_CLOCK_NOT_INJECTED` (existing Phase-3 ErrorCode).

mod common;

use common::manifest_fixtures::minimal_manifest;

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    D-4F-15 transparent-clock-injection at engine surface: \
    PluginManifest::validate_with_clock(&clock_fn) + install_plugin clock-injection \
    seam. Plugin authors call validate() without clock parameter; engine wraps with \
    clock injection at the load boundary; fail-closed E_UCAN_CLOCK_NOT_INJECTED \
    when not injected. Named destination: plan §3 G24-D-FP-1 (plugin_lifecycle + \
    engine clock-injection seam at install/load boundary). HARD RULE 12 clause-(b) \
    BELONGS-NAMED-NOW."]
#[test]
fn manifest_validate_consults_engine_injected_clock_not_plugin_local_clock() {
    let manifest = minimal_manifest();

    // Future G24-D surface:
    //   PluginManifest::validate_with_clock(&clock_fn) -> Result
    // where clock_fn is engine-provided. Plugin authors call
    // PluginManifest::validate() (without explicit clock parameter)
    // and the engine wraps with clock injection at the load boundary.
    //
    // FAILS-IF-NO-OP because the validate() call without injected
    // clock should surface E_UCAN_CLOCK_NOT_INJECTED.
    let _r = manifest.validate();
    panic!(
        "RED-PHASE: G24-D wave must wire engine-surface clock injection at PluginManifest::validate()"
    );
}

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-1 wave un-ignores) — \
    D-4F-15 transparent-clock-injection at engine surface: \
    PluginManifest::validate_with_clock(&clock_fn) + install_plugin clock-injection \
    seam. Plugin authors call validate() without clock parameter; engine wraps with \
    clock injection at the load boundary; fail-closed E_UCAN_CLOCK_NOT_INJECTED \
    when not injected. Named destination: plan §3 G24-D-FP-1 (plugin_lifecycle + \
    engine clock-injection seam at install/load boundary). HARD RULE 12 clause-(b) \
    BELONGS-NAMED-NOW."]
#[test]
fn admin_ui_v0_install_without_clock_injection_surfaces_e_ucan_clock_not_injected() {
    let manifest = common::manifest_fixtures::admin_ui_v0_manifest();

    // Future G24-D surface mirrors sec-3.5-r1-7 closure: install
    // without clock injection surfaces E_UCAN_CLOCK_NOT_INJECTED.
    // Routes through `crates/benten-platform-foundation/src/
    // plugin_lifecycle.rs::install_plugin(manifest, clock_fn)`; if
    // clock_fn returns DEFAULT_NOW_SECS sentinel, fail-closed.
    let _r = manifest.validate();
    panic!("RED-PHASE: G24-D wave must wire fail-closed clock-not-injected at install_plugin");
}

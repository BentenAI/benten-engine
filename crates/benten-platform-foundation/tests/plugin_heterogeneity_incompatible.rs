//! G24-D row pin — heterogeneity contract (ds-r1-8).
//!
//! Per docs/PLUGIN-MANIFEST.md §3.1: if a plugin's requires include
//! `host:sandbox:exec` AND the installing peer is a thin-compute-
//! surface (browser / edge per CLAUDE.md #17), install fails with
//! `E_PLUGIN_HETEROGENEITY_INCOMPATIBLE`.

mod common;

use common::manifest_fixtures::manifest_requires_sandbox_exec;

#[test]
#[ignore = "RED-PHASE: G24-D wave wires heterogeneity check; un-ignore at G24-D landing"]
fn install_on_thin_compute_surface_with_sandbox_exec_require_fails_with_heterogeneity_error() {
    let _manifest = manifest_requires_sandbox_exec();

    // Future surface: install_plugin consults the device's capability
    // envelope (per D-PHASE-3-25 heterogeneity contract); if peer is
    // thin-compute-surface (runs_sandbox=false), refuse install with
    // ErrorCode::PluginHeterogeneityIncompatible.
    //
    // FAILS-IF-NO-OP because a no-op installer would silently accept
    // and only fail at first SANDBOX invocation runtime.
    panic!(
        "RED-PHASE: G24-D wave must wire E_PLUGIN_HETEROGENEITY_INCOMPATIBLE at install on thin-compute-surface"
    );
}

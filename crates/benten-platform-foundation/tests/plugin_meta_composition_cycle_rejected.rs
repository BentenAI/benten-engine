//! G24-D row pin — meta-plugin composition cycle rejection.
//!
//! Per post-R1-triage Q2 ratification: meta-plugins reference sub-
//! plugins recursively. Install-time cycle detection rejects cycles
//! with `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED`.

mod common;

use common::manifest_fixtures::{minimal_manifest, stub_cid_one, stub_cid_two};

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn meta_plugin_composition_cycle_rejected_at_install_with_typed_error() {
    let mut a = minimal_manifest();
    a.composes_plugins = Some(vec![stub_cid_one()]);

    let mut b = minimal_manifest();
    b.composes_plugins = Some(vec![stub_cid_two()]); // points back to a (cycle)

    // Future surface:
    //   plugin_lifecycle::install_plugin(manifest, install_record)
    //     walks composes_plugins recursively; if any descendant
    //     references the manifest being installed, return
    //     ErrorCode::PluginMetaCompositionCycleRejected.
    //
    // FAILS-IF-NO-OP because cycle would cause infinite walk at
    // runtime (not at install).
    panic!("RED-PHASE: G24-D wave must wire meta-composition cycle detection AS REJECTION");
}

#[ignore = "RED-PHASE-BODY: panic-stub body needs substantive G24-D-FP / wave-N rewrite against landed API surface"]
#[test]
fn meta_plugin_recursive_walk_uses_engine_evaluator_no_new_primitive() {
    let mut a = minimal_manifest();
    a.composes_plugins = Some(vec![stub_cid_one()]);

    // Future surface: walking meta-plugin composition uses the existing
    // 12-primitive vocabulary (likely SUBSCRIBE + CALL recursion);
    // NO new primitive minted. Per CLAUDE.md baked-in #1 (12 primitives
    // irreducible).
    //
    // Substance: grep-assert benten-eval/src/ for no new primitive
    // variant; or assertion at the meta_composition surface that the
    // walk is implemented via CALL/SUBSCRIBE.
    panic!("RED-PHASE: G24-D wave must wire recursive walk via existing 12 primitives");
}

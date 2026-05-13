//! G24-D row pin — meta-plugin composition cycle rejection.
//!
//! Per post-R1-triage Q2 ratification: meta-plugins reference sub-
//! plugins recursively. Install-time cycle detection rejects cycles
//! with `E_PLUGIN_META_COMPOSITION_CYCLE_REJECTED`.

mod common;

use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_platform_foundation::plugin_manifest::detect_composition_cycle;
use common::manifest_fixtures::minimal_manifest;

#[test]
fn meta_plugin_composition_cycle_rejected_with_typed_error_code() {
    // SUBSTANTIVE per pim-2 §3.6b: build a cycle A -> B -> A;
    // exercise detect_composition_cycle at HEAD; expect typed
    // PluginMetaCompositionCycleRejected. Would-FAIL if cycle-walk
    // missed (would return Ok or infinite-loop).
    //
    // Use distinct CID values so root_cid != child_cid identity is real.
    let a_cid = Cid::from_blake3_digest([0xAAu8; 32]);
    let b_cid = Cid::from_blake3_digest([0xBBu8; 32]);

    let mut a = minimal_manifest();
    a.composes_plugins = Some(vec![b_cid]);

    let mut b = minimal_manifest();
    b.composes_plugins = Some(vec![a_cid]); // cycle back to a

    // Resolver: returns b's manifest when looking up b_cid; nothing
    // else.
    let resolver = |cid: &Cid| -> Option<_> { if *cid == b_cid { Some(b.clone()) } else { None } };

    let result = detect_composition_cycle(a_cid, &a, &resolver);
    let err = result.expect_err("cycle MUST be rejected");
    assert_eq!(
        err,
        ErrorCode::PluginMetaCompositionCycleRejected,
        "cycle MUST surface typed PluginMetaCompositionCycleRejected; \
         would-FAIL if cycle-walk skipped"
    );
}

#[test]
fn meta_plugin_acyclic_composition_admitted_no_typed_error() {
    // SUBSTANTIVE boundary per pim-2 §3.6b: non-cyclic composition
    // (A -> B -> nothing) admits. Would-FAIL if detector over-rejected.
    let a_cid = Cid::from_blake3_digest([0xAAu8; 32]);
    let b_cid = Cid::from_blake3_digest([0xBBu8; 32]);

    let mut a = minimal_manifest();
    a.composes_plugins = Some(vec![b_cid]);

    let mut b = minimal_manifest();
    b.composes_plugins = None; // leaf

    let resolver = |cid: &Cid| -> Option<_> { if *cid == b_cid { Some(b.clone()) } else { None } };

    let result = detect_composition_cycle(a_cid, &a, &resolver);
    result.expect("acyclic chain MUST admit");
}

#[ignore = "RED-PHASE (Phase 4-Foundation R5 G24-D-FP-2 wave un-ignores) — \
    Verifies recursive composition walk is implemented via engine evaluator's existing \
    12 primitives (CALL/SUBSCRIBE) without minting a new primitive. The grep-walk \
    defense lives in the engine evaluator integration path that G24-D-FP-2 ships \
    (manifest_envelope_chain_validation.rs walks composition via engine evaluator). \
    Named destination: plan §3 G24-D-FP-2 row (manifest envelope chain validator). \
    HARD RULE 12 clause-(b) BELONGS-NAMED-NOW: plan row pre-exists; this pin couples to it."]
#[test]
fn meta_plugin_recursive_walk_uses_engine_evaluator_no_new_primitive() {
    // Future surface at G24-D-FP-2: walking meta-plugin composition
    // uses the existing 12-primitive vocabulary (CALL + SUBSCRIBE
    // recursion); NO new primitive minted. Per CLAUDE.md baked-in #1
    // (12 primitives irreducible).
    //
    // Substance shape: grep-assert benten-eval/src/ for no new
    // primitive variant; verify the composition walk path uses CALL
    // or SUBSCRIBE.
    panic!(
        "Phase 4-Foundation R5 G24-D-FP-2 un-ignores once \
         manifest_envelope_chain_validation.rs wires composition-walk \
         through engine evaluator"
    );
}

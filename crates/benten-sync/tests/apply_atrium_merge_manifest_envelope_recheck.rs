//! **LOAD-BEARING** per R2 §5 Gap fix #4 — defense-in-depth pin at
//! `apply_atrium_merge` boundary.
//!
//! Per T8 defense narrative (sec-3.5-r1-8 + sec-4f-r1-3): even if a
//! chain leaks past the synchronous write boundary, the merge-boundary
//! recheck catches it.
//!
//! Per Phase-3 PR #161 G16-B-F sec-r4r1-2 closure (structural-always-
//! on per-row cap-recheck inside `apply_atrium_merge`): that recheck
//! path EXTENDS at Phase 4-Foundation to also call the manifest-
//! envelope check.
//!
//! ## R4b-FP-1 Seam 3 — substantive grep-walk un-ignore
//!
//! `benten-sync` MUST NOT depend on `benten-engine` (`dependency_edges.rs`
//! arch-r1-11), so this test exercises the integration via grep-walk
//! of `benten-engine/src/`. The integration is observable from source:
//!
//! 1. `manifest_envelope_recheck.rs` defines the
//!    `ManifestEnvelopeRechecker` trait + `ManifestEnvelopeRecheckOutcome`
//!    enum + `NoopManifestEnvelopeRechecker` default + the
//!    `outcome_to_row_reject` helper that surfaces typed
//!    `E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE`.
//! 2. `engine.rs::apply_atrium_merge` consults the configured
//!    rechecker AFTER `policy.check_write` (Layer-3 refinement on
//!    Layer-1 revocation).
//! 3. `Engine::set_manifest_envelope_rechecker` exposes the setter
//!    so production adapters can install the rechecker port.
//!
//! pim-18 §3.6f vacuous-truth defense: every file existence + content
//! sized > 100 bytes asserted BEFORE substantive grep.

#![allow(clippy::unwrap_used)]

#[test]
fn apply_atrium_merge_per_row_recheck_extends_to_manifest_envelope_check() {
    // R4b-FP-1 Seam 3 — would-FAIL-if-no-op'd: regression that removes
    // the integration call OR deletes the port module tripis this.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let engine_root = manifest_dir
        .parent()
        .expect("workspace crates/ parent")
        .join("benten-engine")
        .join("src");
    assert!(
        engine_root.exists() && engine_root.is_dir(),
        "engine src dir MUST exist: {engine_root:?} (pim-18 §3.6f vacuous-truth defense)"
    );

    let engine_rs = engine_root.join("engine.rs");
    let port_module = engine_root.join("manifest_envelope_recheck.rs");

    assert!(engine_rs.exists(), "engine.rs MUST exist");
    assert!(
        port_module.exists(),
        "manifest_envelope_recheck.rs (Seam 3 port module) MUST exist"
    );

    let engine_src = std::fs::read_to_string(&engine_rs).unwrap();
    let port_src = std::fs::read_to_string(&port_module).unwrap();

    assert!(
        engine_src.len() > 1000,
        "engine.rs sized > 1000 bytes (vacuous-truth defense)"
    );
    assert!(
        port_src.len() > 100,
        "manifest_envelope_recheck.rs sized > 100 bytes"
    );

    // Substance A — port module defines the surfaces.
    assert!(
        port_src.contains("trait ManifestEnvelopeRechecker"),
        "MUST define `trait ManifestEnvelopeRechecker`"
    );
    assert!(
        port_src.contains("enum ManifestEnvelopeRecheckOutcome"),
        "MUST define `enum ManifestEnvelopeRecheckOutcome`"
    );
    assert!(
        port_src.contains("NoopManifestEnvelopeRechecker"),
        "MUST provide `NoopManifestEnvelopeRechecker` default"
    );
    assert!(
        port_src.contains("PluginDelegationOutsideManifestEnvelope"),
        "row-reject helper MUST surface typed E_PLUGIN_DELEGATION_OUTSIDE_MANIFEST_ENVELOPE"
    );

    // Substance B — apply_atrium_merge wires the integration call AND
    // engine exposes the setter.
    assert!(
        engine_src.contains("manifest_envelope_recheck::outcome_to_row_reject"),
        "engine.rs MUST call outcome_to_row_reject in apply_atrium_merge's per-row loop"
    );
    assert!(
        engine_src.contains("set_manifest_envelope_rechecker"),
        "engine.rs MUST expose `set_manifest_envelope_rechecker` setter"
    );

    // Substance C — ordering: envelope recheck AFTER cap-recheck.
    let cap_idx = engine_src
        .find("policy.check_write(&ctx)")
        .expect("Phase-3 G16-B-F cap-recheck site MUST remain");
    let env_idx = engine_src
        .find("manifest_envelope_recheck::outcome_to_row_reject")
        .expect("Seam 3 envelope-recheck call MUST be wired");
    assert!(
        env_idx > cap_idx,
        "Seam 3 envelope-recheck MUST run AFTER cap-recheck (Layer-3 \
         refinement on Layer-1 revocation; preserves PR #161 ordering)"
    );
}

#[test]
fn apply_atrium_merge_legitimate_chain_admitted_no_regression() {
    // R4b-FP-1 false-positive defense — outcome enum carries
    // `Admitted` + `NotApplicable` variants; Noop returns NotApplicable
    // so engines without a configured rechecker admit every row
    // (Phase-3 baseline preserved).
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let port_module = manifest_dir
        .parent()
        .expect("workspace crates/ parent")
        .join("benten-engine")
        .join("src")
        .join("manifest_envelope_recheck.rs");
    assert!(port_module.exists());
    let port_src = std::fs::read_to_string(&port_module).unwrap();
    assert!(port_src.len() > 100);

    assert!(
        port_src.contains("Admitted"),
        "outcome enum MUST carry `Admitted` variant (legitimate-chain admit path)"
    );
    assert!(
        port_src.contains("NotApplicable"),
        "outcome enum MUST carry `NotApplicable` variant (engines without configured rechecker)"
    );

    // Noop impl returns NotApplicable.
    let noop_section = port_src
        .split("impl ManifestEnvelopeRechecker for NoopManifestEnvelopeRechecker")
        .nth(1)
        .expect("Noop impl section MUST exist");
    let noop_first_500: String = noop_section.chars().take(500).collect();
    assert!(
        noop_first_500.contains("NotApplicable"),
        "NoopManifestEnvelopeRechecker MUST return NotApplicable so unconfigured \
         engines behave identically to Phase-3 baseline (no false-positive)"
    );
}

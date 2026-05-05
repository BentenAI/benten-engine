//! R3-A RED-PHASE pin: HLC used in AttributionFrame per Inv-14
//! (G14-D wave-5a; D-A G14-pre-D follow-on).
//!
//! Pin source: r2-test-landscape §2.0 wave-1pre row
//! `hlc_used_in_attribution_frame_per_inv_14`; D-A; Inv-14.
//!
//! ## What this pins
//!
//! G14-pre-D shipped HLC infrastructure in `benten-core`. G14-D wires
//! HLC into the `AttributionFrame` so every attributable write carries
//! a `BentenHlc` stamp, supporting:
//!
//! - **Inv-14 device-grain attribution** — every write attributable
//!   to a `(peer_did, device_did, hlc)` triple.
//! - **Loro per-property LWW** at G16-B — uses the `BentenHlc`
//!   ordering as the LWW tiebreaker.
//! - **Cross-process WAIT-resume `cap_snapshot_hash`** — the resume
//!   envelope carries the HLC at suspension time so the resume site
//!   knows whether cap-state moved forward / backward across the
//!   suspension window.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D wire HLC into AttributionFrame; G14-pre-D HLC already shipped"]
fn hlc_used_in_attribution_frame_per_inv_14() {
    // D-A + Inv-14 pin. G14-D implementer wires this against the
    // production-runtime arm:
    //
    //   let dir = tempfile::tempdir().unwrap();
    //   let engine = benten_engine::Engine::open(dir.path()).unwrap();
    //   // ... do a write that triggers an AttributionFrame ...
    //   let attribution = engine.last_attribution_frame_for_test();
    //   // The frame carries an HLC stamp:
    //   assert!(attribution.hlc().physical_ms() > 0);
    //   // The stamp is monotonic with subsequent writes:
    //   let prev_hlc = attribution.hlc();
    //   // ... do another write ...
    //   let next_hlc = engine.last_attribution_frame_for_test().hlc();
    //   assert!(next_hlc > prev_hlc,
    //       "AttributionFrame HLC must be strictly monotonic per Inv-14 / D-A");
    //
    // OBSERVABLE consequence: every attributable write carries a
    // strictly-monotonic HLC stamp on the production runtime path.
    // Defends against the failure shape where AttributionFrame ships
    // with a placeholder zero-HLC field (passing structural sentinel
    // tests but failing the load-bearing per-property LWW + Inv-14
    // device-grain consumers downstream).
    //
    // Per pim-2 §3.6b: this test drives the production entry point
    // (Engine::put_node_with_label or similar) and asserts an
    // OBSERVABLE behavioral consequence (HLC strictly increasing
    // across two writes) that would FAIL if the HLC integration were
    // silently no-op'd.
    unimplemented!(
        "G14-D wires AttributionFrame HLC monotonicity assertion via production runtime path"
    );
}

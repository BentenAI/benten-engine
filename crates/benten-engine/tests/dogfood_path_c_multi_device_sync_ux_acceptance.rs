//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for dogfood path (c):
//! multi-device sync — workflow A→B propagates ≤3s on loopback; per-device
//! CURRENT visible.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 13 (LOAD-BEARING §3.6f substantive); closes ux-r1-1 + ratification
//! #2 (multi-device sync exit-criterion 16 carried into admin UI v0
//! dogfood surface).
//!
//! ## Per pim-18 §3.6f LOAD-BEARING substantive shape
//!
//! Production-runtime arms:
//! 1. **2-peer Atrium real iroh loopback** (ratification #2 + criterion 16
//!    end-to-end). NOT mocked transport.
//! 2. **Write on device A → visible on device B in ≤3s** observable wall-clock.
//! 3. **Each device's CURRENT pointer is independently visible** — admin
//!    UI on device A shows its own CURRENT; same on B. Device-DID-attested
//!    envelope per `DeviceAttestationEnvelope` V2 (Phase-3 G16-D wave-6b).
//! 4. **Replay across reload+sync produces same trace** — close+reopen
//!    both peers; sync-rehydrate; replay matches.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A + G24-F + G16 inheritance wires this. Pin source: r2-test-landscape.md §2.6 row 13 + ratification #2. LOAD-BEARING per pim-18 §3.6f: real iroh-loopback 2-peer; A→B latency ≤3s wall-clock; per-device CURRENT; replay across reload. Would FAIL if iroh transport stubbed."]
fn dogfood_path_c_multi_device_sync_ux_acceptance() {
    // G24-A + G24-F wires this; inherits Phase-3 G16-B-E + G16-D
    // criterion 1 + 15 + 16 infrastructure. Substantive shape:
    //
    //   let harness_a = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   let harness_b = harness_a.spawn_second_peer_in_same_atrium();
    //
    //   // Sanity (1): both peers are full peers running real iroh
    //   // transport per ratification #2 + CLAUDE.md baked-in #17 (a):
    //   assert!(harness_a.uses_real_iroh_transport());
    //   assert!(harness_b.uses_real_iroh_transport());
    //
    //   // (2) Write on A → visible on B within wall-clock budget:
    //   let workflow_cid = harness_a.create_workflow("from-device-a");
    //   let t0 = std::time::Instant::now();
    //   let observed_on_b = harness_b
    //       .await_workflow_visible(workflow_cid, std::time::Duration::from_secs(3));
    //   let propagation_latency = t0.elapsed();
    //   assert!(
    //       observed_on_b.is_some(),
    //       "Dogfood path (c): A→B sync MUST surface workflow within 3s \
    //        wall-clock per ratification #2; not visible after {:?}",
    //       propagation_latency,
    //   );
    //   assert!(
    //       propagation_latency <= std::time::Duration::from_secs(3),
    //       "Dogfood path (c): propagation latency {:?} > 3s wall-clock \
    //        budget per ratification #2",
    //       propagation_latency,
    //   );
    //
    //   // (3) Per-device CURRENT pointers are independently visible on
    //   // each device's admin UI:
    //   let current_on_a = harness_a.admin_ui_workflows_current_pointer();
    //   let current_on_b = harness_b.admin_ui_workflows_current_pointer();
    //   assert_eq!(
    //       current_on_a.workflow_cid, workflow_cid,
    //       "Device A admin UI must show local CURRENT = workflow created here"
    //   );
    //   assert_eq!(
    //       current_on_b.workflow_cid, workflow_cid,
    //       "Device B admin UI must show synced CURRENT after merge"
    //   );
    //   // Per CLAUDE.md baked-in #17 + Phase-3 G16-D wave-6b: each
    //   // device's CURRENT carries its own device-DID-attested envelope:
    //   assert_ne!(
    //       current_on_a.device_attestation.device_did,
    //       current_on_b.device_attestation.device_did,
    //       "Per-device CURRENT pointers must carry distinct \
    //        device-DID-attested envelopes per criterion 16"
    //   );
    //
    //   // (4) Replay across reload+sync produces same trace:
    //   let trace_pre_reload = harness_b.trace_capture(|h| {
    //       h.dispatch_workflow_by_cid(workflow_cid).unwrap()
    //   });
    //   let path_b = harness_b.engine_path();
    //   drop(harness_b);
    //   let reopened_b = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   reopened_b.open_engine_at(&path_b);
    //   let trace_post_reload = reopened_b.trace_capture(|h| {
    //       h.dispatch_workflow_by_cid(workflow_cid).unwrap()
    //   });
    //   assert_eq!(
    //       trace_pre_reload.canonical_event_bytes(),
    //       trace_post_reload.canonical_event_bytes(),
    //       "Dogfood path (c): post-reload replay MUST match pre-reload \
    //        per pim-18 §3.6f production-runtime determinism arm"
    //   );
    //
    // OBSERVABLE consequence: multi-device dogfood path works under
    // real iroh + real cross-peer DAG sync + per-device attestation.
    // Defends against the failure shape where mock transport in tests
    // passes but real-world devices don't see each other.
    unimplemented!(
        "G24-A + G24-F wire dogfood path (c): 2-peer iroh-loopback \
         with 4-arm production-runtime check (≤3s wall-clock + per-device \
         CURRENT + reload replay) per pim-18 §3.6f"
    );
}

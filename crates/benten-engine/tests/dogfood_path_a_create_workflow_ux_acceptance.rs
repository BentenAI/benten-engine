//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for dogfood path (a):
//! create a workflow in admin UI v0 with ≤5 clicks; persisted; invokable.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 11 (LOAD-BEARING §3.6f substantive) + §5 table row 6 (dogfood
//! paths MUST assert production-runtime arm, not `.toBeVisible`);
//! closes ux-r1-1 BLOCKER + D-4F-8 (admin UI v0 as first plugin).
//!
//! ## Per pim-18 §3.6f LOAD-BEARING substantive shape
//!
//! Per R2 §5 table row 6: "each dogfood test asserts production-runtime
//! arm (workflow persisted to redb; reload retrieves same CID; replay
//! produces same trace) — NOT shape-only (`.toBeVisible`)."
//!
//! This pin's body MUST exercise four production-runtime checks:
//! 1. **UX-acceptance click count** — workflow creation is reachable
//!    in ≤5 clicks from admin UI root (ux-r1-1 acceptance criterion).
//! 2. **Persistence to redb** — after the workflow form submit,
//!    inspect the redb backend; assert the workflow content is at a
//!    stable CID.
//! 3. **Reload retrieves same CID** — close + reopen the engine;
//!    re-fetch the workflow; CID matches the original.
//! 4. **Replay produces same trace** — dispatch the workflow via the
//!    evaluator a second time; trace events are byte-identical to first.
//!
//! Defends against the failure shape where admin UI form renders but
//! the workflow either (a) never persists, (b) persists with non-stable
//! CID, or (c) replays differently across reloads.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A + G24-B wave-6/6b wire this. Pin source: r2-test-landscape.md §2.6 row 11 + §5 table row 6. LOAD-BEARING per pim-18 §3.6f: 4 production-runtime arms — ≤5 clicks; persisted-to-redb; reload-retrieves-same-CID; replay-produces-same-trace. Would FAIL if any arm shape-only-shaped."]
fn dogfood_path_a_create_workflow_ux_acceptance() {
    // G24-A + G24-B wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // (1) UX-acceptance click count per ux-r1-1:
    //   let click_recorder = harness.start_click_recording();
    //   let workflow_form = harness.navigate_to_workflow_creation();
    //   workflow_form.fill_fields(&[
    //       ("name", "my_first_workflow"),
    //       ("trigger", "manual"),
    //       ("body", "// 1+1"),
    //   ]);
    //   let workflow_cid = workflow_form.submit().unwrap();
    //   let click_count = click_recorder.stop();
    //   assert!(
    //       click_count <= 5,
    //       "Dogfood path (a) UX-acceptance: workflow creation MUST be \
    //        reachable in ≤5 clicks per ux-r1-1; saw {}",
    //       click_count,
    //   );
    //
    //   // (2) Persistence to redb per pim-18 §3.6f:
    //   let redb_node = harness.backend_for_test().get(&workflow_cid).unwrap();
    //   assert!(
    //       redb_node.is_some(),
    //       "Dogfood path (a): workflow MUST persist to redb after \
    //        form submit; CID {} returned None — write was shape-only \
    //        DOM update without engine persistence",
    //       workflow_cid,
    //   );
    //
    //   // (3) Reload retrieves same CID — close + reopen engine:
    //   let engine_path = harness.engine_path();
    //   drop(harness);
    //   let reopened = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   reopened.open_engine_at(&engine_path);
    //   let reread = reopened
    //       .backend_for_test()
    //       .get(&workflow_cid)
    //       .unwrap();
    //   assert_eq!(
    //       reread, Some(redb_node.unwrap()),
    //       "Dogfood path (a): reopen-engine MUST retrieve same node \
    //        bytes at same CID — content-addressing failure"
    //   );
    //
    //   // (4) Replay produces same trace per pim-18 §3.6f:
    //   let trace_1 = reopened.trace_capture(|h| {
    //       h.dispatch_workflow_by_cid(workflow_cid).unwrap()
    //   });
    //   let trace_2 = reopened.trace_capture(|h| {
    //       h.dispatch_workflow_by_cid(workflow_cid).unwrap()
    //   });
    //   assert_eq!(
    //       trace_1.canonical_event_bytes(),
    //       trace_2.canonical_event_bytes(),
    //       "Dogfood path (a): workflow replay MUST be deterministic \
    //        per pim-18 §3.6f production-runtime arm; trace_1 != trace_2 \
    //        suggests non-deterministic evaluator path or hidden state"
    //   );
    //
    // OBSERVABLE consequence: dogfood path (a) end-to-end works under
    // real engine + real redb + real evaluator. Defends against shape-
    // only "form renders" pass. ux-r1-1 BLOCKER closure.
    unimplemented!(
        "G24-A + G24-B wire dogfood path (a): create-workflow with \
         4-arm production-runtime check (≤5 clicks + persisted + \
         reload-same-CID + replay-same-trace) per pim-18 §3.6f"
    );
}

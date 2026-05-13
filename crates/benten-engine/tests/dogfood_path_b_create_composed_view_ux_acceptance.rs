//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for dogfood path (b):
//! create a composed view in admin UI v0 with ≤4 clicks; preview
//! ≤200ms / ≤1s.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 12 (LOAD-BEARING §3.6f substantive); closes ux-r1-1 + ux-r1-16
//! (composed-view live-preview latency UX bound).
//!
//! ## Per pim-18 §3.6f LOAD-BEARING substantive shape
//!
//! Production-runtime arms:
//! 1. **UX-acceptance click count** ≤4 to reach composed-view creator
//!    + form submit (ux-r1-1).
//! 2. **Live preview latency** — first preview render ≤200ms p50,
//!    ≤1s p99 (ux-r1-16) — measured on real evaluator + materializer
//!    pipeline (not mocked).
//! 3. **Composed view persisted as IVM-subgraph** — CID stable across
//!    reload (G23-0a + D-4F-2 consumer).
//! 4. **Live preview propagates through SUBSCRIBE seam** — a write to
//!    underlying labels triggers a re-render via `on_change_as_with_cursor`
//!    (G23-B materializer + T12 consumer).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A + G24-C wave-6/6b wire this. Pin source: r2-test-landscape.md §2.6 row 12. LOAD-BEARING per pim-18 §3.6f: 4 production-runtime arms — ≤4 clicks; preview latency p50/p99; persisted-as-IVM-subgraph; live-preview via SUBSCRIBE seam. Would FAIL if measured against mock pipeline."]
fn dogfood_path_b_create_composed_view_ux_acceptance() {
    // G24-A + G24-C wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // (1) UX-acceptance click count per ux-r1-1:
    //   let click_recorder = harness.start_click_recording();
    //   let view_form = harness.navigate_to_composed_view_creator();
    //   view_form.select_underlying_labels(&["notes", "tags"]);
    //   view_form.select_view_strategy("filter+project");
    //   let view_cid = view_form.submit().unwrap();
    //   let click_count = click_recorder.stop();
    //   assert!(
    //       click_count <= 4,
    //       "Dogfood path (b): composed-view creation MUST be reachable \
    //        in ≤4 clicks per ux-r1-1; saw {}",
    //       click_count,
    //   );
    //
    //   // (2) Live-preview latency per ux-r1-16:
    //   let mut latencies = vec![];
    //   for _ in 0..100 {
    //       let t0 = std::time::Instant::now();
    //       let _preview = harness.composed_view_render_preview(view_cid);
    //       latencies.push(t0.elapsed());
    //   }
    //   latencies.sort();
    //   let p50 = latencies[50];
    //   let p99 = latencies[99];
    //   assert!(
    //       p50 <= std::time::Duration::from_millis(200),
    //       "Dogfood path (b): live-preview p50 MUST be ≤200ms per \
    //        ux-r1-16; saw {:?}",
    //       p50,
    //   );
    //   assert!(
    //       p99 <= std::time::Duration::from_secs(1),
    //       "Dogfood path (b): live-preview p99 MUST be ≤1s per \
    //        ux-r1-16; saw {:?}",
    //       p99,
    //   );
    //
    //   // (3) View persisted as IVM-subgraph CID stable across reload:
    //   let view_node_before = harness.backend_for_test().get(&view_cid).unwrap();
    //   let engine_path = harness.engine_path();
    //   drop(harness);
    //   let reopened = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //   reopened.open_engine_at(&engine_path);
    //   let view_node_after = reopened.backend_for_test().get(&view_cid).unwrap();
    //   assert_eq!(
    //       view_node_before, view_node_after,
    //       "Dogfood path (b): composed view MUST persist as IVM-subgraph \
    //        node with stable CID across reload"
    //   );
    //
    //   // (4) Live preview propagates via SUBSCRIBE per T12 / G23-B:
    //   let trace = reopened.trace_capture(|h| {
    //       let preview_handle = h.composed_view_open_live_preview(view_cid);
    //       // Cause a write to underlying notes label:
    //       h.write_test_note();
    //       preview_handle.await_re_render();
    //   });
    //   assert!(
    //       trace.calls_to("Engine::on_change_as_with_cursor").len() >= 1,
    //       "Dogfood path (b): live-preview MUST propagate via \
    //        on_change_as_with_cursor seam (cap-recheck enabled); \
    //        ZERO invocations seen — preview is polling or bypassing"
    //   );
    //
    // OBSERVABLE consequence: dogfood path (b) end-to-end works against
    // real materializer + real subscribe. Defends against shape-only
    // "preview pane renders empty" failure mode.
    unimplemented!(
        "G24-A + G24-C wire dogfood path (b): composed-view with 4-arm \
         production-runtime check per pim-18 §3.6f"
    );
}

//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! shell rendering content via the Materializer trait (G23-B consumer).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 3 (substantive); G23-B materializer pipeline consumer.
//!
//! ## What this pin establishes
//!
//! Per Phase-4-Foundation G23-B materializer canary: content render
//! flows through a Materializer trait impl, not bespoke handcoded
//! "renderProperty(node)" logic. Admin UI's Content Types + Views
//! routes consume the materializer to project subgraph data to UI shape.
//!
//! This pin is substantive (not grep-shaped): it actually dispatches a
//! materialization through the admin UI render path + asserts the
//! Materializer trait was on the call stack.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this; depends on G23-B materializer canary (wave-5). Pin source: r2-test-landscape.md §2.6 row 3. Substantive: dispatch admin UI content render + assert Materializer trait was invoked, not bespoke renderProperty."]
fn admin_ui_v0_shell_consumes_materializer_for_content_render() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // Author a fixture note (canonical content-type per
    //   // schema_fixtures.rs canonical-note-type) + dispatch the
    //   // admin UI Content Types route render against it:
    //   let note_cid = harness.write_canonical_note_fixture();
    //
    //   let trace = harness.trace_capture(|h| {
    //       h.dispatch_admin_ui_content_type_render(note_cid)
    //   });
    //
    //   // Per pim-18 §3.6f: SUBSTANCE check — Materializer trait was
    //   // actually invoked, not just imported:
    //   assert!(
    //       trace.invoked_traits.contains("benten_ivm::Materializer"),
    //       "Admin UI content render MUST invoke the Materializer trait \
    //        per G23-B consumer; trace shows ZERO Materializer invocations \
    //        — admin UI is hand-coding renderProperty"
    //   );
    //
    //   // Defense against shape-only pass: assert the materialized
    //   // output bytes actually flowed to the DOM:
    //   let materialized = harness.last_materializer_output();
    //   let rendered_dom = trace.final_dom_text();
    //   assert!(
    //       rendered_dom.contains(&materialized.distinguishing_token()),
    //       "Materializer output MUST flow to DOM; saw rendered DOM \
    //        '{}' but materializer produced '{:?}'",
    //       rendered_dom,
    //       materialized,
    //   );
    //
    // OBSERVABLE consequence: content render delegates to materializer
    // pipeline; admin UI doesn't fork its own render path. Defends
    // against the G23-B-consumer-bypass failure shape.
    unimplemented!(
        "G24-A wires admin UI consumes-materializer pin; depends on \
         G23-B Materializer trait at wave-5"
    );
}

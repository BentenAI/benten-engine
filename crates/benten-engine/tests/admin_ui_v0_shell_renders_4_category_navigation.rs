//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for G24-A admin UI v0
//! shell renders the 4-category navigation IA.
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 1 (LOAD-BEARING substantive); closes ratification #4 + ux-r1-8 +
//! plugin-arch-r1-12 + Family F1 gap-#6 (R2 §5 — 4-category nav IA
//! substance check, NOT shape-only).
//!
//! ## What this pin establishes
//!
//! Per ratification #4 (D-4F-4 admin UI v0 IA): the navigation surface
//! is **4 categories** — Plugins / Workflows / Content Types / Views.
//!
//! Per Family F1 gap-#6 (`r2-test-landscape.md` §5 risk #6): the test
//! MUST assert each route renders a **real component fed by the engine's
//! substrate** — NOT a shape-only `nav.querySelectorAll('a').length == 4`.
//! Per pim-18 §3.6f SHAPE-not-SUBSTANCE pre-flight, the substantive
//! check pairs DOM-shape with production-runtime arm:
//!
//! 1. **4 routes exist** in the admin UI router config.
//! 2. **Each route resolves to a real React component** (not a placeholder
//!    `<div>TODO</div>`).
//! 3. **Each component invokes `Engine::read_node_as`** during render to
//!    fetch its substrate data — verified via runtime-trace harness.
//! 4. **Each route renders content sourced from the engine**, not static
//!    mocks — verified via DOM snapshot vs. engine-fixture data.

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 canary wires this. Pin source: r2-test-landscape.md §2.6 row 1 + ratification #4 + Family F1 gap-#6 substance check. Substantive: pairs 4-route DOM presence with runtime-trace assertion that each route's component invokes Engine::read_node_as + renders engine-sourced (not mock) substrate."]
fn admin_ui_v0_shell_renders_4_category_navigation_substantively() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // (1) Routes exist — DOM-side shape check:
    //   let nav = harness.render_admin_ui_root();
    //   let categories: Vec<&str> = nav
    //       .query_selector_all("[role='navigation'] a")
    //       .iter()
    //       .map(|a| a.text())
    //       .collect();
    //   assert_eq!(
    //       categories,
    //       vec!["Plugins", "Workflows", "Content Types", "Views"],
    //       "Admin UI v0 navigation MUST expose 4 categories in this \
    //        order per ratification #4 (D-4F-4 IA)"
    //   );
    //
    //   // (2) + (3) + (4) Each route renders a real engine-substrate
    //   // component — runtime-trace assertion per Family F1 gap-#6:
    //   for route in ["plugins", "workflows", "content-types", "views"] {
    //       let trace = harness.trace_capture(|h| {
    //           let view = h.navigate_to(route);
    //           // Assert the rendered DOM is non-trivial:
    //           assert!(
    //               !view.is_placeholder(),
    //               "Route {} MUST render a real component, not a \
    //                <div>TODO</div> placeholder",
    //               route,
    //           );
    //           view
    //       });
    //
    //       // Per pim-18 §3.6f: SUBSTANCE pin — assert the trace shows
    //       // `read_node_as` was invoked during render, NOT just that
    //       // the route compiled.
    //       assert!(
    //           trace.invoked_surfaces.contains("Engine::read_node_as"),
    //           "Route {} MUST invoke Engine::read_node_as for cap-scoped \
    //            reads (CLAUDE.md baked-in #18 Class B β seam); shape-only \
    //            DOM presence is INSUFFICIENT per Family F1 gap-#6",
    //           route,
    //       );
    //
    //       // Per Family F1 gap-#6 substance check: assert rendered DOM
    //       // reflects actual engine fixture data (not hand-typed mock):
    //       let dom_text = trace.final_dom_text();
    //       let engine_fixture = harness.read_route_fixture(route);
    //       assert!(
    //           dom_text.contains(&engine_fixture.distinguishing_field()),
    //           "Route {} DOM MUST reflect engine fixture content; saw \
    //            DOM '{}' but engine fixture was '{:?}'",
    //           route,
    //           dom_text,
    //           engine_fixture,
    //       );
    //   }
    //
    // OBSERVABLE consequence: the 4-category nav IA is REAL — each
    // route is a fully-wired engine consumer, not a placeholder. Defends
    // against the failure shape where shell looks done but its sub-views
    // are skeletons (pim-18 §3.6f).
    unimplemented!(
        "G24-A wires admin UI shell 4-category navigation pin. Per \
         Family F1 gap-#6: SUBSTANCE check (4 routes + each renders \
         real component fed by Engine::read_node_as), NOT shape-only \
         'DOM has 4 nav items'."
    );
}

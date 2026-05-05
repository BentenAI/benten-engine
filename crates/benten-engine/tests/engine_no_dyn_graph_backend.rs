//! R4-FP RED-PHASE pin: NO `dyn GraphBackend` references at the engine
//! boundary (closes arch-r4-1 design-lock asymmetry from
//! `.addl/phase-3/r4-r1-architect.json`).
//!
//! ## Why this pin exists
//!
//! `crates/benten-graph/tests/graph_backend_trait.rs::graph_backend_not_object_safe_per_d_phase_3_1_resolution`
//! ships the **positive direction** smoke (generic-cascade works) for
//! D-PHASE-3-1 RESOLVED. Per the trait's by-construction non-object-
//! safety (associated `type Error` + associated `type Snapshot` +
//! associated `type Transaction` + RPITIT `impl Future + Send` returns),
//! the compiler refuses to materialize `dyn GraphBackend` today.
//!
//! BUT a future 3-axis refactor cascade (drop `type Error` AND drop
//! `type Snapshot` AND drop `type Transaction`) WOULD re-enable
//! object-safety; the existing positive smoke would still pass and the
//! engine-side `engine_generic_compiles_with_redb_default` pin would
//! also still pass (the engine still works generically). Only a syntactic
//! grep across the engine sources catches the regression: if a future PR
//! adds `dyn GraphBackend` / `Box<dyn GraphBackend>` / `Arc<dyn GraphBackend>`
//! at the engine boundary, this pin fails loudly.
//!
//! Mirrors the precedent at
//! `crates/benten-engine/tests/no_unauthorized_dyn_error.rs::no_unauthorized_box_dyn_std_error_at_engine_boundary`
//! which protects the EngineError::Backend single-erasure-site contract
//! via the same syntactic-grep shape.
//!
//! ## R4-FP origin
//!
//! - `.addl/phase-3/r4-r1-architect.json` finding `arch-r4-1` (MINOR;
//!   FIX-NOW): "GraphBackend OBJECT-SAFETY DESIGN LOCK IS ONE-SIDED —
//!   positive smoke is pinned, but no syntactic guard asserts the engine
//!   sources don't reference `dyn GraphBackend`."
//! - Recommendation 1: add the syntactic-grep RED-PHASE pin mirroring
//!   `no_unauthorized_dyn_error.rs`.
//! - Pin source: r2-test-landscape §2.1 G13-B (this file is the R4-FP
//!   companion to `engine_generic.rs` + `no_unauthorized_dyn_error.rs`
//!   that closes the design-lock asymmetry arch-r4-1 surfaces).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G13-B wave-2 — closes arch-r4-1 design-lock asymmetry — engine sources MUST NOT reference dyn GraphBackend"]
fn engine_does_not_reference_dyn_graph_backend_at_engine_boundary() {
    // G13-B implementer wires this:
    //
    //   // Sweep all engine source files for dyn-erasure references to
    //   // GraphBackend. Per D-PHASE-3-1 RESOLVED + arch-r1-2 BLOCKER
    //   // closure, the engine consumes GraphBackend EXCLUSIVELY via the
    //   // generic-cascade direction (`<B: GraphBackend>` parameters);
    //   // NEVER via dyn-erasure.
    //   //
    //   // Forbidden substrings:
    //   //   - `dyn GraphBackend`
    //   //   - `Box<dyn GraphBackend>`
    //   //   - `Arc<dyn GraphBackend>`
    //   //   - `Rc<dyn GraphBackend>`
    //   //   - `&dyn GraphBackend` / `&mut dyn GraphBackend`
    //
    //   let engine_src_dir = std::path::Path::new("crates/benten-engine/src");
    //   let mut violations: Vec<(String, usize, String)> = Vec::new();
    //   let forbidden = [
    //       "dyn GraphBackend",
    //       "Box<dyn GraphBackend>",
    //       "Arc<dyn GraphBackend>",
    //       "Rc<dyn GraphBackend>",
    //       "&dyn GraphBackend",
    //   ];
    //   for entry in std::fs::read_dir(engine_src_dir).unwrap() {
    //       let path = entry.unwrap().path();
    //       if path.extension().and_then(|s| s.to_str()) != Some("rs") {
    //           continue;
    //       }
    //       let src = std::fs::read_to_string(&path).unwrap();
    //       for (i, line) in src.lines().enumerate() {
    //           let trimmed = line.trim_start();
    //           // Skip rustdoc + line comments + cite-discipline narratives
    //           // that legitimately quote `dyn GraphBackend` to explain
    //           // why it's forbidden.
    //           if trimmed.starts_with("//") || trimmed.starts_with("///")
    //              || trimmed.starts_with("//!") {
    //               continue;
    //           }
    //           for needle in &forbidden {
    //               if line.contains(needle) {
    //                   violations.push((
    //                       path.display().to_string(),
    //                       i + 1,
    //                       line.to_string(),
    //                   ));
    //               }
    //           }
    //       }
    //   }
    //   assert!(
    //       violations.is_empty(),
    //       "engine sources reference `dyn GraphBackend` outside generic-cascade — \
    //        D-PHASE-3-1 RESOLVED + arch-r1-2 BLOCKER design-lock violation: {:#?}",
    //       violations
    //   );
    //
    // OBSERVABLE consequence: a future 3-axis refactor cascade (drop
    // `type Error` + `type Snapshot` + `type Transaction` from
    // `GraphBackend`) that re-enables object-safety would silently pass
    // the existing positive smoke + the engine-side generic-cascade pin
    // (the engine still works generically). This pin fires loudly the
    // moment ANY engine module adds `dyn GraphBackend` / `Box<dyn>` /
    // `Arc<dyn>` references — closing the asymmetry between the
    // positive-direction `graph_backend_works_under_generic_cascade`
    // smoke and the negative-direction syntactic guard arch-r4-1 named.
    //
    // Mirrors `no_unauthorized_dyn_error.rs::no_unauthorized_box_dyn_std_error_at_engine_boundary`
    // which protects the analogous EngineError::Backend single-site
    // erasure contract via the same syntactic-grep shape.
    unimplemented!(
        "G13-B wires source-grep assertion that engine sources contain ZERO `dyn GraphBackend` references"
    );
}

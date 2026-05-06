//! G13-B GREEN-PHASE pin: NO `dyn GraphBackend` references at the engine
//! boundary (closes arch-r4-1 design-lock asymmetry from
//! `.addl/phase-3/r4-r1-architect.json`).
//!
//! ## Why this pin exists
//!
//! `crates/benten-graph/tests/graph_backend_trait.rs::graph_backend_not_object_safe_per_d_phase_3_1_resolution`
//! ships the **positive direction** smoke (generic-cascade works) for
//! D-PHASE-3-1 RESOLVED. Per the trait's by-construction non-object-
//! safety (associated `type Error` + associated `type Snapshot` +
//! associated `type Transaction`), the compiler refuses to materialize
//! `dyn GraphBackend` today.
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
//! which protects the analogous EngineError::Backend single-erasure-site
//! contract via the same syntactic-grep shape.
//!
//! ## R4-FP origin
//!
//! - `.addl/phase-3/r4-r1-architect.json` finding `arch-r4-1` (MINOR;
//!   FIX-NOW): "GraphBackend OBJECT-SAFETY DESIGN LOCK IS ONE-SIDED".
//! - Recommendation 1: add the syntactic-grep RED-PHASE pin mirroring
//!   `no_unauthorized_dyn_error.rs`.

#![allow(clippy::unwrap_used)]

#[test]
fn engine_does_not_reference_dyn_graph_backend_at_engine_boundary() {
    // Sweep every `.rs` file under `crates/benten-engine/src/` for
    // `dyn`-erasure references to `GraphBackend`. Per D-PHASE-3-1
    // RESOLVED + arch-r1-2 BLOCKER closure, the engine consumes
    // `GraphBackend` EXCLUSIVELY via the generic-cascade direction
    // (`<B: GraphBackend>` parameters); NEVER via dyn-erasure.
    //
    // Forbidden substrings:
    //   - `dyn GraphBackend`
    //   - `Box<dyn GraphBackend>`
    //   - `Arc<dyn GraphBackend>`
    //   - `Rc<dyn GraphBackend>`
    //   - `&dyn GraphBackend` / `&mut dyn GraphBackend`
    //
    // Comments / docstrings that NARRATIVELY quote the forbidden
    // substring (e.g. "the engine consumes `GraphBackend` exclusively
    // via the generic-cascade direction (`<B: GraphBackend>`
    // parameters), never `Arc<dyn GraphBackend>` / `Box<dyn
    // GraphBackend>` — this is the load-bearing per-backend zero-cost-
    // dispatch contract") legitimately reference the substring to
    // explain why it's forbidden — those don't violate the design
    // lock. Skip lines whose trimmed prefix starts with `//`.

    // Use CARGO_MANIFEST_DIR so the test runs from any cwd.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let engine_src_dir = std::path::Path::new(manifest_dir).join("src");
    let engine_src_dir = engine_src_dir.as_path();
    let forbidden = [
        "dyn GraphBackend",
        "Box<dyn GraphBackend",
        "Arc<dyn GraphBackend",
        "Rc<dyn GraphBackend",
        "&dyn GraphBackend",
        "&mut dyn GraphBackend",
    ];

    let mut violations: Vec<(String, usize, String)> = Vec::new();
    let dir_iter = std::fs::read_dir(engine_src_dir)
        .expect("crates/benten-engine/src must exist (run from workspace root)");
    for entry in dir_iter {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let src = std::fs::read_to_string(&path).unwrap();
        for (i, line) in src.lines().enumerate() {
            let trimmed = line.trim_start();
            // Skip comment lines (line, doc, module-doc).
            if trimmed.starts_with("//") {
                continue;
            }
            for needle in &forbidden {
                if line.contains(needle) {
                    violations.push((path.display().to_string(), i + 1, line.to_string()));
                    break;
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "engine sources reference `dyn GraphBackend` outside generic-cascade — \
         D-PHASE-3-1 RESOLVED + arch-r1-2 BLOCKER design-lock violation:\n{}",
        violations
            .iter()
            .map(|(p, l, line)| format!("  {p}:{l}: {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

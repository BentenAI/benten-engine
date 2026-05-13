//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! source NEVER calling `Engine::read_node` (the `pub(crate)` engine-
//! internal seam) — only `Engine::read_node_as` (Class B β public
//! cap-scoped seam shipped at PR #184).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 7; closes cag-r1-9 + CLAUDE.md baked-in #18 (Class B β seam
//! discipline).
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #18 (PR #184 LIVE): `Engine::read_node` is
//! `pub(crate)`. Engine internals (IVM, sync, view materialization,
//! audit) call it directly with no permission check, no overhead on
//! hot paths. **Plugin authors NEVER call either function** — they
//! author graph nodes; the evaluator is the only caller of `_as`.
//!
//! This pin is the **grep-assert** sibling of the runtime-trace pin in
//! `admin_ui_v0_shell_routes_through_engine_read_node_as_for_cap_scoped_reads.rs`.
//! Together they form the §3.6f SHAPE-not-SUBSTANCE pair per R2 §5
//! table row 3 (cag-r1-9 named both grep + runtime trace).

#![allow(clippy::unwrap_used)]

mod common;

#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §2.6 row 7 + cag-r1-9. SHAPE half (grep-assert); pairs with runtime-trace pin in admin_ui_v0_shell_routes_through_engine_read_node_as_for_cap_scoped_reads.rs."]
fn admin_ui_v0_source_never_references_engine_read_node_directly() {
    // G24-A wave wires this. Substantive shape:
    //
    //   // Admin UI v0 plugin source roots (Rust handlers + TS shell):
    //   let roots = [
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..")
    //           .join("benten-platform-foundation")
    //           .join("src")
    //           .join("admin_ui_v0"),
    //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //           .join("..")
    //           .join("..")
    //           .join("packages")
    //           .join("admin-ui-v0")
    //           .join("src"),
    //   ];
    //
    //   let mut found_read_node_as = 0_usize;
    //   for root in &roots {
    //       for entry in walkdir::WalkDir::new(root) {
    //           let entry = entry.unwrap();
    //           if !entry.file_type().is_file() { continue; }
    //           let src = std::fs::read_to_string(entry.path()).unwrap();
    //
    //           // The discrimination is whitespace-precise: `read_node(`
    //           // without `_as` suffix is the violation. Match on
    //           // word-boundaries to avoid false positives on
    //           // `read_node_as`:
    //           let re_bad = regex::Regex::new(
    //               r"\bread_node\s*\("
    //           ).unwrap();
    //           let re_good = regex::Regex::new(
    //               r"\bread_node_as\s*\("
    //           ).unwrap();
    //
    //           // Find any `read_node(` invocation NOT inside a
    //           // `read_node_as(` lexeme:
    //           let bad_count = re_bad.find_iter(&src).count();
    //           let good_count = re_good.find_iter(&src).count();
    //           let bare_count = bad_count - good_count;
    //
    //           assert_eq!(
    //               bare_count, 0,
    //               "Admin UI source MUST NEVER call \
    //                Engine::read_node directly (pub(crate) seam, \
    //                no cap check); found in {}",
    //               entry.path().display(),
    //           );
    //
    //           found_read_node_as += good_count;
    //       }
    //   }
    //
    //   // Positive side: admin UI calls read_node_as at least once
    //   // (otherwise it isn't reading anything):
    //   assert!(
    //       found_read_node_as > 0,
    //       "Admin UI source MUST call Engine::read_node_as at least \
    //        once; ZERO references — admin UI has no reads or is \
    //        bypassing engine surface entirely"
    //   );
    //
    // OBSERVABLE consequence: lexical-level defense that admin UI
    // doesn't reach for the bypass seam. SHAPE half of the §3.6f pair.
    unimplemented!(
        "G24-A wires admin UI never-read_node grep-assert (SHAPE half of \
         pim-18 §3.6f pair; SUBSTANCE half lives in \
         admin_ui_v0_shell_routes_through_engine_read_node_as_for_cap_scoped_reads.rs)"
    );
}

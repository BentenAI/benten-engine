//! R3-B RED-PHASE pin: anchor-store consolidation residual closed
//! (G14-C wave-4b; cov-f3 + phase-2-backlog §6.3).
//!
//! Pin source: r2-test-landscape §2.2 G14-C row
//! `anchor_store_consolidation_cov_f3_no_residual`; cov-f3.
//!
//! ## Architectural intent
//!
//! Phase-2 left a tracked residual (cov-f3 / `docs/future/phase-2-backlog.md`
//! §6.3) where multiple ad-hoc anchor-storage helpers existed across
//! benten-engine + benten-graph. G14-C consolidates these to a single
//! anchor-store API. This test pins the consolidation: only one path
//! exists at G14-C close.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-C
//! implementer un-ignores. The un-ignored test must drive the
//! consolidated API + assert the consolidation landed (no residual
//! ad-hoc helpers).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-C — cov-f3 + phase-2-backlog §6.3 — anchor-store consolidated, no residual"]
fn anchor_store_consolidation_cov_f3_no_residual() {
    // cov-f3 pin. G14-C implementer wires this via SOURCE-CITE
    // assertions over the post-consolidation tree:
    //
    //   // 1. The consolidated API exists at exactly one path:
    //   let anchor_store_module_count = std::fs::read_dir("crates/benten-engine/src/")
    //       .unwrap()
    //       .filter_map(|e| e.ok())
    //       .filter(|e| e.file_name().to_string_lossy().contains("anchor_store"))
    //       .count();
    //   assert_eq!(anchor_store_module_count, 1,
    //       "cov-f3: anchor-store implementation MUST live at exactly one site");
    //
    //   // 2. No residual ad-hoc helpers in benten-graph / benten-eval:
    //   for crate_path in &["crates/benten-graph/src/", "crates/benten-eval/src/"] {
    //       let walk = walkdir::WalkDir::new(crate_path).into_iter()
    //           .filter_map(|e| e.ok())
    //           .filter(|e| e.path().extension().map_or(false, |x| x == "rs"));
    //       for entry in walk {
    //           let src = std::fs::read_to_string(entry.path()).unwrap();
    //           // No file in benten-graph or benten-eval should
    //           // re-implement anchor-store primitives:
    //           assert!(!src.contains("fn put_anchor("),
    //               "cov-f3: residual put_anchor helper at {:?}", entry.path());
    //           assert!(!src.contains("fn fetch_anchor("),
    //               "cov-f3: residual fetch_anchor helper at {:?}", entry.path());
    //       }
    //   }
    //
    //   // 3. The consolidated API is consumed by Engine + handler-
    //   //    version chain (G14-C target):
    //   let engine = benten_engine::Engine::open(store_dir.path()).unwrap();
    //   let anchor_cid = ...;
    //   let _node = engine.anchor_store().fetch(&anchor_cid).unwrap();
    //
    // OBSERVABLE consequence: cov-f3 closes when (a) exactly one
    // anchor-store module exists, (b) no ad-hoc helpers remain in
    // sibling crates, (c) Engine consumes the consolidated API. The
    // test fails loudly if any of these regress.
    unimplemented!(
        "G14-C wires source-grep + Engine consumption assertions for anchor-store consolidation"
    );
}

//! Phase 2b R3-B — engine_sandbox public-surface integration tests (G7-C).
//!
//! Pin sources: plan §3 G7-C, dx-r1-2b SANDBOX.
//!
//! G7-C surface posture (dx-optimizer corrected):
//!   - DSL composition surface ONLY: `subgraph(...).sandbox({ module,
//!     manifest? | caps? })`.
//!   - NO top-level `engine.sandbox(...)` user-facing API — would
//!     bypass evaluator + Inv-4 + AttributionFrame plumbing.
//!   - Top-level engine surface for sandbox-related work is exclusively
//!     `engine.installModule(manifest, manifestCid)` /
//!     `engine.uninstallModule(cid)` (G10-B owned).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "pending G7-A executor wiring; tracks G7-A's phase-2b/g7/a-sandbox-core PR (PR #30) — G7-C delivers the surface; G7-A delivers the executor body that makes E2E run"]
fn engine_sandbox_end_to_end_via_dsl_composition_only() {
    // Plan §3 G7-C — register a SubgraphSpec built via the DSL
    // `subgraph('handler').sandbox({ module: cid, manifest: 'compute-basic' })`.
    // engine.call('handler', input) routes through the evaluator,
    // which dispatches the SANDBOX primitive.
    //
    // No top-level `engine.sandbox(...)` API is invoked — the
    // composition is what's tested.
    todo!("R5 G7-C — DSL builder + register + engine.call + assertion");
}

#[test]
fn sandbox_no_top_level_engine_sandbox_call_site_exists() {
    // dx-r1-2b SANDBOX surface — anti-regression: the public Rust
    // engine surface (`benten_engine::Engine`) MUST NOT carry a
    // `sandbox` method. Only `install_module` / `uninstall_module`
    // (G10-B owned) and the internal `execute_sandbox_*` plumbing
    // (private).
    //
    // Source-grep absence pin via manual recursive walk over
    // `crates/benten-engine/src/` (avoids pulling `walkdir` as a
    // dev-dep just for this single test). Asserts no `pub fn sandbox(`
    // declaration exists. Sufficient for the dx-r1-2b corrected-
    // surface contract per HARD RULE (compile_fail / trybuild
    // harness reserved for Phase 3 if a deeper type-system
    // regression vector surfaces).
    fn walk(dir: &std::path::Path, hits: &mut Vec<String>) {
        let entries =
            std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, hits);
                continue;
            }
            if path.extension().is_some_and(|ext| ext == "rs") {
                let body = std::fs::read_to_string(&path)
                    .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
                for (lineno, line) in body.lines().enumerate() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("pub fn sandbox(")
                        || trimmed.starts_with("pub fn sandbox<")
                        || trimmed.starts_with("pub async fn sandbox(")
                    {
                        hits.push(format!("{}:{}: {}", path.display(), lineno + 1, line));
                    }
                }
            }
        }
    }
    let src_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offending = Vec::new();
    walk(&src_root, &mut offending);
    assert!(
        offending.is_empty(),
        "dx-r1-2b absence pin tripped: top-level `engine.sandbox(...)` surface MUST NOT exist on \
         `Engine`; found {} declaration(s):\n{}",
        offending.len(),
        offending.join("\n")
    );
}

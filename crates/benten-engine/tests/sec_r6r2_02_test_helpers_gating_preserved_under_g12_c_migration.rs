//! G12-C-cont (Phase 2b R6 A1 closure) sec-pre-r1-13 carry: assert the
//! Phase-2a `sec-r6r2-02` test-helpers cfg-gating
//! (`#[cfg(any(test, feature = "test-helpers"))]` etc.) is NOT silently
//! dropped during the `Subgraph` type relocation from `benten-eval` to
//! `benten-core`.
//!
//! Per `r1-security-auditor.json` sec-pre-r1-13: Phase-2a security closures
//! "are MUST-NOT-REOPEN in Phase 2b. Specifically: ... G12-C migration MUST
//! preserve the Phase-2a `#[cfg(any(test, feature = "test-helpers"))]` gates
//! on `testing_*` surfaces (no surface should silently drop a gate during
//! the Subgraph type relocation)."
//!
//! Test approach: scan source trees for every `pub fn testing_*` and assert
//! each is preceded (within 8 lines) by a recognised cfg-gate attribute.

#![allow(clippy::unwrap_used)]

use std::fs;
use std::path::{Path, PathBuf};

const ENGINE_RECOGNISED_GATES: &[&str] = &[
    r#"#[cfg(any(test, feature = "test-helpers"))]"#,
    r#"#[cfg(any(test, feature = "envelope-cache-test-grade"))]"#,
    r#"#[cfg(any(test, feature = "iteration-budget-test-grade"))]"#,
    r#"#[cfg(any(test, feature = "test-helpers", feature = "envelope-cache-test-grade"))]"#,
    r#"#[cfg(any(test, feature = "test-helpers", feature = "iteration-budget-test-grade"))]"#,
    "#[cfg(test)]",
];

const EVAL_RECOGNISED_GATES: &[&str] = &[
    r#"#[cfg(any(test, feature = "testing"))]"#,
    r#"#[cfg(any(test, debug_assertions, feature = "testing"))]"#,
    r"#[cfg(any(test, debug_assertions))]",
    "#[cfg(test)]",
];

fn walk_src(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_src(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR points to the test crate's root (benten-engine).
    // Workspace root is two levels up (crates/benten-engine -> .).
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    Path::new(&manifest_dir)
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn find_testing_helpers(crate_src: &Path) -> Vec<(PathBuf, usize, String)> {
    let mut files = Vec::new();
    walk_src(crate_src, &mut files);

    let mut hits = Vec::new();
    for path in files {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        for (lineno, line) in content.lines().enumerate() {
            let trimmed = line.trim_start();
            // Match `pub fn testing_*` or `pub(crate) fn testing_*` or
            // method forms `pub fn testing_*(`.
            if trimmed.starts_with("pub fn testing_")
                || trimmed.starts_with("pub(crate) fn testing_")
            {
                hits.push((path.clone(), lineno + 1, trimmed.to_string()));
            }
        }
    }
    hits
}

fn assert_each_helper_is_cfg_gated(crate_src: &Path, crate_label: &str, recognised_gates: &[&str]) {
    let hits = find_testing_helpers(crate_src);
    let mut violations = Vec::new();

    for (path, lineno, helper) in hits {
        let content = fs::read_to_string(&path).expect("read");
        let lines: Vec<&str> = content.lines().collect();
        // Look at the 8 lines preceding the `pub fn testing_*` line for a
        // recognised gate attribute, OR check whether the surrounding mod /
        // file is gated.
        let start = lineno.saturating_sub(9);
        let preceding = &lines[start..lineno.saturating_sub(1)];
        let has_inline_gate = preceding
            .iter()
            .any(|l| recognised_gates.contains(&l.trim()));
        // Module-level fallback: if the file or surrounding mod block is
        // cfg-gated, we accept that as the gate.
        let file_starts_with_module_gate = lines
            .iter()
            .take(20)
            .any(|l| recognised_gates.contains(&l.trim()));
        if !(has_inline_gate || file_starts_with_module_gate) {
            violations.push(format!(
                "{}:{}: `{}` lacks a recognised cfg-gate within 8 preceding lines",
                path.display(),
                lineno,
                helper
            ));
        }
    }

    assert!(
        violations.is_empty(),
        "{crate_label}: testing_* surfaces without recognised cfg-gate post-G12-C-cont:\n{}",
        violations.join("\n")
    );
}

#[test]
fn every_pub_testing_helper_in_benten_engine_carries_cfg_test_or_test_helpers_gate() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let engine_src = Path::new(&manifest_dir).join("src");
    assert_each_helper_is_cfg_gated(&engine_src, "benten-engine", ENGINE_RECOGNISED_GATES);
}

#[test]
fn every_pub_testing_helper_in_benten_eval_carries_cfg_test_or_testing_feature_gate() {
    let eval_src = workspace_root().join("crates/benten-eval/src");
    assert_each_helper_is_cfg_gated(&eval_src, "benten-eval", EVAL_RECOGNISED_GATES);
}

#[test]
fn count_of_testing_helpers_post_migration_does_not_drop_to_zero() {
    // sec-pre-r1-13 reinforcement, scoped to the pattern this test scans for:
    // `pub fn testing_*` declarations. The R3 doc's pre-migration count of
    // 85 covered a broader inventory (helpers, constants, types — not just
    // `pub fn testing_*`); this test is the narrow `pub fn testing_*` slice.
    // The G12-C-cont relocation moves Subgraph/SubgraphBuilder/companions
    // into `benten-core` but the testing_* helpers stay where they were —
    // so the narrow `pub fn testing_*` count MUST NOT drop to zero (which
    // would indicate an accidental wholesale removal under the migration).
    let eval_src = workspace_root().join("crates/benten-eval/src");
    let engine_src = workspace_root().join("crates/benten-engine/src");
    let count = find_testing_helpers(&eval_src).len() + find_testing_helpers(&engine_src).len();
    assert!(
        count > 0,
        "expected at least one `pub fn testing_*` helper post-G12-C-cont; \
         found 0 (regression — the relocation should NOT remove testing helpers)"
    );
}

#[test]
fn g12c_parse_counter_cfg_gate_preserved_post_subgraph_migration() {
    // sec-pre-r1-13 explicit named carry from R2 §1.9: pin that the Phase-2a
    // sec-r6r3-02 parse-counter cfg-gate (`testing_parse_counter` /
    // `testing_reset_parse_counter`) survives the Subgraph relocation.
    let eval_src = workspace_root().join("crates/benten-eval/src");
    let mut files = Vec::new();
    walk_src(&eval_src, &mut files);
    let mut found_parse_counter = false;
    for path in files {
        let content = fs::read_to_string(&path).expect("read");
        let lines: Vec<&str> = content.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.contains("fn testing_parse_counter")
                || trimmed.contains("fn testing_reset_parse_counter")
            {
                found_parse_counter = true;
                let start = i.saturating_sub(8);
                let preceding = &lines[start..i];
                let has_gate = preceding.iter().any(|l| {
                    let t = l.trim();
                    EVAL_RECOGNISED_GATES.contains(&t)
                });
                let file_module_gate = lines
                    .iter()
                    .take(20)
                    .any(|l| EVAL_RECOGNISED_GATES.contains(&l.trim()));
                assert!(
                    has_gate || file_module_gate,
                    "{}:{}: `{}` lost its cfg-gate post-G12-C-cont",
                    path.display(),
                    i + 1,
                    trimmed
                );
            }
        }
    }
    // It's fine if the parse-counter helpers don't exist (Phase-2b may not
    // have shipped them yet) — but if they DO exist, they MUST be gated.
    let _ = found_parse_counter; // intentionally tolerated
}

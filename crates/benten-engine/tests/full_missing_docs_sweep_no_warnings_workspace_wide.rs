//! Phase 2b G11-2b — workspace-wide `missing_docs` sweep gate.
//!
//! Asserts the brief-owned crates (`benten-eval`, `benten-engine`,
//! `benten-ivm`) carry **no** crate-root `#![allow(missing_docs)]`
//! and **no** per-item `#[allow(missing_docs)]` annotations.
//!
//! Phase 2b G11-2b owns the cleanup; this test pins the post-cleanup
//! state so a future regression cannot silently re-introduce the
//! suppression.
//!
//! The gate is doc-source structural — it greps the `src/` tree
//! rather than re-running clippy from inside the test. Clippy
//! enforcement happens in CI via `clippy --all-targets -- -D
//! warnings -D missing_docs`; this test is the structural belt that
//! catches "someone added `#[allow]` to silence a doc warning"
//! before CI even runs.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn full_missing_docs_sweep_no_warnings_workspace_wide() {
    let root = workspace_root();
    let crate_src_dirs = [
        root.join("crates/benten-eval/src"),
        root.join("crates/benten-engine/src"),
        root.join("crates/benten-ivm/src"),
    ];

    let mut violations = Vec::new();
    for dir in &crate_src_dirs {
        let mut files = Vec::new();
        collect_rs_files(dir, &mut files);
        for file in files {
            let body = std::fs::read_to_string(&file).unwrap();
            for (lineno, line) in body.lines().enumerate() {
                let trimmed = line.trim_start();
                // Crate-root: `#![allow(missing_docs)]` or
                //             `#![allow(missing_docs, ...)]`.
                // Per-item:   `#[allow(missing_docs)]` or
                //             `#[allow(missing_docs, ...)]`.
                let has_crate_root =
                    trimmed.starts_with("#![allow(") && trimmed.contains("missing_docs");
                let has_per_item =
                    trimmed.starts_with("#[allow(") && trimmed.contains("missing_docs");
                if has_crate_root || has_per_item {
                    violations.push(format!(
                        "{}:{} -> {}",
                        file.strip_prefix(&root).unwrap_or(&file).display(),
                        lineno + 1,
                        trimmed
                    ));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Phase 2b G11-2b sweep regression: \
         {} `allow(missing_docs)` site(s) reintroduced in \
         brief-owned crates (benten-eval / benten-engine / \
         benten-ivm). The G11-2b brief explicitly drops these; \
         re-adding them silently degrades the public-surface \
         doc gate. Sites:\n{}",
        violations.len(),
        violations.join("\n")
    );
}

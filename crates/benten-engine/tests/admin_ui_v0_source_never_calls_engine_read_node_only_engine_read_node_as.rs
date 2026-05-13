//! Phase-4-Foundation G24-A — admin UI v0 source NEVER calls
//! `Engine::read_node` (the `pub(crate)` engine-internal seam) — only
//! `Engine::read_node_as` (Class B β public cap-scoped seam shipped at
//! PR #184).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 7; closes cag-r1-9 + CLAUDE.md baked-in #18 (Class B β seam
//! discipline).
//!
//! Per CLAUDE.md baked-in #18 (PR #184 LIVE): `Engine::read_node` is
//! `pub(crate)`. Engine internals (IVM, sync, view materialization,
//! audit) call it directly. **Plugin authors NEVER call either
//! function** — they author graph nodes; the evaluator is the only
//! caller of `_as`. The admin UI v0 plugin handler module
//! (`crates/benten-platform-foundation/src/admin_ui_v0/`) MUST route
//! through `read_node_as` via the
//! [`benten_platform_foundation::MaterializerEngine`] trait's
//! `read_node_as` method — never reach for `read_node` directly.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn admin_ui_v0_source_roots() -> Vec<PathBuf> {
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace = here.parent().unwrap().parent().unwrap();
    vec![
        // Rust handler module — the plugin handler code itself.
        workspace
            .join("crates")
            .join("benten-platform-foundation")
            .join("src")
            .join("admin_ui_v0"),
        // TS browser-bundle side.
        workspace.join("packages").join("admin-ui-v0").join("src"),
    ]
}

fn collect_source_files(root: &PathBuf) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if !root.exists() {
        return out;
    }
    fn walk(dir: &PathBuf, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if matches!(
                path.extension().and_then(|s| s.to_str()),
                Some("rs" | "ts" | "tsx" | "js" | "mjs")
            ) {
                out.push(path);
            }
        }
    }
    walk(root, &mut out);
    out
}

fn strip_comments(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let bytes = src.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            i += 2;
            while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    out
}

/// Match `engine.read_node(...)` / `.read_node(...)` invocations,
/// **excluding** `.read_node_as(...)` calls. Returns the offending
/// line text on hit.
fn find_read_node_violations(scrubbed: &str) -> Vec<String> {
    let mut hits = Vec::new();
    for line in scrubbed.lines() {
        let mut search_from = 0;
        while let Some(idx) = line[search_from..].find("read_node(") {
            let abs = search_from + idx;
            let prefix = &line[..abs];
            // Tighten to method-dispatch syntax — `.read_node(` or
            // `::read_node(`. Bare identifier `read_node(` (a local
            // function) is filtered to keep the assertion targeted.
            let dispatch_char = prefix.chars().last();
            let is_method_call = dispatch_char.is_some_and(|c| c == '.' || c == ':');
            if is_method_call {
                hits.push(line.to_string());
            }
            search_from = abs + 1;
        }
    }
    hits
}

#[test]
fn admin_ui_v0_source_never_references_engine_read_node_directly() {
    let roots = admin_ui_v0_source_roots();
    let mut scanned_files = 0_usize;
    let mut found_read_node_as = 0_usize;
    for root in &roots {
        for path in collect_source_files(root) {
            scanned_files += 1;
            let src = std::fs::read_to_string(&path).unwrap();
            let scrubbed = strip_comments(&src);
            let violations = find_read_node_violations(&scrubbed);
            assert!(
                violations.is_empty(),
                "Admin UI source MUST NOT call `Engine::read_node` directly per CLAUDE.md \
                 baked-in #18 Class B β; the public cap-scoped seam is `read_node_as`. \
                 Found {} violation(s) in {}:\n{}",
                violations.len(),
                path.display(),
                violations.join("\n")
            );
            for line in scrubbed.lines() {
                if line.contains("read_node_as") {
                    found_read_node_as += 1;
                }
            }
        }
    }
    assert!(
        scanned_files > 0,
        "Admin UI v0 source roots MUST exist (Rust + TS bundle present)"
    );
    assert!(
        found_read_node_as > 0,
        "Admin UI v0 source MUST reference `read_node_as` at least once (otherwise the \
         CLASS B β seam isn't wired — the absence of the violating call is trivial)"
    );
}

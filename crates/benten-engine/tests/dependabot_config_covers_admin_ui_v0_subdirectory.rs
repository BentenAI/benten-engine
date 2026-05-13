//! R4-FP-3 RED-PHASE pin: `.github/dependabot.yml` covers
//! `packages/admin-ui-v0/` subdirectory.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.13 row 4.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - meth-r1-9: admin UI v0 ships with its own npm package at
//!   `packages/admin-ui-v0/`; dependabot must cover the subdirectory to
//!   pick up vitest 4.x / Playwright / browser-mode dep updates.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    PathBuf::from(&manifest_dir)
        .parent()
        .and_then(std::path::Path::parent)
        .map(std::path::Path::to_path_buf)
        .expect("workspace root")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G26-B wave-10 un-ignores. \
    Pin source: r2-test-landscape.md §2.13 row 4 + meth-r1-9. dependabot.yml MUST add \
    packages/admin-ui-v0/ directory entry alongside existing packages/engine + bindings/napi \
    + tools/create-benten-app coverage."]
fn dependabot_config_covers_admin_ui_v0_subdirectory() {
    let root = workspace_root();
    let path = root.join(".github/dependabot.yml");

    let body = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("dependabot.yml MUST exist at {} ({})", path.display(), e);
    });

    // Look for an `updates:` entry with `directory: "/packages/admin-ui-v0"`
    // (or equivalent quoting / npm ecosystem).
    let covers_admin_ui =
        body.contains("/packages/admin-ui-v0") || body.contains("packages/admin-ui-v0");
    assert!(
        covers_admin_ui,
        ".github/dependabot.yml MUST include a `directory: \"/packages/admin-ui-v0\"` \
         entry so weekly minor+patch updates for vitest 4.x / Playwright / browser-mode \
         deps surface (meth-r1-9). At HEAD the file lists /packages/engine and \
         /tools/create-benten-app but not the admin-ui-v0 subdirectory. G26-B wave-10 \
         wires the missing entry."
    );

    // Sanity / SUBSTANCE — verify the file isn't matching by accident
    // (e.g., a comment). Look for the typical structure `directory:` line.
    let has_directory_line = body
        .lines()
        .any(|line| line.trim_start().starts_with("directory:") && line.contains("admin-ui-v0"));
    assert!(
        has_directory_line,
        "dependabot.yml MUST have a `directory:` line specifically naming admin-ui-v0 — \
         comment-only mention does not register the subdirectory with dependabot. The pin \
         catches the failure shape where someone documents the intent in a comment but \
         forgets the actual `updates:` block entry."
    );
}

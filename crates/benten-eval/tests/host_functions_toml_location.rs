//! Phase 2b R3-B — `host-functions.toml` workspace-root location pin
//! (G7-A).
//!
//! Pin source: wsa-16 — TOML lives at workspace root so the existing
//! drift-detect.yml workflow scans it (mirrors the pattern set by other
//! workspace-root TOML sources of truth).
//!
//! **cr-g7a-mr-1 fix-pass:** test FLIPPED from `#[ignore]` `todo!()` to
//! live assertion against the G7-A-landed surface.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn host_functions_toml_at_workspace_root() {
    // wsa-16 — assert `host-functions.toml` exists at workspace root
    // (NOT under `crates/benten-eval/` or any sub-crate).
    let root = workspace_root();
    let path = root.join("host-functions.toml");
    assert!(
        path.exists(),
        "host-functions.toml MUST live at workspace root (wsa-16); \
         expected at {}",
        path.display()
    );
    // Defensively check that it does NOT live in the sub-crate.
    let bad = root.join("crates/benten-eval/host-functions.toml");
    assert!(
        !bad.exists(),
        "host-functions.toml must NOT live at the per-crate path {}; \
         workspace-root single-source-of-truth per wsa-16",
        bad.display()
    );
}

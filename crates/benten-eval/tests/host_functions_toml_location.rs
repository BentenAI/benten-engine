//! Phase 2b R3-B — `host-functions.toml` workspace-root location pin
//! (G7-A).
//!
//! Pin source: wsa-16 — TOML lives at workspace root so the existing
//! drift-detect.yml workflow scans it (mirrors the pattern set by other
//! workspace-root TOML sources of truth).

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables)]

#[test]
#[ignore = "Phase 2b G7-A pending — wsa-16 workspace-root location"]
fn host_functions_toml_at_workspace_root() {
    // wsa-16 — assert `host-functions.toml` exists at workspace root
    // (NOT under `crates/benten-eval/` or any sub-crate).
    //
    // Discovery: walk up from CARGO_MANIFEST_DIR until finding a dir
    // containing `Cargo.toml` with `[workspace]` table; assert
    // `host-functions.toml` exists in that dir.
    //
    // R5 G7-A also needs to extend `.github/workflows/drift-detect.yml`
    // to scan workspace-root TOMLs (separate test in R3-E CI workflow
    // suite).
    todo!("R5 G7-A — workspace-root walk + file existence assertion");
}

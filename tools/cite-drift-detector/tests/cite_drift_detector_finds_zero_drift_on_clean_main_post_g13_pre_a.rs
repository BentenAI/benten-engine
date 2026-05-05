//! Phase-3 cite-cleanup-21-drifts end-to-end pin (per §3.6b): drives
//! the production entry point against the LIVE workspace tree (the
//! cargo-workspace root, located by walking up from `CARGO_MANIFEST_DIR`)
//! and asserts ZERO `cite-drift` findings (the `--cite-only` pass).
//!
//! Couples the cite-cleanup PR's cleanliness claim to a CI-runnable
//! pin that would FAIL if any of the 21 drifts were silently
//! reintroduced — or if a future PR introduced a 22nd. Sister fixture
//! pin lives at `cite_drift_detector_passes_on_clean_tree.rs`
//! (synthetic clean tree); this pin validates the REAL tree.
//!
//! Numeric-claim drift pass is intentionally NOT asserted here — that
//! pass already has its own dedicated fixture pin
//! (`numeric_claim_drift_lint_finds_known_drift_fixture.rs`) plus the
//! full `--all` invocation runs in CI workflow. The cite-only assertion
//! is the load-bearing one for cite-drift cleanliness.

use std::path::PathBuf;

use cite_drift_detector::run_cite_drift_check;

/// Locate the workspace root by walking up from `CARGO_MANIFEST_DIR`
/// until we find a `Cargo.toml` containing a `[workspace]` table OR
/// the `crates/` + `packages/` siblings that mark the benten-engine
/// workspace root. Returns the workspace root path.
fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cur = manifest_dir.as_path();
    loop {
        if cur.join("crates").is_dir() && cur.join("packages").is_dir() {
            return cur.to_path_buf();
        }
        match cur.parent() {
            Some(p) => cur = p,
            None => panic!(
                "could not locate workspace root from CARGO_MANIFEST_DIR={}",
                manifest_dir.display()
            ),
        }
    }
}

#[test]
fn cite_drift_detector_finds_zero_drift_on_clean_main_post_g13_pre_a() {
    let root = workspace_root();
    let findings = run_cite_drift_check(&root);
    assert!(
        findings.is_empty(),
        "expected zero cite-drift findings on the live workspace tree; got {} finding(s):\n{:#?}",
        findings.len(),
        findings
    );
}

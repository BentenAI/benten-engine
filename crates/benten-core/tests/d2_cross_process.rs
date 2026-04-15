//! D2 — Cross-process determinism test.
//!
//! Runs in a separate test binary (integration tests live in `tests/` and are
//! compiled to their own binary, so every `cargo test` / `cargo nextest run`
//! invocation exercises the "new process" property).
//!
//! Strategy: the first time this test runs, it writes the canonical CID to
//! `tests/fixtures/canonical_cid.txt`. On every subsequent run (and on CI,
//! where the fixture is committed to git), it re-computes the CID and asserts
//! byte-for-byte equality against the fixture. Running the test twice in CI
//! therefore actually exercises the property across two separate processes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stderr,
    reason = "tests may use unwrap/expect and the bootstrap branch needs to \
              announce the fixture write on the first run"
)]

use std::env;
use std::fs;
use std::path::PathBuf;

use benten_core::testing::canonical_test_node;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("canonical_cid.txt")
}

#[test]
fn d2_cross_process_determinism() {
    let cid = canonical_test_node().cid().expect("hash succeeds");
    let current = cid.to_base32();

    let path = fixture_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create fixtures dir");
    }

    if !path.exists() {
        // First-run bootstrap: record the canonical CID. Subsequent runs (and
        // all CI runs after this commit) assert against the fixture.
        fs::write(&path, &current).expect("write fixture");
        eprintln!("D2 bootstrap: wrote canonical CID fixture to {path:?}");
        return;
    }

    let recorded = fs::read_to_string(&path).expect("read fixture");
    let recorded = recorded.trim();

    assert_eq!(
        recorded, current,
        "cross-process determinism violated:\n  recorded: {recorded}\n  current:  {current}\n\
         If this failure is intentional (e.g., the hash algorithm or canonical \
         serialization changed), delete the fixture and re-run once to bootstrap."
    );
}

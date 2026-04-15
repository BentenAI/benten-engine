//! D2 — Cross-process determinism test.
//!
//! Runs in a separate test binary (integration tests live in `tests/` and are
//! compiled to their own binary, so every `cargo test` / `cargo nextest run`
//! invocation exercises the "new process" property).
//!
//! Strategy: the fixture file `tests/fixtures/canonical_cid.txt` is committed
//! to the repo. Every test run re-computes the canonical CID and asserts byte-
//! for-byte equality against the committed fixture. If the fixture is missing,
//! the test fails loudly by default — a lost fixture must be a deliberate
//! re-pin, not a silent bootstrap. To intentionally bootstrap a new fixture
//! (when the hash algorithm, canonical serialization, or canonical test Node
//! is deliberately changed), set the environment variable
//! `BENTEN_D2_BOOTSTRAP=1`.

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

    let bootstrap = env::var_os("BENTEN_D2_BOOTSTRAP").is_some();

    if !path.exists() {
        assert!(
            bootstrap,
            "D2 fixture missing at {path:?} and BENTEN_D2_BOOTSTRAP is not set.\n\
             A missing fixture must be a deliberate re-pin, not a silent \
             bootstrap. Set `BENTEN_D2_BOOTSTRAP=1` in the environment to \
             create the fixture from the current canonical CID."
        );
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
         serialization changed), delete the fixture and re-run with \
         BENTEN_D2_BOOTSTRAP=1 to re-pin."
    );
}

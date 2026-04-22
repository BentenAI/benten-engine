//! Phase 2a R4 qa-r4-7 / cov-4 + arch-7: `docs/HOST-FUNCTIONS.md` stub
//! exists with the cap-string format spec (`<prefix>:<domain>:<action>`).
//!
//! The doc is a G11-A deliverable. TDD red-phase: this test fails until
//! the doc lands.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

fn doc_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("repo root")
        .join("docs/HOST-FUNCTIONS.md")
}

#[test]
fn host_functions_doc_stub_present() {
    let contents = fs::read_to_string(doc_path()).expect(
        "docs/HOST-FUNCTIONS.md must exist (G11-A deliverable; see plan §3 G11-A + arch-7)",
    );

    // The doc must at least anchor the cap-string format shape the parser
    // enforces in `benten_errors::parse_cap_string`.
    assert!(
        contents.contains("prefix") && contents.contains("domain") && contents.contains("action"),
        "docs/HOST-FUNCTIONS.md must name the `<prefix>:<domain>:<action>` \
         cap-string shape (arch-7); found only:\n{contents}"
    );
}

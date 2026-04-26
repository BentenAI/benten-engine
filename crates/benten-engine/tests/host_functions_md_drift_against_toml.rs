//! Phase 2b R4-FP B-4 — `docs/HOST-FUNCTIONS.md` → `host-functions.toml`
//! reverse-direction drift detector.
//!
//! TDD red-phase. Pin source: R2 §6 (`host_functions_doc_drift_against_toml`)
//! and qa-r4-08 dispatch note (R3-E's `host_functions_doc_drift_against_toml.rs`
//! covers TOML→MD; this file covers the OPPOSITE direction MD→TOML
//! to close the bidirectional drift gap that asymmetric drift detectors
//! historically miss).
//!
//! The companion file `host_functions_doc_drift_against_toml.rs`
//! (R3-E, landed) asserts every TOML entry has a doc section
//! ("missing in doc"). This file asserts every doc section has a TOML
//! entry ("extra in doc" — operators reading docs see only fictional
//! host-fns the codegen has dropped).
//!
//! Format pins (mirroring the R3-E companion):
//!   * `host-functions.toml`: each host-fn is a top-level table
//!     `[host_fn.<name>]`.
//!   * `docs/HOST-FUNCTIONS.md`: each host-fn gets a header
//!     `## host_fn.<name>` (or `### host_fn.<name>`).
//!
//! Owned by R3-E (CI workflow tests row); test landed by R4-FP B-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(clippy::doc_lazy_continuation)] // R3-FP test scaffolding doc-comment formatting; R5 G11-2b may rewrite

use std::collections::BTreeSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

fn parse_toml_host_fn_names(toml_src: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in toml_src.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("[host_fn.")
            && let Some(name) = rest.strip_suffix(']')
        {
            names.insert(name.to_string());
        }
    }
    names
}

fn parse_doc_host_fn_names(md_src: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in md_src.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        if let Some(name) = trimmed.strip_prefix("host_fn.") {
            let clean = name.split_whitespace().next().unwrap_or(name);
            names.insert(clean.to_string());
        }
    }
    names
}

/// `host_functions_md_to_toml_no_extra_in_doc` — closes the
/// bidirectional drift gap. The companion R3-E file asserts MD ⊇ TOML;
/// this file asserts MD ⊆ TOML.
#[test]
#[ignore = "Phase 2b G7-A + G11-2b-A pending — host-functions.toml + docs/HOST-FUNCTIONS.md unimplemented"]
fn host_functions_md_to_toml_no_extra_in_doc() {
    let root = workspace_root();
    let toml_path = root.join("crates/benten-eval/host-functions.toml");
    let doc_path = root.join("docs/HOST-FUNCTIONS.md");

    let toml_src = std::fs::read_to_string(&toml_path).unwrap_or_else(|e| {
        panic!(
            "host-functions.toml not found at {} ({}). G7-A owns this \
             file as the SANDBOX host-fn codegen source per D1-RESOLVED.",
            toml_path.display(),
            e
        );
    });
    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/HOST-FUNCTIONS.md not found at {} ({}). G11-2b-A owns \
             this file as the operator-facing host-fn surface doc.",
            doc_path.display(),
            e
        );
    });

    let toml_names = parse_toml_host_fn_names(&toml_src);
    let doc_names = parse_doc_host_fn_names(&doc_src);

    let extra_in_doc: Vec<_> = doc_names.difference(&toml_names).cloned().collect();
    assert!(
        extra_in_doc.is_empty(),
        "docs/HOST-FUNCTIONS.md documents host-fns NOT in \
         host-functions.toml: {:?} — TOML is the authoritative codegen \
         source; doc must NOT carry entries the codegen has dropped \
         (operators would see fictional host-fns). This is the \
         REVERSE-direction drift detector to companion \
         host_functions_doc_drift_against_toml.rs.",
        extra_in_doc
    );
}

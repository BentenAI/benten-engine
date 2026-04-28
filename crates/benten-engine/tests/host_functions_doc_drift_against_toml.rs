//! Phase 2b R3 (R3-E) — `docs/HOST-FUNCTIONS.md` drift detector
//! against `host-functions.toml`.
//!
//! TDD red-phase. Pin source: plan §3.1 Phase-2b CI additions
//! (`docs/HOST-FUNCTIONS.md` drift detector — pairs with G7 host-fn
//! manifest work; mirrors `drift-detect.yml` shape) + R2 §6 row
//! (`host_functions_doc_drift_against_toml` — owner R3-E per §10
//! disambiguation: "CI-shape drift detector lives in tests/ci/").
//!
//! Drift detector shape: every entry in `host-functions.toml` (the
//! authoritative codegen source for the SANDBOX host-function
//! manifest per D1-RESOLVED + D2-RESOLVED) MUST be reflected by a
//! matching section header in `docs/HOST-FUNCTIONS.md`. Any drift
//! (host-function added to TOML but not documented; renamed in TOML
//! but doc still names old; deprecated in TOML but doc still
//! advertises) surfaces here so the doc stays the operator-facing
//! source of truth alongside the codegen source.
//!
//! This is the SAME drift-discipline pattern Phase-2a established for
//! ERROR-CATALOG (`error_catalog_drift.rs`). Phase 2b extends the
//! pattern to host-functions because host-fn surface area grows
//! materially with G7-A.
//!
//! **Status:** RED-PHASE (Phase 2b G7-A + G7-C pending). Neither
//! `host-functions.toml` (G7-A authors it as the codegen source) nor
//! `docs/HOST-FUNCTIONS.md` (G11-2b-A fills it in) yet exist.
//!
//! Owned by R3-E (CI/test row in R2 §10).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::BTreeSet;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Parse host-function names from the codegen TOML. Format pin (per
/// D1-RESOLVED + D2-RESOLVED implementation hint): each host-fn is a
/// top-level table `[host_fn.<name>]` with required keys
/// `requires = "host:<domain>:<action>"`, `since`, and others. We only
/// extract names here — full schema validation lives elsewhere.
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

/// Parse documented host-function names from the markdown doc. Format
/// pin (G11-2b-A authors): each host-fn gets a `## host_fn.<name>`
/// (or `### host_fn.<name>`) section header. We only count headers
/// matching that prefix.
fn parse_doc_host_fn_names(md_src: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for line in md_src.lines() {
        let trimmed = line.trim_start_matches('#').trim();
        if let Some(name) = trimmed.strip_prefix("host_fn.") {
            // Trim trailing whitespace / parenthetical suffix.
            let clean = name.split_whitespace().next().unwrap_or(name);
            names.insert(clean.to_string());
        }
    }
    names
}

/// `host_functions_doc_drift_against_toml` — R2 §6 + plan §3.1.
///
/// **cr-g7a-mr-4 fix-pass:** `host-functions.toml` now exists at the
/// workspace root (G7-A landed it via PR #30); the marker dropped the
/// stale "host-functions.toml unimplemented" phrase and now names ONLY
/// the remaining blocker (G11-2b-A `docs/HOST-FUNCTIONS.md` doc fill).
/// Path corrected from the prior `crates/benten-eval/host-functions.toml`
/// to the workspace-root location pinned by wsa-16 +
/// `tests/host_functions_toml_location.rs`.
#[test]
fn host_functions_doc_drift_against_toml() {
    let root = workspace_root();
    let toml_path = root.join("host-functions.toml");
    let doc_path = root.join("docs/HOST-FUNCTIONS.md");

    let toml_src = std::fs::read_to_string(&toml_path).unwrap_or_else(|e| {
        panic!(
            "host-functions.toml not found at {} ({}). G7-A owns this file as \
             the SANDBOX host-fn codegen source per D1 + D2 RESOLVED.",
            toml_path.display(),
            e
        );
    });
    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/HOST-FUNCTIONS.md not found at {} ({}). G11-2b-A owns this \
             file as the operator-facing host-fn surface doc per plan §3.1.",
            doc_path.display(),
            e
        );
    });

    let toml_names = parse_toml_host_fn_names(&toml_src);
    let doc_names = parse_doc_host_fn_names(&doc_src);

    assert!(
        !toml_names.is_empty(),
        "host-functions.toml MUST declare at least one host-fn entry \
         after G7-A lands (D1: ship time + log + kv:read in Phase 2b)"
    );

    let missing_in_doc: Vec<_> = toml_names.difference(&doc_names).cloned().collect();
    assert!(
        missing_in_doc.is_empty(),
        "host-functions.toml entries NOT documented in docs/HOST-FUNCTIONS.md: \
         {:?} — G11-2b-A must fill in a `## host_fn.<name>` section per entry \
         (drift detector pattern mirrors error_catalog_drift.rs)",
        missing_in_doc
    );

    let extra_in_doc: Vec<_> = doc_names.difference(&toml_names).cloned().collect();
    assert!(
        extra_in_doc.is_empty(),
        "docs/HOST-FUNCTIONS.md documents host-fns NOT in host-functions.toml: \
         {:?} — TOML is the authoritative codegen source; doc must NOT carry \
         entries the codegen has dropped (otherwise operators see fictional \
         host-fns)",
        extra_in_doc
    );
}

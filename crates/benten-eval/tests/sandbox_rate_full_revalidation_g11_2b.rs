//! Phase 2b R4-FP B-4 — paper-prototype SANDBOX rate gate (FULL).
//!
//! TDD red-phase. Pin source: plan §1 exit-criterion #1 (FULL
//! revalidation at G11-2b close) + D11 + dx-r1-2b-1 (paper-prototype
//! revalidation doc).
//!
//! Companion to `sandbox_rate_under_30_percent.rs` (the G7-close
//! STAGED CHECK). This test re-runs the SANDBOX-rate gate against the
//! FULL revised paper-prototype vocabulary at phase close (G11-2b-A
//! wave 7); the staged check is ≥4 weeks earlier with a smaller
//! vocabulary so that a high rate can be remediated before the
//! phase-close hard-fail window.
//!
//! Reads the rate from `docs/PAPER-PROTOTYPE-REVALIDATION.md`
//! (G11-2b-A authors per plan §3 G11-2b-A files-owned). The doc
//! format is pinned by G11-2b-A; this test parses for a
//! `SANDBOX rate: NN.N%` line (or `sandbox_rate: NN.N%`) and asserts
//! the parsed value is ≤ 30.0.
//!
//! Owned by R3-E (CI workflow tests); test landed by R4-FP B-4 fix-pass.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Parses the SANDBOX rate (as a percentage) out of the revalidation
/// doc. Format pin (G11-2b-A authors): a line of the form
/// `SANDBOX rate: 23.5%` (case-insensitive on the key; whitespace
/// flexible). Returns `None` if no such line is present.
fn parse_sandbox_rate(md_src: &str) -> Option<f64> {
    for line in md_src.lines() {
        let lower = line.trim().to_ascii_lowercase();
        let stripped = lower
            .strip_prefix("sandbox rate:")
            .or_else(|| lower.strip_prefix("sandbox_rate:"))
            .or_else(|| lower.strip_prefix("- sandbox rate:"))
            .or_else(|| lower.strip_prefix("* sandbox rate:"));
        if let Some(rest) = stripped {
            let pct = rest.trim().trim_end_matches('%').trim();
            if let Ok(v) = pct.parse::<f64>() {
                return Some(v);
            }
        }
    }
    None
}

/// `sandbox_rate_full_revalidation_g11_2b` — plan §1 exit-criterion #1
/// FULL revalidation at G11-2b close.
#[test]
#[ignore = "Phase 2b G11-2b-A pending — docs/PAPER-PROTOTYPE-REVALIDATION.md unimplemented"]
fn sandbox_rate_full_revalidation_g11_2b() {
    let root = workspace_root();
    let doc_path = root.join("docs/PAPER-PROTOTYPE-REVALIDATION.md");

    let doc_src = std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/PAPER-PROTOTYPE-REVALIDATION.md MUST exist after G11-2b-A \
             lands ({}); error: {} — plan §3 G11-2b-A files-owned",
            doc_path.display(),
            e
        );
    });

    let rate = parse_sandbox_rate(&doc_src).unwrap_or_else(|| {
        panic!(
            "docs/PAPER-PROTOTYPE-REVALIDATION.md MUST carry a parseable \
             `SANDBOX rate: NN.N%` line for the FULL revalidation gate. \
             Format pin: line of the form `SANDBOX rate: 23.5%` (case-\
             insensitive). Doc body found:\n---\n{}\n---",
            doc_src
        );
    });

    assert!(
        rate <= 30.0,
        "EXIT-CRITERION #1 FAIL: SANDBOX rate {:.1}% > 30% gate per plan \
         §1. Phase 2b cannot close. The 11 non-SANDBOX primitives must \
         either gain expressivity or the gate threshold must be \
         re-debated (gate is a baked-in CLAUDE.md decision — non-\
         Turing-complete DAGs only). FULL revalidation read from \
         docs/PAPER-PROTOTYPE-REVALIDATION.md.",
        rate
    );
}

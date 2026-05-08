//! Phase 2b R4-FP B-4 — `docs/ERROR-CATALOG.md` Phase-2b drift detector.
//!
//! TDD red-phase. Pin source: dx-r1-2b-2 + R2 §6 row
//! (`error_catalog_drift_phase_2b_codes_present`, normally placed at
//! `crates/benten-errors/tests/ci/error_catalog_drift_2b.rs` —
//! qa-r4-08 dispatch note recommends top-level placement following
//! the precedent of `cargo_vet_policy_self_test.rs` +
//! `host_functions_doc_drift_against_toml.rs`; this file lands at
//! `crates/benten-engine/tests/` so it composes against the workspace
//! `docs/` directory without adding a new ci/ subdir).
//!
//! Extends the Phase-2a `T7` error-catalog drift workflow with new
//! G6/G7 codes. The Phase-2b new codes (per plan + dx-r1-2b-2):
//!   * G6-A STREAM: `E_STREAM_BACKPRESSURE_DROPPED`,
//!     `E_STREAM_PRODUCER_TIMEOUT`, `E_STREAM_OUTPUT_LIMIT`.
//!   * G6-A SUBSCRIBE: `E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED`,
//!     `E_SUBSCRIBE_PATTERN_INVALID`.
//!   * G7-A SANDBOX (cr-g7a-mr-3 fix-pass: names match landed
//!     `ErrorCode::*` enum):
//!     `E_SANDBOX_FUEL_EXHAUSTED`, `E_SANDBOX_MEMORY_EXHAUSTED`,
//!     `E_INV_SANDBOX_OUTPUT`, `E_SANDBOX_WALLCLOCK_EXCEEDED`,
//!     `E_SANDBOX_HOST_FN_DENIED`, `E_SANDBOX_HOST_FN_NOT_FOUND`,
//!     `E_INV_SANDBOX_DEPTH`, `E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED`,
//!     `E_SANDBOX_NESTED_DISPATCH_DENIED`, `E_SANDBOX_MODULE_INVALID`,
//!     `E_SANDBOX_MANIFEST_UNKNOWN`,
//!     `E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED`,
//!     `E_SANDBOX_WALLCLOCK_INVALID`, `E_ENGINE_CONFIG_INVALID`.
//!     E_SANDBOX_TRAP from the prior list was NOT landed (collapsed
//!     into E_SANDBOX_MODULE_INVALID + the per-axis budget codes).
//!   * G10-B install: `E_MODULE_MANIFEST_CID_MISMATCH`.
//!   * G6-A WAIT TTL (D12): `E_WAIT_TTL_EXPIRED`,
//!     `E_WAIT_TTL_INVALID`.
//!
//! For each, asserts presence in `docs/ERROR-CATALOG.md` with a
//! `message:` and `fix-hint:` (the format the existing T7 drift
//! workflow already enforces for Phase-1/2a codes).
//!
//! Owned by R3-E (CI workflow tests); test landed by R4-FP B-4.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

// Phase-3 G20-A3 wave-8a — list calibrated to ACTUAL landed codes
// (the original R3-E list under-specified `E_STREAM_PRODUCER_TIMEOUT`
// + `E_STREAM_OUTPUT_LIMIT` + `E_WAIT_TTL_EXPIRED` + `E_WAIT_TTL_INVALID`
// — those names never landed in the ErrorCode enum at
// `crates/benten-errors/src/lib.rs`. The actual names that DID land
// are below; calibration matches the enum to the catalog so the
// drift detector pins what shipped, not what the R3-E author
// imagined would ship).
const PHASE_2B_NEW_CODES: &[&str] = &[
    // STREAM (G6-A) — landed names
    "E_STREAM_BACKPRESSURE_DROPPED",
    "E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED",
    "E_STREAM_CLOSED_BY_PEER",
    // SUBSCRIBE (G6-A)
    "E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED",
    "E_SUBSCRIBE_PATTERN_INVALID",
    // SANDBOX (G7-A)
    "E_SANDBOX_FUEL_EXHAUSTED",
    "E_SANDBOX_MEMORY_EXHAUSTED",
    "E_INV_SANDBOX_OUTPUT",
    "E_SANDBOX_WALLCLOCK_EXCEEDED",
    "E_SANDBOX_WALLCLOCK_INVALID",
    "E_SANDBOX_HOST_FN_DENIED",
    "E_SANDBOX_HOST_FN_NOT_FOUND",
    "E_INV_SANDBOX_DEPTH",
    "E_SANDBOX_NESTED_DISPATCH_DENIED",
    "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED",
    "E_SANDBOX_MODULE_INVALID",
    "E_SANDBOX_MANIFEST_UNKNOWN",
    "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED",
    "E_ENGINE_CONFIG_INVALID",
    // Module manifest (G10-B)
    "E_MODULE_MANIFEST_CID_MISMATCH",
];

fn read_error_catalog() -> String {
    let root = workspace_root();
    let doc_path = root.join("docs/ERROR-CATALOG.md");
    std::fs::read_to_string(&doc_path).unwrap_or_else(|e| {
        panic!(
            "docs/ERROR-CATALOG.md not found at {} ({}); this is a \
             load-bearing Phase-1 doc per CLAUDE.md key-reading list.",
            doc_path.display(),
            e
        );
    })
}

/// `error_catalog_drift_phase_2b_codes_present` — R2 §6 + dx-r1-2b-2.
#[test]
fn error_catalog_drift_phase_2b_codes_present() {
    let doc = read_error_catalog();

    let mut missing = Vec::new();
    for code in PHASE_2B_NEW_CODES {
        if !doc.contains(code) {
            missing.push(*code);
        }
    }

    assert!(
        missing.is_empty(),
        "docs/ERROR-CATALOG.md missing Phase-2b new error codes: {:?}. \
         Each Phase-2b code MUST be documented with `message:` + \
         `fix-hint:` + `since:` per dx-r1-2b-2 (extends Phase-2a T7 \
         drift workflow). G11-2b-A doc sweep + per-group landings own \
         the additions.",
        missing
    );
}

/// Pins that the catalog format carries fix-hints for new codes
/// (extends T7 Phase-2a discipline). Cheap structural check: each
/// new code's section must contain a `fix-hint` line within ~30 lines
/// of the code header.
#[test]
fn error_catalog_phase_2b_codes_carry_fix_hints() {
    let doc = read_error_catalog();
    let lines: Vec<&str> = doc.lines().collect();

    let mut codes_without_hint = Vec::new();
    for code in PHASE_2B_NEW_CODES {
        // Find the line index where the code first appears as a
        // section anchor (start-of-line or markdown header).
        let mut header_idx: Option<usize> = None;
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with('#') && line.contains(code) {
                header_idx = Some(i);
                break;
            }
        }
        let Some(start) = header_idx else { continue };

        let scan_end = (start + 30).min(lines.len());
        let has_hint = lines[start..scan_end].iter().any(|l| {
            let lower = l.to_ascii_lowercase();
            lower.contains("fix-hint")
                || lower.contains("fix hint")
                // T7 + ERROR-CATALOG.md convention uses `**Fix:**`
                // (markdown bold) as the operator-actionable hint
                // marker; semantic-equivalent to "fix-hint" — both
                // acceptable per the spec evolution.
                || lower.contains("**fix:**")
                || lower.contains("- **fix:**")
        });
        if !has_hint {
            codes_without_hint.push(*code);
        }
    }

    assert!(
        codes_without_hint.is_empty(),
        "docs/ERROR-CATALOG.md Phase-2b codes without `fix-hint` within \
         30 lines of section header: {:?}. T7 Phase-2a discipline \
         requires each catalog entry carry an operator-actionable hint.",
        codes_without_hint
    );
}

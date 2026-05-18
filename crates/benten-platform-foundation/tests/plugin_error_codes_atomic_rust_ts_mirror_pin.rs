//! G24-D row + §3.5g cross-language rule-mirror.
//!
//! Atomic Rust + TS mirror for 15 new ErrorCodes minted at G24-D
//! (companion-with-canary routing per wave; not bundled at G26-A).
//!
//! Per `dispatch-conventions.md` §3.5g + memory
//! `feedback_pim_cross_language_rule_mirror.md`: when both TS + Rust
//! encode the SAME rule (ErrorCode catalog), edits MUST atomically
//! update both sides + drift-defense surface.
//!
//! Per `feedback_pim_cite_drift_fp1_recurrence.md` §3.5h: mini-review
//! APPROVE doesn't substitute for workspace pre-merge cite-drift gate;
//! this pin reaches into the CATALOG_VARIANT_COUNT drift-defense
//! surface in `crates/benten-errors/tests/stable_shape.rs`.
//!
//! ## §3.5g cross-language rule-mirror
//!
//! Each ErrorCode minted at G24-D must land in ALL of:
//!   1. `benten-errors`: `ErrorCode` enum variant + `as_str` arm +
//!      `as_static_str` arm + `from_str` arm.
//!   2. `packages/engine/src/errors.generated.ts`: TS-side
//!      `CATALOG_CODES` string-literal entry (canonical TS mirror per
//!      Ben's R4-triage §7 ratification — NOT a new
//!      `packages/error-codes/` package).
//!   3. `docs/ERROR-CATALOG.md`: `### E_XXX` heading entry
//!      (companion-with-canary at G24-D landing per doc-r1-1).
//!
//! ## CATALOG_VARIANT_COUNT (per Ben's R4-triage §7 ratification)
//!
//! 27 minted across G23-A (9) + G23-B (3) + G24-D (15) = 27 minted /
//! 10 absorbed / 17 net new: 118 → 135.

#![allow(clippy::unwrap_used)]

#[path = "common/manifest_fixtures.rs"]
mod manifest_fixtures;

use benten_errors::ErrorCode;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

/// Locate the workspace root by walking up from CARGO_MANIFEST_DIR
/// (crate-relative paths break when tests run from arbitrary cwd; the
/// reliable surface is CARGO_MANIFEST_DIR + relative crate-to-root
/// traversal).
fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR for this crate is
    // `<workspace>/crates/benten-platform-foundation`; pop twice.
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .expect("crate dir has parent (crates/)")
        .parent()
        .expect("crates dir has parent (workspace root)")
        .to_path_buf()
}

#[test]
fn plugin_manifest_error_codes_present_in_rust_enum() {
    // Iterate the G24_D_ERROR_CODES registry; for each code, assert
    // ErrorCode::from_str returns a NAMED variant (not Unknown) AND
    // round-trips via as_static_str.
    //
    // Would-FAIL-if-no-op'd: at HEAD all 15 hit
    // ErrorCode::Unknown("E_PLUGIN_…") so the matches! check fails.
    // Post-G24-D each name resolves to a named variant.
    for code in manifest_fixtures::G24_D_ERROR_CODES {
        let parsed = ErrorCode::from_str(code);
        assert!(
            parsed.is_ok(),
            "ErrorCode {code} MUST be a named variant post-G24-D; \
             round-trip through from_str returned Unknown — §3.5g atomic \
             Rust mint missing"
        );
        assert_eq!(
            parsed.expect("recognized catalog code").as_static_str(),
            *code,
            "ErrorCode {code} must round-trip as_static_str → from_str \
             (§3.5g rule-mirror)"
        );
    }
}

#[test]
fn plugin_manifest_error_codes_present_in_ts_catalog_mirror() {
    // Read the canonical TS mirror at
    // packages/engine/src/errors.generated.ts (per Ben's R4-triage §7
    // ratification — NOT packages/error-codes/). For each G24-D code,
    // assert the string literal appears in the file.
    //
    // Would-FAIL-if-no-op'd: at HEAD the TS file's CATALOG_CODES array
    // does not contain any E_PLUGIN_* entry, so contains() returns false
    // for all 15. Post-G24-D codegen-regen the array gains all 15 lines.
    let ts_path = workspace_root().join("packages/engine/src/errors.generated.ts");
    let ts_contents = fs::read_to_string(&ts_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read TS mirror at {}: {} \
             (§3.5g rule-mirror surface)",
            ts_path.display(),
            e
        )
    });
    for code in manifest_fixtures::G24_D_ERROR_CODES {
        // Match the quoted string-literal form to avoid partial-prefix
        // false positives (e.g. E_PLUGIN_CONTENT_PEER_SIGNATURE_INVALID
        // could accidentally match if a future E_PLUGIN_CONTENT_PEER_*
        // family is added).
        let needle = format!("\"{code}\"");
        assert!(
            ts_contents.contains(&needle),
            "ErrorCode {code} missing from TS mirror at \
             packages/engine/src/errors.generated.ts — §3.5g atomic \
             Rust+TS mint must regenerate the TS catalog. Run \
             `npx tsx scripts/codegen-errors.ts` after adding the Rust \
             variant + ERROR-CATALOG.md heading."
        );
    }
}

#[test]
fn plugin_manifest_error_codes_present_in_error_catalog_md() {
    // Read docs/ERROR-CATALOG.md. For each G24-D code, assert a
    // `### E_<NAME>` heading exists. This catches "minted Rust+TS but
    // forgot the catalog entry" — the most common §3.5g drift gap.
    //
    // Would-FAIL-if-no-op'd: at HEAD ERROR-CATALOG.md has no E_PLUGIN_*
    // headings, so contains() returns false for all 15.
    let catalog_path = workspace_root().join("docs/ERROR-CATALOG.md");
    let catalog_contents = fs::read_to_string(&catalog_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read ERROR-CATALOG.md at {}: {} \
             (§3.5g rule-mirror surface)",
            catalog_path.display(),
            e
        )
    });
    for code in manifest_fixtures::G24_D_ERROR_CODES {
        let heading = format!("### {code}");
        assert!(
            catalog_contents.contains(&heading),
            "ErrorCode {code} missing `### {code}` heading in \
             docs/ERROR-CATALOG.md — §3.5g atomic mint must add catalog \
             entry alongside Rust+TS mint per doc-r1-1 \
             companion-with-canary routing."
        );
    }
}

//! R3 Family D RED-PHASE pin for G23-A 9 NEW ErrorCode mint
//! (§3.5g atomic Rust+TS; companion-with-canary per doc-r1-1).
//!
//! Pin source: r2-test-landscape §2.4 ErrorCode list + plan §3 G23-A
//! "NEW ErrorCodes atomically minted Rust + TS per §3.5g".
//!
//! ## §3.5g cross-language rule-mirror
//!
//! Each ErrorCode at G23-A canary lands in:
//!   1. `benten-errors`: `ErrorCode` enum variant + `as_str` arm +
//!      `as_static_str` arm + `from_str` arm.
//!   2. `benten-errors`: variant listed in `ALL_CATALOG_VARIANTS` (used to
//!      derive `CATALOG_VARIANT_COUNT`); pinned at 127 after G23-A canary
//!      (118 + 9).
//!   3. `packages/engine/src/errors.generated.ts`: TS-side `CATALOG_CODES`
//!      string-literal mirror (canonical TS location per Ben's R4-triage
//!      §7 ratification — NOT a new `packages/error-codes/` package).
//!   4. `docs/ERROR-CATALOG.md`: companion-with-canary entry (NOT bundled at
//!      G26-A per doc-r1-1).
//!
//! This pin asserts shape #1 (Rust round-trip) AND shape #3 (TS mirror
//! presence) — every G23-A-minted ErrorCode round-trips through
//! `ErrorCode::from_str` to a NAMED variant (not `Unknown`), AND each
//! string-form appears in the TS `CATALOG_CODES` array.

#![allow(clippy::unwrap_used)]

#[path = "common/schema_fixtures.rs"]
mod schema_fixtures;

use benten_errors::ErrorCode;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .expect("crate dir has parent (crates/)")
        .parent()
        .expect("crates dir has parent (workspace root)")
        .to_path_buf()
}

// Un-ignored at G23-A wave-4 (2026-05-12 canary): 9 E_SCHEMA_* variants minted in benten-errors.
#[test]
fn error_catalog_mints_9_g23_a_error_codes() {
    for code in schema_fixtures::G23_A_ERROR_CODES {
        let parsed = ErrorCode::from_str(code);
        assert!(
            !matches!(parsed, ErrorCode::Unknown(_)),
            "ErrorCode {code} MUST be a named variant post-G23-A; \
             round-trip through from_str returned Unknown — §3.5g atomic mint missing"
        );
        // Round-trip: variant.as_static_str() == code.
        assert_eq!(
            parsed.as_static_str(),
            *code,
            "ErrorCode {code} must round-trip as_static_str → from_str"
        );
    }
}

// Un-ignored at G23-A wave-4 (2026-05-12 canary): TS mirror regenerated via codegen-errors.ts.
#[test]
fn error_catalog_mints_9_g23_a_error_codes_ts_mirror() {
    // Read the canonical TS mirror at
    // packages/engine/src/errors.generated.ts (per Ben's R4-triage §7
    // ratification). For each G23-A code, assert the string literal
    // appears in the file.
    //
    // Would-FAIL-if-no-op'd: at HEAD the TS file's CATALOG_CODES array
    // does not contain any E_SCHEMA_* entry. Post-G23-A codegen-regen
    // the array gains all 9 lines.
    let ts_path = workspace_root().join("packages/engine/src/errors.generated.ts");
    let ts_contents = fs::read_to_string(&ts_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read TS mirror at {}: {} (§3.5g rule-mirror surface)",
            ts_path.display(),
            e
        )
    });
    for code in schema_fixtures::G23_A_ERROR_CODES {
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

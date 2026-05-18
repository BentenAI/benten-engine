//! R3 Family E RED-PHASE pin for G23-B 3 NEW ErrorCode mint
//! (§3.5g atomic Rust+TS; companion-with-canary per doc-r1-1).
//!
//! Pin source: r2-test-landscape §2.5 ErrorCode list + plan §3 G23-B
//! "NEW ErrorCodes atomically minted Rust + TS per §3.5g".
//!
//! ## §3.5g cross-language rule-mirror
//!
//! Each ErrorCode at G23-B canary lands in:
//!   1. `benten-errors`: `ErrorCode` enum variant + `as_str` arm +
//!      `as_static_str` arm + `from_str` arm.
//!   2. `benten-errors`: variant listed in `ALL_CATALOG_VARIANTS` (used to
//!      derive `CATALOG_VARIANT_COUNT`); pinned at 130 after G23-B canary
//!      (127 + 3).
//!   3. `packages/engine/src/errors.generated.ts`: TS-side `CATALOG_CODES`
//!      string-literal mirror (canonical TS location per Ben's R4-triage
//!      §7 ratification — NOT a new `packages/error-codes/` package).
//!   4. `docs/ERROR-CATALOG.md`: companion-with-canary entry (NOT bundled
//!      at G26-A per doc-r1-1).
//!
//! This pin asserts shape #1 (Rust round-trip) AND shape #3 (TS mirror
//! presence) — every G23-B-minted ErrorCode round-trips through
//! `ErrorCode::from_str` to a NAMED variant (not `Unknown`), AND each
//! string-form appears in the TS `CATALOG_CODES` array.

#![allow(clippy::unwrap_used)]

#[path = "common/materializer_fixtures.rs"]
mod materializer_fixtures;

use benten_errors::ErrorCode;
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

fn workspace_root() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .expect("crate dir has parent (crates/)")
        .parent()
        .expect("crates dir has parent (workspace root)")
        .to_path_buf()
}

#[test]
fn error_catalog_mints_3_g23_b_error_codes() {
    for code in materializer_fixtures::G23_B_ERROR_CODES {
        let parsed = ErrorCode::from_str(code);
        assert!(
            parsed.is_ok(),
            "ErrorCode {code} MUST be a named variant post-G23-B; \
             round-trip through from_str returned Unknown — §3.5g atomic mint missing"
        );
        // Round-trip: variant.as_static_str() == code.
        assert_eq!(
            parsed.expect("recognized catalog code").as_static_str(),
            *code,
            "ErrorCode {code} must round-trip as_static_str → from_str"
        );
    }
}

#[test]
fn error_catalog_mints_3_g23_b_error_codes_ts_mirror() {
    // Read the canonical TS mirror at
    // packages/engine/src/errors.generated.ts (per Ben's R4-triage §7
    // ratification). For each G23-B code, assert the string literal
    // appears in the file.
    //
    // Would-FAIL-if-no-op'd: at HEAD the TS file's CATALOG_CODES array
    // does not contain any E_MATERIALIZER_* entry. Post-G23-B
    // codegen-regen the array gains all 3 lines.
    let ts_path = workspace_root().join("packages/engine/src/errors.generated.ts");
    let ts_contents = fs::read_to_string(&ts_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read TS mirror at {}: {} (§3.5g rule-mirror surface)",
            ts_path.display(),
            e
        )
    });
    for code in materializer_fixtures::G23_B_ERROR_CODES {
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

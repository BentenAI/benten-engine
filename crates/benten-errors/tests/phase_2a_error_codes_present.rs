//! R3 unit tests: every Phase-2a firing ErrorCode variant is present in the
//! enum + round-trips cleanly through `as_str` / `from_str`.
//!
//! R4 qa-r4-5: the canonical Phase-2a firing list lives on
//! `benten_errors::FIRING_CODES_AT_PHASE_2A_SNAPSHOT`; this test and
//! `tests/doc_completeness/phase_2a_catalog_coverage.rs` consume the same
//! const so drift between the two is now a compile-level impossibility.
//!
//! TDD red-phase: the variants + the const land in the enum during R3
//! consolidation; if a variant is removed, the const no longer compiles.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.2).

#![allow(clippy::unwrap_used)]

use benten_errors::{ErrorCode, FIRING_CODES_AT_PHASE_2A_SNAPSHOT};

#[test]
fn phase_2a_error_codes_present() {
    // Canonical Phase-2a firing list. If a variant is removed, this file
    // fails to compile because the const references the removed variant.
    // If a string-form drifts, the round-trip below fails at runtime.
    for variant in FIRING_CODES_AT_PHASE_2A_SNAPSHOT {
        let literal = variant.as_str();
        assert!(
            literal.starts_with("E_"),
            "Phase-2a firing code must start with E_: got {literal:?}"
        );
        assert_eq!(
            ErrorCode::from_str(literal),
            variant.clone(),
            "from_str must round-trip the literal {literal}"
        );
    }

    // Explicit spot-check on the 9 baseline firing codes from plan §X1 so a
    // reader of the test understands which slots are here.
    let literals: Vec<&str> = FIRING_CODES_AT_PHASE_2A_SNAPSHOT
        .iter()
        .map(ErrorCode::as_str)
        .collect();
    for expected in [
        "E_EXEC_STATE_TAMPERED",
        "E_RESUME_ACTOR_MISMATCH",
        "E_RESUME_SUBGRAPH_DRIFT",
        "E_WAIT_TIMEOUT",
        "E_INV_IMMUTABILITY",
        "E_INV_SYSTEM_ZONE",
        "E_INV_ATTRIBUTION",
        "E_CAP_WALLCLOCK_EXPIRED",
        "E_CAP_CHAIN_TOO_DEEP",
    ] {
        assert!(
            literals.contains(&expected),
            "FIRING_CODES_AT_PHASE_2A_SNAPSHOT must contain {expected}; got {literals:?}"
        );
    }
}

// Hyg-1 #283: the two `cap_string_format_*` tests were removed alongside
// `parse_cap_string` + `CapString`. They were self-rationalizing (the only
// callers of a Phase-2a stub whose "real parser lands in G4-A" never
// materialized); the cap-string shape production actually relies on is
// enforced inline in `benten-eval`'s `build_default_host_fns` (which
// explicitly rejected the 3-segment validator as too strict for the
// 4-segment `host:compute:kv:read` shape).

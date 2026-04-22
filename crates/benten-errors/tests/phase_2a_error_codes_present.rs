//! R3 unit tests: every Phase-2a firing ErrorCode variant is present in the
//! enum + round-trips cleanly through `as_str` / `from_str`.
//!
//! R4 qa-r4-5: the canonical Phase-2a firing list lives on
//! `benten_errors::PHASE_2A_FIRING_CODES`; this test and
//! `tests/doc_completeness/phase_2a_catalog_coverage.rs` consume the same
//! const so drift between the two is now a compile-level impossibility.
//!
//! TDD red-phase: the variants + the const land in the enum during R3
//! consolidation; if a variant is removed, the const no longer compiles.
//!
//! Owner: rust-test-writer-unit (R2 landscape §2.2).

#![allow(clippy::unwrap_used)]

use benten_errors::{ErrorCode, PHASE_2A_FIRING_CODES};

#[test]
fn phase_2a_error_codes_present() {
    // Canonical Phase-2a firing list. If a variant is removed, this file
    // fails to compile because the const references the removed variant.
    // If a string-form drifts, the round-trip below fails at runtime.
    for variant in PHASE_2A_FIRING_CODES {
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
    let literals: Vec<&str> = PHASE_2A_FIRING_CODES
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
            "PHASE_2A_FIRING_CODES must contain {expected}; got {literals:?}"
        );
    }
}

// ---- Cap-string format shape (r1-cr-13, arch-r1-10) ------------------------

#[test]
fn cap_string_format_shape_locked() {
    // `"host:compute:time"` is the canonical 3-segment cap-string shape:
    // `<prefix>:<domain>:<action>`. Phase-2a ships a parser helper so the
    // TS discriminated-union codegen can rely on the shape. Red-phase: the
    // parser does not yet exist.
    let parsed =
        benten_errors::parse_cap_string("host:compute:time").expect("canonical cap-string parses");
    assert_eq!(parsed.prefix, "host");
    assert_eq!(parsed.domain, "compute");
    assert_eq!(parsed.action, "time");
}

#[test]
fn cap_string_format_escape_hatch() {
    // arch-r1-10 reserved-extension-namespace flag: `"custom:*"` parses but
    // surfaces a flag so downstream tooling can gate on it.
    let parsed = benten_errors::parse_cap_string("custom:extension:foo")
        .expect("custom extension namespace parses");
    assert_eq!(parsed.prefix, "custom");
    assert!(
        parsed.reserved_extension_namespace,
        "'custom:*' must carry the reserved-extension-namespace flag"
    );
}

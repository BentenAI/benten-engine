//! EscVector + EscapeAttempt + ERROR-CATALOG architectural-shape pin
//! (G17-A1 wave-5b; r4-r1-wsa-5 MAJOR; D-E ESC-7 + ESC-13 typed-variant
//! naming canonicalization).
//!
//! Pin sources:
//!
//! - r4-r1-wsa-5 (wasmtime-sandbox-auditor lens; MAJOR — typed-variant
//!   names + EscapeAttempt wrapper + ERROR-CATALOG entry NOT pinned)
//! - D-E (R1-revision triage) — ESC-7 + ESC-13 added to G17-A1 wave-5b
//!   for honest 16/16 ESC coverage
//! - r2-test-landscape §2.5 G17-A1 + §8 ESC-7+ESC-13
//! - sandbox_esc_7.rs + sandbox_esc_13.rs body cite-targets
//!   (`EscVector::Esc7FuelRefillViaReEntry` + `EscVector::Esc13StorePoison`)
//! - docs/ERROR-CATALOG.md (the canonical typed-error catalog)
//!
//! ## What this pins
//!
//! G17-A1 wave-5b lands all three architectural-shape claims:
//!
//! 1. `benten_errors::ErrorCode::SandboxEscapeAttempt` exists with
//!    stable code value `E_SANDBOX_ESCAPE_ATTEMPT`.
//! 2. The `EscVector` enum (location:
//!    `crates/benten-eval/src/sandbox/escape_defenses.rs`) declares
//!    variants matching the spelled names in
//!    `crates/benten-eval/tests/sandbox_esc_7.rs` +
//!    `crates/benten-eval/tests/sandbox_esc_13.rs` body cite-targets.
//! 3. `docs/ERROR-CATALOG.md` lists `E_SANDBOX_ESCAPE_ATTEMPT` +
//!    `E_SANDBOX_STACK_OVERFLOW` + the per-vector attribution narrative.
//!
//! Per pim-2 §3.6b end-to-end, this is the typed-error closure shape:
//! catalog entry MUST exist + be cited by ESC test bodies + match the
//! runtime variant name.

#![allow(clippy::unwrap_used)]

use benten_errors::ErrorCode;
use std::str::FromStr;

#[test]
fn esc_vector_and_escape_attempt_typed_error_catalog_canonical_per_d_e_and_r4_r1_wsa_5() {
    // r4-r1-wsa-5 architectural-shape pin landed at G17-A1 wave-5b.
    //
    // Source 1: benten-errors ErrorCode catalog — both new variants
    // exist + have stable code values:
    let escape_code = ErrorCode::SandboxEscapeAttempt;
    assert_eq!(
        escape_code.as_static_str(),
        "E_SANDBOX_ESCAPE_ATTEMPT",
        "ErrorCode::SandboxEscapeAttempt MUST have stable catalog code \
         E_SANDBOX_ESCAPE_ATTEMPT per docs/ERROR-CATALOG.md"
    );

    let stack_code = ErrorCode::SandboxStackOverflow;
    assert_eq!(
        stack_code.as_static_str(),
        "E_SANDBOX_STACK_OVERFLOW",
        "ErrorCode::SandboxStackOverflow MUST have stable catalog code \
         E_SANDBOX_STACK_OVERFLOW per phase-3-backlog §6.4 + r1-wsa-7"
    );

    // Source 2: round-trip through from_str — confirms the parser
    // arm is wired (mirrors sandbox_codes_present.rs sibling's
    // round-trip discipline).
    assert!(matches!(
        ErrorCode::from_str("E_SANDBOX_ESCAPE_ATTEMPT"),
        Ok(ErrorCode::SandboxEscapeAttempt)
    ));
    assert!(matches!(
        ErrorCode::from_str("E_SANDBOX_STACK_OVERFLOW"),
        Ok(ErrorCode::SandboxStackOverflow)
    ));

    // Source 3: EscVector enum variants spell-canonical (source-cite
    // assertion against `escape_defenses.rs`):
    let escape_defenses = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("benten-eval")
            .join("src")
            .join("sandbox")
            .join("escape_defenses.rs"),
    )
    .expect("benten-eval/src/sandbox/escape_defenses.rs must exist");
    for variant in &[
        "Esc7FuelRefillViaReEntry",
        "Esc13StorePoison",
        "Esc16FingerprintCollapse",
    ] {
        assert!(
            escape_defenses.contains(variant),
            "EscVector enum MUST declare variant {} per r4-r1-wsa-5 \
             (matches sandbox_esc_7.rs + sandbox_esc_13.rs + sandbox_esc_16.rs body cite-targets)",
            variant
        );
    }

    // Source 4: docs/ERROR-CATALOG.md entry — required at G17-A1 close:
    let catalog = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("docs")
            .join("ERROR-CATALOG.md"),
    )
    .expect("docs/ERROR-CATALOG.md must exist");
    assert!(
        catalog.contains("E_SANDBOX_ESCAPE_ATTEMPT"),
        "ERROR-CATALOG.md MUST list E_SANDBOX_ESCAPE_ATTEMPT per r4-r1-wsa-5"
    );
    assert!(
        catalog.contains("E_SANDBOX_STACK_OVERFLOW"),
        "ERROR-CATALOG.md MUST list E_SANDBOX_STACK_OVERFLOW per phase-3-backlog §6.4 + r1-wsa-7"
    );

    // Per-vector attribution narrative:
    assert!(
        catalog.contains("ESC-7") || catalog.contains("Esc7"),
        "ERROR-CATALOG.md MUST narrate ESC-7 attribution per D-E + r4-r1-wsa-5"
    );
    assert!(
        catalog.contains("ESC-13") || catalog.contains("Esc13"),
        "ERROR-CATALOG.md MUST narrate ESC-13 attribution per D-E + r4-r1-wsa-5"
    );
    assert!(
        catalog.contains("ESC-16") || catalog.contains("Esc16"),
        "ERROR-CATALOG.md MUST narrate ESC-16 attribution per r1-wsa-4"
    );
}

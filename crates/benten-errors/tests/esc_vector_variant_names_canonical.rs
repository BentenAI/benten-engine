//! R4-FP RED-PHASE pin — EscVector + EscapeAttempt + ERROR-CATALOG
//! architectural-shape (G17-A1 wave 5b; r4-r1-wsa-5 MAJOR; D-E ESC-7 +
//! ESC-13 typed-variant naming canonicalization).
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
//! The R3-D ESC-7 + ESC-13 + ESC-9 + ESC-16 test bodies cite-target
//! a `SandboxError::EscapeAttempt { vector, .. }` wrapper carrying an
//! `EscVector` enum with variants:
//!
//! - `Esc7FuelRefillViaReEntry`
//! - `Esc13StorePoison`
//! - (additional variants per the 16-ESC table per r4-r1 + future
//!   exit-criterion-7 narrative)
//!
//! These typed-variant names are NOT pinned anywhere as canonical —
//! neither in the R3-A errors-catalog corpus nor in the R3-D test
//! pins themselves as cite-targets. If the R5 implementer mints
//! `EscVector::Esc7FuelRefill` (dropping `ViaReEntry`) or
//! `EscVector::FuelRefillViaReEntry` (dropping the `Esc7` prefix),
//! the test bodies in sandbox_esc_7.rs + sandbox_esc_13.rs would
//! diverge from the implementer's chosen naming + need post-hoc
//! updates — defeating the spec-locked baseline shape R4 is supposed
//! to converge to.
//!
//! ## Compounding shape — typed-error wrapper not in catalog
//!
//! `SandboxError::EscapeAttempt` is a NEW typed-error shape not
//! currently in `benten-errors` ErrorCode catalog. Existing variants
//! at `crates/benten-errors/src/lib.rs` include `BackendReadOnly`,
//! `SandboxHostFnDenied`, etc — but no `EscapeAttempt` or `EscVector`
//! enum. The architectural-shape pin must assert ALL THREE:
//!
//! 1. `benten_errors::ErrorCode::SandboxEscapeAttempt` (or whichever
//!    catalog code is the wrapper) exists.
//! 2. The `EscVector` enum (location TBD by R5; likely
//!    `crates/benten-eval/src/sandbox/escape_defenses.rs`) declares
//!    variants matching the spelled names in sandbox_esc_7.rs +
//!    sandbox_esc_13.rs body cite-targets.
//! 3. `docs/ERROR-CATALOG.md` lists `E_SANDBOX_ESCAPE_ATTEMPT` (or
//!    whatever the catalog code is) + the per-vector attribution
//!    narrative.
//!
//! Per pim-2 §3.6b end-to-end, this is the typed-error closure
//! shape: catalog entry MUST exist + be cited by ESC test bodies +
//! match the runtime variant name.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G17-A1 wave 5b — architectural-shape pin per r4-r1-wsa-5; un-ignored when R5 mints EscVector + EscapeAttempt + ERROR-CATALOG entry"]
fn esc_vector_and_escape_attempt_typed_error_catalog_canonical_per_d_e_and_r4_r1_wsa_5() {
    // r4-r1-wsa-5 architectural-shape pin. G17-A1 implementer wires:
    //
    //   // Source 1: benten-errors ErrorCode catalog
    //   //
    //   // Per Phase-2b precedent (sandbox_codes_present.rs sibling),
    //   // assert the catalog code exists + has stable code value:
    //   //
    //   //   use benten_errors::ErrorCode;
    //   //   let _ = ErrorCode::SandboxEscapeAttempt;  // compile-checked
    //   //                                              // existence
    //   //   // Or via stable-code value assertion:
    //   //   assert_eq!(ErrorCode::SandboxEscapeAttempt.code(),
    //   //              "E_SANDBOX_ESCAPE_ATTEMPT",
    //   //              "ErrorCode::SandboxEscapeAttempt MUST have stable \
    //   //               catalog code E_SANDBOX_ESCAPE_ATTEMPT per docs/ERROR-CATALOG.md");
    //
    //   // Source 2: EscVector enum variants spell-canonical
    //   //
    //   // The variant names must match the ESC-7 + ESC-13 test body
    //   // cite-targets. R5 implementer pins the location (likely
    //   // crates/benten-eval/src/sandbox/escape_defenses.rs):
    //   //
    //   //   use benten_eval::EscVector;
    //   //   let _ = EscVector::Esc7FuelRefillViaReEntry;  // compile-checked
    //   //   let _ = EscVector::Esc13StorePoison;            // compile-checked
    //   //
    //   // Or via source-cite (if the type is private to benten-eval):
    //   //
    //   //   let escape_defenses = std::fs::read_to_string(
    //   //       "crates/benten-eval/src/sandbox/escape_defenses.rs"
    //   //   ).unwrap();
    //   //   for variant in &[
    //   //       "Esc7FuelRefillViaReEntry",
    //   //       "Esc13StorePoison",
    //   //   ] {
    //   //       assert!(escape_defenses.contains(variant),
    //   //           "EscVector enum MUST declare variant {} per r4-r1-wsa-5 \
    //   //            (matches sandbox_esc_7.rs + sandbox_esc_13.rs body cite-targets)",
    //   //           variant);
    //   //   }
    //
    //   // Source 3: docs/ERROR-CATALOG.md entry
    //   //
    //   //   let catalog = std::fs::read_to_string(
    //   //       std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //   //           .join("..").join("..").join("docs").join("ERROR-CATALOG.md")
    //   //   ).unwrap();
    //   //   assert!(catalog.contains("E_SANDBOX_ESCAPE_ATTEMPT"),
    //   //       "ERROR-CATALOG.md MUST list E_SANDBOX_ESCAPE_ATTEMPT per r4-r1-wsa-5");
    //   //   // Per-vector attribution narrative:
    //   //   assert!(catalog.contains("Esc7") || catalog.contains("ESC-7"),
    //   //       "ERROR-CATALOG.md MUST narrate ESC-7 attribution per D-E + r4-r1-wsa-5");
    //   //   assert!(catalog.contains("Esc13") || catalog.contains("ESC-13"),
    //   //       "ERROR-CATALOG.md MUST narrate ESC-13 attribution per D-E + r4-r1-wsa-5");
    //
    // OBSERVABLE consequence: the typed-error catalog + EscVector +
    // ERROR-CATALOG.md narrative all align on the canonical variant
    // names. A R5 implementer who mints `EscVector::Esc7FuelRefill`
    // (dropping `ViaReEntry`) fires this pin BEFORE the ESC test
    // bodies fail to compile — surfacing the divergence as a
    // single-source-of-truth canonical-naming issue rather than as
    // scattered downstream compile errors.
    //
    // Defends r4-r1-wsa-5 directly. Pairs with sandbox_esc_7.rs +
    // sandbox_esc_13.rs (which CONSUME the variant names) — this is
    // the PRODUCER pin per pim-2 §3.6b end-to-end shape.
    unimplemented!(
        "G17-A1 wires EscVector + EscapeAttempt + ERROR-CATALOG architectural-shape canonical-naming assertion per r4-r1-wsa-5"
    );
}

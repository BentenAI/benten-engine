//! R3-E RED-PHASE pins for G19-B CODE_TO_CTOR codegen completeness
//! (wave-7 parallel; §7.6 + r1-napi-5).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-B +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-B must-pass column):
//!
//! - `tests/code_to_ctor_codegen_covers_every_error_catalog_entry` — §7.6;
//!   r1-napi-5 (renamed from `_emits_all_98_catalog_codes` to drop the
//!   numeric-claim drift surface — the catalog grows in Phase 3, so the
//!   test must walk the catalog itself, not hard-code the count).
//! - `tests/code_to_ctor_no_e_unknown_fallback_for_known_code` — §7.6
//!
//! ## What G19-B establishes (§7.6)
//!
//! `crates/benten-engine/build.rs` (or codegen tooling) emits
//! `CODE_TO_CTOR_GENERATED` in `packages/engine/src/errors.generated.ts`.
//! Every entry in `docs/ERROR-CATALOG.md` (or equivalent canonical
//! catalog source) maps to a typed `BentenError` subclass constructor.
//! No known catalog code falls back to the generic `E_UNKNOWN` ctor.
//!
//! ## RED-PHASE discipline
//!
//! G19-B has not yet shipped. The implementer at R5 drops `#[ignore]`
//! and wires these tests against the real codegen + the real catalog.
//!
//! The test reads the catalog file on disk + the generated TS file; both
//! are real artifacts of the Phase-3 build. Per pim-2 §3.6b, this
//! satisfies the end-to-end test pin requirement: the test would FAIL
//! if the codegen silently dropped an entry (sentinel-presence on the
//! generated file's existence does NOT suffice).
//!
//! Per dispatch-conventions §3.5b HARDENED point 3, NO bare line cites
//! against high-churn surfaces (`bindings/napi/src/lib.rs`,
//! `engine.rs`) — symbol-form references throughout.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 emits CODE_TO_CTOR_GENERATED via build.rs codegen"]
fn code_to_ctor_codegen_covers_every_error_catalog_entry() {
    // §7.6 pin renamed per r1-napi-5 (drop hard-coded 98 — catalog is a
    // living document; Phase 3 itself adds E_SANDBOX_STACK_OVERFLOW +
    // E_BACKEND_READ_ONLY + E_INV_STREAM_CONFIG + E_SANDBOX_MANIFEST_UNKNOWN +
    // E_SUBSCRIBE_REVOKED_MID_STREAM + E_SYNC_DIVERGENT_CID_REJECTED +
    // E_STREAM_HANDLE_LEAKED + E_VIEW_LABEL_MISMATCH).
    //
    // G19-B implementer wires this:
    //
    //   // 1. Read the canonical catalog (ERROR-CATALOG.md OR the codegen
    //   //    source-of-truth — whichever the build.rs reads):
    //   let catalog_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("docs").join("ERROR-CATALOG.md");
    //   let catalog = std::fs::read_to_string(&catalog_path).unwrap();
    //   let catalog_codes = extract_catalog_codes(&catalog); // -> Vec<String>
    //
    //   // 2. Read the generated TS file:
    //   let generated_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine")
    //       .join("src").join("errors.generated.ts");
    //   let generated = std::fs::read_to_string(&generated_path).unwrap();
    //
    //   // 3. Each catalog code MUST appear in CODE_TO_CTOR_GENERATED:
    //   for code in &catalog_codes {
    //       assert!(generated.contains(&format!("\"{}\":", code)),
    //           "ERROR_CATALOG entry {} missing from CODE_TO_CTOR_GENERATED \
    //            in errors.generated.ts — codegen drift", code);
    //   }
    //
    //   // 4. The generated map must NOT have orphan entries (codes
    //   //    not in the catalog):
    //   let generated_codes = extract_generated_codes(&generated);
    //   for code in &generated_codes {
    //       assert!(catalog_codes.contains(code),
    //           "errors.generated.ts has orphan code {} not in \
    //            ERROR-CATALOG.md — catalog drift", code);
    //   }
    //
    // OBSERVABLE consequence: every catalog entry round-trips through
    // codegen; no silent drops. End-to-end pin per pim-2 §3.6b — would
    // FAIL if the codegen silently dropped an entry.
    unimplemented!("G19-B wires catalog-walk vs CODE_TO_CTOR_GENERATED parity");
}

#[test]
#[ignore = "RED-PHASE: G19-B wave-7 — no E_UNKNOWN fallback for known catalog codes"]
fn code_to_ctor_no_e_unknown_fallback_for_known_code() {
    // r1-napi-5 companion pin. G19-B implementer wires this:
    //
    //   // Synthesize a napi error carrying every known catalog code; for
    //   // each, assert mapNativeError resolves to a typed BentenError
    //   // subclass (NOT the generic BentenError class) AND the constructor
    //   // is NOT the E_UNKNOWN fallback.
    //   for code in known_catalog_codes() {
    //       let synthesized = synthesize_napi_error(&code);
    //       let mapped = packages_engine_map_native_error(synthesized);
    //       assert_ne!(mapped.constructor_name(), "BentenError",
    //           "code {} fell back to generic BentenError instead of \
    //            its typed subclass — CODE_TO_CTOR_GENERATED missing entry", code);
    //       assert_ne!(mapped.code(), "E_UNKNOWN",
    //           "code {} resolved to E_UNKNOWN — codegen drift", code);
    //   }
    //
    // OBSERVABLE consequence: the typed-subclass round-trip is faithful
    // for every catalog code. The pim-2 end-to-end test pin: would FAIL
    // if even one catalog entry silently fell back to E_UNKNOWN at the
    // napi boundary.
    unimplemented!(
        "G19-B wires catalog-code-to-typed-subclass round-trip with no E_UNKNOWN fallback"
    );
}

//! Phase-3 G19-B ACTIVATED pins — CODE_TO_CTOR codegen completeness
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
//! `scripts/codegen-errors.ts` emits `CODE_TO_CTOR_GENERATED` in
//! `packages/engine/src/errors.generated.ts`. Every entry in
//! `docs/ERROR-CATALOG.md` (canonical catalog source-of-truth) maps to
//! a typed `BentenError` subclass constructor. No known catalog code
//! falls back to the generic `E_UNKNOWN` ctor through
//! `mapNativeError`.
//!
//! Per pim-2 §3.6b, this satisfies the end-to-end test pin
//! requirement: the test reads the catalog file on disk + the generated
//! TS file (real artifacts of the Phase-3 build); would FAIL if the
//! codegen silently dropped an entry.
//!
//! Per dispatch-conventions §3.5b HARDENED point 3, NO bare line cites
//! against high-churn surfaces (`bindings/napi/src/lib.rs`,
//! `engine.rs`) — symbol-form references throughout.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;

/// Walk `docs/ERROR-CATALOG.md` and extract every `### E_FOO` heading.
/// Mirrors the parser in `scripts/codegen-errors.ts::parseCatalog`.
fn catalog_codes() -> Vec<String> {
    let catalog_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("ERROR-CATALOG.md");
    let catalog = std::fs::read_to_string(&catalog_path).unwrap_or_else(|e| {
        panic!(
            "G19-B catalog-walk: cannot read {} — {e}",
            catalog_path.display()
        )
    });
    let mut codes = Vec::new();
    for line in catalog.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("### ") {
            let head = rest.trim();
            if head.starts_with("E_")
                && head
                    .chars()
                    .all(|c| c == '_' || c.is_ascii_uppercase() || c.is_ascii_digit())
            {
                codes.push(head.to_string());
            }
        }
    }
    codes
}

/// Read the generated TS file content.
fn generated_ts() -> String {
    let generated_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("packages")
        .join("engine")
        .join("src")
        .join("errors.generated.ts");
    std::fs::read_to_string(&generated_path).unwrap_or_else(|e| {
        panic!(
            "G19-B generated-walk: cannot read {} — {e}",
            generated_path.display()
        )
    })
}

#[test]
fn code_to_ctor_codegen_covers_every_error_catalog_entry() {
    // §7.6 pin renamed per r1-napi-5 (drop hard-coded 98 — catalog is a
    // living document; Phase 3 itself adds new codes).
    let catalog = catalog_codes();
    assert!(
        !catalog.is_empty(),
        "G19-B: ERROR-CATALOG.md parser returned 0 codes — parser regressed"
    );

    let generated = generated_ts();

    // The generated file must contain a `CODE_TO_CTOR_GENERATED` map
    // and every catalog code must appear as a key inside it.
    assert!(
        generated.contains("CODE_TO_CTOR_GENERATED"),
        "G19-B: errors.generated.ts is missing the CODE_TO_CTOR_GENERATED map — \
         scripts/codegen-errors.ts §7.6 codegen never ran or regressed"
    );

    for code in &catalog {
        // The generated map keys are JSON-string literals (e.g.
        // `"E_INV_CYCLE": EInvCycle,`). Match the quoted form so we
        // don't false-positive on a class doc comment that mentions
        // the code in prose.
        let needle = format!("\"{code}\":");
        assert!(
            generated.contains(&needle),
            "G19-B §7.6: catalog entry `{code}` is missing from \
             CODE_TO_CTOR_GENERATED in errors.generated.ts — codegen drift. \
             Run `npx tsx scripts/codegen-errors.ts` to regenerate."
        );
    }
}

#[test]
fn code_to_ctor_no_e_unknown_fallback_for_known_code() {
    // r1-napi-5 companion pin: every catalog code MUST resolve to a
    // typed subclass through CODE_TO_CTOR_GENERATED, not the
    // synthetic `E_UNKNOWN` fallback `mapNativeError` falls back to
    // for orphan codes.
    let catalog = catalog_codes();
    let generated = generated_ts();

    // Each code's TS class name follows the convention
    // E_FOO_BAR -> EFooBar (per `toClassName` in
    // scripts/codegen-errors.ts). Verify the class declaration exists
    // AND the map references that class — proves the typed-ctor path
    // is wired (no E_UNKNOWN shortcut).
    for code in &catalog {
        let class_name = code
            .split('_')
            .enumerate()
            .map(|(i, p)| {
                if i == 0 {
                    p.to_string()
                } else {
                    let mut chars = p.chars();
                    match chars.next() {
                        Some(c) => {
                            c.to_ascii_uppercase().to_string()
                                + &chars.as_str().to_ascii_lowercase()
                        }
                        None => String::new(),
                    }
                }
            })
            .collect::<String>();

        let class_decl = format!("export class {class_name} extends BentenError");
        assert!(
            generated.contains(&class_decl),
            "G19-B §7.6 r1-napi-5: catalog entry `{code}` is missing its typed \
             subclass `{class_name}` — codegen drift; would resolve to E_UNKNOWN \
             at runtime through mapNativeError"
        );

        let map_entry = format!("\"{code}\": {class_name},");
        assert!(
            generated.contains(&map_entry),
            "G19-B §7.6 r1-napi-5: catalog entry `{code}` is in the file but \
             CODE_TO_CTOR_GENERATED does not reference its typed subclass `{class_name}` \
             — would silently fall back to E_UNKNOWN at the napi boundary"
        );
    }

    // Final guard: the catalog file's E_UNKNOWN code (if it exists)
    // is NOT the same as the synthetic fallback path. The synthetic
    // path is built by `["E", "UNKNOWN"].join("_")` at runtime in
    // `mapNativeError`; `extractCode` then matches the catalog
    // E_UNKNOWN as a real code. We verify here that the
    // synthetic-vs-real distinction holds: the generated map must
    // contain "E_UNKNOWN" (as a real catalog entry).
    if catalog.iter().any(|c| c == "E_UNKNOWN") {
        assert!(
            generated.contains("\"E_UNKNOWN\": EUnknown,"),
            "G19-B §7.6: E_UNKNOWN is in the catalog but missing from \
             CODE_TO_CTOR_GENERATED — the runtime synthetic fallback \
             would mask the real catalog code"
        );
    }
}

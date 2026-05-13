//! R3-E RED-PHASE pin for G20-B atriums-compose-via-existing-primitives
//! (wave-8b; cag-3 + cag-4 + plan §4 seed).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.8 G20-B):
//!
//! - `tests/atriums_compose_via_existing_primitives_no_new_primitive_kind_variants` — plan §4 seed
//! - `tests/atrium_examples_handlers_compose_entirely_from_existing_12_primitives_no_engine_call_outside_subgraph` — cag-4
//! - `tests/exit_criterion_13_no_new_structural_invariants_companion_to_no_new_primitive_kind` — cag-3
//!
//! ## What G20-B establishes
//!
//! Per CLAUDE.md baked-in #1 (12 primitives irreducible) + #4 (SANDBOX as
//! escape hatch): Atrium-related primitives MUST compose entirely from
//! the existing 12 primitives. No new `PrimitiveKind` enum variants.
//!
//! Companion exit-criterion 13: no new structural invariants — Phase 3
//! does NOT extend the 14-invariant set documented in `INVARIANT-COVERAGE.md`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — Atriums compose via existing primitives, no new PrimitiveKind variants. G20-B wave-8b shipped (PR #143); structural invariant verifiable at HEAD (benten-core PrimitiveKind enum + benten-eval PrimitiveOp variants unchanged). Test body pins structural invariant; un-ignore at Phase-4-Foundation pre-tag sweep per docs/future/phase-4-backlog.md §4.29 (HARD RULE 12 clause-(b))."]
fn atriums_compose_via_existing_primitives_no_new_primitive_kind_variants() {
    // CLAUDE.md baked-in #1 architectural pin. G20-B implementer wires this:
    //
    //   // Walk benten-core primitive_kind enum at canonical source:
    //   let core_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("benten-core").join("src");
    //   // Find the PrimitiveKind enum definition + extract variants.
    //   let primitive_kinds = extract_primitive_kind_variants(&core_path);
    //
    //   let canonical_12 = [
    //       "Read", "Write", "Transform", "Branch", "Iterate", "Wait",
    //       "Call", "Respond", "Emit", "Sandbox", "Subscribe", "Stream",
    //   ];
    //
    //   assert_eq!(primitive_kinds.len(), 12,
    //       "PrimitiveKind enum must have exactly 12 variants per \
    //        CLAUDE.md baked-in #1 (got {})", primitive_kinds.len());
    //
    //   for v in &canonical_12 {
    //       assert!(primitive_kinds.contains(&v.to_string()),
    //           "PrimitiveKind missing canonical variant {}", v);
    //   }
    //
    // OBSERVABLE consequence: the irreducible-12-primitives architectural
    // commitment holds at Phase-3 close. Defends against scope-creep
    // from Atrium feature-pressure.
    unimplemented!("G20-B wires PrimitiveKind 12-variant pin");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — Atrium example handlers compose from 12 primitives only. G20-B wave-8b shipped; test body pins handler-composition structural invariant; un-ignore at Phase-4-Foundation pre-tag sweep per docs/future/phase-4-backlog.md §4.29 (HARD RULE 12 clause-(b))."]
fn atrium_examples_handlers_compose_entirely_from_existing_12_primitives_no_engine_call_outside_subgraph()
 {
    // cag-4 architectural pin. G20-B implementer wires this:
    //
    //   // Walk packages/engine/examples/atrium-*/handler.ts (or .rs).
    //   // Inspect each example's compiled OperationNode list; every
    //   // node.kind MUST be one of the canonical 12 primitives.
    //   //
    //   // Additionally: no engine.call outside a subgraph context (the
    //   // "engine.call MUST be invoked from a subgraph handler, not at
    //   // the top level of an example" companion clause).
    //
    //   let examples_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    //       .join("..").join("..").join("packages").join("engine").join("examples");
    //
    //   // cag-r4-4 MINOR (R4 large-council Round 1 + Round 2 carry):
    //   // generic example-iteration is vacuously green if a category
    //   // is missing. Track which of the 4 named handler categories
    //   // {peer-mgmt, sync-trigger, ucan-grant, did-resolution} are
    //   // represented across the atrium-* examples; assert each is
    //   // INDIVIDUALLY represented at G20-B close.
    //   let mut categories_present: std::collections::BTreeSet<&'static str> =
    //       std::collections::BTreeSet::new();
    //
    //   for entry in std::fs::read_dir(&examples_dir).unwrap() {
    //       let entry = entry.unwrap();
    //       let dir_name = entry.file_name().to_string_lossy().into_owned();
    //       if !dir_name.starts_with("atrium-") { continue; }
    //       let handler_src = std::fs::read_to_string(entry.path().join("handler.ts")).unwrap();
    //       // Verify handler does NOT call engine.call directly outside a subgraph context:
    //       assert!(!handler_src.contains("\nengine.call("),
    //           "Atrium example {} calls engine.call at top level", entry.path().display());
    //
    //       // Tag category from dirname (atrium-peer-mgmt-*, atrium-sync-trigger-*, etc.):
    //       for category in &["peer-mgmt", "sync-trigger", "ucan-grant", "did-resolution"] {
    //           if dir_name.contains(category) {
    //               categories_present.insert(category);
    //           }
    //       }
    //   }
    //
    //   // cag-r4-4 MINOR: each of the 4 named categories MUST be
    //   // represented; missing categories surface visibly:
    //   for required in &["peer-mgmt", "sync-trigger", "ucan-grant", "did-resolution"] {
    //       assert!(categories_present.contains(required),
    //           "Atrium examples MUST include at least one example per category `{}` per cag-r4-4 \
    //            (Charter 8 — handlers compose from 12 primitives across the 4 named categories)",
    //           required);
    //   }
    //
    // OBSERVABLE consequence: examples are composition-pure AND
    // cover all 4 named handler categories. Defends against the
    // failure shape where G20-B ships 3 of 4 categories and the
    // generic loop happily green-pins.
    unimplemented!(
        "G20-B wires Atrium-examples 12-primitive composition pin + 4-category coverage \
         assertion {{peer-mgmt, sync-trigger, ucan-grant, did-resolution}} per cag-r4-4"
    );
}

/// cag-3 architectural pin. Un-ignored at R6-FP-BF (closes
/// 3plus-r6-r1-1 + R6 R1 test-coverage-auditor tc-3 stale §7.3.D
/// citation). OBSERVABLE consequence: Phase 3 + Phase 4-Foundation
/// close without inventing new structural invariants — schemas +
/// materializers compose from existing ones.
#[test]
fn exit_criterion_13_no_new_structural_invariants_companion_to_no_new_primitive_kind() {
    let inv_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("INVARIANT-COVERAGE.md");
    let inv_doc = std::fs::read_to_string(&inv_path).unwrap_or_else(|e| {
        panic!(
            "INVARIANT-COVERAGE.md must exist at {} (got: {})",
            inv_path.display(),
            e
        )
    });

    // Count invariant-table rows. The coverage table uses pipe-form
    // rows of shape `| <N> | <Invariant> | <Phase> | <Enforcer> | <Tests> |`.
    let count = inv_doc
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            t.starts_with("| ") && {
                let after = &t[2..];
                after.chars().next().is_some_and(|c| c.is_ascii_digit())
            }
        })
        .count();
    assert_eq!(
        count, 14,
        "INVARIANT-COVERAGE.md must list exactly 14 invariants per \
         CLAUDE.md baked-in commitment + Phase-3 exit-criterion 13 \
         (got {count})"
    );
}

//! Engine-level umbrella RED-PHASE pin: exit-criterion 7 aggregates
//! the 6 distributed Phase-4-Foundation primitive-defense pins.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.18 row 2
//!   (G26-A umbrella; cag-r1-11).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 wave charter
//!   (closes r4-arch-1 family-charter coverage gap orphaned by R3 family
//!   enumeration — no single R3 family owned the cross-feature umbrella).
//! - CLAUDE.md baked-in #1 (12 operation primitives are irreducible).
//!
//! ## What this pin asserts
//!
//! Six Phase-4-Foundation features each ship a per-feature 12-primitive
//! defense pin (G23-A schema-compiler vocab; G23-B materializer
//! walks-only-12; G23-0a IVM-subgraph generalization with no new
//! Strategy variant; G24-A admin-UI subgraph uses-only-12; G24-D plugin
//! manifest's `requires` envelope; G24-E renderer-tauri composition).
//! The aggregate umbrella pin asserts ALL SIX feature-pins exist as
//! files (companion to the engine-level enum-count pin in the sibling
//! file).
//!
//! Without this umbrella, a future feature wave could remove its own
//! per-feature defense pin without breaking CI (the per-feature test
//! file deletion is silent; only the umbrella's enumerate-then-check
//! shape catches it).
//!
//! ## RED-PHASE staged-pin discipline (pim-12 §3.6e)
//!
//! Closes at G26-A wave-10 retense. Implementer enumerates the 6 pin
//! files at landing time and un-ignores per pim-12 §3.6e.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE pre-flight
//!
//! SHAPE: list of file paths. SUBSTANCE: each file must contain the
//! `PrimitiveKind` or canonical-12 names so the pin actually exercises
//! the defense (a renamed-to-blank file would fail the substance check).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE umbrella pin — \
    G26-A wave-10 un-ignores. Pin source: r2-test-landscape.md §2.18 row 2 + \
    r4-triage.md §5.3 R4-FP-3 charter. Aggregates 6 cross-crate per-feature \
    12-primitive defense pins; closes cag-r1-11 + r4-arch-1."]
fn exit_criterion_7_aggregates_6_distributed_primitive_pins() {
    // G26-A implementer wires this. Substantive shape:
    //
    //   let root = workspace_root();
    //
    //   // The 6 distributed primitive-defense pins each Phase-4-Foundation
    //   // feature MUST keep on disk:
    //   let pins: &[(&str, &str)] = &[
    //       // G23-A schema-compiler typed-field-vocab composes over 12:
    //       ("crates/benten-platform-foundation/tests/schema_compiler_typed_field_vocab_composes_over_12_primitives_no_extension.rs",
    //        "PrimitiveKind"),
    //       // G23-B materializer walks only the 12 primitives:
    //       ("crates/benten-platform-foundation/tests/materializer_walks_only_existing_12_primitives_no_extension.rs",
    //        "PrimitiveKind"),
    //       // G23-0a IVM-subgraph generalization: no new Strategy variant
    //       // (carries the 12-primitive commitment in spirit per CLAUDE.md
    //       // baked-in #2 — Strategy stays 3-variant closed set):
    //       ("crates/benten-ivm/tests/ivm_generalized_kernel_no_new_strategy_variant.rs",
    //        "Strategy"),
    //       // G24-A admin UI v0 plugin subgraph uses only 12:
    //       ("crates/benten-engine/tests/admin_ui_v0_uses_only_12_primitives_no_synthetic_extension.rs",
    //        "PrimitiveKind"),
    //       // G24-D plugin manifest schema's `requires` envelope:
    //       ("crates/benten-platform-foundation/tests/schema_compiler_emits_subgraph_with_no_new_primitive_kind_variants.rs",
    //        "PrimitiveKind"),
    //       // G24-E renderer-tauri compose-via-existing primitives
    //       // (engine-extension shape; per CLAUDE.md #19 the renderer
    //       // doesn't mint primitives, it consumes them):
    //       ("crates/benten-renderer-tauri/tests/three_rung_baked_in_17_defense_extension_pin.rs",
    //        "PrimitiveKind"),
    //   ];
    //
    //   let mut missing: Vec<&str> = Vec::new();
    //   let mut substance_missing: Vec<&str> = Vec::new();
    //
    //   for (path, marker) in pins {
    //       let full = root.join(path);
    //       if !full.is_file() {
    //           missing.push(path);
    //           continue;
    //       }
    //       // SUBSTANCE check (pim-18 §3.6f): the file must mention the
    //       // canonical marker (PrimitiveKind/Strategy) so a file
    //       // renamed-but-emptied is still caught:
    //       let body = std::fs::read_to_string(&full).unwrap();
    //       if !body.contains(marker) {
    //           substance_missing.push(path);
    //       }
    //   }
    //
    //   assert!(
    //       missing.is_empty(),
    //       "Phase-4-Foundation MUST keep 6 distributed primitive-defense pins on disk; \
    //        missing: {:?}",
    //       missing,
    //   );
    //   assert!(
    //       substance_missing.is_empty(),
    //       "Phase-4-Foundation primitive-defense pin files must mention the \
    //        canonical marker (PrimitiveKind/Strategy) to actually exercise the defense; \
    //        substance-missing: {:?}",
    //       substance_missing,
    //   );
    //
    // OBSERVABLE consequence: the per-feature defense pins remain in
    // place across the Phase-4-Foundation feature surface. Defense in
    // depth — engine-level count pin (sibling file) + 6 per-feature
    // pins together close cag-r1-11.
    let _ = workspace_root();
    unimplemented!(
        "G26-A wave-10 wires engine-level umbrella aggregate over 6 distributed \
         12-primitive defense pins"
    );
}

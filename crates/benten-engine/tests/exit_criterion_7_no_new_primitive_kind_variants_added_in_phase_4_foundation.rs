//! Engine-level umbrella RED-PHASE pin: exit-criterion 7 — no new
//! `PrimitiveKind` variants are minted in Phase 4-Foundation.
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.18 row 1
//!   (cross-cutting 12-primitive irreducibility regression-defense).
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 wave charter
//!   (closes r4-tc-6 + r4-tc-8 + r4-tc-9 + r4-arch-1 family-charter
//!   coverage gaps orphaned from R3 family enumeration).
//! - CLAUDE.md baked-in #1 (12 operation primitives are irreducible).
//!
//! ## What this pin asserts
//!
//! Per CLAUDE.md baked-in #1: the engine recognises exactly 12
//! `PrimitiveKind` variants — READ, WRITE, TRANSFORM, BRANCH, ITERATE,
//! WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM. Phase
//! 4-Foundation introduces: admin UI v0 (G24-A), plugin manifest schema
//! (G24-D), schema-driven rendering compiler (G23-A), materializer
//! pipeline (G23-B), IVM-subgraph generalization (G23-0a/b),
//! Tauri renderer backend (G24-E). Each of these surfaces could
//! conceivably bring with it pressure to mint a new primitive kind. This
//! pin is the engine-level umbrella regression-guard that the count
//! remains exactly 12.
//!
//! Distinct from the per-feature pins in §2.4-§2.9 (schema_compiler /
//! admin_ui_v0 / atrium / etc. each ALSO pin this from their own
//! consumer angle). The engine-level umbrella verifies the property
//! ONCE at the canonical source-of-truth (the `PrimitiveKind` enum
//! definition in `benten-core`).
//!
//! ## RED-PHASE staged-pin discipline (pim-12 §3.6e)
//!
//! At HEAD the enum should already be 12-variant (Phase 3 SHIPPED). The
//! pin is staged here so any Phase-4-Foundation wave that adds a
//! variant breaks CI before merge. The implementer un-ignores at G26-A
//! retense wave-10 (when the umbrella aggregates land alongside other
//! docs-shape pins) per pim-12 §3.6e wave-completion checklist.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE pre-flight
//!
//! SHAPE: grep the enum body in `crates/benten-core/src/` for variant
//! identifiers. SUBSTANCE: assert variant count == 12 AND each of the
//! 12 canonical names is present. Catches the failure shape where a
//! generic `count > 0` would vacuously green on a renamed-out enum.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE umbrella pin — \
    G26-A wave-10 un-ignores. Pin source: r2-test-landscape.md §2.18 row 1 + \
    r4-triage.md §5.3 R4-FP-3 charter. CLAUDE.md baked-in #1 12-primitive \
    irreducibility regression-defense at engine-level umbrella."]
fn exit_criterion_7_no_new_primitive_kind_variants_added_in_phase_4_foundation() {
    // G26-A implementer wires this. Substantive shape:
    //
    //   let core_src = workspace_root().join("crates/benten-core/src");
    //
    //   // Locate the PrimitiveKind enum definition. The canonical
    //   // source is `crates/benten-core/src/primitives.rs` (or wherever
    //   // the enum lives at retense-time).
    //   let enum_body = find_primitive_kind_enum_body(&core_src);
    //
    //   // Extract identifiers from the enum body (e.g. `Read,`, `Write,`).
    //   let variants = extract_variant_idents(&enum_body);
    //
    //   assert_eq!(
    //       variants.len(),
    //       12,
    //       "PrimitiveKind enum MUST have exactly 12 variants per \
    //        CLAUDE.md baked-in #1 (got {} after Phase-4-Foundation: {:?})",
    //       variants.len(),
    //       variants,
    //   );
    //
    //   // SUBSTANCE half: each canonical name must be present (not
    //   // just count == 12 with random names).
    //   let canonical_12 = [
    //       "Read", "Write", "Transform", "Branch", "Iterate", "Wait",
    //       "Call", "Respond", "Emit", "Sandbox", "Subscribe", "Stream",
    //   ];
    //   for v in &canonical_12 {
    //       assert!(
    //           variants.iter().any(|x| x == v),
    //           "PrimitiveKind missing canonical variant `{}` after \
    //            Phase-4-Foundation retense — exit-criterion 7 violation",
    //           v,
    //       );
    //   }
    //
    //   // Sanity-canary against vacuously-passing shape: the source-file
    //   // walk must have found a non-empty enum body.
    //   assert!(
    //       !enum_body.is_empty(),
    //       "PrimitiveKind enum body MUST be non-empty (smoke-check); \
    //        empty body means the grep walk found nothing — pin is \
    //        vacuous-truth (pim-18 §3.6f failure mode)"
    //   );
    //
    // OBSERVABLE consequence: the irreducible-12-primitives architectural
    // commitment holds at Phase-4-Foundation close. Defends against
    // scope-creep from admin-UI / plugin-manifest / materializer /
    // schema-compiler / IVM-subgraph / renderer feature pressure.
    let _ = workspace_root();
    unimplemented!("G26-A wave-10 wires engine-level umbrella PrimitiveKind 12-variant pin");
}

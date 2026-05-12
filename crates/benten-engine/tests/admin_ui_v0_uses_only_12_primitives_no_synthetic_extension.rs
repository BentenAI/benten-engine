//! Phase-4-Foundation R3 Family F1 — RED-PHASE pin for admin UI v0
//! subgraph using ONLY the 12 canonical primitives (no synthetic
//! extension, no per-plugin operation type).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 6 + §5 SHAPE-not-SUBSTANCE pairing (table row 2); closes
//! plugin-arch-r1-14 + CLAUDE.md baked-in #1 (12 primitives irreducible)
//! + Family F1 substance discipline.
//!
//! ## What this pin establishes
//!
//! Per CLAUDE.md baked-in #1: the engine recognises exactly **12
//! `PrimitiveKind` variants** — READ, WRITE, TRANSFORM, BRANCH, ITERATE,
//! WAIT, CALL, RESPOND, EMIT, SANDBOX, SUBSCRIBE, STREAM. Per baked-in
//! #18: plugins are subgraphs of these primitives, NOT separate
//! runtimes. The admin UI v0 plugin is the canary test of this
//! commitment.
//!
//! This pin protects against the failure shape where admin UI smuggles
//! in a per-plugin operation type (e.g., a synthetic `AdminUiRender`
//! variant) instead of composing from existing primitives.
//!
//! ## SHAPE+SUBSTANCE pairing per R2 §5 table row 2 (pim-18 §3.6f)
//!
//! - **SHAPE half** — cargo-graph parse / static AST walk: admin UI
//!   subgraph CIDs decode to OperationNodes whose `kind` field
//!   discriminant maps to one of the 12 `PrimitiveKind` variants.
//! - **SUBSTANCE half** — production-runtime walk: dispatch the admin
//!   UI subgraph against the evaluator + record every `PrimitiveOp`
//!   variant the evaluator dispatches over. Assert each is in the
//!   canonical 12-set.

#![allow(clippy::unwrap_used)]

mod common;

/// SHAPE half — static walk of admin UI subgraph CIDs; each OperationNode
/// kind must be a canonical PrimitiveKind variant.
#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §2.6 row 6 + §5 table row 2 SHAPE half. CLAUDE.md baked-in #1 + #18 12-primitive-irreducibility defense at admin UI canary plugin."]
fn admin_ui_v0_subgraph_static_walk_uses_only_canonical_primitive_kinds() {
    // G24-A wave wires this. Substantive shape:
    //
    //   use benten_core::PrimitiveKind;
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // Walk every OperationNode in the admin UI v0 subgraph:
    //   let nodes = harness.walk_admin_ui_subgraph_nodes();
    //   assert!(
    //       !nodes.is_empty(),
    //       "Admin UI subgraph MUST be non-empty (smoke-check)"
    //   );
    //
    //   // The canonical 12 variants per CLAUDE.md baked-in #1:
    //   let canonical = [
    //       PrimitiveKind::Read, PrimitiveKind::Write,
    //       PrimitiveKind::Transform, PrimitiveKind::Branch,
    //       PrimitiveKind::Iterate, PrimitiveKind::Wait,
    //       PrimitiveKind::Call, PrimitiveKind::Respond,
    //       PrimitiveKind::Emit, PrimitiveKind::Sandbox,
    //       PrimitiveKind::Subscribe, PrimitiveKind::Stream,
    //   ];
    //
    //   for node in &nodes {
    //       assert!(
    //           canonical.contains(&node.kind),
    //           "Admin UI OperationNode {} has PrimitiveKind {:?} \
    //            — not one of the canonical 12 per CLAUDE.md baked-in \
    //            #1. Admin UI is smuggling a synthetic primitive.",
    //           node.cid,
    //           node.kind,
    //       );
    //   }
    //
    // OBSERVABLE consequence: structural defense against synthetic
    // primitive smuggling in admin UI plugin subgraph. SHAPE half of
    // §3.6f pair.
    unimplemented!("G24-A wires admin UI subgraph SHAPE walk (12-primitive defense)");
}

/// SUBSTANCE half — production-runtime walk: dispatch admin UI subgraph
/// against evaluator + assert PrimitiveOp discriminants are in 12-set.
#[test]
#[ignore = "phase-4-foundation R3 RED-PHASE — G24-A wave-6 wires this. Pin source: r2-test-landscape.md §5 table row 2 SUBSTANCE half. Production-runtime walk pairs with static SHAPE check per pim-18 §3.6f."]
fn admin_ui_v0_evaluator_dispatches_only_canonical_primitive_op_discriminants() {
    // G24-A wave wires this. Substantive shape:
    //
    //   let harness = common::admin_ui_v0_harness::AdminUiV0TestHarness::new();
    //
    //   // Capture the evaluator's PrimitiveOp dispatch trace as the
    //   // admin UI subgraph executes (not just inspects nodes):
    //   let trace = harness.trace_capture(|h| {
    //       h.dispatch_admin_ui_full_render_sweep()
    //   });
    //
    //   // The evaluator's PrimitiveOp variants — these mirror
    //   // PrimitiveKind but on the eval side. Per pim-18 §3.6f
    //   // production-runtime check:
    //   let canonical_op_discriminants: std::collections::BTreeSet<&str> =
    //       ["Read", "Write", "Transform", "Branch", "Iterate", "Wait",
    //        "Call", "Respond", "Emit", "Sandbox", "Subscribe", "Stream"]
    //           .into_iter()
    //           .collect();
    //
    //   for dispatched in trace.primitive_op_discriminants() {
    //       assert!(
    //           canonical_op_discriminants.contains(dispatched.as_str()),
    //           "Evaluator dispatched non-canonical PrimitiveOp \
    //            discriminant `{}` during admin UI subgraph walk — \
    //            CLAUDE.md baked-in #1 violation",
    //           dispatched,
    //       );
    //   }
    //
    //   // Sanity: trace was non-empty (smoke against shape-only pass):
    //   assert!(
    //       !trace.primitive_op_discriminants().is_empty(),
    //       "Production-runtime walk MUST observe at least one \
    //        PrimitiveOp dispatch; empty trace indicates pin is \
    //        shape-only (failure mode pim-18 §3.6f defends against)"
    //   );
    //
    // OBSERVABLE consequence: production-runtime defense against
    // synthetic primitive injected via evaluator extension. Required
    // SUBSTANCE companion per R2 §5 table row 2.
    unimplemented!(
        "G24-A wires admin UI evaluator-dispatch SUBSTANCE walk \
         (PrimitiveOp discriminant check). Pairs with static SHAPE \
         walk in same file."
    );
}

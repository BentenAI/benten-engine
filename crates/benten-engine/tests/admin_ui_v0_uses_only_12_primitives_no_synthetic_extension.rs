//! Phase-4-Foundation G24-A — admin UI v0 subgraph uses ONLY the 12
//! canonical primitives (no synthetic extension, no per-plugin
//! operation type).
//!
//! Pin source: `.addl/phase-4-foundation/r2-test-landscape.md` §2.6
//! row 6 + §5 SHAPE-not-SUBSTANCE pairing; closes plugin-arch-r1-14
//! + CLAUDE.md baked-in #1 (12 primitives irreducible).
//!
//! ## SHAPE+SUBSTANCE pair (pim-18 §3.6f)
//!
//! - **SHAPE** — static walk of the admin UI v0 subgraph: every
//!   `OperationNode.kind` is one of the 12 canonical `PrimitiveKind`
//!   variants.
//! - **SUBSTANCE** — exercise the production builder + verify the
//!   walked subgraph carries the expected primitive shape
//!   (READ+TRANSFORM+RESPOND repeated per category × 4 categories).

#![allow(clippy::unwrap_used)]

use benten_core::PrimitiveKind;
use benten_platform_foundation::{NAV_CATEGORIES, build_admin_ui_v0_subgraph};

#[test]
fn admin_ui_v0_subgraph_static_walk_uses_only_canonical_primitive_kinds() {
    let sg = build_admin_ui_v0_subgraph();
    assert!(
        !sg.nodes().is_empty(),
        "admin UI subgraph MUST be non-empty"
    );
    for op in sg.nodes() {
        match op.kind {
            PrimitiveKind::Read
            | PrimitiveKind::Write
            | PrimitiveKind::Transform
            | PrimitiveKind::Branch
            | PrimitiveKind::Iterate
            | PrimitiveKind::Wait
            | PrimitiveKind::Call
            | PrimitiveKind::Respond
            | PrimitiveKind::Emit
            | PrimitiveKind::Sandbox
            | PrimitiveKind::Subscribe
            | PrimitiveKind::Stream => {}
            other => panic!(
                "admin UI v0 subgraph contains non-canonical PrimitiveKind variant `{other:?}` \
                 — violates CLAUDE.md baked-in #1 (12 primitives irreducible) + #18 \
                 (plugins are subgraphs of primitives, not new runtimes)"
            ),
        }
    }
}

#[test]
fn admin_ui_v0_subgraph_substantively_carries_read_transform_respond_per_category() {
    let sg = build_admin_ui_v0_subgraph();
    let mut counts: std::collections::HashMap<PrimitiveKind, usize> = Default::default();
    for op in sg.nodes() {
        *counts.entry(op.kind).or_insert(0) += 1;
    }
    let categories = NAV_CATEGORIES.len();
    assert_eq!(
        counts.get(&PrimitiveKind::Read).copied().unwrap_or(0),
        categories,
        "admin UI v0 subgraph MUST have one READ per category"
    );
    assert_eq!(
        counts.get(&PrimitiveKind::Transform).copied().unwrap_or(0),
        categories,
        "admin UI v0 subgraph MUST have one TRANSFORM per category"
    );
    assert_eq!(
        counts.get(&PrimitiveKind::Respond).copied().unwrap_or(0),
        categories,
        "admin UI v0 subgraph MUST have one RESPOND per category"
    );
}

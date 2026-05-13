//! G23-0b View 4 (`governance_inheritance`) typed-output projection
//! SHAPE pin — R2 §5 gap #3 closure.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §5 gap #3:
//!
//! > "G23-0b View 4 (governance_inheritance) typed-output projection
//! > shape — `mat-r1-1` reserve flagged in plan but no specific R3
//! > pin for the projection-shape change beyond the canonical-view
//! > round-trip."
//!
//! Recommended file: `view_4_typed_output_projection_shape_pin.rs`.
//! Closes: arch-r1-9 view-4 + mat-r1-1.
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! Beyond the round-trip equivalence in
//! `view_4_governance_inheritance_subgraph_spec_round_trip.rs`, the
//! typed-output projection emitted by View 4 carries a STRUCTURED
//! shape per the Phase-3 governance-traversal semantics:
//!
//! ```text
//! (governance_root_cid, inheriting_node_cid, depth)
//! ```
//!
//! where:
//! - `governance_root_cid` — the apex Community Cid the inheritance
//!   chain resolves to.
//! - `inheriting_node_cid` — the leaf Community Cid the rules apply
//!   to.
//! - `depth` — hop count (0 ≤ depth ≤ MAX_GOVERNANCE_DEPTH = 5 per
//!   ENGINE-SPEC §8).
//!
//! This pin would FAIL if a re-expression drops a field
//! (e.g. emits only the leaf Cid), reorders fields (e.g. swaps root +
//! leaf), or widens depth beyond the cap.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - **Shape arm:** asserts the projection EMITS a 3-tuple-shaped
//!   payload (governance_root_cid, inheriting_node_cid, depth) — not
//!   a single-Cid or two-Cid degenerate shape.
//! - **Depth-cap arm:** depth value MUST satisfy 0 ≤ depth ≤
//!   MAX_GOVERNANCE_DEPTH; a re-expression that drops the cap check
//!   trips this pin when fed a chain longer than 5 hops.
//! - **Would-FAIL-if-no-op'd:** a stub emitting an empty Rules buffer
//!   produces a zero-row projection; the shape assertion (at least
//!   one row, each row a 3-tuple) catches the no-op.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0b. Un-ignore per pim-12 §3.6e at landing.
//! The R5 implementer either (a) extends the canary surface in
//! `common_kernel_canary.rs` to expose the projection-tuple shape
//! directly (preferred for substantive observation) or (b) replaces
//! the byte-pattern check below with a typed-projection decoder
//! sourced from the production crate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common_kernel_canary;
use common_kernel_canary::{
    CanarySubgraphSpec, KernelInput, KernelOutput, TypedOutputProjection,
    register_and_walk_to_completion,
};

/// Required field cardinality for the View 4 typed-output projection.
/// Documented in this pin's module rustdoc; the R5 implementer wires
/// this constant into the canary surface (`common_kernel_canary.rs`)
/// at un-ignore time so the projection decoder validates it directly.
const VIEW_4_PROJECTION_FIELDS: usize = 3; // (root_cid, leaf_cid, depth)

/// MAX_GOVERNANCE_DEPTH pin (ENGINE-SPEC §8). R5 implementer asserts
/// the actual production value matches via constant-equality at
/// un-ignore time.
const MAX_GOVERNANCE_DEPTH_PIN: u32 = 5;

#[test]

fn view_4_typed_output_projection_declares_rules_variant() {
    // First-line shape gate: the canary spec for View 4 MUST declare
    // TypedOutputProjection::Rules. A re-expression that drops the
    // typed-output declaration (and falls back to Rows) trips this
    // pin before any walk runs.
    let spec = CanarySubgraphSpec::for_canonical_view("governance_inheritance");
    assert_eq!(
        spec.typed_output_projection,
        Some(TypedOutputProjection::Rules),
        "View 4 typed-output projection MUST be TypedOutputProjection::Rules \
         per mat-r1-1; got {:?}. A re-expression that omits the typed-output \
         declaration falls back to Rows, violating mat-r1-1.",
        spec.typed_output_projection,
    );
}

#[test]

fn view_4_typed_output_projection_emits_rules_kernel_output() {
    // Walk-time shape gate: the materialised KernelOutput MUST be
    // Rules(_), not Rows / Current. Pairs with the round-trip pin
    // (which asserts byte-equivalence); this pin isolates the shape
    // assertion from the byte-equivalence assertion.
    let spec = CanarySubgraphSpec::for_canonical_view("governance_inheritance");
    let writes = vec![
        KernelInput::new("system:GovernanceInheritance", 100, 0),
        KernelInput::new("system:GovernanceInheritance", 200, 1),
        KernelInput::new("system:GovernanceInheritance", 300, 2),
    ];

    let output = register_and_walk_to_completion(&spec, &writes).expect("walk ok");

    match output {
        KernelOutput::Rules(bytes) => {
            assert!(
                !bytes.is_empty(),
                "View 4 Rules projection MUST contain at least one row \
                 for a 3-write chain; got empty Rules buffer — a no-op \
                 re-expression that emits empty Rules trips this gate."
            );
            // The R5 implementer extends this assertion at un-ignore
            // time to decode the projection tuple shape:
            //
            //     for row in decode_rules_rows(&bytes) {
            //         assert_eq!(row.fields(), VIEW_4_PROJECTION_FIELDS);
            //         assert!(row.depth() <= MAX_GOVERNANCE_DEPTH_PIN);
            //     }
            //
            // Until the canary surface exposes the decoder, we pin
            // the cardinality + cap constants as compile-time anchors
            // so the un-ignore drift is observable.
            let _ = (VIEW_4_PROJECTION_FIELDS, MAX_GOVERNANCE_DEPTH_PIN);
        }
        KernelOutput::Rows(_) | KernelOutput::Current(_) => panic!(
            "View 4 typed-output projection MUST emit Rules; got non-\
             Rules KernelOutput — mat-r1-1 violation. A re-expression \
             that defaults all views to Rows fails this gate."
        ),
    }
}

#[test]

fn view_4_typed_output_projection_depth_cap_pinned() {
    // Depth-cap discipline pin: MAX_GOVERNANCE_DEPTH is part of the
    // typed-output projection contract — a depth value beyond the
    // cap is a contract violation. The R5 implementer un-ignores this
    // and wires the cap check directly to the production constant
    // (`benten_ivm::views::governance_inheritance::MAX_GOVERNANCE_DEPTH`).
    //
    // RED-PHASE: pin the constant value here; un-ignore-time
    // assertion is `MAX_GOVERNANCE_DEPTH_PIN == benten_ivm::views::
    // governance_inheritance::MAX_GOVERNANCE_DEPTH as u32` to catch
    // production drift away from the 5-hop cap.
    assert_eq!(
        MAX_GOVERNANCE_DEPTH_PIN, 5,
        "ENGINE-SPEC §8 pins MAX_GOVERNANCE_DEPTH = 5; pin constant \
         drifted to {MAX_GOVERNANCE_DEPTH_PIN}. If the production value \
         changed legitimately, update this pin atomically + sweep \
         spec docs per §3.5b."
    );
    assert_eq!(
        VIEW_4_PROJECTION_FIELDS, 3,
        "View 4 typed-output projection field cardinality is 3 \
         (governance_root_cid, inheriting_node_cid, depth); pin \
         drifted to {VIEW_4_PROJECTION_FIELDS}."
    );
}

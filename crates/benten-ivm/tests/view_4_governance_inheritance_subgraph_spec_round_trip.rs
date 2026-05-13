//! G23-0b View 4 (`governance_inheritance`) re-expressed as
//! `SubgraphSpec` — round-trip equivalence vs the hand-written
//! `GovernanceInheritanceView`.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 row 4. Closes:
//! arch-r1-9 view-4 + mat-r1-1 + D-4F-2.
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! View 4 (`governance_inheritance`) maintains the effective-rules
//! transitive closure with the Phase-1 depth cap (MAX_GOVERNANCE_DEPTH
//! = 5 hops per ENGINE-SPEC §8). Per mat-r1-1, View 4 carries a
//! typed-output projection (`TypedOutputProjection::Rules`) — distinct
//! from the Rows output of Views 1/2/3 and the Current output of
//! View 5.
//!
//! Re-expressed as a `SubgraphSpec` running through the generalized
//! Algorithm B kernel (Family B G23-0a), View 4 MUST:
//! 1. Produce KernelOutput::Rules (not Rows / Current).
//! 2. Match the hand-written `GovernanceInheritanceView` byte-for-byte
//!    for the same write sequence (including depth-cap discipline).
//!
//! Companion: `view_4_typed_output_projection_shape_pin.rs` asserts
//! the projection SHAPE (the tuple-shape of the rules snapshot).
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - Production-runtime arm: registers + walks via Family B canary.
//! - Observable consequence: KernelOutput::Rules with bytes matching
//!   hand-written baseline.
//! - Would-FAIL-if-no-op'd: if the implementer routes View 4 to the
//!   Rows path (mat-r1-1 violation), the typed-output-shape assertion
//!   trips. If the rules content drifts, the byte-equivalence trips.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0b. Un-ignore per pim-12 §3.6e at landing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common_kernel_canary;
use common_kernel_canary::{
    CanarySubgraphSpec, KernelInput, KernelOutput, TypedOutputProjection,
    assert_round_trip_equivalent_to_handwritten, register_and_walk_to_completion,
};

fn handwritten_baseline_for_writes(writes: &[KernelInput]) -> KernelOutput {
    // RED-PHASE placeholder. R5 implementer wires to a live
    // `GovernanceInheritanceView`-fed run, exercising the
    // MAX_GOVERNANCE_DEPTH=5 cap discipline.
    let mut bytes = Vec::with_capacity(writes.len() * 16);
    for w in writes {
        bytes.extend_from_slice(w.label.as_bytes());
        bytes.extend_from_slice(&w.created_at.to_le_bytes());
        bytes.extend_from_slice(&w.disambiguator.to_le_bytes());
    }
    KernelOutput::Rules(bytes)
}

#[test]

fn view_4_governance_inheritance_subgraph_spec_round_trip_matches_handwritten() {
    let spec = CanarySubgraphSpec::for_canonical_view("governance_inheritance");
    assert!(spec.is_canonical, "governance_inheritance is canonical");
    assert_eq!(
        spec.typed_output_projection,
        Some(TypedOutputProjection::Rules),
        "View 4 MUST declare TypedOutputProjection::Rules per mat-r1-1; \
         got {:?}",
        spec.typed_output_projection,
    );

    let writes = vec![
        KernelInput::new("system:GovernanceInheritance", 100, 0),
        KernelInput::new("system:GovernanceInheritance", 200, 1),
        KernelInput::new("system:GovernanceInheritance", 300, 2),
    ];

    let expected = handwritten_baseline_for_writes(&writes);
    assert_round_trip_equivalent_to_handwritten(&spec, &writes, &expected);
}

#[test]

fn view_4_subgraph_spec_emits_rules_not_rows_or_current() {
    // mat-r1-1 LOAD-BEARING: View 4 MUST route to the typed-output
    // Rules path, NOT to the default Rows path. A no-op re-expression
    // that defaults all canonical views to Rows fails this gate.
    let spec = CanarySubgraphSpec::for_canonical_view("governance_inheritance");

    let writes = vec![KernelInput::new("system:GovernanceInheritance", 1, 0)];
    let output = register_and_walk_to_completion(&spec, &writes).expect("walk ok");

    match output {
        KernelOutput::Rules(_) => { /* expected typed-output shape */ }
        KernelOutput::Rows(_) => panic!(
            "View 4 (governance_inheritance) MUST emit Rules (typed-\
             output projection per mat-r1-1); got Rows — implementer \
             defaulted View 4 to the Rows-output path, violating \
             mat-r1-1 typed-output-shape contract."
        ),
        KernelOutput::Current(_) => panic!(
            "View 4 MUST emit Rules; got Current — mis-routed through \
             View 5's typed-output path."
        ),
    }
}

#[test]

fn view_4_subgraph_spec_distinct_inputs_produce_distinct_outputs() {
    let spec = CanarySubgraphSpec::for_canonical_view("governance_inheritance");

    let empty: Vec<KernelInput> = Vec::new();
    let populated = vec![
        KernelInput::new("system:GovernanceInheritance", 100, 0),
        KernelInput::new("system:GovernanceInheritance", 200, 1),
    ];

    let empty_output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");
    let populated_output =
        register_and_walk_to_completion(&spec, &populated).expect("populated walk ok");

    assert_ne!(
        empty_output, populated_output,
        "View 4 SubgraphSpec re-expression MUST observably differ by \
         write sequence; got identical `{empty_output:?}`"
    );
}

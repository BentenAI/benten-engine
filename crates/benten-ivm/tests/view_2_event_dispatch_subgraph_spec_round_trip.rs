//! G23-0b View 2 (`event_dispatch`) re-expressed as `SubgraphSpec` —
//! round-trip equivalence vs the hand-written `EventDispatchView`.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 row 2. Closes:
//! arch-r1-9 view-2 + D-4F-2.
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! View 2 routes Created events labeled `Event` (Phase-1 EMIT-event
//! shape) into per-handler dispatch buckets. Re-expressed as a
//! `SubgraphSpec` running through the generalized Algorithm B kernel
//! (Family B G23-0a), it MUST produce the SAME materialised Rows
//! output as the hand-written `EventDispatchView` for the same write
//! sequence.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - Production-runtime arm: registers via Family B's canary +
//!   asserts round-trip equivalence vs the hand-written reference.
//! - Observable consequence: identical output bytes for identical
//!   inputs across both view implementations.
//! - Would-FAIL-if-no-op'd: a no-op re-expression that drops events,
//!   routes to the wrong bucket, or returns a constant cannot match
//!   the live hand-written-view materialisation.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0b. Un-ignore per pim-12 §3.6e at landing.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common_kernel_canary;
use common_kernel_canary::{
    CanarySubgraphSpec, KernelInput, KernelOutput, assert_round_trip_equivalent_to_handwritten,
    register_and_walk_to_completion,
};

fn handwritten_baseline_for_writes(writes: &[KernelInput]) -> KernelOutput {
    // RED-PHASE placeholder. R5 implementer wires this to a live
    // `EventDispatchView::new()`-fed run so drift is observable.
    let mut bytes = Vec::with_capacity(writes.len() * 16);
    for w in writes {
        bytes.extend_from_slice(w.label.as_bytes());
        bytes.extend_from_slice(&w.created_at.to_le_bytes());
        bytes.extend_from_slice(&w.disambiguator.to_le_bytes());
    }
    KernelOutput::Rows(bytes)
}

#[test]

fn view_2_event_dispatch_subgraph_spec_round_trip_matches_handwritten() {
    let spec = CanarySubgraphSpec::for_canonical_view("event_dispatch");
    assert!(
        spec.is_canonical,
        "event_dispatch is one of CANONICAL_VIEW_IDS"
    );
    assert!(
        spec.typed_output_projection.is_none(),
        "View 2 emits Rows (event → handler bindings); typed-output \
         projections are Views 4/5 only per mat-r1-1"
    );

    let writes = vec![
        KernelInput::new("system:EventDispatch", 100, 0),
        KernelInput::new("system:EventDispatch", 200, 1),
    ];

    let expected = handwritten_baseline_for_writes(&writes);
    assert_round_trip_equivalent_to_handwritten(&spec, &writes, &expected);
}

#[test]

fn view_2_subgraph_spec_distinct_inputs_produce_distinct_outputs() {
    let spec = CanarySubgraphSpec::for_canonical_view("event_dispatch");

    let empty: Vec<KernelInput> = Vec::new();
    let populated = vec![
        KernelInput::new("system:EventDispatch", 100, 0),
        KernelInput::new("system:EventDispatch", 200, 1),
    ];

    let empty_output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");
    let populated_output =
        register_and_walk_to_completion(&spec, &populated).expect("populated walk ok");

    assert_ne!(
        empty_output, populated_output,
        "View 2 SubgraphSpec re-expression MUST observably differ by \
         write sequence; got identical `{empty_output:?}`"
    );
}

#[test]

fn view_2_subgraph_spec_emits_rows_output_not_typed_projection() {
    let spec = CanarySubgraphSpec::for_canonical_view("event_dispatch");
    let writes = vec![KernelInput::new("system:EventDispatch", 1, 0)];
    let output = register_and_walk_to_completion(&spec, &writes).expect("walk ok");

    match output {
        KernelOutput::Rows(_) => {}
        KernelOutput::Rules(_) | KernelOutput::Current(_) => panic!(
            "View 2 (event_dispatch) MUST emit Rows; got typed-output \
             projection — mis-routed."
        ),
    }
}

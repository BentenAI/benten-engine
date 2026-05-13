//! G23-0b View 1 (`capability_grants`) re-expressed as `SubgraphSpec` —
//! round-trip equivalence vs the hand-written `CapabilityGrantsView`.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 row 1
//! (`canonical_view_capability_grants_round_trip_post_generalization.rs`
//! intent; file renamed per R3 Family C brief to
//! `view_1_capability_grants_subgraph_spec_round_trip.rs`).
//! Closes: arch-r1-9 view-1 + D-4F-2 (materializer view IS IVM view).
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! Per D-4F-2 ratification: the 5 canonical IVM views and any user-
//! registered views use the SAME generalized Algorithm B kernel
//! (Family B G23-0a). View 1 (`capability_grants`) re-expressed as a
//! `SubgraphSpec` MUST produce byte-identical materialised output to
//! the pre-generalization hand-written `CapabilityGrantsView` for the
//! same write sequence.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - **Production-runtime arm:** registers the canonical view via
//!   `register_and_walk_to_completion` (Family B canary surface) and
//!   asserts round-trip equivalence vs the hand-written baseline.
//! - **Observable consequence:** same write sequence in → same
//!   materialised Rows output as `CapabilityGrantsView`.
//! - **Would-FAIL-if-no-op'd:** if the subgraph-spec re-expression of
//!   View 1 returns empty/wrong output (constant return, dropped
//!   labels, mis-routed projection), the equivalence assertion against
//!   the hand-written reference fails — the no-op cannot pretend to
//!   match the live `CapabilityGrantsView` materialisation.
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

/// Build the expected hand-written-baseline output for View 1
/// (`capability_grants`) given the write sequence. The R5 G23-0b
/// implementer wires this helper's right-hand side to the actual
/// `CapabilityGrantsView::new()`-fed equivalent materialisation so
/// drift between the hand-written view + the SubgraphSpec re-expression
/// is observable.
///
/// **RED-PHASE:** returns a placeholder byte sequence the round-trip
/// helper compares against. At un-ignore time the R5 implementer
/// either (a) replaces this with a live hand-written-view runner
/// (preferred — exercises both sides) or (b) hard-codes the canonical
/// expected bytes derived from a one-time hand-written-view run.
fn handwritten_baseline_for_writes(writes: &[KernelInput]) -> KernelOutput {
    // Placeholder until R5 G23-0b lands. The shape MUST be Rows for
    // View 1 (capability_grants emits a Cids set per ViewResult::Cids;
    // canary maps to KernelOutput::Rows). Bytes are write-sequence-
    // dependent so distinct inputs produce distinct expected outputs.
    let mut bytes = Vec::with_capacity(writes.len() * 16);
    for w in writes {
        bytes.extend_from_slice(w.label.as_bytes());
        bytes.extend_from_slice(&w.created_at.to_le_bytes());
        bytes.extend_from_slice(&w.disambiguator.to_le_bytes());
    }
    KernelOutput::Rows(bytes)
}

#[test]

fn view_1_capability_grants_subgraph_spec_round_trip_matches_handwritten() {
    // Substantive production-runtime arm: register the canonical view 1
    // as a SubgraphSpec via Family B's kernel + assert byte-equivalent
    // to the hand-written CapabilityGrantsView output for the same
    // write sequence.
    let spec = CanarySubgraphSpec::for_canonical_view("capability_grants");
    assert!(
        spec.is_canonical,
        "capability_grants is one of CANONICAL_VIEW_IDS; spec must \
         classify as canonical (Strategy::A fast-path)"
    );
    assert!(
        spec.typed_output_projection.is_none(),
        "View 1 emits Rows (Cids set per ViewResult::Cids), NOT a \
         typed-output projection (those are Views 4/5 only per mat-r1-1)"
    );

    let writes = vec![
        KernelInput::new("system:CapabilityGrant", 100, 0),
        KernelInput::new("system:CapabilityGrant", 200, 1),
        KernelInput::new("system:CapabilityGrant", 300, 2),
    ];

    let expected = handwritten_baseline_for_writes(&writes);
    assert_round_trip_equivalent_to_handwritten(&spec, &writes, &expected);
}

#[test]

fn view_1_subgraph_spec_distinct_inputs_produce_distinct_outputs() {
    // §3.6b would-FAIL-if-no-op'd: feeding empty vs populated write
    // sequences through the SAME canonical-view spec MUST produce
    // observably-distinct KernelOutput. A no-op re-expression
    // (constant Rows(vec![])) fails this.
    let spec = CanarySubgraphSpec::for_canonical_view("capability_grants");

    let empty: Vec<KernelInput> = Vec::new();
    let populated = vec![
        KernelInput::new("system:CapabilityGrant", 100, 0),
        KernelInput::new("system:CapabilityGrant", 200, 1),
    ];

    let empty_output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");
    let populated_output =
        register_and_walk_to_completion(&spec, &populated).expect("populated walk ok");

    assert_ne!(
        empty_output, populated_output,
        "View 1 SubgraphSpec re-expression MUST observably differ by \
         write sequence — empty vs populated produced identical output \
         `{empty_output:?}`; no-op would alias both."
    );
}

#[test]

fn view_1_subgraph_spec_emits_rows_output_not_typed_projection() {
    // View 1 is a Rows-producing view (set of grant Cids). The
    // generalized kernel must route View 1 to Rows-output, not to
    // Rules/Current typed-output paths (those are Views 4/5 per
    // mat-r1-1).
    let spec = CanarySubgraphSpec::for_canonical_view("capability_grants");

    let writes = vec![KernelInput::new("system:CapabilityGrant", 1, 0)];
    let output = register_and_walk_to_completion(&spec, &writes).expect("walk ok");

    match output {
        KernelOutput::Rows(_) => { /* expected shape */ }
        KernelOutput::Rules(_) | KernelOutput::Current(_) => panic!(
            "View 1 (capability_grants) MUST emit Rows; got typed-output \
             projection — implementer mis-routed View 1 through the \
             Views 4/5 typed-output paths."
        ),
    }
}

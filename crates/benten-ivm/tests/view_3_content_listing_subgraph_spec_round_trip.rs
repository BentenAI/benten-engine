//! G23-0b View 3 (`content_listing`) re-expressed as `SubgraphSpec` —
//! round-trip equivalence vs the hand-written `ContentListingView`.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 row 3. Closes:
//! arch-r1-9 view-3 + D-4F-2.
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! View 3 maintains a per-label sorted Cids set (e.g. all `Post`
//! nodes in creation order). Re-expressed as a `SubgraphSpec` running
//! through the generalized Algorithm B kernel (Family B G23-0a), it
//! MUST produce the SAME materialised Rows output as the hand-written
//! `ContentListingView` for the same write sequence — in particular,
//! the secondary-sort order (created_at, then disambiguator) must be
//! preserved exactly.
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - Production-runtime arm: registers via Family B's canary +
//!   asserts round-trip equivalence vs the hand-written reference.
//! - Observable consequence: identical sorted-bytes output for
//!   identical inputs.
//! - Would-FAIL-if-no-op'd: an empty/wrong-order return cannot match
//!   the hand-written view's sort discipline.
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
    // RED-PHASE placeholder. R5 implementer wires to a live
    // `ContentListingView::new("post")`-fed run so drift is observable
    // (in particular: the sort discipline (created_at, disambiguator)).
    let mut sorted_inputs: Vec<&KernelInput> = writes.iter().collect();
    sorted_inputs.sort_by_key(|w| (w.created_at, w.disambiguator));
    let mut bytes = Vec::with_capacity(sorted_inputs.len() * 16);
    for w in sorted_inputs {
        bytes.extend_from_slice(w.label.as_bytes());
        bytes.extend_from_slice(&w.created_at.to_le_bytes());
        bytes.extend_from_slice(&w.disambiguator.to_le_bytes());
    }
    KernelOutput::Rows(bytes)
}

#[test]

fn view_3_content_listing_subgraph_spec_round_trip_matches_handwritten() {
    let spec = CanarySubgraphSpec::for_canonical_view("content_listing");
    assert!(spec.is_canonical, "content_listing is canonical");
    assert!(
        spec.typed_output_projection.is_none(),
        "View 3 emits Rows; not a typed-output projection"
    );

    let writes = vec![
        KernelInput::new("post", 100, 0),
        KernelInput::new("post", 200, 1),
        KernelInput::new("post", 150, 2),
    ];

    let expected = handwritten_baseline_for_writes(&writes);
    assert_round_trip_equivalent_to_handwritten(&spec, &writes, &expected);
}

#[test]

fn view_3_subgraph_spec_distinct_inputs_produce_distinct_outputs() {
    let spec = CanarySubgraphSpec::for_canonical_view("content_listing");

    let empty: Vec<KernelInput> = Vec::new();
    let populated = vec![
        KernelInput::new("post", 100, 0),
        KernelInput::new("post", 200, 1),
    ];

    let empty_output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");
    let populated_output =
        register_and_walk_to_completion(&spec, &populated).expect("populated walk ok");

    assert_ne!(
        empty_output, populated_output,
        "View 3 SubgraphSpec re-expression MUST observably differ by \
         write sequence; got `{empty_output:?}`"
    );
}

#[test]

fn view_3_subgraph_spec_sort_discipline_preserved() {
    // Insert OUT-OF-ORDER by created_at; round-trip must still produce
    // the SAME bytes as the hand-written view, which sorts on
    // (created_at, disambiguator) before materialisation. If the
    // re-expression drops the sort step, the bytes diverge from the
    // baseline.
    let spec = CanarySubgraphSpec::for_canonical_view("content_listing");

    let out_of_order = vec![
        KernelInput::new("post", 300, 0),
        KernelInput::new("post", 100, 1),
        KernelInput::new("post", 200, 2),
    ];
    let expected = handwritten_baseline_for_writes(&out_of_order);
    assert_round_trip_equivalent_to_handwritten(&spec, &out_of_order, &expected);
}

//! G23-0b View 5 (`version_current`) re-expressed as `SubgraphSpec` —
//! round-trip equivalence vs the hand-written `VersionCurrentView`.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 row 5. Closes:
//! arch-r1-9 view-5 + mat-r1-1 + D-4F-2.
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! View 5 (`version_current`) maintains anchor → current-version Cid
//! map (Phase-1 anchor + Version Node pattern; CLAUDE.md baked-in #8).
//! Per mat-r1-1, View 5 carries a typed-output projection
//! (`TypedOutputProjection::Current`) — emitting `Option<Cid>` rather
//! than a Cids set.
//!
//! Re-expressed as a `SubgraphSpec` running through the generalized
//! Algorithm B kernel (Family B G23-0a), View 5 MUST:
//! 1. Produce `KernelOutput::Current(Option<Vec<u8>>)` (not Rows /
//!    Rules).
//! 2. Match the hand-written `VersionCurrentView` byte-for-byte for
//!    the same write sequence (including the `None`-when-no-pointer
//!    case).
//!
//! Companion: `view_5_typed_output_projection_shape_pin.rs` asserts
//! the projection SHAPE (the `(anchor_cid, current_version_cid,
//! current_version_index)` tuple).
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - Production-runtime arm: registers + walks via Family B canary.
//! - Observable consequence: KernelOutput::Current with bytes
//!   matching hand-written baseline.
//! - Would-FAIL-if-no-op'd: implementer routing View 5 to Rows
//!   (mat-r1-1 violation) trips the typed-output-shape assertion;
//!   pointer drift trips the byte-equivalence assertion.
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
    // `VersionCurrentView`-fed run; `None` when no NEXT_VERSION events
    // have been observed.
    if writes.is_empty() {
        return KernelOutput::Current(None);
    }
    let mut bytes = Vec::with_capacity(writes.len() * 16);
    for w in writes {
        bytes.extend_from_slice(w.label.as_bytes());
        bytes.extend_from_slice(&w.created_at.to_le_bytes());
        bytes.extend_from_slice(&w.disambiguator.to_le_bytes());
    }
    KernelOutput::Current(Some(bytes))
}

#[test]

fn view_5_version_current_subgraph_spec_round_trip_matches_handwritten() {
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");
    assert!(spec.is_canonical, "version_current is canonical");
    assert_eq!(
        spec.typed_output_projection,
        Some(TypedOutputProjection::Current),
        "View 5 MUST declare TypedOutputProjection::Current per \
         mat-r1-1; got {:?}",
        spec.typed_output_projection,
    );

    let writes = vec![
        KernelInput::new("NEXT_VERSION", 100, 0),
        KernelInput::new("NEXT_VERSION", 200, 1),
    ];

    let expected = handwritten_baseline_for_writes(&writes);
    assert_round_trip_equivalent_to_handwritten(&spec, &writes, &expected);
}

#[test]

fn view_5_subgraph_spec_emits_current_not_rows_or_rules() {
    // mat-r1-1 LOAD-BEARING: View 5 MUST route to the typed-output
    // Current path, NOT to the default Rows path or to View 4's
    // Rules path. A no-op re-expression that defaults all canonical
    // views to Rows fails this gate.
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");

    let writes = vec![KernelInput::new("NEXT_VERSION", 1, 0)];
    let output = register_and_walk_to_completion(&spec, &writes).expect("walk ok");

    match output {
        KernelOutput::Current(_) => { /* expected typed-output shape */ }
        KernelOutput::Rows(_) => panic!(
            "View 5 (version_current) MUST emit Current (typed-output \
             projection per mat-r1-1); got Rows — implementer defaulted \
             View 5 to the Rows-output path, violating mat-r1-1."
        ),
        KernelOutput::Rules(_) => panic!(
            "View 5 MUST emit Current; got Rules — mis-routed through \
             View 4's typed-output path."
        ),
    }
}

#[test]

fn view_5_subgraph_spec_empty_walk_emits_current_none() {
    // View 5 distinct shape arm: zero NEXT_VERSION events MUST
    // produce Current(None), not Current(Some(empty_bytes)). The
    // hand-written baseline returns None when no anchor → current
    // mapping exists; the SubgraphSpec re-expression must match.
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");
    let empty: Vec<KernelInput> = Vec::new();
    let output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");

    match output {
        KernelOutput::Current(None) => { /* expected */ }
        KernelOutput::Current(Some(bytes)) => panic!(
            "View 5 empty-walk MUST emit Current(None) (no CURRENT \
             pointer); got Current(Some({} bytes)) — implementer \
             initialised storage with empty-pointer bytes instead of \
             distinguishing absence.",
            bytes.len()
        ),
        other => panic!("View 5 MUST emit Current; got `{other:?}` — mis-routed."),
    }
}

#[test]

fn view_5_subgraph_spec_distinct_inputs_produce_distinct_outputs() {
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");

    let empty: Vec<KernelInput> = Vec::new();
    let populated = vec![
        KernelInput::new("NEXT_VERSION", 100, 0),
        KernelInput::new("NEXT_VERSION", 200, 1),
    ];

    let empty_output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");
    let populated_output =
        register_and_walk_to_completion(&spec, &populated).expect("populated walk ok");

    assert_ne!(
        empty_output, populated_output,
        "View 5 SubgraphSpec re-expression MUST observably differ by \
         write sequence — empty (Current(None)) vs populated \
         (Current(Some(_))) collapsed to `{empty_output:?}`."
    );
}

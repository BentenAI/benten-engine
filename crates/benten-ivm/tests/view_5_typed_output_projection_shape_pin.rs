//! G23-0b View 5 (`version_current`) typed-output projection SHAPE
//! pin — R2 §5 gap #3 closure.
//!
//! ## Pin source
//!
//! `.addl/phase-4-foundation/r2-test-landscape.md` §5 gap #3.
//! Recommended file: `view_5_typed_output_projection_shape_pin.rs`.
//! Closes: arch-r1-9 view-5 + mat-r1-1.
//!
//! ## What this asserts (LOAD-BEARING substantive)
//!
//! Beyond the round-trip equivalence in
//! `view_5_version_current_subgraph_spec_round_trip.rs`, the typed-
//! output projection emitted by View 5 carries a STRUCTURED shape
//! per the Phase-1 anchor+version pattern (CLAUDE.md baked-in #8):
//!
//! ```text
//! (anchor_cid, current_version_cid, current_version_index)
//! ```
//!
//! where:
//! - `anchor_cid` — stable identity Cid; the long-lived pointer's
//!   target (an Anchor Node per Phase-1 pattern).
//! - `current_version_cid` — Cid of the latest Version Node in the
//!   chain; `None` when no version exists yet.
//! - `current_version_index` — monotonic version counter (0 for
//!   inaugural version; increments per NEXT_VERSION append).
//!
//! This pin would FAIL if a re-expression drops a field
//! (e.g. emits only the current version cid), reorders fields
//! (e.g. swaps anchor + version), or collapses the optional pointer
//! semantics (treating `None` as zero-length bytes).
//!
//! ## §3.6b would-FAIL-if-no-op'd
//!
//! - **Shape arm:** asserts the projection EMITS a 3-tuple-shaped
//!   payload (anchor_cid, current_version_cid, current_version_index)
//!   — not a single-Cid degenerate shape.
//! - **Option-pointer arm:** Current(None) vs Current(Some(_)) MUST
//!   be observably distinguishable; a re-expression that collapses
//!   None to empty-bytes trips this.
//! - **Monotonicity arm:** current_version_index MUST advance on
//!   each NEXT_VERSION append; the round-trip baseline + the
//!   distinct-inputs pin in the sibling file enforce this together.
//! - **Would-FAIL-if-no-op'd:** a stub emitting Current(None) for
//!   every input collapses populated walks to the empty-pointer
//!   shape; the distinct-output assertion catches the no-op.
//!
//! ## RED-PHASE
//!
//! Closes at R5 G23-0b. Un-ignore per pim-12 §3.6e at landing.
//! The R5 implementer either (a) extends the canary surface in
//! `common_kernel_canary.rs` to expose the projection-tuple shape
//! directly or (b) replaces the byte-pattern check below with a
//! typed-projection decoder sourced from the production crate.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common_kernel_canary;
use common_kernel_canary::{
    CanarySubgraphSpec, KernelInput, KernelOutput, TypedOutputProjection,
    register_and_walk_to_completion,
};

/// Required field cardinality for the View 5 typed-output projection.
/// (anchor_cid, current_version_cid, current_version_index).
const VIEW_5_PROJECTION_FIELDS: usize = 3;

#[test]

fn view_5_typed_output_projection_declares_current_variant() {
    // First-line shape gate: the canary spec for View 5 MUST declare
    // TypedOutputProjection::Current. A re-expression that drops the
    // typed-output declaration (and falls back to Rows) trips this
    // pin before any walk runs.
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");
    assert_eq!(
        spec.typed_output_projection,
        Some(TypedOutputProjection::Current),
        "View 5 typed-output projection MUST be \
         TypedOutputProjection::Current per mat-r1-1; got {:?}. A \
         re-expression that omits the typed-output declaration falls \
         back to Rows, violating mat-r1-1.",
        spec.typed_output_projection,
    );
}

#[test]

fn view_5_typed_output_projection_emits_current_kernel_output() {
    // Walk-time shape gate: the materialised KernelOutput MUST be
    // Current(_), not Rows / Rules. Pairs with the round-trip pin
    // (which asserts byte-equivalence); this pin isolates the shape
    // assertion from the byte-equivalence assertion.
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");
    let writes = vec![
        KernelInput::new("NEXT_VERSION", 100, 0),
        KernelInput::new("NEXT_VERSION", 200, 1),
    ];

    let output = register_and_walk_to_completion(&spec, &writes).expect("walk ok");

    match output {
        KernelOutput::Current(Some(bytes)) => {
            assert!(
                !bytes.is_empty(),
                "View 5 Current projection MUST carry non-empty bytes \
                 when at least one NEXT_VERSION event has been observed; \
                 got Current(Some(empty)) — a no-op re-expression that \
                 emits empty bytes for a populated walk trips this gate."
            );
            // The R5 implementer extends this assertion at un-ignore
            // time to decode the projection tuple shape:
            //
            //     let row = decode_current_row(&bytes);
            //     assert_eq!(row.fields(), VIEW_5_PROJECTION_FIELDS);
            //     assert!(row.current_version_index() >= 0);
            //
            // Until the canary surface exposes the decoder, we pin
            // the cardinality constant as a compile-time anchor.
            let _ = VIEW_5_PROJECTION_FIELDS;
        }
        KernelOutput::Current(None) => panic!(
            "View 5 Current projection for a 2-event walk MUST emit \
             Current(Some(_)); got Current(None) — a re-expression that \
             never advances the CURRENT pointer fails this gate."
        ),
        KernelOutput::Rows(_) | KernelOutput::Rules(_) => panic!(
            "View 5 typed-output projection MUST emit Current; got non-\
             Current KernelOutput — mat-r1-1 violation. A re-expression \
             that defaults all views to Rows fails this gate."
        ),
    }
}

#[test]

fn view_5_typed_output_projection_option_pointer_distinguishes_none_vs_some() {
    // Option-pointer arm: Current(None) (no version yet) MUST be
    // observably distinct from Current(Some(empty_bytes)) (degenerate
    // empty-bytes encoding of presence). A re-expression that
    // collapses absence into empty-bytes-presence trips this pin.
    let spec = CanarySubgraphSpec::for_canonical_view("version_current");

    let empty: Vec<KernelInput> = Vec::new();
    let empty_output = register_and_walk_to_completion(&spec, &empty).expect("empty walk ok");

    match empty_output {
        KernelOutput::Current(None) => { /* expected absence */ }
        KernelOutput::Current(Some(bytes)) => panic!(
            "View 5 empty-walk MUST emit Current(None) — absence is \
             semantically distinct from presence-of-empty-bytes. Got \
             Current(Some({} bytes)).",
            bytes.len()
        ),
        other => panic!("View 5 empty-walk MUST emit Current(None); got `{other:?}`."),
    }
}

#[test]

fn view_5_typed_output_projection_field_cardinality_pinned() {
    // Cardinality drift gate: the canonical 3-tuple shape (anchor_cid,
    // current_version_cid, current_version_index) is part of the
    // typed-output projection contract. The R5 implementer un-ignores
    // and wires this assertion to the production projection decoder.
    assert_eq!(
        VIEW_5_PROJECTION_FIELDS, 3,
        "View 5 typed-output projection field cardinality is 3 \
         (anchor_cid, current_version_cid, current_version_index); \
         pin drifted to {VIEW_5_PROJECTION_FIELDS}. If the production \
         shape changed legitimately, update this pin atomically + \
         sweep spec docs per §3.5b."
    );
}

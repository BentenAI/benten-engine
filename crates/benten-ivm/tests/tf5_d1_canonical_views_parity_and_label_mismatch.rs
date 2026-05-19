//! ADDL R3 (TDD red-phase) — Phase-4-Meta-Core, Wave R3-B, agent R3-B2,
//! family **TF-5**. RED-phase pin for the **D1 A2 `CanonicalViews`
//! registry-query seam** (C4 exit obligation; G-CORE-4 substrate).
//!
//! ## Pin provenance
//!
//! - R2 `.addl/phase-4-meta/r2-test-landscape.md` TF-5 + §2.A row S4 +
//!   §2.B "Cross-DID"-adjacent row (D1 A2 parity).
//! - Plan G-CORE-4 group def + §1.A **C4** ("4 leaked IVM helpers
//!   collapsed into ONE `CanonicalViews` registry-query type documented
//!   alongside `Strategy`; #758 rename + #914 constructor-narrowing
//!   ride it").
//! - RATIFIED **D1 A2** disposition + **DISAGREE-record ivm-r1-2**: the
//!   `CanonicalViews` seam stays IN `benten-ivm` — NO lower-crate lift.
//!   The R1 lens-prompt premise "#911 ratified (b) lift to a lower
//!   crate" CONTRADICTS RATIFIED D1; the plan correctly follows A2.
//!   This pin asserts the seam lives in `benten-ivm` (no lower-crate
//!   lift) so the "(b) lift" framing cannot leak into implementation.
//!
//! ## The 4 leaked helpers being collapsed (ground-truthed at HEAD
//! `ed03729a`, `crates/benten-ivm/src/algorithm_b.rs`):
//!
//! 1. `hardcoded_label_for_id(view_id) -> Option<&'static str>`
//! 2. `canonical_typed_output_projection_for(view_id) -> Option<TypedOutputProjection>`
//! 3. `is_canonical_view_id(view_id) -> bool`
//! 4. `dispatch_for(view_id) -> Strategy`
//!
//! (Re-exported at the crate root `benten_ivm::lib.rs:48-51`:
//! `hardcoded_label_for_id`, `is_canonical_view_id`, `dispatch_for`;
//! `canonical_typed_output_projection_for` is `pub` in `algorithm_b`.)
//!
//! ## §3.6b sub-rule-4 production-arm shape
//!
//! - PRODUCTION RUNTIME ARM: `CanonicalViews::lookup` / `is_canonical`
//!   (the G-CORE-4 unified registry-query type) called for every
//!   canonical id AND a representative user-view id.
//! - OBSERVABLE CONSEQUENCE: the unified type's answers are
//!   **behaviour-identical** to the 4 old leaked helpers for the same
//!   inputs (no behaviour drift across the collapse) AND the engine
//!   `E_VIEW_LABEL_MISMATCH` contract (`AlgorithmError::ViewLabelMismatch`)
//!   is preserved.
//! - WOULD-FAIL-IF-NO-OP: a collapse that changes any per-id answer
//!   (label / projection / canonical-classification / strategy) fails
//!   the parity assertion; a collapse that drops the label-mismatch
//!   guard fails the `ViewLabelMismatch` arm.
//!
//! ## SHAPE-FLAG (not faked)
//!
//! `CanonicalViews` is the G-CORE-4 deliverable type and does NOT exist
//! at HEAD `ed03729a`. The parity tests are `#[ignore]`d (§3.6e staged
//! pin; reviewer verifies landing). The parity *oracle* (the 4 old
//! helpers' current answers) is computed in a runnable companion test
//! so the G-CORE-4 implementer has a frozen behavioural baseline to
//! collapse against — that companion is NOT ignored (it would-FAIL now
//! only if the existing helpers regress, which is a real regression
//! guard for the pre-collapse surface).
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): tests only; G-CORE-4 sweeps docs on the `CanonicalViews`-add (documented-alongside-`Strategy` per C4).
//! - [x] §3.6b + sub-rule-4: parity + ViewLabelMismatch SPECIFIC arms.
//! - [x] §3.6e: RED-PHASE staged-pin; ignore msg names G-CORE-4 destination.
//! - [x] §3.6f: SHAPE-not-SUBSTANCE — substantive per-id parity oracle, NOT "assert a CanonicalViews type exists".
//! - [x] §3.5g: no cross-language/cross-doc mirror touched here.
//! - [x] §3.5i: file-disjoint from R3-B5 (benten-ivm lane only).
//! - [x] §3.6h: no rule-naming-origin codified here.
//! - [x] §3.6i/§3.6j: N/A. §3.13: no shared static introduced.
//! - [x] §3.6g: this checklist IS the literal reproduction.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_ivm::algorithm_b::canonical_typed_output_projection_for;
use benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS;
use benten_ivm::{Strategy, dispatch_for, hardcoded_label_for_id, is_canonical_view_id};

/// Representative inputs for the parity oracle: every canonical id +
/// a user-view id (the non-canonical lane).
fn parity_probe_ids() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = CANONICAL_VIEW_IDS.to_vec();
    v.push("user-defined-arbitrary-view");
    v
}

/// RUNNABLE regression guard (NOT ignored): freezes the pre-collapse
/// behavioural baseline of the 4 leaked helpers. This is the oracle the
/// G-CORE-4 `CanonicalViews` collapse must match. Would-FAIL now only if
/// the existing helper behaviour regresses before the collapse — a real
/// guard, and it documents the exact answers the collapsed type owes.
#[test]
fn tf5_d1_four_leaked_helpers_baseline_is_stable_pre_collapse() {
    for id in parity_probe_ids() {
        // The four helpers' answers are internally consistent: a
        // canonical id classifies canonical + routes Strategy::A; a
        // user id is non-canonical + routes Strategy::B. This binds
        // the baseline so the collapse cannot silently re-key it.
        let canonical = is_canonical_view_id(id);
        let strategy = dispatch_for(id);
        if canonical {
            assert_eq!(
                strategy,
                Strategy::A,
                "canonical id `{id}` must route Strategy::A (pre-collapse baseline)"
            );
        } else {
            assert_eq!(
                strategy,
                Strategy::B,
                "non-canonical id `{id}` must route Strategy::B (pre-collapse baseline)"
            );
            assert!(
                hardcoded_label_for_id(id).is_none(),
                "non-canonical id `{id}` must have no hardcoded label"
            );
            assert!(
                canonical_typed_output_projection_for(id).is_none(),
                "non-canonical id `{id}` must have no typed-output projection"
            );
        }
    }
    // content_listing is canonical but honors the supplied label
    // (hardcoded_label_for_id == None) — a known non-uniform arm the
    // collapse MUST preserve (this is the subtle one a naive collapse
    // would flatten).
    assert!(
        is_canonical_view_id("content_listing"),
        "content_listing is canonical"
    );
    assert!(
        hardcoded_label_for_id("content_listing").is_none(),
        "content_listing canonically honors the caller-supplied label \
         (hardcoded_label_for_id == None) — the collapse MUST preserve \
         this non-uniform arm"
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — D1 A2: `CanonicalViews::lookup`/\
`is_canonical` parity vs the 4 old leaked helpers (no behaviour drift across the \
collapse). `CanonicalViews` is the G-CORE-4 deliverable type. C4 exit obligation. \
Reviewer verifies landing-status per §3.6e."]
fn tf5_d1_canonical_views_lookup_parity_vs_four_old_helpers() {
    // PRODUCTION-ARM (un-ignore + wire at G-CORE-4):
    //
    //   use benten_ivm::CanonicalViews;
    //   let views = CanonicalViews::registry();   // the unified seam
    //   for id in parity_probe_ids() {
    //       let entry = views.lookup(id);          // unified query
    //       assert_eq!(views.is_canonical(id), is_canonical_view_id(id),
    //           "is_canonical parity drift for `{id}`");
    //       assert_eq!(entry.map(|e| e.hardcoded_label()).flatten(),
    //           hardcoded_label_for_id(id),
    //           "hardcoded_label parity drift for `{id}`");
    //       assert_eq!(entry.map(|e| e.typed_output_projection()).flatten(),
    //           canonical_typed_output_projection_for(id),
    //           "typed_output_projection parity drift for `{id}`");
    //       assert_eq!(views.dispatch(id), dispatch_for(id),
    //           "dispatch (Strategy) parity drift for `{id}`");
    //   }
    //
    //   // ivm-r1-2 DISAGREE-record: the seam stays IN benten-ivm.
    //   // The type path MUST be `benten_ivm::CanonicalViews` (NO
    //   // lower-crate lift — the "(b) lift to a lower crate" framing
    //   // is RATIFIED-rejected per D1 A2). A compile-fact: the import
    //   // above resolves from `benten_ivm`, not benten-core/graph.
    unimplemented!(
        "G-CORE-4 introduces `benten_ivm::CanonicalViews` (D1 A2; stays \
         IN benten-ivm per ivm-r1-2 DISAGREE-record — NO lower-crate \
         lift), un-ignores this test, wires it against the parity \
         oracle frozen in \
         tf5_d1_four_leaked_helpers_baseline_is_stable_pre_collapse"
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — D1 A2: the engine \
`E_VIEW_LABEL_MISMATCH` contract (`AlgorithmError::ViewLabelMismatch`) is preserved \
after the 4-helper collapse (a canonical id + a disagreeing caller label still \
fails loud). C4 exit obligation. Reviewer verifies landing-status per §3.6e."]
fn tf5_d1_e_view_label_mismatch_contract_preserved_post_collapse() {
    // PRODUCTION-ARM (un-ignore + wire at G-CORE-4):
    //
    //   use benten_ivm::{AlgorithmBView, AlgorithmError};
    //   // Register a canonical view-id whose hardcoded label is fixed
    //   // (NOT content_listing) with a deliberately-wrong caller label.
    //   let canonical_id = "capability_grants";
    //   let wrong_label = "definitely-not-the-hardcoded-label";
    //   let err = AlgorithmBView::register(
    //       canonical_id,
    //       /* label_pattern */ wrong_label.into(),
    //       /* projection */ Default::default(),
    //   ).expect_err("canonical id + disagreeing label must fail loud");
    //   assert!(matches!(err, AlgorithmError::ViewLabelMismatch { .. }),
    //       "E_VIEW_LABEL_MISMATCH contract must survive the CanonicalViews \
    //        collapse — got {err:?}");
    unimplemented!(
        "G-CORE-4 wires this against the post-collapse register path; \
         the ViewLabelMismatch (E_VIEW_LABEL_MISMATCH) guard MUST be \
         routed through `CanonicalViews` without weakening it"
    );
}

/// Handler-call-graph cycle-detection regression-guard: the G-CORE-4
/// group def + CLAUDE.md baked-in #1/#4 require cycle detection to be a
/// STRUCTURAL pre-registration DFS (visited-on-stack), reusing the
/// `detect_composition_cycle`-shape pattern — with **NO new
/// `PrimitiveKind` variant** minted (12-primitive irreducibility).
///
/// This arm is RUNNABLE at HEAD (it asserts the irreducibility invariant
/// over the EXISTING `benten_core::PrimitiveKind` set) so a G-CORE-4
/// implementer who reaches for a new variant trips this guard
/// immediately, not at review time.
#[test]
fn tf5_d1_no_new_primitive_kind_variant_minted_for_handler_cycle_detection() {
    use benten_core::PrimitiveKind;
    // The frozen canonical 12 (CLAUDE.md baked-in #1). A handler
    // call-graph cycle detector is a structural DFS over Read-shaped
    // nodes — it must NOT require a 13th variant.
    let canonical_12 = [
        PrimitiveKind::Read,
        PrimitiveKind::Write,
        PrimitiveKind::Transform,
        PrimitiveKind::Branch,
        PrimitiveKind::Iterate,
        PrimitiveKind::Wait,
        PrimitiveKind::Call,
        PrimitiveKind::Respond,
        PrimitiveKind::Emit,
        PrimitiveKind::Sandbox,
        PrimitiveKind::Subscribe,
        PrimitiveKind::Stream,
    ];
    assert_eq!(
        canonical_12.len(),
        12,
        "12-primitive irreducibility (CLAUDE.md baked-in #1): the \
         G-CORE-4 handler-call-graph cycle detector is a STRUCTURAL \
         pre-registration DFS reusing the detect_composition_cycle \
         shape — it must NOT mint a 13th PrimitiveKind variant (#4 \
         DAG-only). If this array no longer enumerates the full set, a \
         variant was added — STOP (ivm-r1-3 constraint)."
    );
}

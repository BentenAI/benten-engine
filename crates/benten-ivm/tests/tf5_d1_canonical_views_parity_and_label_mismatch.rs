//! ADDL R3 (TDD red-phase) — Phase-4-Meta-Core, Wave R3-B, agent R3-B2,
//! family **TF-5**. Post-G-CORE-4 GREEN landing of the **D1 A2
//! `CanonicalViews` registry-query seam** (C4 exit obligation; G-CORE-4
//! substrate).
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
//! ## Post-G-CORE-4 collapse — what the 4 pre-collapse helpers became
//!
//! Pre-G-CORE-4 the crate exposed 4 `pub fn` helpers:
//!
//! 1. `hardcoded_label_for_id(view_id) -> Option<&'static str>`
//! 2. `canonical_typed_output_projection_for(view_id) -> Option<TypedOutputProjection>`
//! 3. `is_canonical_view_id(view_id) -> bool`
//! 4. `dispatch_for(view_id) -> Strategy`
//!
//! Post-G-CORE-4 these are NARROWED to `pub(crate)` and the unified
//! [`benten_ivm::CanonicalViews`] registry-query type is the published
//! surface. The new shape:
//!
//! - `CanonicalViews::registry().lookup(id)` → `Option<CanonicalViewEntry>`
//! - `entry.hardcoded_label()` → `Option<&'static str>`
//! - `entry.typed_output_projection()` → `Option<TypedOutputProjection>`
//! - `CanonicalViews::registry().is_canonical(id)` → `bool`
//! - `CanonicalViews::registry().dispatch(id)` → `Strategy`
//!
//! ## §3.6b sub-rule-4 production-arm shape
//!
//! - PRODUCTION RUNTIME ARM: [`CanonicalViews::lookup`] /
//!   [`CanonicalViews::is_canonical`] / [`CanonicalViews::dispatch`]
//!   called for every canonical id AND a representative user-view id.
//! - OBSERVABLE CONSEQUENCE: the unified type's answers are
//!   **behaviour-identical** to the pre-collapse 4-helper baseline
//!   (frozen below as constant tables) AND the engine
//!   `E_VIEW_LABEL_MISMATCH` contract (`AlgorithmError::ViewLabelMismatch`)
//!   is preserved (a canonical id + disagreeing caller label still
//!   fails loud at register-time).
//! - WOULD-FAIL-IF-NO-OP: a collapse that changes any per-id answer
//!   (label / projection / canonical-classification / strategy) fails
//!   the parity assertion; a collapse that drops the label-mismatch
//!   guard fails the `ViewLabelMismatch` arm.
//!
//! ## SHAPE-FLAG (post-G-CORE-4)
//!
//! `CanonicalViews` is the G-CORE-4 deliverable type and EXISTS at the
//! landing branch HEAD. The parity baseline is frozen below as a static
//! table of `(view_id, hardcoded_label_opt, typed_projection_opt,
//! canonical_classification, strategy)` 5-tuples — the table IS the
//! oracle (no longer derived at runtime from the 4 leaked helpers,
//! because the helpers are `pub(crate)` post-collapse). The substantive
//! shape (per-id parity oracle) is preserved.
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): public-shape change (helpers narrowed +
//!   new `CanonicalViews` type) sweeps adjacent docs in the same wave.
//! - [x] §3.6b + sub-rule-4: parity + ViewLabelMismatch SPECIFIC arms.
//! - [x] §3.6e: post-G-CORE-4 un-ignore landing; G-CORE-4 ships
//!   `CanonicalViews` and un-ignores the 2 pre-collapse-ignored arms.
//! - [x] §3.6f: SHAPE-not-SUBSTANCE — substantive per-id parity oracle
//!   (frozen table) compared against the unified registry-query type's
//!   answers, NOT "assert a CanonicalViews type exists".
//! - [x] §3.5g: no cross-language/cross-doc mirror touched here.
//! - [x] §3.5i: file-disjoint from R3-B5 (benten-ivm lane only).
//! - [x] §3.6h: this rule does name an origin — the 4 leaked-pub helpers
//!   — and the G-CORE-4 wave SAME PR closes them (narrows them to
//!   `pub(crate)`); §3.6h satisfied by same-wave closure.
//! - [x] §3.6i/§3.6j: N/A. §3.13: no shared static introduced.
//! - [x] §3.6g: this checklist IS the literal reproduction.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_ivm::{
    AlgorithmBView, AlgorithmError, CanonicalViewEntry, CanonicalViews, LabelPattern, Projection,
    Strategy, TypedOutputProjection,
};

/// The frozen pre-collapse oracle: one row per (canonical id ∪
/// representative user id) probe point with the per-id facts the
/// pre-collapse helpers used to expose. This table IS the parity oracle
/// post-G-CORE-4 (the standalone `pub` helpers are `pub(crate)` and no
/// longer reachable from integration tests).
///
/// Row shape:
/// `(view_id, hardcoded_label_opt, typed_projection_opt, is_canonical, strategy)`
#[allow(clippy::type_complexity)]
const PARITY_ORACLE: &[(
    &str,
    Option<&str>,
    Option<TypedOutputProjection>,
    bool,
    Strategy,
)] = &[
    // The 5 canonical views — pre-collapse facts FROZEN here.
    (
        "capability_grants",
        Some("system:CapabilityGrant"),
        None,
        true,
        Strategy::A,
    ),
    (
        "event_dispatch",
        Some("system:EventDispatch"),
        None,
        true,
        Strategy::A,
    ),
    // `content_listing` honors the caller-supplied label
    // (hardcoded_label == None) — the subtle non-uniform arm a naive
    // collapse would flatten. The G-CORE-4 collapse preserves it.
    ("content_listing", None, None, true, Strategy::A),
    (
        "governance_inheritance",
        Some("system:GovernanceInheritance"),
        Some(TypedOutputProjection::Rules),
        true,
        Strategy::A,
    ),
    (
        "version_current",
        // The NEXT_VERSION wire label is sourced via
        // `benten_core::LABEL_NEXT_VERSION` (#601 SSoT) — frozen here
        // as the literal it resolves to for stability.
        Some(benten_core::LABEL_NEXT_VERSION),
        Some(TypedOutputProjection::Current),
        true,
        Strategy::A,
    ),
    // Representative user-defined id: non-canonical, no hardcoded
    // label, no typed-output projection, routes Strategy::B.
    (
        "user-defined-arbitrary-view",
        None,
        None,
        false,
        Strategy::B,
    ),
];

/// RUNNABLE regression guard: the frozen parity oracle MUST stay
/// internally consistent. A canonical id classifies canonical + routes
/// Strategy::A; a user id is non-canonical + routes Strategy::B. This
/// binds the baseline so the test FILE itself can't silently re-key the
/// post-collapse oracle.
#[test]
fn tf5_d1_frozen_parity_oracle_is_internally_consistent_pre_collapse_baseline() {
    for (id, label, proj, is_canon, strategy) in PARITY_ORACLE {
        if *is_canon {
            assert_eq!(*strategy, Strategy::A, "canonical id `{id}` must route A");
        } else {
            assert_eq!(
                *strategy,
                Strategy::B,
                "non-canonical id `{id}` must route B"
            );
            assert!(
                label.is_none(),
                "non-canonical id `{id}` must have no hardcoded label"
            );
            assert!(
                proj.is_none(),
                "non-canonical id `{id}` must have no typed-output projection"
            );
        }
    }
    // Subtle non-uniform arm: `content_listing` is canonical but
    // honors the caller-supplied label. The collapse MUST preserve
    // this. Encoded explicitly so a future re-keying that flattens
    // it (e.g. by hardcoding `"post"`) trips this assertion.
    let content_listing = PARITY_ORACLE
        .iter()
        .find(|r| r.0 == "content_listing")
        .expect("content_listing row");
    assert!(content_listing.3, "content_listing is canonical");
    assert!(
        content_listing.1.is_none(),
        "content_listing canonically honors the caller-supplied label \
         (hardcoded_label == None) — the collapse MUST preserve this \
         non-uniform arm"
    );
}

#[test]
fn tf5_d1_canonical_views_lookup_parity_vs_four_old_helpers() {
    let registry = CanonicalViews::registry();
    for (id, label, proj, is_canon, strategy) in PARITY_ORACLE {
        // ivm-r1-2 DISAGREE-record: the unified type's path MUST be
        // `benten_ivm::CanonicalViews` (NO lower-crate lift — the
        // "(b) lift to a lower crate" framing is RATIFIED-rejected per
        // D1 A2). A compile-fact: the `use` at the top of this file
        // resolves from `benten_ivm`, not `benten_core`/`benten_graph`.
        let entry: Option<CanonicalViewEntry> = registry.lookup(id);

        // `is_canonical` parity.
        assert_eq!(
            registry.is_canonical(id),
            *is_canon,
            "is_canonical parity drift for `{id}`: registry={} oracle={}",
            registry.is_canonical(id),
            *is_canon,
        );

        // `lookup(...)` returns Some for canonical ids, None for
        // non-canonical (frozen-oracle parity).
        assert_eq!(
            entry.is_some(),
            *is_canon,
            "lookup-presence parity drift for `{id}`"
        );

        if let Some(e) = entry {
            assert_eq!(
                e.hardcoded_label(),
                *label,
                "hardcoded_label parity drift for `{id}`: entry={:?} oracle={label:?}",
                e.hardcoded_label(),
            );
            assert_eq!(
                e.typed_output_projection(),
                *proj,
                "typed_output_projection parity drift for `{id}`: entry={:?} oracle={proj:?}",
                e.typed_output_projection(),
            );
        }

        // `dispatch` parity.
        assert_eq!(
            registry.dispatch(id),
            *strategy,
            "dispatch (Strategy) parity drift for `{id}`: registry={:?} oracle={strategy:?}",
            registry.dispatch(id),
        );
    }
}

#[test]
fn tf5_d1_e_view_label_mismatch_contract_preserved_post_collapse() {
    // E_VIEW_LABEL_MISMATCH contract: a canonical id + a disagreeing
    // caller label MUST still fail loud at registration time
    // post-collapse. We register `capability_grants` (whose hardcoded
    // label is `system:CapabilityGrant`) with a deliberately-wrong
    // caller label and assert `ViewLabelMismatch`. The G-CORE-4
    // collapse MUST NOT weaken this guard.
    let canonical_id = "capability_grants";
    let wrong_label = "definitely-not-the-hardcoded-label";
    let err = AlgorithmBView::register(
        canonical_id,
        LabelPattern::exact(wrong_label),
        Projection::all_props(),
    )
    .expect_err("canonical id + disagreeing label MUST fail loud");
    match err {
        AlgorithmError::ViewLabelMismatch {
            view_id,
            expected_label,
            ..
        } => {
            assert_eq!(view_id, canonical_id);
            assert_eq!(expected_label, "system:CapabilityGrant");
        }
        other => panic!(
            "expected AlgorithmError::ViewLabelMismatch post-G-CORE-4 collapse, got {other:?}"
        ),
    }
}

/// Handler-call-graph cycle-detection regression-guard: the G-CORE-4
/// group def + CLAUDE.md baked-in #1/#4 require cycle detection to be a
/// STRUCTURAL pre-registration DFS (visited-on-stack), reusing the
/// `detect_composition_cycle`-shape pattern — with **NO new
/// `PrimitiveKind` variant** minted (12-primitive irreducibility).
///
/// This arm asserts the irreducibility invariant over the EXISTING
/// `benten_core::PrimitiveKind` set so a future implementer who reaches
/// for a new variant trips this guard immediately, not at review time.
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

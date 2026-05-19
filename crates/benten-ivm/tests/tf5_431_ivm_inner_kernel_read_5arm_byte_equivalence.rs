//! ADDL R3 (TDD red-phase) — Phase-4-Meta-Core, Wave R3-B, agent R3-B2,
//! family **TF-5**. RED-phase pin for the **§4.31 IVM inner-kernel-read
//! 5-arm byte-equivalence** (C9 exit obligation; G-CORE-4 substrate).
//!
//! ## Pin provenance
//!
//! - R2 test-landscape `.addl/phase-4-meta/r2-test-landscape.md` TF-5 +
//!   §2.A row S4 + §4-A SHAPE-not-SUBSTANCE trap table (TF-5 row).
//! - Plan `.addl/phase-4-meta/00-implementation-plan.md` G-CORE-4 group
//!   def + §1.A C9 exit criterion.
//! - Origin pin cluster: `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs`
//!   (R6 R1 test-coverage-auditor tc-1; 5 arms) →
//!   `docs/future/phase-4-backlog.md §4.31`. This file is the
//!   Phase-4-Meta-Core substantive landing of that named-destination
//!   carry (the §4.31 row's 5-arm byte-equivalence companion).
//! - ivm-materializer-r1-1 / r1-triage row 46.
//!
//! ## ⚠️ SHAPE-not-SUBSTANCE — THE EXPLICIT TRAP (pim-18 / §3.6f)
//!
//! The R2 §4-A trap table flags THIS family as "the easiest place in
//! the phase to ship a structural sentinel". The required substantive
//! arm is: drive the **production** `materialize_inner_kernel_read`
//! seam over BOTH the `SubgraphSpec`-routed walk (G23-0a
//! `Algorithm::register_subgraph`) AND the legacy G15-A path-view walk
//! (`Algorithm::register`), feed identical write sequences, and assert
//! the inner-kernel-read outputs are **byte-identical** for each of the
//! 5 canonical views. It is NOT sufficient to assert "a `CanonicalViews`
//! type is constructible" or "a SubgraphSpec round-trips" — those are
//! the sentinel forms r1-triage explicitly rejected.
//!
//! ## §3.6b sub-rule-4 production-arm shape
//!
//! - PRODUCTION RUNTIME ARM: `AlgorithmBView::register_subgraph(spec)`
//!   (SubgraphSpec-routed) ∥ `AlgorithmBView::register(view_id,
//!   label_pattern, projection)` (G15-A path-view), identical
//!   `walk_writes` sequence, then the **production**
//!   `materialize_inner_kernel_read` seam (G-CORE-4 deliverable) over
//!   each view.
//! - OBSERVABLE CONSEQUENCE: per-view inner-kernel-read bytes are
//!   byte-equal across both walks (× 5 canonical views).
//! - WOULD-FAIL-IF-NO-OP: if either walk drifts the emission shape
//!   (e.g. `view_4` ViewResult::Rules field order, `view_5`
//!   ViewResult::Current variant tag) the byte-inequality assertion
//!   fires. The G23-0b round-trip pins do NOT catch this (they prove
//!   wrapper-construction-equivalence by construction-identity, which
//!   bypasses the inner kernel's read — see the origin file's module
//!   doc). Only this arm catches an inner-kernel emission regression.
//!
//! ## SHAPE-FLAG (not faked)
//!
//! `materialize_inner_kernel_read` is the G-CORE-4 deliverable seam and
//! does NOT exist at HEAD `ed03729a`. These tests are written against
//! the **intended production surface** and are `#[ignore]`d until
//! G-CORE-4 wires the seam (per §3.6e RED-PHASE staged-pin; reviewer
//! verifies landing-status, not just spec-pin presence). The test
//! bodies encode the substantive byte-equality assertion (NOT a
//! sentinel) so the G-CORE-4 implementer un-ignores + wires the real
//! seam against an already-correct substantive shape.
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal, per the plan §3 NON-NEGOTIABLE R3/R5 BRIEF-TEMPLATE DIRECTIVE — reproduced, not §-referenced)
//!
//! - [x] §3.5b HARDENED (pim-1): no public-shape change here (tests only); the G-CORE-4 implementer sweeps adjacent docs on the real seam-add.
//! - [x] §3.6b + sub-rule-4 (pim-2): production-arm + observable + would-FAIL exercising the SPECIFIC 5-arm byte-equiv (above).
//! - [x] §3.6e (pim-12): RED-PHASE staged-pin; ignore message names the G-CORE-4 un-ignore destination; reviewer verifies landing.
//! - [x] §3.6f (pim-18): SHAPE-not-SUBSTANCE — substantive byte-equal arm, NOT a constructible-type sentinel (the explicit trap).
//! - [x] §3.5g: no ErrorCode/type-name/dual-config mirror touched.
//! - [x] §3.5i: file-disjoint from R3-B5 (this lane = benten-ivm/tests + materializer + dsl vocab-fixture; NO benten-engine/benten-caps).
//! - [x] §3.6h: this file codifies no rule naming an origin instance.
//! - [x] §3.6i/§3.6j: N/A (no JSON artifact authored here).
//! - [x] §3.13: per-test statics — none introduced (no shared static).
//! - [x] §3.5h/§3.5l/§3.5m/§3.5n: orchestrator/merge-time gates; N/A at test-write time.
//! - [x] §3.11: N/A (R3-B2 is not the large/long-running G-CORE-8 lane).
//! - [x] §3.6g: this checklist IS the literal reproduction.

#![allow(clippy::unwrap_used, clippy::expect_used)]

// Canonical view ids — single source of truth is
// `benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS`. The 5-arm coverage
// is one test per id; a per-arm helper keeps each arm a distinct
// would-FAIL pin (pim-2 amendment per-finding granularity).
const CANONICAL_VIEWS: [&str; 5] = [
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
];

/// Substantive byte-equivalence assertion for ONE canonical view.
///
/// SHAPE-FLAG: `materialize_inner_kernel_read` is the G-CORE-4 seam
/// (absent at HEAD `ed03729a`). The G-CORE-4 implementer un-ignores the
/// per-arm tests below + wires the production seam.
///
/// ## R4.1 fix-pass (L1 MINOR coverage-completeness-r4.1-3 / pim-18
/// pattern-induction-3): converted from the `unimplemented!()` +
/// comment-code idiom to the §4.28-template panic!-hold-with-shipped-
/// surface-exercise idiom (the L3-affirmed-sound pattern; pure cfg-
/// gate was the L1-named alternative but a cfg-gate would require a
/// Cargo.toml [features] addition which is out-of-scope for the tests-
/// only fix-pass — fix-pass directive explicitly authorizes the
/// `panic!`-hold fallback when the feature gate is the wrong shape).
/// The body now exercises SHIPPED `benten_ivm::subgraph_spec::
/// CANONICAL_VIEW_IDS` substrate to assert the per-view-id input is
/// a recognized canonical view (the substrate the G-CORE-4
/// `materialize_inner_kernel_read` seam will dispatch on), THEN
/// panics with the production-seam-undelivered narrative. This
/// converts the un-ignore-time correctness from "implementer writes
/// whatever body" to "exercise the SHIPPED canonical-view-id
/// substrate with a real would-FAIL signal" + the standing
/// pim-18-mostly-undelivered-target-surface-hybrid pattern.
#[allow(dead_code)]
fn assert_inner_kernel_read_byte_equivalent_across_both_walks(view_id: &str) {
    use benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS;

    // -----------------------------------------------------------------
    // SHIPPED-SURFACE EXERCISE (substantive-arm anchor for pim-18 §3.6f):
    // the SHIPPED `CANONICAL_VIEW_IDS` substrate IS the dispatch table
    // the future `materialize_inner_kernel_read` seam consults. Assert
    // `view_id` is a recognized canonical view (real assertion on real
    // input). Would-FAIL signal: if the canonical-view set shrinks or
    // the requested view_id is dropped from it, this assertion fires —
    // the same regression the 5-arm byte-equivalence obligation depends
    // on.
    // -----------------------------------------------------------------
    assert!(
        CANONICAL_VIEW_IDS.contains(&view_id),
        "shipped surface exercise (substrate for §4.31): the requested \
         canonical view id `{view_id}` MUST be present in the SHIPPED \
         `benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS` dispatch table \
         (the substrate the future `materialize_inner_kernel_read` seam \
         dispatches on). Would-FAIL if the canonical-view set shrinks \
         or the requested view_id is silently dropped."
    );

    // PRODUCTION-ARM (un-ignore + wire at G-CORE-4):
    //
    //   use benten_ivm::{AlgorithmBView, SubgraphSpec};
    //   use benten_platform_foundation::materializer::materialize_inner_kernel_read;
    //
    //   // (1) SubgraphSpec-routed walk (G23-0a path).
    //   let spec = SubgraphSpec::for_canonical_view(view_id).unwrap();
    //   let mut subgraph_view = AlgorithmBView::register_subgraph(spec).unwrap();
    //
    //   // (2) Legacy G15-A path-view walk (same canonical id).
    //   let (label_pattern, projection) =
    //       benten_ivm::canonical_g15a_args_for(view_id);
    //   let mut g15a_view =
    //       AlgorithmBView::register(view_id, label_pattern, projection).unwrap();
    //
    //   // Identical write sequence through BOTH.
    //   let writes = benten_ivm::testing::canonical_inputs_for(view_id);
    //   subgraph_view.walk_writes(&writes).unwrap();
    //   g15a_view.walk_writes(&writes).unwrap();
    //
    //   // The G-CORE-4 PRODUCTION seam (NOT a sentinel): raw
    //   // inner-kernel-read bytes for each walk.
    //   let lhs = materialize_inner_kernel_read(&subgraph_view).unwrap();
    //   let rhs = materialize_inner_kernel_read(&g15a_view).unwrap();
    //
    //   // SUBSTANTIVE byte-equality (the explicit trap — NOT
    //   // "assert a CanonicalViews type is constructible").
    //   assert_eq!(
    //       lhs, rhs,
    //       "inner-kernel-read byte-equivalence regression for canonical \
    //        view `{view_id}`: SubgraphSpec-routed walk and legacy G15-A \
    //        path-view walk emitted different bytes (would-FAIL if either \
    //        walk's emission shape drifts)"
    //   );
    panic!(
        "§4.31 inner-kernel-read 5-arm byte-equivalence undelivered: \
         the SHIPPED `CANONICAL_VIEW_IDS` substrate is exercised above \
         (view_id `{view_id}` is a recognized canonical view), but the \
         G-CORE-4 production `materialize_inner_kernel_read` seam that \
         emits the inner-kernel-read bytes across BOTH the SubgraphSpec- \
         routed walk and the legacy G15-A path-view walk is undelivered. \
         G-CORE-4 wires the seam + un-ignores the 5 per-arm tests; \
         substantive byte-equality shape is fixed in the commented \
         block above (SHAPE-not-SUBSTANCE trap guard, R2 §4-A)."
    );
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.31 IVM inner-kernel-read 5-arm \
byte-equivalence; production seam `materialize_inner_kernel_read` is the G-CORE-4 \
deliverable. Named destination: docs/future/phase-4-backlog.md §4.31 (HARD RULE 12 \
clause-(b)). Reviewer verifies landing-status per §3.6e, not just spec-pin presence."]
fn tf5_431_inner_kernel_read_byte_equiv_view_1_capability_grants() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[0]);
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.31 arm: view_2 event_dispatch \
inner-kernel-read byte-equivalence across SubgraphSpec-routed + legacy G15-A walks."]
fn tf5_431_inner_kernel_read_byte_equiv_view_2_event_dispatch() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[1]);
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.31 arm: view_3 content_listing \
inner-kernel-read byte-equivalence (couples View-3 stale-with-last-known-good \
generalization; the byte-equiv must hold including the stale-fallback emission)."]
fn tf5_431_inner_kernel_read_byte_equiv_view_3_content_listing() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[2]);
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.31 arm: view_4 governance_inheritance \
(ViewResult::Rules emission); would-FAIL if Rules field order drifts on either walk."]
fn tf5_431_inner_kernel_read_byte_equiv_view_4_governance_inheritance() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[3]);
}

#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-4 — §4.31 arm: view_5 version_current \
(ViewResult::Current emission); would-FAIL if Current variant tag drifts on either walk."]
fn tf5_431_inner_kernel_read_byte_equiv_view_5_version_current() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[4]);
}

/// Coverage-completeness guard: the 5-arm set MUST cover exactly the
/// canonical view ids (no arm dropped, none invented). This arm is
/// runnable at HEAD (no production seam dependency) so a future
/// reduction of the canonical-view set surfaces immediately rather than
/// silently shrinking the byte-equivalence obligation.
#[test]
fn tf5_431_five_arm_set_covers_exactly_the_canonical_view_ids() {
    use benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS;
    let mut expected: Vec<&str> = CANONICAL_VIEW_IDS.to_vec();
    let mut covered: Vec<&str> = CANONICAL_VIEWS.to_vec();
    expected.sort_unstable();
    covered.sort_unstable();
    assert_eq!(
        covered, expected,
        "the §4.31 5-arm byte-equivalence set must cover EXACTLY the \
         canonical IVM view ids (one would-FAIL pin per view); a \
         mismatch means an arm was dropped or the canonical set changed \
         without the byte-equiv obligation following it"
    );
}

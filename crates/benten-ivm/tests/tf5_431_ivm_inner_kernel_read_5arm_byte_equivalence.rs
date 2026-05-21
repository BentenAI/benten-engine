//! Phase-4-Meta-Core G-CORE-4 GREEN landing of the §4.31 IVM
//! inner-kernel-read 5-arm byte-equivalence pin (C9 exit obligation).
//!
//! ## Pin provenance
//!
//! - R2 test-landscape `.addl/phase-4-meta/r2-test-landscape.md` TF-5 +
//!   §2.A row S4 + §4-A SHAPE-not-SUBSTANCE trap table (TF-5 row).
//! - Plan `.addl/phase-4-meta/00-implementation-plan.md` G-CORE-4 group
//!   def + §1.A C9 exit criterion.
//! - Origin pin cluster: `inner_kernel_read_equivalence_post_subgraph_spec_round_trip.rs`
//!   (R6 R1 test-coverage-auditor tc-1; 5 arms) →
//!   `docs/future/phase-4-backlog.md §4.31`. This file is the substantive
//!   landing of that named-destination carry (the §4.31 row's 5-arm
//!   byte-equivalence companion).
//! - ivm-materializer-r1-1 / r1-triage row 46.
//!
//! ## SHAPE-FLAG closed (G-CORE-4 lands the production seam)
//!
//! Pre-G-CORE-4 the seam `benten_ivm::materialize_inner_kernel_read` did
//! NOT exist; this file was `#[ignore]`d (§3.6e RED-PHASE staged-pin).
//! G-CORE-4 lands `materialize_inner_kernel_read` (the 4-arm
//! `KernelOutput` byte-envelope) and un-ignores the 5 per-arm tests.
//! Each test now:
//!   1. Registers the same canonical view via BOTH paths
//!      (`AlgorithmBView::register_subgraph` G23-0a + `AlgorithmBView::register`
//!      G15-A).
//!   2. Walks an identical write sequence through each.
//!   3. Calls the PRODUCTION `materialize_inner_kernel_read` seam over
//!      each walk + asserts byte-equality.
//!
//! Substantive arm — NOT a sentinel. A regression on either walk's
//! emission shape (e.g. `view_4` ViewResult::Rules field order, `view_5`
//! ViewResult::Current variant tag) fires the byte-inequality assertion.
//! The G23-0b round-trip pins do NOT catch this (they prove
//! wrapper-construction-equivalence by construction-identity, which
//! bypasses the inner kernel's read — see the origin file's module doc).
//!
//! ## §3.6g inherited-discipline pre-flight checklist (literal)
//!
//! - [x] §3.5b HARDENED (pim-1): adjacent docs swept by the G-CORE-4 wave
//!   in the same PR.
//! - [x] §3.6b + sub-rule-4 (pim-2): production runtime arm
//!   (`materialize_inner_kernel_read` over both walks) + observable
//!   consequence (byte-equality) + would-FAIL (drift in either walk's
//!   emission fires).
//! - [x] §3.6e (pim-12): RED-PHASE pins un-ignored at G-CORE-4 (this file).
//! - [x] §3.6f (pim-18): SHAPE-not-SUBSTANCE — byte-equal arm,
//!   construction-equivalence is NOT what's asserted.
//! - [x] §3.5g: no cross-language mirror touched.
//! - [x] §3.5i: file-disjoint from G-CORE-7 (G-CORE-7 owns
//!   plugin_lifecycle/manifest_store/plugin_manifest in
//!   benten-platform-foundation; G-CORE-4 owns materializer +
//!   vocab-fixture; this file lives in benten-ivm).
//! - [x] §3.6h: this file does not codify a rule.
//! - [x] §3.6i/§3.6j: N/A (no JSON artifact authored).
//! - [x] §3.13: no per-test static introduced.
//! - [x] §3.11: N/A (small lane).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS;
use benten_ivm::{
    AlgorithmBView, CanonicalViews, KernelInput, LabelPattern, Projection, SubgraphSpec,
    materialize_inner_kernel_read,
};

/// The 5 canonical view ids exercised one-test-per-id (pim-2 amendment
/// per-finding granularity).
const CANONICAL_VIEWS: [&str; 5] = [
    "capability_grants",
    "event_dispatch",
    "content_listing",
    "governance_inheritance",
    "version_current",
];

/// Resolve the canonical hardcoded label for a canonical view id using
/// the G-CORE-4 `CanonicalViews` registry-query type. `content_listing`
/// honors a caller-supplied label; for the 4 hardcoded-label views the
/// registry returns the canonical fixed label.
fn label_for_canonical_view(view_id: &str) -> String {
    if view_id == "content_listing" {
        return "post".to_string();
    }
    CanonicalViews::registry()
        .lookup(view_id)
        .and_then(|e| e.hardcoded_label())
        .unwrap_or_else(|| panic!("no hardcoded label for `{view_id}`"))
        .to_string()
}

/// Build a deterministic write sequence for one canonical view id.
/// Uses the canonical hardcoded label so the SubgraphSpec-routed walk
/// and the legacy G15-A path-view walk both admit the events.
fn canonical_writes_for(view_id: &str) -> Vec<KernelInput> {
    let label = label_for_canonical_view(view_id);
    (0u32..4)
        .map(|i| KernelInput::new(label.clone(), i64::from(i) * 100, u64::from(i)))
        .collect()
}

/// Substantive byte-equivalence assertion for ONE canonical view: drive
/// both registration paths over the same write sequence, then compare
/// the production `materialize_inner_kernel_read` seam's output.
fn assert_inner_kernel_read_byte_equivalent_across_both_walks(view_id: &str) {
    // Shipped-surface sanity: the requested view_id is in the canonical
    // dispatch table the seam consults.
    assert!(
        CANONICAL_VIEW_IDS.contains(&view_id),
        "shipped surface exercise: the requested canonical view id `{view_id}` MUST \
         be present in `benten_ivm::subgraph_spec::CANONICAL_VIEW_IDS` (the substrate \
         the `materialize_inner_kernel_read` seam dispatches on)"
    );

    let writes = canonical_writes_for(view_id);

    // (1) SubgraphSpec-routed walk (G23-0a path).
    let spec = SubgraphSpec::for_canonical_view(view_id)
        .expect("SubgraphSpec::for_canonical_view succeeds for canonical id");
    let mut subgraph_view = AlgorithmBView::register_subgraph(spec)
        .expect("Algorithm::register_subgraph succeeds for canonical SubgraphSpec");
    subgraph_view
        .walk_writes(&writes)
        .expect("walk_writes (SubgraphSpec lane) succeeds");

    // (2) Legacy G15-A path-view walk (same canonical id + same label
    //     + same projection).
    let label = label_for_canonical_view(view_id);
    let mut g15a_view =
        AlgorithmBView::register(view_id, LabelPattern::exact(label), Projection::all_props())
            .expect("Algorithm::register (G15-A lane) succeeds for canonical id + canonical label");
    g15a_view
        .walk_writes(&writes)
        .expect("walk_writes (G15-A lane) succeeds");

    // (3) PRODUCTION seam: per-walk inner-kernel-read bytes (the G-CORE-4
    //     deliverable). Substantive byte-equality assertion.
    let lhs = materialize_inner_kernel_read(&subgraph_view);
    let rhs = materialize_inner_kernel_read(&g15a_view);
    assert_eq!(
        lhs, rhs,
        "inner-kernel-read byte-equivalence regression for canonical view `{view_id}`: \
         the SubgraphSpec-routed walk and the legacy G15-A path-view walk emitted \
         different bytes (would-FAIL if either walk's emission shape drifts; the \
         G23-0b round-trip pins prove wrapper-construction-equivalence by \
         construction-identity, which bypasses this assertion)."
    );

    // (4) Sanity: the byte envelope is non-trivial (a regression that
    //     silently returned `Vec::new()` from the seam would still match
    //     itself; assert at least the arm-discriminator byte is present).
    assert!(
        !lhs.is_empty(),
        "materialize_inner_kernel_read MUST emit at least the 1-byte \
         arm-discriminator for any walk (got empty bytes for `{view_id}`)"
    );
}

#[test]
fn tf5_431_inner_kernel_read_byte_equiv_view_1_capability_grants() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[0]);
}

#[test]
fn tf5_431_inner_kernel_read_byte_equiv_view_2_event_dispatch() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[1]);
}

#[test]
fn tf5_431_inner_kernel_read_byte_equiv_view_3_content_listing() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[2]);
}

#[test]
fn tf5_431_inner_kernel_read_byte_equiv_view_4_governance_inheritance() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[3]);
}

#[test]
fn tf5_431_inner_kernel_read_byte_equiv_view_5_version_current() {
    assert_inner_kernel_read_byte_equivalent_across_both_walks(CANONICAL_VIEWS[4]);
}

/// Coverage-completeness guard: the 5-arm set MUST cover exactly the
/// canonical view ids (no arm dropped, none invented).
#[test]
fn tf5_431_five_arm_set_covers_exactly_the_canonical_view_ids() {
    let mut expected: Vec<&str> = CANONICAL_VIEW_IDS.to_vec();
    let mut covered: Vec<&str> = CANONICAL_VIEWS.to_vec();
    expected.sort_unstable();
    covered.sort_unstable();
    assert_eq!(
        covered, expected,
        "the §4.31 5-arm byte-equivalence set must cover EXACTLY the \
         canonical IVM view ids (one would-FAIL pin per view); a mismatch \
         means an arm was dropped or the canonical set changed without \
         the byte-equiv obligation following it"
    );
}

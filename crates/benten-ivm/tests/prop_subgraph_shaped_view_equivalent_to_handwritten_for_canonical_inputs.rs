//! R4-FP-3 → G23-0b: proptest equivalence — subgraph-shaped view result
//! MUST equal hand-written-G15-A view result for arbitrary canonical
//! inputs (post-G23-0b 5-view re-expression).
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 G23-0b row
//!   (proptest equivalence over arbitrary canonical inputs).
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0b
//!   must-pass tests.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter
//!   (closes r4-tc-6 Family C missing IVM pin #1 of 3).
//! - r1-architect-reviewer.json arch-r1-9: post-generalization safety
//!   requires PROPTEST equivalence over arbitrary canonical inputs.
//!
//! ## What this pin asserts
//!
//! G23-0b re-expresses 5 hand-written views (`capability_grants`,
//! `event_dispatch`, `content_listing`, `governance_inheritance`,
//! `version_current`) as `SubgraphSpec`-shaped views consuming the
//! G23-0a generalized kernel. This pin asserts the
//! `Algorithm::register_subgraph(SubgraphSpec)` output matches the
//! `Algorithm::register(view_id, label_pattern, projection)` (G15-A
//! API) output over a wide set of arbitrary canonical inputs.
//!
//! Critical safety net for the generalization: the 5 fixed-fixture
//! round-trip pins (Family C round-trip pins) verify equivalence at
//! canonical inputs; the proptest sweep catches edge-case input shapes.
//!
//! ## §3.6f SHAPE-not-SUBSTANCE
//!
//! SUBSTANCE: each prop_case ACTUALLY runs both kernel paths AND
//! compares canonical bytes — not "kernel exists" / "got non-empty
//! result" placeholder shape. SHAPE: proptest harness compiles.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common_kernel_canary;
use common_kernel_canary::{
    CanarySubgraphSpec, KernelInput, handwritten_baseline_via_register_g15a,
    register_and_walk_to_completion,
};
use proptest::prelude::*;

/// Per-canonical-view arbitrary KernelInput strategy. For canonical
/// views with hardcoded labels we use those labels; for `content_listing`
/// we use `"post"`. The created_at + disambiguator vary freely.
fn arbitrary_kernel_inputs_for(view_id: &'static str) -> impl Strategy<Value = Vec<KernelInput>> {
    let label: &'static str = match view_id {
        "capability_grants" => "system:CapabilityGrant",
        "event_dispatch" => "system:EventDispatch",
        "content_listing" => "post",
        "governance_inheritance" => "system:GovernanceInheritance",
        "version_current" => "NEXT_VERSION",
        _ => panic!("unknown canonical view id: {view_id}"),
    };
    prop::collection::vec(
        (any::<i64>(), any::<u32>()).prop_map(move |(t, d)| {
            // Keep timestamps in a reasonable range so logical-ordering
            // never overflows (BTreeMap key comparison cost is unaffected
            // either way; this is for human-readable debug output).
            let bounded_t = t.rem_euclid(1_000_000_000);
            KernelInput::new(label.to_string(), bounded_t, u64::from(d))
        }),
        0..16usize,
    )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 200,
        ..ProptestConfig::default()
    })]

    /// View 1 — capability_grants. SubgraphSpec path must match G15-A
    /// register path byte-for-byte.
    #[test]
    fn capability_grants_subgraph_spec_matches_handwritten_g15a(
        writes in arbitrary_kernel_inputs_for("capability_grants"),
    ) {
        let spec = CanarySubgraphSpec::for_canonical_view("capability_grants");
        let lhs = register_and_walk_to_completion(&spec, &writes).expect("SubgraphSpec walk");
        let rhs = handwritten_baseline_via_register_g15a(&spec, &writes).expect("G15-A walk");
        prop_assert_eq!(
            lhs, rhs,
            "View 1 (capability_grants) SubgraphSpec path diverged from G15-A baseline"
        );
    }

    /// View 2 — event_dispatch.
    #[test]
    fn event_dispatch_subgraph_spec_matches_handwritten_g15a(
        writes in arbitrary_kernel_inputs_for("event_dispatch"),
    ) {
        let spec = CanarySubgraphSpec::for_canonical_view("event_dispatch");
        let lhs = register_and_walk_to_completion(&spec, &writes).expect("SubgraphSpec walk");
        let rhs = handwritten_baseline_via_register_g15a(&spec, &writes).expect("G15-A walk");
        prop_assert_eq!(
            lhs, rhs,
            "View 2 (event_dispatch) SubgraphSpec path diverged from G15-A baseline"
        );
    }

    /// View 3 — content_listing. The G15-A inner kernel preserves sort
    /// order; SubgraphSpec path must match.
    #[test]
    fn content_listing_subgraph_spec_matches_handwritten_g15a(
        writes in arbitrary_kernel_inputs_for("content_listing"),
    ) {
        let spec = CanarySubgraphSpec::for_canonical_view("content_listing");
        let lhs = register_and_walk_to_completion(&spec, &writes).expect("SubgraphSpec walk");
        let rhs = handwritten_baseline_via_register_g15a(&spec, &writes).expect("G15-A walk");
        prop_assert_eq!(
            lhs, rhs,
            "View 3 (content_listing) SubgraphSpec path diverged from G15-A baseline"
        );
    }

    /// View 4 — governance_inheritance. Typed-output projection
    /// (Rules) MUST be selected by both paths.
    #[test]
    fn governance_inheritance_subgraph_spec_matches_handwritten_g15a(
        writes in arbitrary_kernel_inputs_for("governance_inheritance"),
    ) {
        let spec = CanarySubgraphSpec::for_canonical_view("governance_inheritance");
        let lhs = register_and_walk_to_completion(&spec, &writes).expect("SubgraphSpec walk");
        let rhs = handwritten_baseline_via_register_g15a(&spec, &writes).expect("G15-A walk");
        prop_assert_eq!(
            lhs, rhs,
            "View 4 (governance_inheritance) SubgraphSpec path diverged from G15-A baseline"
        );
    }

    /// View 5 — version_current. Typed-output projection (Current)
    /// MUST be selected by both paths.
    #[test]
    fn version_current_subgraph_spec_matches_handwritten_g15a(
        writes in arbitrary_kernel_inputs_for("version_current"),
    ) {
        let spec = CanarySubgraphSpec::for_canonical_view("version_current");
        let lhs = register_and_walk_to_completion(&spec, &writes).expect("SubgraphSpec walk");
        let rhs = handwritten_baseline_via_register_g15a(&spec, &writes).expect("G15-A walk");
        prop_assert_eq!(
            lhs, rhs,
            "View 5 (version_current) SubgraphSpec path diverged from G15-A baseline"
        );
    }
}

/// Smoke check that the harness compiles and the proptest blocks are
/// hooked up correctly. Sanity-check on the assertion shape.
#[test]
fn prop_subgraph_shaped_view_equivalent_to_handwritten_for_canonical_inputs() {
    // The five proptest blocks above each cover one canonical view.
    // This wrapper test exists for the test-name pin: the file's
    // name is `prop_subgraph_shaped_view_equivalent_to_handwritten_for_canonical_inputs`
    // and the spec pins the name + invokes a smoke run of each view's
    // fixed-fixture lane to catch register-time regressions early.
    for view_id in [
        "capability_grants",
        "event_dispatch",
        "content_listing",
        "governance_inheritance",
        "version_current",
    ] {
        let spec = CanarySubgraphSpec::for_canonical_view(view_id);
        let writes: Vec<KernelInput> = Vec::new();
        let lhs = register_and_walk_to_completion(&spec, &writes)
            .unwrap_or_else(|e| panic!("smoke walk for `{view_id}`: {e}"));
        let rhs = handwritten_baseline_via_register_g15a(&spec, &writes)
            .unwrap_or_else(|e| panic!("smoke baseline for `{view_id}`: {e}"));
        assert_eq!(lhs, rhs, "empty-walk equivalence smoke for `{view_id}`");
    }
}

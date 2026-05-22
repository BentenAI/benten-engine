//! TF-3w (R3-W2) — Combinator proptest: `intersect / union / filter`
//! structural-containment-preserving.
//!
//! Family: TF-3w — G-CORE-3w `intersect / union / filter` combinators +
//! authoring-ergonomics arm.
//!
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3w def
//! (line 334) — "builder DSL + `query! / walk! / predicate! /
//! stop_when!` macros + `intersect / union / filter` combinators."
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3w (P-3) — "`intersect /
//! union / filter` combinators: structural-containment-preserving
//! (intersect narrows; union widens; filter narrows); proptest over
//! generated SubgraphSpecs." + (P-4) authoring ergonomics: macros
//! produce the same SubgraphSpec as the raw struct-literal.
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3w
//! production surface = `benten_core::subgraph_spec::combinators::{
//! intersect, union, filter}`. R5 implementer mints
//! `crates/benten-core/src/subgraph_spec/combinators.rs` + un-ignores
//! these pins.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Full R3 BRIEF-TEMPLATE CHECKLIST in
//! `tf3b_restricted_spec_contains_decidable.rs` head comment.
//!
//! Special pin shape: this is the proptest member of the W2 partition.
//! Cases reduced to ≥64 per the in-tree convention (`proptests.rs:3`).
//! SHAPE-flag-don't-fake: RED-PHASE.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use benten_core::Cid;
// RED: `benten_core::subgraph_spec` does NOT exist at HEAD.
use benten_core::subgraph_spec::{
    Spec, SubgraphSpecError,
    combinators::{filter, intersect, union},
};
use proptest::prelude::*;

fn cid_for(label: &str) -> Cid {
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

/// Strategy: a small `Spec` over a finite root pool + per-Spec
/// max_depth + per-Spec label_allowlist subset.
fn arb_spec() -> impl Strategy<Value = Spec> {
    // Finite root pool of 4 deterministic CIDs.
    let root_pool: Vec<Cid> = ["proptest-R0", "proptest-R1", "proptest-R2", "proptest-R3"]
        .iter()
        .map(|s| cid_for(s))
        .collect();
    let labels_pool: Vec<String> = ["A", "B", "C", "D"].iter().map(|s| s.to_string()).collect();

    (
        prop::collection::vec(0usize..root_pool.len(), 1..=root_pool.len()),
        1u32..=10,
        prop::collection::vec(0usize..labels_pool.len(), 0..=labels_pool.len()),
    )
        .prop_filter_map(
            "well-formed spec",
            move |(root_idxs, max_depth, label_idxs)| {
                let mut roots: Vec<Cid> = root_idxs.iter().map(|i| root_pool[*i]).collect();
                roots.sort_by_key(|c| *c.as_bytes());
                roots.dedup();

                let mut labels: Vec<String> =
                    label_idxs.iter().map(|i| labels_pool[*i].clone()).collect();
                labels.sort();
                labels.dedup();

                let mut b = Spec::builder().with_max_depth(max_depth);
                for r in &roots {
                    b = b.with_root(*r);
                }
                if !labels.is_empty() {
                    b = b.with_label_allowlist(labels);
                }
                b.build().ok()
            },
        )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        .. ProptestConfig::default()
    })]

    // -------------------------------------------------------------------
    // Arm P-3.1 — intersect NARROWS: `intersect(a, b)` is contained by
    // BOTH a and b. (Standard intersection invariant.)
    // -------------------------------------------------------------------

    /// RED until G-CORE-3w: `intersect(a, b)` satisfies `a.contains_spec(
    /// &intersect(a, b)) && b.contains_spec(&intersect(a, b))`.
    /// WOULD-FAIL if `intersect` was implemented as union by mistake.
    #[test]
    fn intersect_narrows_proptest(a in arb_spec(), b in arb_spec()) {
        let ab = match intersect(&a, &b) {
            Ok(s) => s,
            Err(SubgraphSpecError::EmptyIntersection) => return Ok(()), // degenerate; not a violation
            Err(e) => return Err(TestCaseError::Fail(
                format!("intersect failed unexpectedly: {:?}", e).into(),
            )),
        };
        prop_assert!(a.contains_spec(&ab), "intersect ⊆ a (P-3.1)");
        prop_assert!(b.contains_spec(&ab), "intersect ⊆ b (P-3.1)");
    }

    // -------------------------------------------------------------------
    // Arm P-3.2 — union WIDENS: BOTH a and b are contained by `union(a, b)`.
    // -------------------------------------------------------------------

    /// RED until G-CORE-3w: `union(a, b)` satisfies `union.contains_spec(&a)
    /// && union.contains_spec(&b)`. WOULD-FAIL if `union` was implemented
    /// as intersect by mistake.
    #[test]
    fn union_widens_proptest(a in arb_spec(), b in arb_spec()) {
        let ab = match union(&a, &b) {
            Ok(s) => s,
            Err(e) => return Err(TestCaseError::Fail(
                format!("union failed unexpectedly: {:?}", e).into(),
            )),
        };
        prop_assert!(ab.contains_spec(&a), "a ⊆ union (P-3.2)");
        prop_assert!(ab.contains_spec(&b), "b ⊆ union (P-3.2)");
    }

    // -------------------------------------------------------------------
    // Arm P-3.3 — filter NARROWS: `filter(a, predicate)` is contained by a.
    // -------------------------------------------------------------------

    /// RED until G-CORE-3w: `filter(a, p)` is always contained by a; the
    /// predicate can only narrow. WOULD-FAIL if filter loosened constraints.
    #[test]
    fn filter_narrows_proptest(a in arb_spec()) {
        // Synthetic predicate: deny one specific label (always narrows
        // unless the label wasn't present, in which case a == filter(a)).
        let filtered = filter(&a, |spec_builder| {
            spec_builder.deny_label("A")
        }).expect("filter compose");
        prop_assert!(
            a.contains_spec(&filtered),
            "filter ⊆ a (P-3.3 / narrowing-only)"
        );
    }

    // -------------------------------------------------------------------
    // Arm P-3.4 — `intersect` is commutative.
    // -------------------------------------------------------------------

    /// RED until G-CORE-3w: `intersect(a, b)` canonical bytes ==
    /// `intersect(b, a)` canonical bytes. (Commutativity holds at the
    /// content-addressed shape level.)
    #[test]
    fn intersect_commutative_proptest(a in arb_spec(), b in arb_spec()) {
        let ab = intersect(&a, &b);
        let ba = intersect(&b, &a);
        match (ab, ba) {
            (Ok(s1), Ok(s2)) => {
                let bytes_1 = s1.to_canonical_bytes().unwrap();
                let bytes_2 = s2.to_canonical_bytes().unwrap();
                prop_assert_eq!(
                    bytes_1, bytes_2,
                    "intersect commutative on canonical bytes (P-3.4)"
                );
            }
            (Err(_), Err(_)) => {
                // Both empty — degenerate but consistent.
            }
            (other_a, other_b) => {
                return Err(TestCaseError::Fail(
                    format!("intersect not commutative: ab={:?}, ba={:?}", other_a, other_b).into(),
                ));
            }
        }
    }

    // -------------------------------------------------------------------
    // Arm P-3.5 — `intersect` is associative on byte-equality.
    // -------------------------------------------------------------------

    /// RED until G-CORE-3w: `intersect(intersect(a, b), c)` ==
    /// `intersect(a, intersect(b, c))` on canonical bytes.
    #[test]
    fn intersect_associative_proptest(a in arb_spec(), b in arb_spec(), c in arb_spec()) {
        let ab_c = match intersect(&a, &b).and_then(|ab| intersect(&ab, &c)) {
            Ok(s) => s,
            Err(_) => return Ok(()), // degenerate; skip
        };
        let a_bc = match intersect(&b, &c).and_then(|bc| intersect(&a, &bc)) {
            Ok(s) => s,
            Err(_) => return Ok(()),
        };
        let bytes_left = ab_c.to_canonical_bytes().unwrap();
        let bytes_right = a_bc.to_canonical_bytes().unwrap();
        prop_assert_eq!(
            bytes_left, bytes_right,
            "intersect associative on canonical bytes (P-3.5)"
        );
    }
}

// ---------------------------------------------------------------------------
// Arm P-4 — Authoring ergonomics: macros produce the same SubgraphSpec
// as the raw struct-literal. (Not a proptest — a single positive pin.)
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: the `query!` / `walk!` macros produce a Spec
/// whose canonical bytes equal the raw `Spec::builder()` chain. (Spike F
/// flagged raw struct-literal as painful; the macros must be a pure
/// syntactic affordance, not a semantic divergence.)
#[test]
fn macros_byte_equal_raw_builder() {
    use benten_core::subgraph_spec::query; // RED: macro doesn't exist yet

    let r = cid_for("macro-R");
    let raw = Spec::builder()
        .with_root(r)
        .with_max_depth(3)
        .with_label_allowlist(["Recipe"])
        .build()
        .expect("raw spec");

    // The macro form is the same logical Spec.
    let via_macro = query!(roots: [r], max_depth: 3, allow_labels: ["Recipe"]).expect("macro spec");

    let bytes_raw = raw.to_canonical_bytes().unwrap();
    let bytes_macro = via_macro.to_canonical_bytes().unwrap();
    assert_eq!(
        bytes_raw, bytes_macro,
        "query! macro byte-equal raw builder (P-4)"
    );
}

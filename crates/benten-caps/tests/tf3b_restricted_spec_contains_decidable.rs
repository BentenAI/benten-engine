//! TF-3b (R3-W2) — `RestrictedScope::contains()` decidable across all 6 dimensions.
//!
//! Family: TF-3b — G-CORE-3b `benten_caps::RestrictedScope` shape pins.
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3b def
//! (line 333) + §1.A.FROZEN item 15(b) (lines 133, 138-139) + §5 D-list
//! row D-4M-R1 (line 397).
//!
//! Ratified inputs (cannot be re-debated; cite the record):
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §R1 ("Path (a) restricted-spec language only"; Path (b) refinement-
//!   witness over opaque specs is structurally unsound) + Spike H+1.1.
//!
//! R2-seed: `R2-test-landscape.md` §2 G-CORE-3b (P-1) — "RestrictedScope::
//! contains(other) decidable across all 6 dimensions (roots + edge-allowlist
//! + max_depth + label-allowlist + label-denylist + property-equalities);
//! composition with `&&` is correct. Spike H+1.1's 7 use cases (A-G) each
//! round-trip through `contains` with the documented outcome."
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3b
//! production surface = `benten_caps::restricted_spec::RestrictedScope` +
//! its inherent `pub fn contains(&self, other: &Self) -> bool` and the
//! six per-dimension predicate constructors. The R5 G-CORE-3b implementer
//! mints those files (`crates/benten-caps/src/restricted_spec.rs`) and
//! un-ignores these pins per pim-12 (§3.6e).
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NON-NEGOTIABLE R3 BRIEF-TEMPLATE CHECKLIST (literal pre-flight lines):
//!  1. §3.5b HARDENED (pim-1) — public-shape change ⇒ adjacent-doc sweep.
//!     N/A in R3 (the G-CORE-3b implementer owns ENGINE-SPEC / SECURITY-
//!     POSTURE / GLOSSARY retense when the surface lands).
//!  2. §3.6b + sub-rule 4 (pim-2 + amendment) — every pin is a production-
//!     arm against the (post-G-CORE-3b) `RestrictedScope::contains`, with
//!     an OBSERVABLE byte/bool consequence, WOULD-FAIL-IF-NO-OP'd for the
//!     specific 6-dim arm (not an umbrella sentinel).
//!  3. §3.6e (pim-12) — every arm is `#[ignore]`d with the literal
//!     "un-ignore at G-CORE-3b" marker; the G-CORE-3b mini-reviewer MUST
//!     verify landing-status (un-ignore), not just spec-pin presence.
//!  4. §3.6f (pim-18) — SHAPE-not-SUBSTANCE: each test body exercises the
//!     real (future) production `contains` over the SPECIFIC dimension +
//!     asserts an observable bool consequence; NO aspirational-prose-only
//!     arm; production call-site is `RestrictedScope::contains`.
//!  5. §3.5g — cross-language rule-mirror N/A here (RestrictedScope is a
//!     Rust-internal cap surface for v1-beta; future TS mirror lives with
//!     PR-B / freeze).
//!  6. §3.5i — mini-reviewer FIRST action = tree-state-freshness vs
//!     merge-base. (Directive to the G-CORE-3b mini-reviewer.)
//!  7. §3.5j — pre-push runs `cargo +stable clippy --workspace
//!     --all-targets -- -D warnings` IN ADDITION to MSRV 1.95 clippy.
//!  8. §3.6g — (this checklist) prior-phase pim-N reproduced literally.
//!  9. §3.6h — N/A: this file codifies no new rule.
//! 10. §3.6i — R3 report JSON canonical schema applies to the W2 return.
//! 11. §3.6j — sweep-completeness validator: N/A at R3 file scope.
//! 12. §3.13 — per-test static decomposition: this file introduces ZERO
//!     statics; every test constructs `RestrictedScope`s locally.
//! 13. §3.5h — base pre-push 5-check; the W2 return JSON must `jq .`-validate.
//! 14. §3.11 — checkpoint-pre-flight: N/A (small file).
//! 15. §3.5l — mega-batch combined-branch verify directive to orchestrator.
//! 16. §3.5m — fork-disposition: P-I (RestrictedScope is part of the v1
//!     FROZEN-INTERFACE per §1.A.FROZEN item 15(b) — no dual track).
//! 17. §3.5n — orchestrator ground-truth-verifies every finding.
//! 18. Iterate-to-convergence (CLAUDE.md rule 9) — R3 single-pass.
//! 19. Canary-first — G-CORE-3b waits on G-CORE-3a canary (§3.13 budget).
//! ─────────────────────────────────────────────────────────────────────────
//!
//! SHAPE-flag-don't-fake: the `benten_caps::restricted_spec` module does
//! NOT exist at HEAD. These tests intentionally reference it so they
//! compile-fail NOW and behaviour-pin the contract once the surface lands.
//! Do NOT stub the module to make this compile — that defeats RED phase
//! (pim-18 SHAPE-not-SUBSTANCE).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
// RED: `benten_caps::restricted_spec` does NOT exist at HEAD. G-CORE-3b
// creates it with `RestrictedScope` carrying the 6 dimensions: roots,
// edge_allowlist, max_depth, label_allowlist, label_denylist,
// property_equalities (per §1.A.FROZEN item 15(b) + D-4M-R1).
use benten_caps::restricted_spec::{PropertyValue, RestrictedScope};

fn sample_root(label: &str) -> Cid {
    // Use the public, always-available `from_blake3_digest` constructor
    // rather than the `Cid::sample_for_label` test-helper (gated on the
    // `testing` feature, which benten-caps does not enable). BLAKE3 of
    // the label gives the same deterministic-by-label property without
    // pulling in a feature dep.
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

// ---------------------------------------------------------------------------
// Arm P-1.1 — roots dimension: subset-of-roots ⇒ contains-narrowing.
// (Spike H+1.1 use case A: producer narrows roots set; recipient's spec
// roots ⊆ producer's; contains() == true.)
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): a `RestrictedScope` whose roots are a STRICT SUBSET
/// of `parent`'s roots is contained by `parent`. WOULD-FAIL if the roots
/// dimension is omitted from `contains` (umbrella-sentinel masking).
#[test]
fn contains_roots_subset_narrows() {
    let r1 = sample_root("root-1");
    let r2 = sample_root("root-2");
    let r3 = sample_root("root-3");

    let parent = RestrictedScope::new().with_roots(vec![r1, r2, r3]);
    let child = RestrictedScope::new().with_roots(vec![r1, r2]);

    // Subset on roots → contains == true.
    assert!(
        parent.contains(&child),
        "roots subset must be contained (P-1.1)"
    );
    // Superset on roots → contains == false (widening rejected).
    let widened = RestrictedScope::new().with_roots(vec![r1, r2, r3, sample_root("root-extra")]);
    assert!(
        !parent.contains(&widened),
        "roots superset must NOT be contained (widening rejected)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.2 — edge-allowlist dimension: subset narrows.
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): edge_allowlist subset ⇒ contains; superset ⇒ NOT
/// contains. WOULD-FAIL if the edge-allowlist dim is collapsed into
/// label-allowlist.
#[test]
fn contains_edge_allowlist_subset_narrows() {
    let parent = RestrictedScope::new()
        .with_edge_allowlist(vec!["VERSION_OF".to_string(), "AUTHORED_BY".to_string()]);
    let child = RestrictedScope::new().with_edge_allowlist(vec!["VERSION_OF".to_string()]);

    assert!(
        parent.contains(&child),
        "edge_allowlist subset narrows (P-1.2)"
    );
    let widened = RestrictedScope::new().with_edge_allowlist(vec![
        "VERSION_OF".to_string(),
        "AUTHORED_BY".to_string(),
        "INSTALL_RECORD".to_string(),
    ]);
    assert!(
        !parent.contains(&widened),
        "edge_allowlist widening rejected"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.3 — max_depth dimension: lower depth narrows.
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): max_depth(child) <= max_depth(parent) ⇒ contains;
/// max_depth(child) > max_depth(parent) ⇒ NOT contains.
#[test]
fn contains_max_depth_subset_narrows() {
    let parent = RestrictedScope::new().with_max_depth(8);
    let child = RestrictedScope::new().with_max_depth(4);
    let widened = RestrictedScope::new().with_max_depth(9);

    assert!(parent.contains(&child), "max_depth narrows (P-1.3)");
    assert!(
        !parent.contains(&widened),
        "max_depth widening rejected (P-1.3)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.4 — label-allowlist dimension.
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): label_allowlist subset narrows.
#[test]
fn contains_label_allowlist_subset_narrows() {
    let parent = RestrictedScope::new()
        .with_label_allowlist(vec!["Recipe".to_string(), "Ingredient".to_string()]);
    let child = RestrictedScope::new().with_label_allowlist(vec!["Recipe".to_string()]);

    assert!(
        parent.contains(&child),
        "label_allowlist subset narrows (P-1.4)"
    );
    let widened = RestrictedScope::new().with_label_allowlist(vec![
        "Recipe".to_string(),
        "Ingredient".to_string(),
        "Equipment".to_string(),
    ]);
    assert!(
        !parent.contains(&widened),
        "label_allowlist widening rejected"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.5 — label-denylist dimension: superset of denied labels narrows.
// (A denylist gets STRICTER by ADDING entries — the inverse semantic of
// allowlist. Critical to test the inverse-direction reasoning is correct.)
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): label_denylist SUPERSET narrows (more denied =
/// narrower). label_denylist SUBSET widens (fewer denied = wider).
/// WOULD-FAIL if the implementer applied subset-semantics to denylist
/// (a critical inverse-direction bug class).
#[test]
fn contains_label_denylist_superset_narrows() {
    let parent = RestrictedScope::new().with_label_denylist(vec!["Draft".to_string()]);
    let child = RestrictedScope::new()
        .with_label_denylist(vec!["Draft".to_string(), "Internal".to_string()]);

    // child denies MORE labels → child is narrower → parent contains it.
    assert!(
        parent.contains(&child),
        "label_denylist SUPERSET narrows (P-1.5 inverse)"
    );
    // child denies FEWER labels → child is WIDER → NOT contained.
    let widened = RestrictedScope::new().with_label_denylist(vec![]);
    assert!(
        !parent.contains(&widened),
        "label_denylist SUBSET widens (rejected)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.6 — property_equalities dimension: superset (more equalities)
// narrows; equalities must MATCH exactly on overlapping keys.
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): property_equalities SUPERSET (more required-equal
/// properties) narrows. Conflicting values on an overlapping key ⇒ NOT
/// contained.
#[test]
fn contains_property_equalities_superset_narrows_and_conflicts_reject() {
    let parent = RestrictedScope::new()
        .with_property_equality("status".to_string(), PropertyValue::text("published"));
    let child = RestrictedScope::new()
        .with_property_equality("status".to_string(), PropertyValue::text("published"))
        .with_property_equality("language".to_string(), PropertyValue::text("en"));

    // child requires MORE equalities → narrower → contained.
    assert!(
        parent.contains(&child),
        "property_equalities SUPERSET narrows (P-1.6)"
    );

    // Conflicting value on shared key ⇒ not contained.
    let conflict = RestrictedScope::new()
        .with_property_equality("status".to_string(), PropertyValue::text("draft"));
    assert!(
        !parent.contains(&conflict),
        "property_equalities value-conflict rejected (P-1.6)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.7 — composition with `&&` correct (Spike H+1.1's per-dim
// composability claim).
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): a multi-dimensional child (narrowed on >=2 dims)
/// is contained by the corresponding multi-dimensional parent IFF each
/// per-dim contains-check passes. WOULD-FAIL if `contains` short-circuits
/// any one dim or uses `||` instead of `&&`.
#[test]
fn contains_composes_per_dimension_with_and() {
    let r = sample_root("compose-root");
    let parent = RestrictedScope::new()
        .with_roots(vec![r])
        .with_max_depth(8)
        .with_label_allowlist(vec!["Recipe".to_string(), "Ingredient".to_string()]);

    // Narrowed on roots (same), depth (4 ≤ 8), labels (subset) — contained.
    let child_all_narrow = RestrictedScope::new()
        .with_roots(vec![r])
        .with_max_depth(4)
        .with_label_allowlist(vec!["Recipe".to_string()]);
    assert!(
        parent.contains(&child_all_narrow),
        "all-dimensions narrowed ⇒ contained"
    );

    // Narrowed on labels + depth, but WIDENED on roots ⇒ NOT contained.
    let child_widens_roots = RestrictedScope::new()
        .with_roots(vec![r, sample_root("extra-root")])
        .with_max_depth(4)
        .with_label_allowlist(vec!["Recipe".to_string()]);
    assert!(
        !parent.contains(&child_widens_roots),
        "one-dim widening breaks AND-composition (P-1.7)"
    );
}

// ---------------------------------------------------------------------------
// Arm P-1.8 — reflexivity: a spec contains itself.
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): `s.contains(&s) == true` for every constructable
/// RestrictedScope (reflexivity is required for chain validation's
/// no-op-step admission).
#[test]
fn contains_is_reflexive() {
    let r = sample_root("reflexive-root");
    let s = RestrictedScope::new()
        .with_roots(vec![r])
        .with_max_depth(5)
        .with_label_allowlist(vec!["Recipe".to_string()])
        .with_label_denylist(vec!["Draft".to_string()])
        .with_edge_allowlist(vec!["VERSION_OF".to_string()])
        .with_property_equality("status".to_string(), PropertyValue::text("published"));

    assert!(s.contains(&s), "contains MUST be reflexive (P-1.8)");
}

// ---------------------------------------------------------------------------
// Arm P-1.9 — transitivity: if A contains B and B contains C then A
// contains C. (Chain validator depends on this for >2-step chains.)
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): contains is transitive across the 6-dim product.
/// WOULD-FAIL if any dimension's check breaks ordering (e.g. an unsigned-
/// vs-signed depth comparison off-by-one).
#[test]
fn contains_is_transitive() {
    let a = RestrictedScope::new()
        .with_max_depth(10)
        .with_label_allowlist(
            vec!["Recipe", "Ingredient", "Equipment", "Step"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
    let b = RestrictedScope::new()
        .with_max_depth(6)
        .with_label_allowlist(
            vec!["Recipe", "Ingredient", "Step"]
                .into_iter()
                .map(String::from)
                .collect(),
        );
    let c = RestrictedScope::new()
        .with_max_depth(3)
        .with_label_allowlist(vec!["Recipe".to_string()]);

    assert!(a.contains(&b), "a contains b");
    assert!(b.contains(&c), "b contains c");
    assert!(a.contains(&c), "transitivity: a contains c (P-1.9)");
}

// ---------------------------------------------------------------------------
// Arm P-1.10 — Spike H+1.1 use case smoke: 7 documented A-G outcomes.
// (A single per-dim arm per use case; the per-use-case detail lives in
// the spike doc; this pin asserts the named outcomes survive contains.)
// ---------------------------------------------------------------------------

/// LANDED at G-CORE-3b (pim-12 / §3.6e closure): Spike H+1.1 documented use-case A (roots
/// narrowing) survives contains. Companion arms B-G live above; this
/// pin is the "spike cases round-trip" marker per R2 §2 G-CORE-3b (P-1).
#[test]
fn spike_h_one_one_use_case_a_roots_narrowing_round_trips() {
    let r1 = sample_root("A1");
    let r2 = sample_root("A2");
    let parent = RestrictedScope::new().with_roots(vec![r1, r2]);
    let child = RestrictedScope::new().with_roots(vec![r1]);
    assert!(
        parent.contains(&child),
        "Spike H+1.1 use case A: roots-subset narrows"
    );
}

//! TF-3b (R3-W2) — Chain-validator narrowing pins (R1 chain non-widening).
//!
//! Family: TF-3b — G-CORE-3b chain validator (Path (a) restricted-spec only).
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3b def
//! (line 333) + §1.A.FROZEN item 15(b/c) (lines 133-134) + §5 D-list
//! D-4M-R1 (line 397).
//!
//! Ratified inputs:
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §R1 — "SubgraphSelector chain non-widening shape = Path (a)
//!   restricted-spec language only" + Spike H+1.1 (Path (b) refinement-
//!   witness over opaque specs structurally unsound).
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3b (P-2) — "Chain validator:
//! a delegation chain that *narrows* RestrictedSpec is accepted; one that
//! *widens* is rejected with typed `ChainNotNarrowing`."
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3b
//! production surface = `benten_caps::chain_validator::
//! validate_chain_narrowing(...)` returning `Ok` for monotonic-narrowing
//! chains and typed `ChainNotNarrowing { step_index, ... }` for any
//! widening step. The R5 implementer mints
//! `crates/benten-caps/src/chain_validator.rs` (NEW; distinct from the
//! existing `chain_authority.rs` envelope-ceiling seam — this is the
//! structured-`Scope`/`RestrictedSpec` validator) and un-ignores these
//! pins.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Full R3 BRIEF-TEMPLATE CHECKLIST reproduced in sibling
//! `tf3b_restricted_spec_contains_decidable.rs` head comment; abbreviated
//! here for file-length hygiene. All 19 rules apply.
//! SHAPE-flag-don't-fake: surface does NOT exist at HEAD.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
// RED: `benten_caps::chain_validator` does NOT exist at HEAD.
use benten_caps::chain_validator::{
    ChainValidationError, ChainValidatorOutcome, validate_chain_narrowing,
};
use benten_caps::restricted_spec::RestrictedSpec;
use benten_caps::scope::Scope;

fn root_cid(label: &str) -> Cid {
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

fn scope_with_max_depth(d: u32) -> Scope {
    Scope::RestrictedSelector(RestrictedSpec::new().with_max_depth(d))
}

fn scope_with_labels(labels: &[&str]) -> Scope {
    Scope::RestrictedSelector(
        RestrictedSpec::new().with_label_allowlist(labels.iter().map(|s| s.to_string()).collect()),
    )
}

// ---------------------------------------------------------------------------
// Arm P-2.1 — Monotonic-narrowing chain accepted.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: a chain `Root → Step1 → Step2` where each step's
/// `RestrictedSelector` is contained by its predecessor's is admitted by
/// `validate_chain_narrowing` (`Ok(Admitted)`). The narrowing is along
/// the max_depth dimension (10 → 6 → 3) — covered by P-1.3.
#[test]
#[ignore = "un-ignore at G-CORE-3b: chain-validator narrowing accept (P-2.1)"]
fn narrowing_chain_admitted() {
    let chain = vec![
        scope_with_max_depth(10),
        scope_with_max_depth(6),
        scope_with_max_depth(3),
    ];
    let outcome = validate_chain_narrowing(&chain).expect("monotonic-narrow chain ⇒ Ok");
    assert!(
        matches!(outcome, ChainValidatorOutcome::Admitted),
        "monotonic narrowing chain must produce Admitted (P-2.1); got {:?}",
        outcome
    );
}

// ---------------------------------------------------------------------------
// Arm P-2.2 — Widening step rejected with typed `ChainNotNarrowing`.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: a chain whose middle step WIDENS the spec
/// (max_depth 4 → 6) is rejected with typed `ChainNotNarrowing`,
/// carrying `step_index` of the offending edge. WOULD-FAIL if validator
/// returns `Ok` (the load-bearing chain non-widening invariant).
#[test]
#[ignore = "un-ignore at G-CORE-3b: chain-validator widening reject typed (P-2.2)"]
fn widening_step_rejected_with_typed_chain_not_narrowing() {
    let chain = vec![
        scope_with_max_depth(8),
        scope_with_max_depth(4), // narrows
        scope_with_max_depth(6), // WIDENS — must reject
        scope_with_max_depth(2), // narrows
    ];
    let err = validate_chain_narrowing(&chain).expect_err("widening step ⇒ Err");
    match err {
        ChainValidationError::ChainNotNarrowing { step_index, .. } => {
            assert_eq!(
                step_index, 2,
                "step_index must name the offending edge (4→6 at index 2)"
            );
        }
        other => panic!("expected ChainNotNarrowing; got {:?}", other),
    }
}

// ---------------------------------------------------------------------------
// Arm P-2.3 — Mixed-dim narrowing: ALL dimensions must narrow; widening
// ANY single dimension breaks the chain.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: a chain step that narrows depth but WIDENS the
/// label allowlist is rejected. WOULD-FAIL if validator only checks one
/// dimension (a partial-check bug class that R2 §2 P-2 specifically pins).
#[test]
#[ignore = "un-ignore at G-CORE-3b: chain-validator one-dim-widens-rejects (P-2.3)"]
fn mixed_dim_one_widening_rejects() {
    let parent_scope = Scope::RestrictedSelector(
        RestrictedSpec::new()
            .with_max_depth(8)
            .with_label_allowlist(vec!["Recipe".to_string()]),
    );
    let widens_labels = Scope::RestrictedSelector(
        RestrictedSpec::new()
            .with_max_depth(4) // narrows
            .with_label_allowlist(vec!["Recipe".to_string(), "Ingredient".to_string()]), // WIDENS
    );
    let chain = vec![parent_scope, widens_labels];
    let err = validate_chain_narrowing(&chain).expect_err("one-dim widening ⇒ Err");
    assert!(
        matches!(err, ChainValidationError::ChainNotNarrowing { .. }),
        "got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Arm P-2.4 — Hashes-scope chain non-widening (Scope::Hashes arm).
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: a `Scope::Hashes` chain where each step's hash
/// set is a SUBSET of its predecessor narrows; ADDING a hash widens.
#[test]
#[ignore = "un-ignore at G-CORE-3b: Scope::Hashes chain narrowing arm (P-2.4)"]
fn hashes_scope_chain_widening_rejects() {
    let h1 = root_cid("hash-1");
    let h2 = root_cid("hash-2");
    let h3 = root_cid("hash-3");

    let parent = Scope::Hashes(vec![h1, h2, h3]);
    let narrow = Scope::Hashes(vec![h1, h2]); // subset OK
    let widens = Scope::Hashes(vec![h1, h2, h3, root_cid("hash-extra")]); // adds — must fail

    let ok_chain = vec![parent.clone(), narrow];
    assert!(
        validate_chain_narrowing(&ok_chain).is_ok(),
        "subset-of-hashes ⇒ Ok (P-2.4)"
    );

    let bad_chain = vec![parent, widens];
    let err = validate_chain_narrowing(&bad_chain).expect_err("hash-widen ⇒ Err");
    assert!(
        matches!(err, ChainValidationError::ChainNotNarrowing { .. }),
        "got {:?}",
        err
    );
}

// ---------------------------------------------------------------------------
// Arm P-2.5 — Empty chain rejected (degenerate). Single-step chain
// accepted (no edge to widen).
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: empty chain ⇒ typed `EmptyChain`. Single-step
/// chain ⇒ `Admitted` (no edges, no widening possible).
#[test]
#[ignore = "un-ignore at G-CORE-3b: chain-validator empty + single-step (P-2.5)"]
fn empty_chain_rejected_single_step_admitted() {
    let empty: Vec<Scope> = vec![];
    let err = validate_chain_narrowing(&empty).expect_err("empty chain ⇒ Err");
    assert!(
        matches!(err, ChainValidationError::EmptyChain),
        "got {:?}",
        err
    );

    let single = vec![scope_with_labels(&["Recipe"])];
    let ok = validate_chain_narrowing(&single).expect("single-step ⇒ Ok");
    assert!(
        matches!(ok, ChainValidatorOutcome::Admitted),
        "single-step ⇒ Admitted (no edge to widen)"
    );
}

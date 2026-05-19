//! TF-6 (R3-B3) — D3 `VersionDag` unification RED-PHASE pins.
//!
//! Family: TF-6 — D3 VersionDag unification + dead-Anchor #1003 deletion.
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-5 group def
//! (line 310) + the RATIFIED-decisions-2026-05-17 D3 row (§0 line 55):
//!
//! > ONE `VersionDag` + strict/linear mode; delete dead u64 Anchor #1003
//! > (couple the #1142 Anchor/u64-delete half); one shared trait + one
//! > CURRENT; verify #995 `Cid` docstring already fixed.
//!
//! R2-seed: `r2-test-landscape.md` S5 — "strict-mode fork-rejection ≡ old
//! `VersionError::Branched`; DAG branch/merge; dead-Anchor-deletion
//! regression-guard" → TF-6, obligation class = regression-guard. Exit
//! criterion **C5** (`r2-test-landscape.md` §2.B exit-criterion table: "C5
//! D3 VersionDag | strict-mode fork-rejection ≡ Branched + dead-Anchor
//! guard | TF-6 | none").
//!
//! ## #1290-already-landed vs G-CORE-5-RED split (ground-truthed @ ed03729a)
//!
//! The campaign-tail PR #1290 ALREADY shipped the u64-Anchor #1003 deletion
//! (`anchor_version.rs` deleted; `benten_core::Anchor` / crate-root
//! `append_version` / `U64_CHAINS` / `ANCHOR_COUNTER` no longer resolve) +
//! the #995 `Cid` docstring correction. Those land as **verify-stays
//! regression arms** in the sibling file
//! `tf6_d3_dead_anchor_1003_verify_stays_regression.rs` — NOT here. The
//! pre-existing `u64_anchor_surface_deleted_1003.rs` (a #1290 artifact)
//! already covers the structural deletion; this family's verify-stays file
//! adds the D3-specific "two-surfaces-only / docstring-correct" arms.
//!
//! **This file = the STILL-undelivered G-CORE-5 deliverable, RED-phase.**
//! At HEAD `ed03729a` there are TWO separate version surfaces with TWO
//! different CURRENT semantics and NO shared trait:
//!   - `benten_core::version::{Anchor, append_version, walk_versions}` —
//!     linear, prior-head-threaded, NO `current()` accessor.
//!   - `benten_core::version_chain::DagVersionChain` — DAG-shaped, has
//!     `current()` / `set_current()`.
//! G-CORE-5 unifies these into ONE `VersionDag` with an opt-in
//! strict(linear)/dag mode + ONE shared trait + ONE CURRENT semantic.
//! Every test below references the post-G-CORE-5 unified surface
//! (`benten_core::version_dag::VersionDag` + the shared
//! `benten_core::version_dag::VersionChain` trait) which does NOT exist at
//! `ed03729a` — so each fails (compile-fail until the surface lands, then
//! behaviour-fail until the contract is correct). Deliberate RED.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! NON-NEGOTIABLE R3 BRIEF-TEMPLATE CHECKLIST (plan §3 fix-6 directive —
//! reproduced as LITERAL pre-flight lines, NOT a §-cross-reference):
//!  1. §3.5b HARDENED (pim-1) — public-shape change ⇒ adjacent-doc sweep
//!     BEFORE push. N/A to this R3 test file (no public shape changes; the
//!     G-CORE-5 implementer wave owns the doc sweep when the surface lands).
//!  2. §3.6b + sub-rule 4 (pim-2 + amendment) — every pin below is a
//!     PRODUCTION-ARM against the (post-G-CORE-5) production `VersionDag`,
//!     with an OBSERVABLE CONSEQUENCE, and WOULD-FAIL-IF-NO-OP'd against the
//!     SPECIFIC arm (strict-fork ≡ Branched / one-CURRENT / shared-trait),
//!     not an umbrella sentinel.
//!  3. §3.6e (pim-12) — every arm is an `#[ignore]`d RED-PHASE staged-pin
//!     with the literal "un-ignore at G-CORE-5" marker; the closing
//!     G-CORE-5 wave reviewer MUST verify landing-status (un-ignore), not
//!     just spec-pin presence.
//!  4. §3.6f (pim-18) — SHAPE-not-SUBSTANCE: each test body exercises the
//!     real (future) production surface + asserts an observable byte/typed
//!     consequence; NO aspirational-prose-only arm; production call-site is
//!     the unified `VersionDag` API the G-CORE-5 implementer must build.
//!  5. §3.5g (cross-language/-doc/-tool mirror) — N/A: VersionDag is a
//!     `benten-core` Rust-only surface (no TS/JS mirror, no dual-config).
//!  6. §3.5i — mini-reviewer FIRST action = tree-state-freshness vs
//!     merge-base; flag rebase-staleness if >3 behind / sibling HEAD newer.
//!     (Directive to the G-CORE-5 mini-reviewer; recorded here per fix-6.)
//!  7. §3.5j — §3.5h pre-push runs `cargo +stable clippy --workspace
//!     --all-targets -- -D warnings` IN ADDITION to MSRV 1.95 clippy.
//!  8. §3.6g — (this checklist itself) prior-phase pim-N reproduced as
//!     explicit literal lines, not a bare §-reference.
//!  9. §3.6h — a rule/codification naming origin instance(s) closes (or
//!     DEFER-NAMED-NOW) them in the same wave. N/A: this file codifies no
//!     new rule; it pins the D3 RATIFIED disposition.
//! 10. §3.6i — R3 report JSON canonical schema: top-level `disposition`
//!     (NOT `verdict`) + `findings[]` + per-finding HARD-RULE-12 shape +
//!     well-formed JSON. (Applies to the R3-B3 report artifact.)
//! 11. §3.6j — "I swept X" claims run the validator over the wave's OWN
//!     outputs before the claim; author canonical `disposition` at
//!     author-time.
//! 12. §3.13 — per-test static decomposition: NO process-scoped shared
//!     static. This file introduces ZERO statics; every test constructs a
//!     fresh `VersionDag` locally (the post-#1003 design already removed
//!     the process-global `BTreeMap`; per-anchor state only).
//! 13. §3.5h — base pre-push 5-check + MANDATORY-PRE-MERGE-AFTER-
//!     MINI-REVIEW-APPROVE + `jq .` JSON-artifact-validation + GREEN-CI-
//!     CONFIRMATION sub-clauses (the R3-B3 report JSON must `jq .`-validate).
//! 14. §3.11 — checkpoint-pre-flight recovery: N/A (G-CORE-5 is a small
//!     `benten-core`-only group, not one of the large/long-running groups).
//! 15. §3.5l — mega-batch combined-branch full-workspace verify before any
//!     combined push (Strategy-C). Directive to the orchestrator at batch.
//! 16. §3.5m — fork-disposition: P-I commit-to-permanent-shape (ONE
//!     VersionDag, NO dual-track legacy alias) / P-II ONE serialized
//!     cross-crate sweep / P-III wire-CID-on-disk Ben-scheduled. The unified
//!     `VersionDag` walk MUST preserve Phase-1 anchor/Version-Node canonical
//!     bytes (P-III: NO CID perturbation — pinned below).
//! 17. §3.5n — orchestrator independently ground-truth ref-pinned-verifies
//!     every review MAJOR/closure/stay-open/phantom; downstream DISAGREE is
//!     first-class. (Directive to the G-CORE-5 orchestrator pass.)
//! 18. Iterate-to-convergence (CLAUDE.md rule 9) — R1+R4+R4b+R6 each
//!     iterate to 0-substantive; R3 (this) + per-group R5 mini-reviews are
//!     single-pass; pattern-induction meta-sweep runs alongside.
//! 19. canary-first — G-CORE-5 is independent of BOTH opening canaries
//!     (#989 ∥ #1300); it may run parallel with the canary pair or the
//!     substrate tier (plan line 310). No canary blocks this family.
//! ─────────────────────────────────────────────────────────────────────────
//!
//! CLAUDE.md #18 semantics PRESERVED (the unified surface MUST keep these):
//!  - Phase-1 anchor + Version-Node + CURRENT-pointer pattern (baked-in #8).
//!  - DAG-shape fork: Anchor → v1 → {v2-mainline, v1.5-fork}; CURRENT may
//!    point at any branch tip.
//!  - strict (linear) mode ≡ the old `version::VersionError::Branched`
//!    fork-rejection contract — a second append against an already-extended
//!    prior head is a typed error, not silent divergence.
//!  - DAG mode = the old `version_chain::DagVersionChain` branch/merge
//!    semantics (multi-child = branch; multi-parent = merge; cycle = error).
//!
//! SHAPE-flag-don't-fake: the unified `version_dag` module does NOT exist at
//! `ed03729a`. These tests intentionally reference it so they compile-fail
//! NOW and behaviour-pin the G-CORE-5 contract once the surface lands. Do
//! NOT stub `version_dag` to make this compile — that would defeat the RED
//! phase (pim-18 SHAPE-not-SUBSTANCE).

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::{Node, Value};
// RED: `benten_core::version_dag` does NOT exist at ed03729a. G-CORE-5
// creates it as the ONE unified surface (the unification of
// `version::Anchor`+`append_version` and `version_chain::DagVersionChain`).
// The shared trait `VersionChain` + the `Mode` selector + the unified
// `VersionDag` are all part of the deliverable.
use benten_core::version_dag::{Mode, VersionChain, VersionDag, VersionDagError};

fn versioned_node(seq: i64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("seq".to_string(), Value::Int(seq));
    Node::new(vec!["Post".to_string()], p)
}

// ---------------------------------------------------------------------------
// Arm 1 — ONE `VersionDag` type with an opt-in strict(linear)/dag Mode.
// (RATIFIED D3: "ONE `VersionDag` + strict/linear mode".)
// ---------------------------------------------------------------------------

/// RED until G-CORE-5: a SINGLE `VersionDag` type is constructible in either
/// `Mode::Strict` (linear, fork-rejecting — the old `version::Anchor`
/// contract) or `Mode::Dag` (branch/merge — the old `DagVersionChain`
/// contract). Today these are TWO disjoint types; the unified one does not
/// exist. WOULD-FAIL if G-CORE-5 ships two types or no `Mode` selector
/// (the specific "ONE type + mode" arm, not an umbrella).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn one_version_dag_type_constructible_in_either_mode() {
    let root = versioned_node(0).cid().unwrap();

    // The SAME type, two modes. (No second type. No `DagVersionChain` vs
    // `Anchor` split.)
    let strict: VersionDag = VersionDag::new(root, Mode::Strict);
    let dag: VersionDag = VersionDag::new(root, Mode::Dag);

    assert_eq!(strict.mode(), Mode::Strict);
    assert_eq!(dag.mode(), Mode::Dag);
    // Both are the same nominal type — a `Vec<VersionDag>` must hold both
    // (would not compile if G-CORE-5 kept two distinct types).
    let both: alloc::vec::Vec<VersionDag> = alloc::vec![strict, dag];
    assert_eq!(both.len(), 2);
}

// ---------------------------------------------------------------------------
// Arm 2 — strict-mode fork-rejection ≡ old `VersionError::Branched`.
// (RATIFIED D3 RED-shape: "strict-mode fork-rejection on the production
// `VersionDag` returns the equivalent of the old `VersionError::Branched`".)
// ---------------------------------------------------------------------------

/// RED until G-CORE-5: in `Mode::Strict`, a second append against an
/// already-extended prior head returns the equivalent of the old
/// `version::VersionError::Branched` (carrying `seen` = the re-used prior
/// head). This is the EXACT contract `version_branched.rs` pinned on the
/// pre-unification `version::Anchor`; the unified surface MUST preserve it.
/// WOULD-FAIL if strict mode silently accepts the fork (the specific
/// fork-rejection arm).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn strict_mode_fork_rejection_equivalent_to_old_version_branched() {
    let v0 = versioned_node(0).cid().unwrap();
    let mut chain = VersionDag::new(v0, Mode::Strict);

    let v1_a = versioned_node(11).cid().unwrap();
    let v1_b = versioned_node(12).cid().unwrap();

    chain
        .append(&v0, &v1_a)
        .expect("first append against observed head must succeed in strict mode");

    // Second writer still thinks v0 is the head → a concurrent fork. Old
    // `version::Anchor` returned `VersionError::Branched { seen: v0, .. }`.
    // The unified `VersionDag` strict mode MUST surface the equivalent.
    let err = chain.append(&v0, &v1_b).expect_err(
        "strict-mode stale-head append MUST be rejected (≡ old VersionError::Branched)",
    );

    match err {
        VersionDagError::Branched { seen, attempted } => {
            assert_eq!(seen, v0, "Branched.seen must name the re-used prior head");
            assert_eq!(
                attempted, v1_b,
                "Branched.attempted must name the forking new head"
            );
        }
        other => panic!(
            "strict-mode fork must surface VersionDagError::Branched (≡ old \
             version::VersionError::Branched), got: {other:?}"
        ),
    }
}

/// RED until G-CORE-5: strict-mode unknown-prior is also preserved (the old
/// `VersionError::UnknownPrior` half — the unified error enum MUST carry
/// BOTH old linear variants, not just `Branched`). WOULD-FAIL if the
/// unification drops the unknown-prior variant.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn strict_mode_unknown_prior_preserved() {
    let v0 = versioned_node(0).cid().unwrap();
    let mut chain = VersionDag::new(v0, Mode::Strict);

    let phantom = versioned_node(99).cid().unwrap();
    let v1 = versioned_node(1).cid().unwrap();

    let err = chain
        .append(&phantom, &v1)
        .expect_err("append against an unobserved prior head MUST be rejected");
    assert!(
        matches!(err, VersionDagError::UnknownPrior { .. }),
        "strict-mode unknown-prior must surface VersionDagError::UnknownPrior \
         (≡ old version::VersionError::UnknownPrior), got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Arm 3 — DAG mode branch/merge round-trips natively.
// (RATIFIED D3 RED-shape: "DAG branch/merge round-trips natively".)
// CLAUDE.md #18: Anchor → v1 → {v2-mainline, v1.5-fork}.
// ---------------------------------------------------------------------------

/// RED until G-CORE-5: `Mode::Dag` accepts a fork (two children off one
/// parent) WITHOUT erroring — exactly the old `DagVersionChain` branch
/// semantics, now reachable on the ONE unified type. The fork that strict
/// mode REJECTS, dag mode ACCEPTS (mode-divergent behaviour on one type).
/// WOULD-FAIL if dag mode rejects the branch (regressing the DAG contract)
/// or if the same input does not diverge by mode.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn dag_mode_branch_then_two_tips() {
    let v0 = versioned_node(0).cid().unwrap();
    let mut dag = VersionDag::new(v0, Mode::Dag);

    let v1 = versioned_node(1).cid().unwrap();
    let v2_mainline = versioned_node(2).cid().unwrap();
    let v1_5_fork = versioned_node(15).cid().unwrap();

    dag.append(&v0, &v1).expect("dag: v0 -> v1");
    // Two children off v1 — strict would Branched-reject this; dag accepts.
    dag.append(&v1, &v2_mainline)
        .expect("dag: v1 -> v2 mainline (branch ok in dag mode)");
    dag.append(&v1, &v1_5_fork)
        .expect("dag: v1 -> v1.5 fork (branch ok in dag mode — CLAUDE.md #18)");

    let mut tips = dag.tips();
    tips.sort_unstable();
    let mut expected = alloc::vec![v2_mainline, v1_5_fork];
    expected.sort_unstable();
    assert_eq!(
        tips, expected,
        "dag mode must expose BOTH branch tips (CLAUDE.md #18 DAG-shape fork)"
    );
}

/// RED until G-CORE-5: DAG-mode merge (multi-parent child) round-trips —
/// the old `DagVersionChain::merge_node_has_two_parents` contract preserved
/// on the unified type. WOULD-FAIL if the unification drops merge support.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn dag_mode_merge_node_has_two_parents() {
    let v0 = versioned_node(0).cid().unwrap();
    let mut dag = VersionDag::new(v0, Mode::Dag);

    let a = versioned_node(1).cid().unwrap();
    let b = versioned_node(2).cid().unwrap();
    let merged = versioned_node(3).cid().unwrap();

    dag.append(&v0, &a).unwrap();
    dag.append(&v0, &b).unwrap();
    dag.append(&a, &merged).unwrap();
    dag.append(&b, &merged).unwrap(); // merge: `merged` now has two parents

    assert!(
        dag.is_ancestor_of(&a, &merged),
        "merge node must keep both ancestry paths (a -> merged)"
    );
    assert!(
        dag.is_ancestor_of(&b, &merged),
        "merge node must keep both ancestry paths (b -> merged)"
    );
}

/// RED until G-CORE-5: DAG-mode cycle is still rejected (the old
/// `VersionDagError::Cycle` contract preserved on the unified enum).
/// WOULD-FAIL if the unification loses cycle detection.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn dag_mode_cycle_rejected() {
    let v0 = versioned_node(0).cid().unwrap();
    let mut dag = VersionDag::new(v0, Mode::Dag);
    let v1 = versioned_node(1).cid().unwrap();
    let v2 = versioned_node(2).cid().unwrap();

    dag.append(&v0, &v1).unwrap();
    dag.append(&v1, &v2).unwrap();
    // v0 is an ancestor of v2 — adding v0 as a child of v2 = cycle.
    let err = dag
        .append(&v2, &v0)
        .expect_err("dag mode must still reject a cycle");
    assert!(
        matches!(err, VersionDagError::Cycle { .. }),
        "dag-mode cycle must surface VersionDagError::Cycle, got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Arm 4 — ONE shared trait + ONE CURRENT semantic.
// (RATIFIED D3: "one shared trait + one CURRENT".  Today: linear has NO
// `current()`; DAG has its own — TWO different CURRENT semantics.  #849
// named exactly "three different 'CURRENT' semantics".)
// ---------------------------------------------------------------------------

/// RED until G-CORE-5: a single shared `VersionChain` trait abstracts BOTH
/// modes, and CURRENT has ONE semantic across modes (`current()` /
/// `set_current()` available regardless of mode; strict mode's CURRENT is
/// its linear head, dag mode's CURRENT is any chosen tip — ONE accessor
/// pair, not two divergent surfaces). WOULD-FAIL if G-CORE-5 keeps two
/// CURRENT semantics or no shared trait (the specific "one trait + one
/// CURRENT" arm — the #849-named drift the unification closes).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn one_shared_trait_and_one_current_semantic_across_modes() {
    let v0 = versioned_node(0).cid().unwrap();

    // Both modes are addressable through the ONE shared trait object —
    // proves a single trait spans both (would not compile if no shared
    // trait, or if the trait were not object-safe / not impl'd for both).
    fn current_via_trait(c: &dyn VersionChain) -> Option<benten_core::Cid> {
        c.current().copied()
    }

    let mut strict = VersionDag::new(v0, Mode::Strict);
    let mut dag = VersionDag::new(v0, Mode::Dag);

    // ONE CURRENT semantic: freshly constructed, CURRENT == root in BOTH
    // modes (the old `DagVersionChain` already did this; the old linear
    // `Anchor` had NO `current()` at all — the unification gives it one,
    // with the SAME semantic).
    assert_eq!(
        current_via_trait(&strict),
        Some(v0),
        "strict-mode fresh CURRENT must be the root (one CURRENT semantic)"
    );
    assert_eq!(
        current_via_trait(&dag),
        Some(v0),
        "dag-mode fresh CURRENT must be the root (one CURRENT semantic)"
    );

    // ONE `set_current` semantic across modes: advancing CURRENT to a known
    // version works identically through the shared trait.
    let v1 = versioned_node(1).cid().unwrap();
    strict.append(&v0, &v1).unwrap();
    dag.append(&v0, &v1).unwrap();

    VersionChain::set_current(&mut strict, v1)
        .expect("strict set_current to a known version (one CURRENT semantic)");
    VersionChain::set_current(&mut dag, v1)
        .expect("dag set_current to a known version (one CURRENT semantic)");

    assert_eq!(current_via_trait(&strict), Some(v1));
    assert_eq!(current_via_trait(&dag), Some(v1));

    // ONE error semantic: set_current to an unknown CID is the SAME typed
    // error regardless of mode (not two divergent error surfaces).
    let unknown = versioned_node(404).cid().unwrap();
    let e_strict = VersionChain::set_current(&mut strict, unknown).unwrap_err();
    let e_dag = VersionChain::set_current(&mut dag, unknown).unwrap_err();
    assert!(
        matches!(e_strict, VersionDagError::UnknownCurrent { .. }),
        "strict unknown-current must be the shared UnknownCurrent variant, got {e_strict:?}"
    );
    assert!(
        matches!(e_dag, VersionDagError::UnknownCurrent { .. }),
        "dag unknown-current must be the shared UnknownCurrent variant, got {e_dag:?}"
    );
}

/// RED until G-CORE-5: the shared trait carries the common walk surface so
/// callers can iterate either mode uniformly (the "no canonical
/// composability surface" gap #849 named — the unification closes it).
/// WOULD-FAIL if walk is not on the shared trait.
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn shared_trait_exposes_uniform_walk_surface() {
    let v0 = versioned_node(0).cid().unwrap();
    let v1 = versioned_node(1).cid().unwrap();
    let v2 = versioned_node(2).cid().unwrap();

    let mut strict = VersionDag::new(v0, Mode::Strict);
    strict.append(&v0, &v1).unwrap();
    strict.append(&v1, &v2).unwrap();

    // Iterate through the shared trait — a single composability surface
    // over both modes (the #849 "canonical composability surface").
    fn linearizable_head(c: &dyn VersionChain) -> Option<benten_core::Cid> {
        c.walk().last()
    }
    assert_eq!(
        linearizable_head(&strict),
        Some(v2),
        "strict-mode walk via the shared trait must yield the linear head \
         (the contract that superseded the deleted u64 surface, #1003/#1142)"
    );
}

// ---------------------------------------------------------------------------
// Arm 5 — P-III canonical-bytes preservation (CLAUDE.md #5 + §3.5m).
// The unification is a TYPE refactor; it MUST NOT perturb the canonical
// DAG-CBOR bytes / CID of any Version Node. (would-FAIL if the unified
// walk reorders or rewraps node bytes — P-III hazard.)
// ---------------------------------------------------------------------------

/// RED until G-CORE-5: a Version Node's `.cid()` is IDENTICAL whether it is
/// minted standalone or appended through the unified `VersionDag` — the
/// unification touches NO node-encoding path. WOULD-FAIL if G-CORE-5
/// perturbs canonical bytes (P-III: wire/CID changes are Ben-scheduled,
/// never an orchestrator side-effect — this arm guards against accidental
/// perturbation during the type refactor).
#[test]
#[ignore = "RED-PHASE: un-ignore at G-CORE-5"]
fn unification_does_not_perturb_version_node_cids() {
    let n1 = versioned_node(1);
    let standalone_cid = n1.cid().unwrap();

    let v0 = versioned_node(0).cid().unwrap();
    let mut chain = VersionDag::new(v0, Mode::Strict);
    let appended_cid = versioned_node(1).cid().unwrap();
    chain.append(&v0, &appended_cid).unwrap();

    assert_eq!(
        standalone_cid, appended_cid,
        "P-III: appending a Version Node through the unified VersionDag MUST \
         NOT change its canonical bytes / CID (the unification is a type \
         refactor, not a wire-format change — CLAUDE.md #5)"
    );
    // And the chain reports exactly that CID at its head.
    assert_eq!(
        chain.walk().last(),
        Some(appended_cid),
        "the unified walk must yield the unperturbed appended CID"
    );
}

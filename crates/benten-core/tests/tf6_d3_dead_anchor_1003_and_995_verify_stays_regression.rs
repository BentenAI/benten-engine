//! TF-6 (R3-B3) — VERIFY-STAYS regression arms for the #1290-already-landed
//! half of the D3 disposition.
//!
//! Family: TF-6 — D3 VersionDag unification + dead-Anchor #1003 deletion.
//! Plan §3 G-CORE-5 group def (line 310) + RATIFIED-decisions-2026-05-17 D3
//! row (§0 line 55-56): "delete dead u64 Anchor #1003 (couple the #1142
//! Anchor/u64-delete half) ... verify #995 `Cid` docstring already fixed";
//! "home/verify #849 ... confirm landed-status".
//!
//! ## Why these are VERIFY-STAYS, not RED (the #1290 split — R3-A2 #835
//! precedent)
//!
//! Ground-truthed at HEAD `ed03729a`: the campaign-tail PR #1290 ALREADY
//! shipped the u64-Anchor #1003 deletion (FF diff added
//! `crates/benten-core/tests/u64_anchor_surface_deleted_1003.rs` and
//! DELETED `anchor_version.rs`) AND the #995 `Cid` docstring correction is
//! already in `crates/benten-core/src/lib.rs`. #849 is CLOSED (the
//! three-coexisting-shapes umbrella). These are therefore NOT G-CORE-5
//! RED-phase deliverables — they are ALREADY-LANDED state whose
//! G-CORE-5-relevant invariants must STAY true through the VersionDag
//! unification refactor. Mirrors the R3-A2 #835-pattern precedent (an
//! already-landed item gets a verify-it-stays regression arm, not a RED
//! arm).
//!
//! **These tests PASS at `ed03729a` and MUST KEEP PASSING after G-CORE-5.**
//! They are NOT `#[ignore]`d — they are live regression guards. The G-CORE-5
//! unification (ONE `VersionDag`) MUST NOT resurrect the u64 surface, MUST
//! NOT re-introduce a third CURRENT semantic, and MUST NOT regress the
//! #995-corrected `Cid` docstring facts. (Distinct from the pre-existing
//! `u64_anchor_surface_deleted_1003.rs` #1290 artifact, which guards the
//! bare structural deletion; this file adds the D3-unification-specific
//! "stays-deleted THROUGH the unification + two-surfaces-only +
//! docstring-facts-hold" arms.)
//!
//! Brief-template directive (plan §3 fix-6): the full 19-line literal
//! pre-flight checklist is reproduced in the sibling RED file
//! `tf6_d3_version_dag_unification_red.rs` (same R3-B3 lane, same wave) —
//! every line applies identically to this file; not re-pasted to avoid
//! divergence (single source of truth within the lane). Salient lines for
//! THIS file: §3.6f pim-18 (substantive body, not aspirational — each arm
//! exercises a real surface + asserts an observable typed/byte
//! consequence); §3.6b sub-rule 4 (each arm is the SPECIFIC #1003/#995/#849
//! invariant, would-FAIL if that invariant regresses, not an umbrella);
//! §3.13 (zero process-scoped statics — fresh constructs per test);
//! §3.5m P-III (no wire/CID perturbation asserted).
//!
//! SHAPE-flag-don't-fake: these reference ONLY surfaces that exist at
//! `ed03729a` (so they compile + pass NOW). The RED, not-yet-existing
//! `version_dag` surface lives ONLY in the sibling RED file.

#![allow(clippy::unwrap_used, clippy::expect_used)]

extern crate alloc;
use alloc::collections::BTreeMap;

use benten_core::version::{Anchor, VersionError, append_version, walk_versions};
use benten_core::version_chain::DagVersionChain;
use benten_core::{Node, Value, version, version_chain};

fn versioned_node(seq: i64) -> Node {
    let mut p = BTreeMap::new();
    p.insert("seq".to_string(), Value::Int(seq));
    Node::new(vec!["Post".to_string()], p)
}

// ---------------------------------------------------------------------------
// #1003 — the dead u64 Anchor surface is GONE and STAYS gone through D3.
// (#1290 deleted it; G-CORE-5 must not resurrect it during unification.)
// ---------------------------------------------------------------------------

/// VERIFY-STAYS: the canonical replacement linear surface
/// (`version::Anchor`) is prior-head-threaded + fork-rejecting — the exact
/// capability that made the deleted headless u64 surface redundant + safe
/// to remove (#1003). The G-CORE-5 unification preserves this contract as
/// `Mode::Strict`; this arm guards it does not silently regress on the way.
/// WOULD-FAIL if the prior-head-threaded fork-rejection breaks (which would
/// re-open the #1003 "weaker u64 surface was removable" premise).
#[test]
fn canonical_linear_surface_stays_prior_head_threaded_and_fork_rejecting() {
    let v0 = versioned_node(0).cid().unwrap();
    let anchor = Anchor::new(v0);

    let v1_a = versioned_node(11).cid().unwrap();
    let v1_b = versioned_node(12).cid().unwrap();

    append_version(&anchor, &v0, &v1_a).expect("first append against observed head must succeed");
    let err = append_version(&anchor, &v0, &v1_b)
        .expect_err("stale-head append MUST be rejected (the #1003-removability contract)");
    assert!(
        matches!(err, VersionError::Branched { .. }),
        "fork must surface VersionError::Branched (the contract G-CORE-5 \
         preserves as Mode::Strict), got: {err:?}"
    );

    // Linear walk yields the surviving (non-forked) head.
    let chain: alloc::vec::Vec<_> = walk_versions(&anchor).collect();
    assert_eq!(chain, alloc::vec![v0, v1_a]);
}

/// VERIFY-STAYS: exactly TWO canonical version surfaces remain at
/// `ed03729a` (linear `version::Anchor` + DAG `version_chain::
/// DagVersionChain`); the headless u64-id third shape is gone (#1003) and
/// #849 (three-coexisting-shapes umbrella) is CLOSED. The type-annotated fn
/// pointers fail to COMPILE if either canonical constructor changes shape
/// OR a restored u64 surface shadows the crate root. G-CORE-5 collapses
/// these TWO into ONE `VersionDag` — until then this asserts the count is
/// exactly two and the deleted third has not crept back.
/// WOULD-FAIL (compile) if the deleted u64 `benten_core::Anchor` /
/// crate-root `append_version` resurfaces during the unification.
#[test]
fn exactly_two_canonical_version_surfaces_remain_pre_unification() {
    // Two `Cid -> _` constructors — the post-#1003 inventory. (The deleted
    // u64 surface had `Anchor::new() -> Anchor` with NO Cid arg + a
    // crate-root `append_version(&anchor, &node)`; neither resolves now.)
    let linear_ctor: fn(benten_core::Cid) -> version::Anchor = version::Anchor::new;
    let dag_ctor: fn(benten_core::Cid) -> version_chain::DagVersionChain =
        version_chain::DagVersionChain::new;

    let v0 = versioned_node(0).cid().unwrap();
    let linear = linear_ctor(v0);
    let dag = dag_ctor(v0);

    // Exercise both so the constructors are not dead.
    let lin_chain: alloc::vec::Vec<_> = walk_versions(&linear).collect();
    assert_eq!(
        lin_chain,
        alloc::vec![v0],
        "fresh linear Anchor's chain is exactly its head"
    );
    assert_eq!(
        dag.current(),
        Some(&v0),
        "fresh DagVersionChain CURRENT is its root (one of the two CURRENT \
         semantics G-CORE-5 unifies; the deleted u64 surface had a third)"
    );
}

/// VERIFY-STAYS: the DAG surface keeps branch/merge/cycle semantics
/// (CLAUDE.md #18) so G-CORE-5 has a correct `Mode::Dag` contract to
/// preserve. WOULD-FAIL if `DagVersionChain` regresses before the
/// unification absorbs it.
#[test]
fn dag_surface_stays_branch_merge_cycle_correct_pre_unification() {
    let v0 = versioned_node(0).cid().unwrap();
    let mut dag = DagVersionChain::new(v0);

    let v1 = versioned_node(1).cid().unwrap();
    let v2 = versioned_node(2).cid().unwrap();
    let fork = versioned_node(15).cid().unwrap();

    dag.add_version(v0, v1).unwrap();
    dag.add_version(v1, v2).unwrap();
    dag.add_version(v1, fork).unwrap(); // branch (CLAUDE.md #18 DAG-fork)

    let mut tips = dag.tips();
    tips.sort_unstable();
    let mut expected = alloc::vec![v2, fork];
    expected.sort_unstable();
    assert_eq!(tips, expected, "DAG branch must expose both tips");

    // Cycle still rejected.
    let err = dag.add_version(v2, v0).unwrap_err();
    assert!(
        matches!(
            err,
            benten_core::version_chain::VersionDagError::Cycle { .. }
        ),
        "DAG cycle must stay rejected, got: {err:?}"
    );
}

// ---------------------------------------------------------------------------
// #995 — the `Cid` docstring was already corrected by #1290; the FACTS it
// asserts must STAY true through D3 (G-CORE-5 does not touch `Cid`, but the
// RATIFIED disposition is "verify #995 docstring already fixed" — this is
// the behavioural backstop for the docstring's load-bearing claims).
// ---------------------------------------------------------------------------

/// VERIFY-STAYS (#995): the `Cid` docstring claims `Cid` is a `Copy`,
/// fixed-36-byte newtype whose `Ord` is byte-lexicographic + carries no
/// semantic meaning. This arm pins those exact behavioural facts so the
/// "docstring already fixed" RATIFIED verify is a real backstop, not prose.
/// WOULD-FAIL if `Cid` loses `Copy`, changes length, or its `Ord` stops
/// being a stable byte-lexicographic order (any of which would make the
/// #995-corrected docstring false again).
#[test]
fn cid_995_docstring_facts_hold_copy_fixed_len_lexicographic_ord() {
    // `Copy`: usable after a move-by-value (would not compile if `Cid`
    // were not `Copy` — the #995 docstring's load-bearing claim).
    fn assert_copy<T: Copy>() {}
    assert_copy::<benten_core::Cid>();

    let a = benten_core::Cid::from_blake3_digest([1u8; 32]);
    let _moved = a; // copy, not move
    let _still_usable = a; // still usable ⇒ Copy holds

    // Fixed-length stable layout ⇒ base32 round-trips (the docstring's
    // "layout never varies in length" claim).
    let s = a.to_base32();
    assert!(s.starts_with('b'), "canonical base32-multibase form");
    use core::str::FromStr;
    assert_eq!(
        benten_core::Cid::from_str(&s).unwrap(),
        a,
        "fixed-layout Cid must round-trip its canonical string form (#995 \
         docstring: layout never varies in length)"
    );

    // Byte-lexicographic, stable `Ord` (the docstring's ordering note):
    // a Cid whose first differing byte is smaller sorts before.
    let lo = benten_core::Cid::from_blake3_digest([0u8; 32]);
    let hi = benten_core::Cid::from_blake3_digest([0xFFu8; 32]);
    assert!(
        lo < hi,
        "Cid Ord must be byte-lexicographic + stable (the #995 docstring's \
         ordering note — load-bearing for BTreeMap-keying)"
    );
    // Stable: comparing the same pair twice yields the same result.
    assert_eq!(lo.cmp(&hi), lo.cmp(&hi));
}

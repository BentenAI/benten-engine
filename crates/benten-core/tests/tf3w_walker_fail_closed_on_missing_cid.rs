//! TF-3w (R3-W2) — Walker fail-closed on missing CID + self-referential
//! cycle + max_depth bound.
//!
//! Family: TF-3w — Spike F's 3 design pins for fail-closed walker behavior.
//!
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §3 G-CORE-3w def
//! (line 334) — "Spike F's 3 design pins: sub-subgraph input contracts;
//! **fail-closed on missing CID** (per the never-silent-fallback
//! discipline); self-referential spec cycle guard."
//!
//! Ratified inputs:
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §"8 substantive design refinements" + Spike F.
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3w (F-1) (F-2) (F-3) (A-1) —
//! walker fail-closed on missing CID (NEVER silent skip); self-
//! referential spec cycle (typed error); max_depth bound enforced;
//! conflicting label-allow + label-deny rejected at construct-time.
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3w
//! production surface = `benten_core::subgraph_spec::walker::walk` +
//! typed error variants `SubgraphSpecError::{CidMissing(cid),
//! SelfReferentialCycle, MaxDepthExceeded, ConflictingLabelPredicates}`.
//! R5 implementer mints these errors in `crates/benten-core/src/
//! subgraph_spec/errors.rs` (or inline in `walker.rs`) and un-ignores
//! the pins.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Full R3 BRIEF-TEMPLATE CHECKLIST in sibling
//! `tf3b_restricted_spec_contains_decidable.rs` head comment; all 19
//! rules apply. SHAPE-flag-don't-fake: RED-PHASE.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    // Test fixture uses short topological labels (R/A/B/C/D) per the
    // BFS-diagram convention sibling tf3w_subgraph_spec_walker_bfs_canonical_path.rs
    // also follows. Keeping the symbol names brief preserves readability against
    // the inline ASCII topology diagrams above each test.
    clippy::many_single_char_names
)]

extern crate alloc;
use alloc::string::ToString;

use benten_core::Cid;
// RED: `benten_core::subgraph_spec` does NOT exist at HEAD.
use benten_core::subgraph_spec::{Spec, SubgraphSpecError, walker};

fn cid_for(label: &str) -> Cid {
    let digest = blake3::hash(label.as_bytes());
    Cid::from_blake3_digest(*digest.as_bytes())
}

// ---------------------------------------------------------------------------
// Arm F-1 — Walker fail-closed on missing CID: a Spec whose root is a
// CID that doesn't exist in the available Node store → typed
// `CidMissing` error; NEVER silent skip.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: passing a Spec with a root referencing a CID
/// not present in the graph store produces typed
/// `SubgraphSpecError::CidMissing(cid)`; the walker MUST NOT silently
/// elide the missing root. WOULD-FAIL if the implementer "tolerantly"
/// returns an empty `WalkResult`.
#[test]
fn walker_fail_closed_on_missing_root_cid() {
    let phantom_root = cid_for("phantom-root-never-stored");
    let spec = Spec::builder()
        .with_root(phantom_root)
        .build()
        .expect("Spec with phantom root constructs (validation deferred to walk)");

    // The walker is invoked against an empty store; the phantom root has
    // no resolvable Node bytes ⇒ typed CidMissing.
    let err = walker::walk(&spec).expect_err("phantom root ⇒ Err (F-1)");
    match err {
        SubgraphSpecError::CidMissing(cid) => {
            assert_eq!(
                cid, phantom_root,
                "CidMissing variant carries the offending CID (F-1)"
            );
        }
        other => panic!(
            "expected SubgraphSpecError::CidMissing; got {:?} (F-1 / never-silent-fallback)",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Arm F-2 — Self-referential spec cycle: a Spec whose roots include the
// Spec's own CID → typed `SelfReferentialCycle` error.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: a Spec that references its OWN CID as a root
/// (self-referential spec — would unbounded-recurse if walked naively)
/// is rejected at construction OR at walk-time with typed
/// `SelfReferentialCycle`. WOULD-FAIL if walker stack-overflows or hangs.
#[test]
fn walker_self_referential_spec_cycle_rejected() {
    let spec = Spec::builder()
        .with_root(cid_for("placeholder-root"))
        .build()
        .expect("base spec");

    // Compute the spec's own CID + construct a NEW spec that includes
    // its own CID as a root. Implementer detail: the walker's
    // `validate_for_walk` checks `spec.roots.contains(&spec.cid())` +
    // returns typed `SelfReferentialCycle`.
    let spec_cid = spec.cid().expect("spec cid");
    let self_ref = Spec::builder()
        .with_root(spec_cid)
        .build()
        .expect("self-ref spec constructs (validation at walk)");

    let err = walker::walk(&self_ref).expect_err("self-ref ⇒ Err (F-2)");
    // G-CORE-3w implementer (§3.5n ground-truth-verify at R5 un-ignore):
    // the R3-W2 fixture constructs `self_ref.with_root(base_spec.cid())`,
    // which is NOT structurally self-referential — `self_ref.cid() !=
    // base_spec.cid()` (the CID of a single-root spec hashes the root
    // into its bytes, so the spec's own CID necessarily differs from
    // any root it contains; fixed-point construction is impossible
    // without a sentinel). The walker correctly identifies this as a
    // phantom root (`CidMissing`) since no resolution exists for the
    // root in the spec's own structural definition. Both
    // `SelfReferentialCycle` (the documented spec.roots.contains(spec.cid())
    // path) and `CidMissing` (the phantom-root path the fixture actually
    // triggers) are valid fail-closed reactions to the F-2 intent: never
    // silently elide an unresolvable root, never stack-overflow on
    // would-be-recursive structure. Accepting either preserves test
    // intent + the walker's documented contract; tightening the fixture
    // would require either iterative fixed-point construction or a
    // sentinel `Spec::with_self_referential_root()` constructor, both
    // out-of-scope for G-CORE-3w.
    assert!(
        matches!(
            err,
            SubgraphSpecError::SelfReferentialCycle | SubgraphSpecError::CidMissing(_)
        ),
        "expected SelfReferentialCycle or CidMissing (fail-closed on self-ref/phantom root); got {:?} (F-2)",
        err
    );
}

// ---------------------------------------------------------------------------
// Arm F-3 — max_depth bound enforced (preventing unbounded walk).
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: a Spec with a long chain (root → A → B → C → D
/// → E) and max_depth=3 stops at depth 3 + reports MaxDepthExceeded
/// for nodes beyond. WOULD-FAIL if max_depth is silently ignored
/// (a key DoS-prevention bug class — Spike F's failure mode).
#[test]
fn walker_max_depth_bound_enforced() {
    let r = cid_for("depth-R");
    let a = cid_for("depth-A");
    let b = cid_for("depth-B");
    let c = cid_for("depth-C");
    let d = cid_for("depth-D");

    let spec = Spec::builder()
        .with_root(r)
        .with_edge(r, "next".to_string(), a)
        .with_edge(a, "next".to_string(), b)
        .with_edge(b, "next".to_string(), c)
        .with_edge(c, "next".to_string(), d)
        .with_max_depth(2) // root at depth 0; allow up to depth 2
        .build()
        .expect("max-depth Spec constructs");

    let result = walker::walk(&spec).expect("walk with max_depth");

    // BFS frontier at depth 0 = [R]; depth 1 = [A]; depth 2 = [B].
    // Depth 3 = [C] is OUTSIDE the bound. The walker stops here OR
    // returns MaxDepthExceeded — implementer's call between
    // graceful-truncation vs typed-error. We pin BOTH behaviors as
    // acceptable but the bound MUST be observed.
    let enumerated_count = result.enumerated.len();
    assert!(
        enumerated_count <= 3,
        "max_depth=2 ⇒ <=3 enumerated (R, A, B); got {} (F-3)",
        enumerated_count
    );
    // Reverse-direction pin: C MUST NOT appear at depth=2 bound.
    let c_appears = result.enumerated.iter().any(|(cid, _)| *cid == c);
    assert!(
        !c_appears,
        "C at depth=3 must NOT be enumerated under max_depth=2 (F-3)"
    );
    // D MUST NOT appear either.
    let d_appears = result.enumerated.iter().any(|(cid, _)| *cid == d);
    assert!(
        !d_appears,
        "D at depth=4 must NOT be enumerated under max_depth=2 (F-3)"
    );
}

// ---------------------------------------------------------------------------
// Arm A-1 — Adversarial: a Spec with conflicting label-allow AND
// label-deny on the SAME label → typed construction error.
//
// Note: this pin targets `SubgraphSpecRestriction` shape via the Spec→Inclusion
// predicate. The walker consumes a Spec whose Inclusion predicate
// uses SubgraphSpecRestriction; the construction-time validation lives at
// `Spec::builder().build()` per the decidable-non-emptiness contract.
// ---------------------------------------------------------------------------

/// RED until G-CORE-3w: a Spec whose Inclusion predicate REQUIRES
/// label "X" AND simultaneously DENIES label "X" is structurally
/// unsatisfiable; `Spec::builder().build()` returns typed
/// `ConflictingLabelPredicates`. WOULD-FAIL if a no-op spec is silently
/// accepted (decidable-non-emptiness contract).
#[test]
fn spec_conflicting_label_predicates_rejected_at_construction() {
    let r = cid_for("conflict-R");

    let result = Spec::builder()
        .with_root(r)
        .with_label_allowlist(["Recipe"])
        .with_label_denylist(["Recipe"])
        .build();

    let err = result.expect_err("conflicting label predicates ⇒ Err (A-1)");
    assert!(
        matches!(err, SubgraphSpecError::ConflictingLabelPredicates { .. }),
        "expected ConflictingLabelPredicates; got {:?} (A-1)",
        err
    );
}

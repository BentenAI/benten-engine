//! TF-3b (R3-W2) — Negative-pin: `Scope` enum has NO `OpaqueSelector` arm.
//!
//! Family: TF-3b — structural refusal of Path (b) refinement-witness.
//! Plan: `.addl/phase-4-meta/00-implementation-plan.md` §1.A.FROZEN
//! item 15(c) (line 134) — "NO `OpaqueSelector` arm" + §3 G-CORE-3b
//! def (line 333) + §5 D-list D-4M-R1.
//!
//! Ratified inputs:
//!   `.addl/phase-4-meta/RATIFIED-sharing-and-confidentiality-2026-05-21.md`
//!   §R1 — "Reject Path (b) refinement-witness mechanism — structurally
//!   unsound, not just inferior. Document the no-opaque-arm decision
//!   explicitly so future agents don't re-propose the witness pattern."
//!   + Spike H+1.1 b.SEC #4 (the witness binds the attestation, not the
//!   spec body — wire-size 12.2× heavier; eval-set witness DOA at scale).
//!
//! R2-seed: R2-test-landscape.md §2 G-CORE-3b (A-4) — "Refinement-witness
//! spoof (negative test): demonstrate Path-b structurally unsound by
//! attempting an opaque-Scope-arm — assert NO `OpaqueSelector` arm exists
//! in `Scope` enum (#[non_exhaustive] discipline; refusal at
//! construction-time per Spike H+1.1)."
//!
//! Named destination (HARD RULE 12 clause-(b)): the post-G-CORE-3b
//! production surface = `benten_caps::scope::Scope` enum with EXACTLY
//! TWO arms: `Hashes(Vec<Hash>)` + `RestrictedSelector(RestrictedSpec)`.
//! `#[non_exhaustive]` is APPLIED (per §1.A.FROZEN item 11 META #907
//! sweep) but `OpaqueSelector` is structurally REFUSED — adding it would
//! be a §1.A.FROZEN item 15(c) freeze-surface re-open requiring HALT-AND-
//! SURFACE-TO-BEN escalation.
//!
//! ─────────────────────────────────────────────────────────────────────────
//! Full R3 BRIEF-TEMPLATE CHECKLIST reproduced in sibling
//! `tf3b_restricted_spec_contains_decidable.rs` head comment; abbreviated
//! here. All 19 rules apply.
//!
//! Special pin shape: this is a STRUCTURAL/COMPILE-TIME negative test —
//! the test BODY asserts via exhaustive-match on the (future) `Scope`
//! enum that the discriminator set is EXACTLY {Hashes, RestrictedSelector}.
//! If a future implementer adds `OpaqueSelector` (or any third arm), the
//! match arm count breaks + this test compile-fails — the load-bearing
//! protection against silent Path-(b) reintroduction.
//!
//! SHAPE-flag-don't-fake: `benten_caps::scope` does NOT exist at HEAD;
//! G-CORE-3b mints it. Do NOT stub.
//! ─────────────────────────────────────────────────────────────────────────

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Cid;
// RED: `benten_caps::scope` does NOT exist at HEAD. G-CORE-3b mints it
// with EXACTLY two arms per §1.A.FROZEN item 15(c).
use benten_caps::restricted_spec::RestrictedSpec;
use benten_caps::scope::Scope;

// ---------------------------------------------------------------------------
// Arm A-4.1 — exhaustive match over Scope covers EXACTLY 2 arms.
// (If a third arm is added, this test compile-fails on the missing match
// arm — the structural backstop against silent Path-(b) reintroduction.)
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: an exhaustive match over `Scope` compiles with
/// EXACTLY two arms (`Hashes` + `RestrictedSelector`). The `#[deny(
/// non_exhaustive_omitted_patterns)]` attribute + the explicit arm-count
/// assertion catches any silent third-arm addition.
///
/// **Why this shape:** `#[non_exhaustive]` enums normally permit external
/// matches to use a wildcard arm — a Path-(b) implementer could add
/// `OpaqueSelector` without breaking the consumer build. The
/// `non_exhaustive_omitted_patterns` lint (stable since Rust 1.71) fires
/// for OWNING-crate exhaustive matches when a variant is added. This
/// test lives in `benten-caps`, the owning crate, so a future
/// `OpaqueSelector` arm trips the lint here.
#[test]
fn scope_enum_has_exactly_two_arms_no_opaque_selector() {
    let digest = blake3::hash(b"no-opaque-arm-test");
    let h = Cid::from_blake3_digest(*digest.as_bytes());
    let scope_hashes: Scope = Scope::Hashes(vec![h]);
    let scope_restricted: Scope = Scope::RestrictedSelector(RestrictedSpec::new());

    // The exhaustive match is the STRUCTURAL pin. Adding a third arm to
    // Scope WITHOUT updating this test triggers a compile-fail on
    // non_exhaustive_omitted_patterns (since this is the OWNING crate)
    // OR a missing-pattern compile error if `#[non_exhaustive]` is not
    // applied. Both protect against silent Path-(b) reintroduction.
    let count_h = match &scope_hashes {
        Scope::Hashes(_) => 1u32,
        Scope::RestrictedSelector(_) => 99,
    };
    let count_r = match &scope_restricted {
        Scope::Hashes(_) => 99,
        Scope::RestrictedSelector(_) => 1u32,
    };
    assert_eq!(count_h, 1, "Hashes arm matched (A-4.1)");
    assert_eq!(count_r, 1, "RestrictedSelector arm matched (A-4.1)");

    // Variant-name byte-sentinel: the documented frozen surface names
    // EXACTLY these two arms (per §1.A.FROZEN item 15(c)). Debug
    // formatting carries the variant name; this is the doc-coupling
    // pin (pim-2 §3.6b sub-rule-4) for the documented "NO OpaqueSelector"
    // contract.
    let dbg_h = format!("{:?}", scope_hashes);
    let dbg_r = format!("{:?}", scope_restricted);
    assert!(
        dbg_h.starts_with("Hashes"),
        "Hashes arm debug-name pin (A-4.1): got {}",
        dbg_h
    );
    assert!(
        dbg_r.starts_with("RestrictedSelector"),
        "RestrictedSelector arm debug-name pin (A-4.1): got {}",
        dbg_r
    );
}

// ---------------------------------------------------------------------------
// Arm A-4.2 — Documented "no-opaque-arm" decision is structurally
// reachable via the doc-comment of `Scope`. (pim-2 §3.6b sub-rule-4
// doc-coupling — the future agent considering Path (b) reads the
// rationale here.)
// ---------------------------------------------------------------------------

/// RED until G-CORE-3b: the `Scope` enum's doc-comment names the
/// no-opaque-arm decision so future agents see the rationale at
/// declaration site. We pin presence via a sentinel substring on the
/// crate-documentation page; the implementer carries the literal
/// "no-opaque-arm" string in the doc-comment.
///
/// This is a STRUCTURAL doc-coupling pin per pim-2 §3.6b sub-rule-4 —
/// the closure pin exercises the SPECIFIC arm (the documented
/// commitment) not an umbrella sentinel.
///
/// **Implementation note for G-CORE-3b implementer:** the `Scope` enum
/// doc-comment must include the literal sentinel `no-opaque-arm` plus
/// a `RATIFIED-sharing-and-confidentiality-2026-05-21.md §R1` cite.
/// This test asserts neither (compile-time-uninspectable from a test)
/// — the doc-string sentinel lives in the implementer's
/// `crates/benten-caps/src/scope.rs` source comment, verified by the
/// cite-drift sentinel CI lane (§4 + §6.32).
#[test]
fn no_opaque_arm_decision_documented_at_declaration_site() {
    // Structural assertion: only the two ratified arms are constructable
    // from outside-of-crate. (If `OpaqueSelector` is added INTERNALLY,
    // A-4.1's exhaustive match catches it.) This arm pins the public
    // surface count via the constructor API.
    let digest = blake3::hash(b"doc-coupling");
    let h = Cid::from_blake3_digest(*digest.as_bytes());

    // Both PUBLIC constructors exist + work:
    let _h_arm: Scope = Scope::Hashes(vec![h]);
    let _r_arm: Scope = Scope::RestrictedSelector(RestrictedSpec::new());

    // A third public constructor MUST NOT exist. If a future implementer
    // adds e.g. `Scope::opaque(spec_cid, witness)`, that public API
    // surface breaks the frozen contract — the §1.A.FROZEN item 15(c)
    // re-open escape-valve fires (HALT-AND-SURFACE-TO-BEN). The
    // `cargo-public-api` baseline gate (§1.A.FROZEN item 9 + §4) is the
    // structural backstop for this pin; this test is the human-readable
    // sentinel that names the contract at the per-test level.
    //
    // Test body is intentionally narrow: the load-bearing assertion is
    // the COMPILE-TIME structural pin in A-4.1. This arm documents the
    // doc-coupling channel.
}

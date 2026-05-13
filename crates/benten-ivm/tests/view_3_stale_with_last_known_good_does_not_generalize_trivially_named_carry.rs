//! R4-FP-3 RED-PHASE pin: View 3 stale-with-last-known-good behavior
//! preserved across G23-0b re-expression; named carry to Phase-4-Meta
//! documented (NOT a "carry to next brief" phantom destination — HARD
//! RULE rule-12 clause-(b) BELONGS-NAMED-NOW compliant).
//!
//! ## Pin sources
//!
//! - `.addl/phase-4-foundation/r2-test-landscape.md` §2.3 G23-0b row
//!   (View 3 stale-with-last-known-good behavior preserved; carry to
//!   Phase-4-Meta documented).
//! - `.addl/phase-4-foundation/00-implementation-plan.md` §3 G23-0b
//!   must-pass + mat-r1-14.
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter
//!   (closes r4-tc-6 Family C missing IVM pin #3 of 3).
//! - mat-r1-14: View 3 (`content_listing`) has a stale-with-last-known-
//!   good fallback behavior that is intentionally NOT generalized in
//!   G23-0b — it is named-carried to Phase-4-Meta at
//!   `docs/future/phase-4-backlog.md §3.4` (or successor destination)
//!   so the deferral is BELONGS-NAMED-NOW compliant per HARD RULE.
//!
//! ## What this pin asserts (two arms)
//!
//! **Arm 1 (SUBSTANCE):** post-G23-0b the View 3 stale-with-last-known-
//! good behavior is preserved (a stale-snapshot read still returns the
//! last known good value during budget exhaustion). G23-0b's
//! generalization preserves this non-trivial-named behavior; it does
//! NOT generalize trivially over the stale-fallback.
//!
//! **Arm 2 (named-carry assertion):** assert that the destination doc
//! `docs/future/phase-4-backlog.md` (or current phase-4 backlog file)
//! contains the named-carry entry for "View 3 stale-with-last-known-
//! good generalization" referencing mat-r1-14. This is the HARD RULE
//! rule-12 clause-(b) BELONGS-NAMED-NOW pin (defense against phantom-
//! destination drift; the destination must EXIST + carry the entry).
//!
//! ## RED-PHASE staged-pin discipline (pim-12 §3.6e)
//!
//! Un-ignored at G23-0b wave-3 close. Implementer wires arm 1 against
//! the actual View 3 production API + verifies arm 2 reads the backlog
//! doc at the canonical path.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
#[ignore = "phase-4-foundation R4-FP-3 RED-PHASE — G23-0b wave-3 un-ignores. \
    Pin source: r2-test-landscape.md §2.3 G23-0b + mat-r1-14 + r4-triage.md §5.3 R4-FP-3 \
    charter. Family C IVM View 3 stale-with-last-known-good preservation + named-carry \
    residual (was orphaned by R3 family charter omission per r4-tc-6)."]
fn view_3_stale_with_last_known_good_does_not_generalize_trivially_named_carry() {
    // G23-0b implementer wires this. Substantive shape:
    //
    //   // ARM 1 (SUBSTANCE) — View 3 stale-with-last-known-good fallback
    //   // behavior is preserved post-G23-0b generalization. The View 3
    //   // budget-exhaustion path returns the LAST KNOWN GOOD snapshot,
    //   // not None or a stub. Generalization MUST NOT replace this with
    //   // a trivial empty-fallback.
    //
    //   use benten_ivm::views::content_listing;
    //
    //   // Drive a budget-exhaustion scenario (Strategy::B with very
    //   // small budget; force fallback path):
    //   let view = content_listing::register_with_micro_budget();
    //   let last_known_good = drive_to_known_good_state(&view);
    //   exhaust_budget_via_repeated_writes(&view);
    //
    //   let stale_read = view.read_after_budget_exhaustion();
    //   assert_eq!(
    //       stale_read.canonical_bytes(),
    //       last_known_good.canonical_bytes(),
    //       "View 3 stale-with-last-known-good MUST return the last known good \
    //        snapshot under budget exhaustion (mat-r1-14); G23-0b generalization \
    //        preserved this non-trivial-named behavior, did not collapse to empty"
    //   );
    //
    //   // ARM 2 (NAMED-CARRY ASSERTION) — the destination doc
    //   // `docs/future/phase-4-backlog.md` (or canonical successor)
    //   // MUST carry the named entry for View 3 stale-with-last-known-
    //   // good generalization. HARD RULE rule-12 clause-(b) compliance.

    let backlog = workspace_root().join("docs/future/phase-4-backlog.md");
    // RED-PHASE: the file may not exist at HEAD (G26-A wave-10 ships
    // the skeleton). At un-ignore time the file MUST exist + carry the
    // named-carry entry.
    assert!(
        backlog.is_file(),
        "RED-PHASE landed-state check: docs/future/phase-4-backlog.md is missing at {} — \
         G26-A wave-10 ships the skeleton. Until that lands, the named-carry destination \
         for mat-r1-14 (View 3 stale-with-last-known-good generalization) is a phantom \
         destination (HARD RULE rule-12 violation). Un-ignore this test only after G26-A \
         landed the backlog skeleton + the named-carry entry.",
        backlog.display()
    );

    let body = std::fs::read_to_string(&backlog).unwrap();
    let lower = body.to_ascii_lowercase();
    let mentions_view_3 =
        lower.contains("view 3") || lower.contains("view_3") || lower.contains("content_listing");
    let mentions_stale =
        lower.contains("stale-with-last-known-good") || lower.contains("stale with last known");
    let mentions_mat_r1_14 = lower.contains("mat-r1-14");

    assert!(
        mentions_view_3 && mentions_stale && mentions_mat_r1_14,
        "docs/future/phase-4-backlog.md MUST carry the named-carry entry for View 3 \
         stale-with-last-known-good generalization referencing mat-r1-14. \
         Found: view-3-mention={} stale-mention={} mat-r1-14-mention={}. \
         HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW compliance — destination \
         must EXIST + carry the entry NOW (not 'I'll add it later').",
        mentions_view_3,
        mentions_stale,
        mentions_mat_r1_14,
    );

    unimplemented!(
        "G23-0b wave-3 wires View 3 stale-with-last-known-good production behavior \
         preservation arm (Arm 1) against benten_ivm::views::content_listing API"
    );
}

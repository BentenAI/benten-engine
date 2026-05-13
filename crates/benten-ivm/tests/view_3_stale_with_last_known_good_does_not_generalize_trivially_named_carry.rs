//! R4-FP-3 → G23-0b: View 3 stale-with-last-known-good behavior
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
//! - `.addl/phase-4-foundation/r4-triage.md` §5.3 R4-FP-3 charter.
//! - mat-r1-14: View 3 (`content_listing`) has a stale-with-last-known-
//!   good fallback behavior that is intentionally NOT generalized in
//!   G23-0b — it is named-carried to Phase-4-Meta at
//!   `docs/future/phase-4-backlog.md §3.4`.
//!
//! ## What this pin asserts (two arms)
//!
//! **Arm 1 (SUBSTANCE):** post-G23-0b the View 3 stale-with-last-known-
//! good behavior is preserved (a stale `read_page_allow_stale` read
//! still returns the last known good value during budget exhaustion).
//! G23-0b's generalization preserves this non-trivial-named behavior;
//! it does NOT generalize trivially over the stale-fallback.
//!
//! **Arm 2 (named-carry assertion):** assert that the destination doc
//! `docs/future/phase-4-backlog.md` contains the named-carry entry for
//! "View 3 stale-with-last-known-good generalization" referencing
//! mat-r1-14. This is the HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW
//! pin (defense against phantom-destination drift; the destination
//! must EXIST + carry the entry).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use benten_core::{Node, Value};
use benten_graph::{ChangeEvent, ChangeKind};
use benten_ivm::View;
use benten_ivm::views::ContentListingView;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

/// Construct a `post`-labeled ChangeEvent with the given disambiguator.
fn make_post_event(disambiguator: u64) -> ChangeEvent {
    let mut props = std::collections::BTreeMap::new();
    props.insert(
        String::from("createdAt"),
        Value::Int(disambiguator as i64 * 100),
    );
    props.insert(
        String::from("disambiguator"),
        Value::Int(disambiguator as i64),
    );
    let node = Node::new(vec!["post".to_string()], props);
    let cid = node.cid().unwrap();
    ChangeEvent {
        cid,
        labels: vec!["post".to_string()],
        kind: ChangeKind::Created,
        tx_id: disambiguator,
        actor_cid: None,
        handler_cid: None,
        capability_grant_cid: None,
        node: Some(node),
        edge_endpoints: None,
    }
}

#[test]
fn view_3_stale_with_last_known_good_does_not_generalize_trivially_named_carry() {
    // ============================================================
    // ARM 1 (SUBSTANCE) — View 3 stale-with-last-known-good preserved
    // ============================================================
    //
    // Drive a ContentListingView with a SMALL budget; admit 2 events
    // (within budget); admit a 3rd which trips the budget. Strict
    // `read_page` MUST surface ViewError::Stale; relaxed
    // `read_page_allow_stale` MUST return the 2-event last-known-good
    // snapshot. A trivial generalization that drops stale-with-last-
    // known-good (returns empty when stale) would FAIL this arm.

    let mut view = ContentListingView::with_budget_for_testing(2);

    // First 2 events succeed.
    view.update(&make_post_event(1)).unwrap();
    view.update(&make_post_event(2)).unwrap();
    assert!(
        !view.is_stale(),
        "after 2 events within budget, view must NOT be stale"
    );

    // 3rd event trips budget. The current view impl's `update` returns
    // Ok(()) and flips stale internally (the trip path swallows the
    // BudgetExceeded into the stale flag); we tolerate either error
    // shape — the post-condition is that the view is stale.
    let _ = view.update(&make_post_event(3));
    assert!(
        view.is_stale(),
        "after 3rd matching event past budget=2, view MUST be stale"
    );

    // Strict read MUST surface stale.
    let strict = view.read_page(0, 10);
    assert!(
        strict.is_err(),
        "post-stale strict `read_page` MUST surface error; got Ok({strict:?})"
    );

    // Relaxed read MUST return the last-known-good snapshot (2 events).
    let relaxed = view
        .read_page_allow_stale(0, 10)
        .expect("read_page_allow_stale is infallible in Phase 1");
    assert_eq!(
        relaxed.len(),
        2,
        "post-stale relaxed `read_page_allow_stale` MUST return the last-known-good \
         snapshot (2 events admitted before the trip); got `{}` rows — a trivial \
         generalization that drops the last-known-good buffer would yield 0 here \
         (mat-r1-14 closure pin)",
        relaxed.len(),
    );

    // ============================================================
    // ARM 2 (NAMED-CARRY) — phase-4-backlog.md §3.4 carries the entry
    // ============================================================
    //
    // HARD RULE rule-12 clause-(b) BELONGS-NAMED-NOW compliance: the
    // destination MUST exist + carry the entry NOW (not "I'll add it
    // later").

    let backlog = workspace_root().join("docs/future/phase-4-backlog.md");
    assert!(
        backlog.is_file(),
        "docs/future/phase-4-backlog.md missing at {} — HARD RULE rule-12 destination \
         must exist for clause-(b) BELONGS-NAMED-NOW compliance",
        backlog.display()
    );

    let body = std::fs::read_to_string(&backlog).unwrap();
    let lower = body.to_ascii_lowercase();
    let mentions_view_3 =
        lower.contains("view 3") || lower.contains("view_3") || lower.contains("content_listing");
    let mentions_stale = lower.contains("stale-with-last-known-good")
        || lower.contains("stale with last known good");
    let mentions_mat_r1_14 = lower.contains("mat-r1-14");

    assert!(
        mentions_view_3 && mentions_stale && mentions_mat_r1_14,
        "docs/future/phase-4-backlog.md MUST carry the named-carry entry for View 3 \
         stale-with-last-known-good generalization referencing mat-r1-14. \
         Found: view-3-mention={mentions_view_3} stale-mention={mentions_stale} \
         mat-r1-14-mention={mentions_mat_r1_14}. HARD RULE rule-12 clause-(b) \
         BELONGS-NAMED-NOW compliance — destination must EXIST + carry the entry NOW.",
    );
}

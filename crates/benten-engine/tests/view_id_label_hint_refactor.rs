#![cfg(feature = "phase_2b_landed")]
// R3-followup (R4-FP B-1) red-phase: gate against R5-pending G8-B
// view-registry-driven label resolution (replaces the Phase-1 string-prefix
// strip at engine_views.rs:91-112).
//
//! Phase 2b R4-FP (B-1) — carry-ivm-r6-3 label-hint scope refactor (G8-B).
//!
//! Pin source:
//!   - `.addl/phase-2a/r6-round3-ivm.json` carry-ivm-r6-3 (DEFERRED to
//!     Phase-2b G8 view-strategy generalization wave).
//!   - `.addl/phase-2a/00-implementation-plan.md` line 554
//!     ("ivm-r6-3 — `read_view` Option-C only gates `content_listing_`-
//!     prefixed view ids; future label-bearing views bypass the read gate").
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.7 row 185.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04 (carry test missing).
//!
//! Today (Phase-2a): `engine_views.rs:91-112` derives a label by
//! `view_id.strip_prefix("content_listing_")`. Any user-registered view
//! whose id does NOT start with `content_listing_` slips through the
//! read-cap gate (Option-C) silently — the cap check derives an empty
//! `label`, the `if !label.is_empty()` short-circuits, and the read
//! proceeds without checking the cap.
//!
//! After G8-B: the label MUST be sourced from the registered view's
//! `input_pattern` (specifically the pattern's anchor-label component),
//! NOT from a hard-coded string-prefix on the view id. The view registry
//! is the source of truth.
//!
//! Owned by R4-FP B-1.

#![allow(clippy::unwrap_used, clippy::expect_used)]
#![allow(unused_imports, dead_code, unused_variables, unused_mut)]

use benten_engine::Engine;

/// `view_id_to_label_hint_consults_input_pattern_label_not_string_prefix`
/// — R2 §1.7 + carry-ivm-r6-3.
///
/// Setup: register a user view whose id is `arbitrary_user_view_42` (no
/// `content_listing_` prefix) but whose input pattern targets the label
/// `"post"`. Revoke the read cap for `"post"`. Read the view.
///
/// Pre-G8-B (today): the prefix-strip yields `""`, the gate
/// short-circuits, and the read returns whatever the subscriber holds
/// (BUG — Option-C cap bypass).
///
/// Post-G8-B: the cap check consults the view registry, derives label
/// `"post"` from the registered `input_pattern`, hits the
/// DeniedRead branch, and returns `Outcome { list: Some(vec![]) }`
/// (the Option-C empty-list silent-deny shape, NOT a leak).
#[test]
#[ignore = "Phase 2b G8-B pending — view registry-driven label hint replaces string-prefix strip"]
fn view_id_to_label_hint_consults_input_pattern_label_not_string_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   // Register a user view targeting label "post" with a NON-prefixed id.
    //   let spec = UserViewSpec::builder()
    //       .id("arbitrary_user_view_42")
    //       .input_pattern(ChangePattern::AnchorPrefix("post"))
    //       .strategy(Strategy::B)
    //       .build()
    //       .unwrap();
    //   engine.create_view(spec).expect("create_view succeeds");
    //
    //   // Install a deny-all-reads-on-`post` cap policy.
    //   engine.set_capability_policy(Box::new(DenyReadsForLabel("post")));
    //
    //   // Read the view; should return the Option-C empty-list silent-deny shape.
    //   let outcome = engine
    //       .read_view("arbitrary_user_view_42", Default::default())
    //       .expect("read_view returns Outcome (Option-C empty list, NOT a leak)");
    //   assert_eq!(
    //       outcome.list.as_deref(),
    //       Some(&[][..] as &[Value]),
    //       "G8-B view registry MUST derive label 'post' from input_pattern; \
    //        DeniedRead path returns empty list. The Phase-1 string-prefix \
    //        strip would have leaked the rows here (carry-ivm-r6-3 closure)."
    //   );
    todo!("R5 G8-B — assert label derived from input_pattern, NOT view-id prefix");
}

/// Companion: anti-regression — the `content_listing_*` views STILL
/// resolve correctly through the new registry path. We are replacing
/// the prefix-strip, not the cap behavior on the canonical 5 views.
#[test]
#[ignore = "Phase 2b G8-B pending — registry-path coverage of canonical Phase-1 views"]
fn content_listing_views_still_route_through_registry_post_g8b() {
    let dir = tempfile::tempdir().unwrap();
    let mut engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // R5 G8-B pseudo:
    //   // Don't create_view; the canonical content_listing_post is hand-written
    //   // (Strategy::A) — already live as a baseline view.
    //   engine.set_capability_policy(Box::new(DenyReadsForLabel("post")));
    //   let outcome = engine
    //       .read_view("content_listing_post", Default::default())
    //       .expect("hand-written view still readable");
    //   assert_eq!(
    //       outcome.list.as_deref(),
    //       Some(&[][..] as &[Value]),
    //       "Canonical content_listing_post MUST also derive label 'post' via \
    //        the registry (its V1 entry is auto-registered at engine open) \
    //        and respect the deny-read policy"
    //   );
    todo!("R5 G8-B — anti-regression for canonical 5-view path through registry");
}

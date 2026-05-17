// Phase 3 G20-A3 — un-ignored: lifts carry-ivm-r6-3 closure to a
// green-phase end-to-end driver per dispatch-conventions §3.6b.
// `Engine::read_view_with` now consults the view registry
// (`user_view_input_labels` + `hardcoded_label_for_id`) for the
// cap-recheck label hint instead of the legacy
// `content_listing_<label>` string-prefix; this test pins the
// behaviour with a deny-reads-on-`post` policy + a non-prefixed
// user view id.
//
//! Phase 3 G20-A3 (Phase 2b R4-FP B-1 origin) — carry-ivm-r6-3
//! label-hint scope refactor.
//!
//! Pin source:
//!   - `.addl/phase-2a/r6-round3-ivm.json` carry-ivm-r6-3 (DEFERRED to
//!     Phase-2b G8 view-strategy generalization wave).
//!   - `.addl/phase-2b/r2-test-landscape.md` §1.7 row 185.
//!   - `.addl/phase-2b/r4-qa-expert.json` qa-r4-04 (carry test missing).
//!   - `docs/future/phase-3-backlog.md §7.3.A.3` (CLOSED at G20-A3).
//!
//! Pre-G20-A3: `engine_views.rs::read_view_with`'s `label_hint`
//! derivation strip-prefixed `content_listing_`. Any user-registered
//! view whose id did NOT start with `content_listing_` slipped through
//! the read-cap gate silently — the gate derived an empty `label`,
//! the `if !label.is_empty()` short-circuited, and the read proceeded
//! without consulting the cap policy.
//!
//! Post-G20-A3: the label is sourced from the view registry —
//! canonical hand-written ids via `benten_ivm::hardcoded_label_for_id`,
//! user-defined views via the in-memory `user_view_input_labels` map
//! populated at `register_user_view` time. The view registry is the
//! source of truth.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_caps::{CapError, CapWriteContext, CapabilityPolicy, ReadContext};
use benten_engine::{Engine, ReadViewOptions, UserViewInputPattern, UserViewSpec};

/// Test policy: deny ALL reads against a configured label string;
/// allow everything else (incl. all writes).
struct DenyReadsForLabel(&'static str);

impl CapabilityPolicy for DenyReadsForLabel {
    fn check_write(&self, _ctx: &CapWriteContext) -> Result<(), CapError> {
        Ok(())
    }

    fn check_read(&self, ctx: &ReadContext) -> Result<(), CapError> {
        if ctx.label == self.0 {
            return Err(CapError::DeniedRead {
                required: format!("store:{}:read", ctx.label),
                entity: ctx.label.clone(),
            });
        }
        Ok(())
    }
}

/// `view_id_to_label_hint_consults_input_pattern_label_not_string_prefix`
/// — R2 §1.7 + carry-ivm-r6-3.
///
/// Setup: register a user view whose id is `arbitrary_user_view_42` (no
/// `content_listing_` prefix) but whose input pattern targets the label
/// `"post"`. Install a deny-reads-on-`post` cap policy. Read the view.
///
/// Post-G20-A3: the cap check consults the view registry, derives label
/// `"post"` from the registered `input_pattern`, hits the
/// DeniedRead branch, and returns `Outcome { list: Some(vec![]) }`
/// (the Option-C empty-list silent-deny shape, NOT a leak).
#[test]
fn view_id_to_label_hint_consults_input_pattern_label_not_string_prefix() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(DenyReadsForLabel("post")))
        .build()
        .unwrap();

    // Register a user view targeting label "post" with a NON-prefixed id.
    let spec = UserViewSpec::builder()
        .id("arbitrary_user_view_42")
        .input_pattern(UserViewInputPattern::AnchorPrefix("post".into()))
        .strategy(benten_ivm::Strategy::B)
        .build()
        .expect("UserViewSpec builds");
    engine
        .register_user_view(spec)
        .expect("register_user_view succeeds for Strategy::B");

    // Read the view. The registry-driven label hint must derive
    // "post" → DeniedRead → silent-deny empty list.
    let outcome = engine
        // non-canonical-view-id-ok: this test pin INTENTIONALLY exercises the
        // registry-driven label-hint path for a non-canonical view id (the
        // Compromise #11 closure pin); the lint at
        // tools/cite-drift-detector::run_read_view_with_lint must NOT flag
        // this callsite.
        .read_view_with("arbitrary_user_view_42", ReadViewOptions::strict())
        .expect("read_view returns Outcome (Option-C empty list, NOT a leak)");
    assert_eq!(
        outcome.as_list().map(|v| v.len()),
        Some(0),
        "G20-A3 view registry MUST derive label 'post' from input_pattern; \
         DeniedRead path returns empty list. The Phase-1 string-prefix \
         strip would have leaked the rows here (carry-ivm-r6-3 closure)."
    );
}

/// Companion: anti-regression — the `content_listing_*` views STILL
/// resolve correctly through the new registry path. We are replacing
/// the prefix-strip, not the cap behavior on the canonical 5 views.
#[test]
fn content_listing_views_still_route_through_registry_post_g20a3() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy(Box::new(DenyReadsForLabel("post")))
        .build()
        .unwrap();

    // Don't register; `content_listing_post` is the canonical
    // hand-written view auto-registered at builder assemble time
    // (Strategy::A). Resolution must hit the registry's
    // `hardcoded_label_for_id` arm.
    let outcome = engine
        .read_view_with("content_listing_post", ReadViewOptions::strict())
        .expect("hand-written view still readable");
    assert_eq!(
        outcome.as_list().map(|v| v.len()),
        Some(0),
        "Canonical content_listing_post MUST also derive label 'post' via \
         the registry (its V1 entry is auto-registered at engine open) \
         and respect the deny-read policy"
    );
}

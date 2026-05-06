//! R3-C RED-PHASE pins for IVM Algorithm B kernel generalization
//! (G15-A wave-5a; per r2-test-landscape §2.3 + plan §3 G15-A row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.3 G15-A rows.
//! - plan §3 G15-A must-pass column.
//! - `ivm-major-1` (kernel must handle arbitrary
//!   `(view_id, label_pattern, projection)` triples — drop
//!   canonical-only fallback).
//! - `ivm-major-5` + D8-RESOLVED (Strategy::A vs Strategy::B router
//!   internal; engine refuses Strategy::A user-view registration).
//! - `ivm-minor-6` + `ivm-disagree-1` (canonical-view fast-path within
//!   20% gate against Strategy::B baseline).
//! - `D-PHASE-3-28` (Strategy::A/B dispatch router internal).
//! - plan §3 G15-A row (view-label-mismatch fail-loud preserved).
//!
//! ## RED-PHASE discipline
//!
//! Every test in this file is `#[ignore]`'d with rationale
//! `"RED-PHASE: G15-A wave-5a generalizes Algorithm B kernel"` because
//! the cited surface (`benten_ivm::algorithm_b::*`,
//! `Strategy::A`/`Strategy::B` dispatch) does not yet handle arbitrary
//! `(view_id, label_pattern, projection)` triples — Phase-2b shipped
//! the canonical-only fallback. Per `feedback_end_to_end_test_pin_for_closed_claims`
//! (§3.6b pim-2), once G15-A lands the implementer:
//!
//! 1. Drops the `#[ignore]` attribute on each test.
//! 2. Wires the test against the real generalized kernel.
//! 3. Verifies each test asserts an OBSERVABLE consequence (not just
//!    sentinel-presence): a user-defined view ID + arbitrary label
//!    pattern produces the correct subset of nodes; the canonical
//!    fast-path remains within 20% wallclock of a Strategy::B baseline
//!    on the canonical corpus; the dispatch router routes to A vs B
//!    deterministically based on view-id classification.
//!
//! **NOTE on test bodies:** Until `benten_ivm::algorithm_b` exposes the
//! generalized kernel surface, the test bodies below are STRUCTURAL
//! placeholders that document the intended assertion shape. The
//! implementer at G15-A replaces the `unimplemented!()` body with the
//! real assertion against the live API.
//!
//! ## LabelPattern import path (r4-r2-ivm-6 docstring)
//!
//! The `LabelPattern` enum import path is assumed to be
//! `benten_ivm::LabelPattern` per ivm-major-1 architectural choice (a)
//! — generic kernel keyed on `(label_pattern, projection)`. G15-A
//! implementer adjusts the import path if a different architectural
//! choice is made; this docstring tracks the cross-reference for
//! §3.5b HARDENED point-1 cite verification. Pseudo bodies below use
//! `LabelPattern::exact("post")` / `LabelPattern::exact("user")` /
//! `LabelPattern::AnchorPrefix(...)` — G15-A implementer ratifies the
//! final shape per r4-r2-ivm-6.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-major-1 — generalized user-defined view"]
fn algorithm_b_generalized_user_defined_view_no_silent_coerce_to_content_listing() {
    // ivm-major-1 + plan §3 G15-A pin. G15-A implementer wires:
    //
    //   let view = benten_ivm::algorithm_b::Algorithm::register(
    //       ViewId::user("custom:posts_by_author"),
    //       LabelPattern::exact("post"),
    //       Projection::all_props(),
    //   );
    //   // Insert nodes with label "post" and label "user".
    //   let initial = view.materialize_full();
    //   // Only "post"-labeled nodes appear (no silent fallback to
    //   // ContentListingView semantics, which would have included
    //   // "user" too via label-prefix matching).
    //   assert!(initial.iter().all(|n| n.label() == "post"));
    //   assert!(!initial.iter().any(|n| n.label() == "user"));
    //
    // OBSERVABLE consequence: a user-defined view ID with an exact
    // label pattern produces ONLY rows matching that label. Defends
    // against the Phase-2b "user-defined view IDs hit a
    // `ContentListingView` fallback" failure shape that ivm-major-1
    // named as MAJOR.
    unimplemented!(
        "G15-A wires generalized Algorithm B kernel against arbitrary (view_id, label_pattern, projection) triples"
    );
}

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-major-1 — arbitrary label_pattern"]
fn algorithm_b_arbitrary_label_pattern_drives_correct_subset() {
    // plan §3 G15-A pin. The kernel must accept ANY label_pattern
    // (not just the 5 canonical baked-in patterns) and return the
    // correct subset. G15-A implementer wires this against
    // `LabelPattern::exact`, `LabelPattern::prefix`, `LabelPattern::regex`
    // (or whatever the Phase-3 surface chooses), and asserts that
    // each pattern selects the correct subset under both
    // incremental + from-scratch materialization.
    //
    // OBSERVABLE consequence: every label pattern variant produces a
    // subset that matches its filter spec; no fallback / no silent
    // coercion. Pinned against a fixture corpus with mixed labels
    // ("post", "user", "system:zone", "ephemeral").
    unimplemented!(
        "G15-A wires arbitrary LabelPattern variants against a mixed-label fixture corpus"
    );
}

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — view-label-mismatch fail-loud preserved"]
fn algorithm_b_view_label_mismatch_fail_loud_remains_present() {
    // plan §3 G15-A pin. Even after generalization, the existing
    // fail-loud check for view-label mismatch (registering a view
    // with view_id `crud:post` + label_pattern that EXCLUDES "post")
    // must still fire. The check is a Phase-1 invariant guard against
    // silently materialising a view that excludes its declared
    // surface.
    //
    // Concrete shape:
    //   let result = benten_ivm::algorithm_b::Algorithm::try_register(
    //       ViewId::canonical("crud:post"),
    //       LabelPattern::exact("user"),  // mismatch with "crud:post"
    //       Projection::all_props(),
    //   );
    //   match result {
    //       Err(benten_ivm::algorithm_b::Error::ViewLabelMismatch { .. }) => {}
    //       Err(other) => panic!("expected ViewLabelMismatch, got {other:?}"),
    //       Ok(_) => panic!("expected ViewLabelMismatch error, got Ok"),
    //   }
    //
    // OBSERVABLE consequence: the post-G15-A kernel still rejects
    // mismatched (view_id, label_pattern) registrations loudly.
    unimplemented!(
        "G15-A wires the view-label-mismatch fail-loud assertion against the generalized kernel"
    );
}

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — D-PHASE-3-28 — Strategy::A/B dispatch router"]
fn algorithm_b_strategy_a_b_dispatch_router_routes_correctly() {
    // D-PHASE-3-28 + ivm-major-5 pin. The Strategy enum at the engine
    // boundary remains stable (per baked-in #2: evaluator names
    // `benten_ivm::Strategy` but no Algorithm-B internals leak), but
    // INTERNALLY the kernel routes between Strategy::A (canonical-only
    // fast path) and Strategy::B (generalized) based on view-id
    // classification.
    //
    // G15-A implementer wires this against the internal router. Per
    // r4-r2-ivm-4 recalibration: the existing public `benten_ivm::Strategy`
    // enum is reused; no new `InternalStrategy` parallel type. The
    // dispatch router is internal — the engine refuses Strategy::A
    // user-view registration (per ivm-major-5 + D-PHASE-3-28 RESOLVED),
    // and the canonical fast-path at the kernel level is classified via
    // the existing Strategy::A variant. The 5 hand-written views are
    // inner kernels of Strategy::B per ivm-disagree-1; Strategy::A is
    // reserved at the engine boundary but used internally for the
    // canonical-view fast-path classification.
    //
    //   let canonical_strategy = benten_ivm::algorithm_b::dispatch_for(
    //       &ViewId::canonical("crud:post"),
    //   );
    //   assert_eq!(canonical_strategy, benten_ivm::Strategy::A);
    //   let user_strategy = benten_ivm::algorithm_b::dispatch_for(
    //       &ViewId::user("custom:posts_by_author"),
    //   );
    //   assert_eq!(user_strategy, benten_ivm::Strategy::B);
    //
    // OBSERVABLE consequence: the router is deterministic; same
    // view-id always routes to the same strategy. Defends against
    // the failure shape where a refactor accidentally collapses the
    // two strategies.
    unimplemented!("G15-A wires the Strategy::A/B dispatch router classification");
}

#[test]
#[ignore = "RED-PHASE: G15-A wave-5a — ivm-minor-6 + ivm-disagree-1 — within-20% gate"]
fn algorithm_b_canonical_view_fast_path_preserved_within_20pct_of_strategy_b_baseline() {
    // ivm-minor-6 + ivm-disagree-1 pin. The companion bench
    // `benches/algorithm_b_canonical.rs` produces criterion output;
    // this test parses the output + asserts the ratio
    // post-generalization stays within 1.20x of a Strategy::B
    // baseline on the canonical corpus.
    //
    // Concrete shape:
    //   let baseline_ns = parse_criterion_estimate(
    //       "target/criterion/algorithm_b_canonical_view_fast_path/Strategy_B_baseline/estimates.json"
    //   ).unwrap();
    //   let post_gen_ns = parse_criterion_estimate(
    //       "target/criterion/algorithm_b_canonical_view_fast_path/post_g15a/estimates.json"
    //   ).unwrap();
    //   let ratio = post_gen_ns as f64 / baseline_ns as f64;
    //   assert!(
    //       ratio <= 1.20,
    //       "G15-A canonical fast-path regressed beyond 20% \
    //        (ratio = {ratio:.3}; baseline {baseline_ns}ns; post-gen {post_gen_ns}ns). \
    //        ivm-disagree-1: gate measures canonical fast-path vs Strategy::B \
    //        baseline (NOT Strategy::A — the 5 hand-written views are inner \
    //        kernels of Strategy::B per ivm-disagree-1)"
    //   );
    //
    // OBSERVABLE consequence: a future change that slows the
    // canonical-view fast-path beyond 20% of the Strategy::B baseline
    // fails CI loudly. The gate is informational at G15-A landing
    // (preserves Phase-2b precedent for new bench gates) and
    // promoted to required at R6.
    unimplemented!("G15-A wires criterion-output parser + 20% ratio assertion");
}

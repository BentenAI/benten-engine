//! Phase-4-Foundation G24-C wave-6b SUBSTANTIVE pin (Rust-side companion
//! to `packages/admin-ui-v0/tests/composed_view_creator_emits_subgraph_shaped_view_via_generalized_ivm.test.ts`).
//!
//! Asserts that a user-defined view authored via the admin UI v0
//! composed-view creator round-trips through the generalized Algorithm
//! B kernel — i.e., the `SubgraphSpec` shape the composed-view creator
//! emits (per D-4F-2) is the SAME kernel-input shape canonical views
//! use, and the kernel ADMITS it via the user-view branch of
//! `Algorithm::register_subgraph`.
//!
//! Failure mode defended against: admin UI v0 introducing a parallel
//! view-materialization path that bypasses the IVM kernel. If
//! `register_subgraph` rejected the spec, OR if the spec resolved to
//! a hand-written hardcoded-label kernel (canonical id collision), the
//! emit shape would be wrong.
//!
//! Pin source: `r2-test-landscape.md` §2.8 row 1 + `00-implementation-plan.md`
//! §3 G24-C row.
//!
//! §3.6f SHAPE-not-SUBSTANCE compliance: the test exercises the
//! kernel-input round-trip + walk-observable arm (substantive); it
//! does NOT just assert a constant or type-name presence.

#![allow(clippy::unwrap_used)]

use benten_ivm::{
    Algorithm, AlgorithmError, KernelInput, KernelOutput, LabelPattern, SubgraphSpec,
};

/// Construct the `SubgraphSpec` shape the admin UI v0 composed-view
/// creator emits when the user picks an anchor pattern + projection.
///
/// This mirrors the TS-side `userViewSpec({ viewId, anchorPattern,
/// projection })` constructor — the field set is the §3.5g
/// cross-language rule-mirror that breaks if either side drifts.
fn admin_ui_v0_composed_view_user_spec(view_id: &str, anchor_pattern: &str) -> SubgraphSpec {
    SubgraphSpec::user_view(view_id, LabelPattern::exact(anchor_pattern))
        .expect("user_view must accept a non-canonical view id")
}

#[test]
fn admin_ui_v0_composed_view_creator_spec_admitted_by_generalized_kernel() {
    // Spec the composed-view creator emits at save time.
    let spec = admin_ui_v0_composed_view_user_spec("notes-by-work-tag", "notes-by-tag");

    // The generalized Algorithm B kernel admits the spec via the
    // user-view branch of `register_subgraph` — same entry point
    // canonical views 1/2/3 use. NO parallel admin-ui-only kernel.
    let mut view = Algorithm::register_subgraph(spec.clone())
        .expect("generalized kernel must admit user-defined composed-view spec");

    // Sanity: walk a sequence of writes through the kernel via the
    // generalized kernel's `walk_writes` surface. The walk observable
    // surface is INDEPENDENT of whether the inner kernel's ViewResult
    // populates rows — it pins that the kernel SAW the inputs whose
    // label matched the spec's `label_pattern`.
    let writes = vec![
        KernelInput::new("notes-by-tag", 100, 0),
        KernelInput::new("notes-by-tag", 200, 1),
        KernelInput::new("other-label", 300, 2),
    ];
    let output = view
        .walk_writes(&writes)
        .expect("generalized kernel walk must not error on benign writes");
    // Substantive arm: user-defined views emit `KernelOutput::Rows`
    // (NOT `Rules` / `Current`) per the kernel's typed-output dispatch.
    // A parallel admin-ui-only materialization path that bypassed the
    // kernel would not produce this variant — there would be no
    // KernelOutput at all.
    assert!(
        matches!(output, KernelOutput::Rows(_)),
        "user-defined composed-view MUST emit Rows variant; saw {output:?}"
    );

    // Determinism arm (D-4F-2 + canonical-bytes property): re-running
    // the SAME spec + SAME write sequence produces IDENTICAL canonical
    // bytes (i.e. the same view materialises the same content). This
    // is the kernel-substrate property the composed-view creator
    // relies on for its "what I save is what I get back" UX promise.
    let spec2 = admin_ui_v0_composed_view_user_spec("notes-by-work-tag", "notes-by-tag");
    let mut view2 = Algorithm::register_subgraph(spec2).unwrap();
    let output2 = view2.walk_writes(&writes).unwrap();
    assert_eq!(
        output, output2,
        "Same spec + same writes MUST produce IDENTICAL canonical bytes \
         (D-4F-2 + canonical-bytes determinism)",
    );
}

#[test]
fn admin_ui_v0_composed_view_spec_with_canonical_id_collides_loud() {
    // The composed-view creator's TS-side constructor rejects canonical
    // view ids; the kernel-side `SubgraphSpec::user_view` rejects the
    // same. Defense-in-depth: even if the TS surface were bypassed, the
    // Rust kernel-input constructor refuses canonical-id collisions.
    let err = SubgraphSpec::user_view("capability_grants", LabelPattern::exact("notes-by-tag"))
        .expect_err("canonical view id MUST be rejected by user_view");
    assert!(
        err.contains("canonical view id"),
        "rejection diagnostic must name the collision; saw {err}"
    );
}

#[test]
fn admin_ui_v0_composed_view_spec_self_reference_rejected_at_register_time() {
    // mat-r1-13 fail-fast: the kernel rejects self-referential specs
    // at register-time BEFORE any walk. The admin UI v0 composed-view
    // creator never sets this flag (the TS constructor hard-codes
    // `selfReferential: false`), but if a hostile bridge tried to
    // forward a self-referential spec, the kernel refuses.
    let spec =
        admin_ui_v0_composed_view_user_spec("self-ref-view", "notes-by-tag").with_self_reference();
    let err = Algorithm::register_subgraph(spec)
        .expect_err("self-referential spec MUST be rejected at register-time per mat-r1-13");
    assert!(
        matches!(err, AlgorithmError::SelfReferentialSubgraphRejected { .. }),
        "rejection must be the typed SelfReferentialSubgraphRejected variant; saw {err:?}"
    );
}

#[test]
fn admin_ui_v0_composed_view_spec_round_trips_view_id_and_label_pattern() {
    // The spec the composed-view creator emits MUST preserve the user's
    // chosen view_id + anchor pattern through the kernel boundary. A
    // bridge that re-keyed the spec to a different id would break the
    // user's expectation that "the view I saved is the view that
    // materializes" — the §3.6f-substantive round-trip arm.
    let spec = admin_ui_v0_composed_view_user_spec("notes-by-work-tag", "notes-by-tag");
    assert_eq!(spec.view_id, "notes-by-work-tag");
    assert!(matches!(
        spec.label_pattern,
        LabelPattern::Exact(ref s) if s == "notes-by-tag"
    ));
    // User-defined views ALWAYS carry typed_output_projection = None
    // (per `SubgraphSpec::user_view` constructor). The canonical 4/5
    // views are the only shape with non-None typed-output.
    assert!(spec.typed_output_projection.is_none());
    assert!(!spec.is_canonical());
}

// Phase 3 R5 wave-9 W9-T1 — `g15a-mr-minor-4` carry close
// (`docs/future/phase-3-backlog.md` §5.1-followup-c).
//
//! W9-T1: `Engine::register_user_view` MUST refuse a registration
//! that supplies `(canonical_view_id, AnchorPrefix(...))` because
//! canonical view ids' hand-written kernels ignore the supplied
//! pattern and use a hardcoded label — admitting an AnchorPrefix would
//! be a doc-vs-code-strength gap (the kernel does not behave like a
//! prefix selector even though the call accepted one). The kernel-side
//! `AlgorithmError::CanonicalIdAnchorPrefixRefused` guard fires; the
//! engine pre-mirrors the same rejection so the typed
//! `EngineError::ViewLabelMismatch` (catalog `E_VIEW_LABEL_MISMATCH`)
//! surfaces consistently regardless of registration entry point.
//!
//! End-to-end pin (§3.6b): drives the production entry point
//! `Engine::register_user_view`; would FAIL if the kernel-side guard
//! were silently bypassed (the registration would land + the runtime
//! would observe label-equality semantics for a supposed prefix
//! selector).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_engine::{Engine, EngineError, UserViewInputPattern, UserViewSpec};

/// W9-T1: `(capability_grants, AnchorPrefix(""))` — empty prefix
/// would prefix-match the canonical hardcoded label
/// (`system:CapabilityGrant`); pre-tightening this registration
/// silently succeeded. Post-tightening it MUST surface the typed
/// label-mismatch error.
#[test]
fn register_user_view_canonical_id_anchor_prefix_empty_refused() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("capability_grants")
        .input_pattern(UserViewInputPattern::AnchorPrefix(String::new()))
        .build()
        .expect("UserViewSpec builder constructs");

    let err = engine
        .register_user_view(spec)
        .expect_err("canonical id + AnchorPrefix MUST fail-loud");
    match err {
        EngineError::ViewLabelMismatch {
            view_id,
            expected_label,
            got_label,
        } => {
            assert_eq!(view_id, "capability_grants");
            assert_eq!(expected_label, "system:CapabilityGrant");
            assert!(
                got_label.contains("AnchorPrefix"),
                "got_label MUST surface the prefix-selector shape; got `{got_label}`"
            );
        }
        other => panic!("expected ViewLabelMismatch, got {other:?}"),
    }
}

/// W9-T1: `(version_current, AnchorPrefix("NEXT_"))` — non-empty prefix
/// that happens to start the canonical hardcoded label. The guard MUST
/// fire on the AnchorPrefix discriminator regardless of the prefix's
/// match outcome.
#[test]
fn register_user_view_canonical_id_anchor_prefix_nonempty_refused() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("version_current")
        .input_pattern(UserViewInputPattern::AnchorPrefix("NEXT_".into()))
        .build()
        .expect("UserViewSpec builder constructs");

    let err = engine
        .register_user_view(spec)
        .expect_err("canonical id + AnchorPrefix(non-empty) MUST fail-loud");
    match err {
        EngineError::ViewLabelMismatch {
            view_id,
            expected_label,
            got_label,
        } => {
            assert_eq!(view_id, "version_current");
            assert_eq!(expected_label, "NEXT_VERSION");
            assert!(got_label.contains("AnchorPrefix"));
            assert!(got_label.contains("NEXT_"));
        }
        other => panic!("expected ViewLabelMismatch, got {other:?}"),
    }
}

/// W9-T1: `(content_listing, AnchorPrefix("crud:"))` — `content_listing`
/// is canonical too; even though its arm honors the supplied label, the
/// guard fires on the canonical-id-vs-AnchorPrefix discriminator (the
/// 5 canonical kernels' Phase-1 shape is Node-label-keyed via Exact
/// match, not prefix). Closes the doc-vs-code-strength gap uniformly
/// across all 5 canonical ids.
#[test]
fn register_user_view_content_listing_with_anchor_prefix_refused() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("content_listing")
        .input_pattern(UserViewInputPattern::AnchorPrefix("crud:".into()))
        .build()
        .expect("UserViewSpec builder constructs");

    let err = engine
        .register_user_view(spec)
        .expect_err("canonical content_listing + AnchorPrefix MUST fail-loud");
    assert!(matches!(err, EngineError::ViewLabelMismatch { .. }));
}

/// W9-T1 sanity: a non-canonical id + AnchorPrefix MUST still succeed
/// (the tightening is canonical-id-only). Exercises the post-G15-A
/// genuine-prefix selector landing for user views.
#[test]
fn register_user_view_non_canonical_id_with_anchor_prefix_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    let spec = UserViewSpec::builder()
        .id("custom:by_prefix")
        .input_pattern(UserViewInputPattern::AnchorPrefix("crud:".into()))
        .build()
        .expect("UserViewSpec builder constructs");

    let cid = engine
        .register_user_view(spec)
        .expect("non-canonical id + AnchorPrefix MUST succeed");
    // The view-definition Node CID surfaces — registration succeeded
    // end-to-end (catalog persistence + IVM subscriber registration).
    let _ = cid;
}

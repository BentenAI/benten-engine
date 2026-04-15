//! Phase 1 R3 integration — Stale view returns error, not stale data.
//!
//! Force View 3 into stale state (via the budget-exceeded path I8). Assert
//! that `engine.read_view(...)` returns E_IVM_VIEW_STALE and that
//! `engine.read_view_allow_stale(...)` returns the last-known-good result.
//!
//! Exercises I8 (per-view budget + stale fallback) wired through the engine
//! public API (N3).
//!
//! **Status:** FAILING until I8 + N3 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Node;
use benten_engine::Engine;

#[test]
fn stale_view_returns_error_not_stale_data() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .ivm_max_work_per_update(1) // force-stale: any non-trivial event exceeds budget
        .build().unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    // Force a handful of writes; at least one will exceed the budget and mark
    // View 3 stale. R4 triage (m4): collect non-ok outcomes via `filter` so
    // an assertion failure names the offending iteration rather than bailing
    // on the first inner `assert!` misfire.
    let failed: Vec<u32> = (0..5u32)
        .filter(|i| {
            let mut props = std::collections::BTreeMap::new();
            props.insert("title".into(), benten_core::Value::Text(format!("p{i}")));
            let outcome = engine
                .call(
                    &handler_id,
                    "post:create",
                    Node::new(vec!["post".into()], props),
                )
                .unwrap();
            !outcome.is_ok_edge()
        })
        .collect();
    assert!(
        failed.is_empty(),
        "write must commit even if IVM is stale; non-ok at iterations: {failed:?}"
    );

    // Strict read_view surfaces stale error.
    let strict = engine.read_view_strict("content_listing_post");
    assert!(strict.is_err(), "strict read_view must error on stale view");
    let err = strict.unwrap_err();
    assert_eq!(err.code(), "E_IVM_VIEW_STALE");

    // Permissive read returns last-known-good (possibly empty).
    let lkg = engine
        .read_view_allow_stale("content_listing_post")
        .expect("allow-stale always returns Ok");
    let _ = lkg
        .as_list()
        .expect("last-known-good is a list (possibly empty)");
}

//! Edge-case tests: subgraph AST cache invalidation on re-registration.
//!
//! R2 landscape §2.6.2 row "AST cache invalidates on re-registration".
//!
//! The AST cache (G2-B, N6) memoises parsed-handler subgraphs keyed by
//! `(handler_id, op, subgraph_cid)`. When the handler entry's CID flips
//! (e.g. via the `testing_force_reregister_with_different_cid` hook used
//! here), the cache key axis on `subgraph_cid` changes, so subsequent
//! lookups miss and the handler is re-parsed.
//!
//! Concerns pinned:
//! - A re-registration that flips the stored `subgraph_cid` for an existing
//!   `(handler_id, op)` pair causes the next call to miss the cache and
//!   re-parse (observable via `testing_parse_counter`).
//! - Re-registration with the SAME content (no CID flip) is a true no-op:
//!   the parse counter does not advance on the next call.
//! - The handler's resolved CID after the flip is the new CID, never the
//!   old one — the call path cannot reach a stale subgraph through the
//!   cached lookup.
//!
//! G11-A Wave 3a rewrite (D12.x): the previous shape called
//! `register_subgraph` twice with different content for the same
//! `handler_id`, contradicting the Phase-1 `DuplicateHandler` contract
//! (`register_subgraph_failures::register_duplicate_handler_id_errors`).
//! The decision is to preserve `DuplicateHandler` and exercise the
//! cache-invalidation contract through the dedicated
//! `testing_force_reregister_with_different_cid` hook on `Engine`.
//! That hook is gated behind `cfg(any(test, feature = "test-helpers"))`
//! so the surface is not reachable from release builds.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Value;
use benten_engine::Engine;
use tempfile::tempdir;

fn engine() -> (tempfile::TempDir, Engine) {
    let dir = tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("ast_inv.redb"))
        .without_versioning()
        .build()
        .unwrap();
    (dir, engine)
}

#[test]
fn ast_cache_invalidates_on_reregister_with_different_cid() {
    let (_dir, engine) = engine();

    // Register a CRUD handler so the dispatch path materialises a real
    // cache entry. `crud("post")` produces handler_id `"crud:post"` and
    // routes through `subgraph_for_crud`, which consults the AST cache
    // keyed on `(handler_id, op, subgraph_cid)`.
    let handler = engine.register_crud("post").unwrap();
    assert_eq!(handler, "crud:post");

    // First call → cache miss, parse counter advances.
    engine.testing_reset_parse_counter();
    engine
        .call_for_test(&handler, "get", Value::unit())
        .unwrap();
    let parse_count_after_first = engine.testing_parse_counter();
    assert!(
        parse_count_after_first > 0,
        "first call must miss the cache and bump the parse counter"
    );

    // Second call at the SAME registered CID → cache hit, parse count
    // unchanged.
    engine
        .call_for_test(&handler, "get", Value::unit())
        .unwrap();
    assert_eq!(
        engine.testing_parse_counter(),
        parse_count_after_first,
        "second call at same subgraph_cid must hit the cache"
    );

    // Force-flip the stored handler CID. This simulates the cache-relevant
    // bit of a re-registration without violating the Phase-1
    // `DuplicateHandler` contract that `register_subgraph` enforces.
    engine
        .testing_force_reregister_with_different_cid(&handler)
        .expect("hook must succeed for a registered handler");

    // Call after the flip → cache miss on the new key, parse counter
    // advances again.
    engine
        .call_for_test(&handler, "get", Value::unit())
        .unwrap();
    assert!(
        engine.testing_parse_counter() > parse_count_after_first,
        "re-registration (flipped subgraph_cid) must invalidate the cached \
         AST and cause a re-parse"
    );
}

#[test]
fn ast_cache_noop_on_reregister_with_identical_cid() {
    // Re-registering the exact same content (no CID change) must be a true
    // no-op for the cache: the parse counter does not advance on the next
    // call. The dispatch path consults `(handler_id, op, subgraph_cid)`,
    // and identical content keeps `subgraph_cid` stable.
    let (_dir, engine) = engine();
    let handler = engine.register_crud("post").unwrap();

    engine.testing_reset_parse_counter();
    engine
        .call_for_test(&handler, "get", Value::unit())
        .unwrap();
    let parse_count = engine.testing_parse_counter();

    // Re-register the same crud label. `register_crud` is idempotent for
    // identical content — the handler entry's CID does not change.
    let handler_again = engine.register_crud("post").unwrap();
    assert_eq!(handler, handler_again);

    engine
        .call_for_test(&handler, "get", Value::unit())
        .unwrap();
    assert_eq!(
        engine.testing_parse_counter(),
        parse_count,
        "identical-CID re-registration must not invalidate the cache"
    );
}

#[test]
fn old_subgraph_cid_not_reachable_through_cached_call_after_reregister() {
    // After a CID flip, the resolver path returns the NEW CID. The engine
    // may still hold the old bytes for audit (Phase-3 concern), but a call
    // via `(handler_id, op)` must resolve to the post-flip CID.
    let (_dir, engine) = engine();
    let handler = engine.register_crud("post").unwrap();

    let old_cid = engine
        .resolve_subgraph_cid_for_test(&handler, "get")
        .expect("handler must resolve");

    // Force-flip the stored handler CID via the test hook.
    engine
        .testing_force_reregister_with_different_cid(&handler)
        .expect("hook must succeed for a registered handler");

    let new_cid = engine
        .resolve_subgraph_cid_for_test(&handler, "get")
        .expect("handler must still resolve after flip");

    assert_ne!(
        new_cid, old_cid,
        "force-reregister must produce a distinct CID"
    );
}

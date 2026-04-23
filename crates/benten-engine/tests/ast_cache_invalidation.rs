//! Edge-case tests: subgraph AST cache invalidation on re-registration.
//!
//! R2 landscape §2.6.2 row "AST cache invalidates on re-registration".
//!
//! The AST cache (G2-B, N6) memoises parsed-handler subgraphs keyed by
//! `SubgraphCacheKey { handler_id, op, subgraph_cid }`. Re-registering a
//! handler under the same `(handler_id, op)` with a different `subgraph_cid`
//! must invalidate the prior cache entry so stale ASTs are not served.
//!
//! Concerns pinned:
//! - Second `register_subgraph` with the same `handler_id+op` but a new
//!   `subgraph_cid` invalidates the prior cache entry.
//! - A subsequent `Engine::call` for that handler re-parses (the cache miss
//!   on the new key is observable via the test harness's parse-counter).
//! - The old subgraph CID is no longer reachable via the cached path.
//! - Re-registration with the SAME subgraph_cid is a no-op — cache stays.
//!
//! R3 red-phase contract: R5 (G2-B) lands `SubgraphCacheKey` keyed by
//! `subgraph_cid`. Tests compile; they fail because the `subgraph_cid` axis
//! on the cache key does not exist yet.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::Value;
use benten_engine::Engine;
use benten_eval::SubgraphBuilder;
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

fn simple_subgraph(handler_id: &str, tag: &str) -> benten_eval::Subgraph {
    let mut sb = SubgraphBuilder::new(handler_id);
    let r = sb.read(tag);
    sb.respond(r);
    sb.build_validated().expect("validation")
}

#[test]
fn ast_cache_invalidates_on_reregister_with_different_cid() {
    let (_dir, engine) = engine();

    // Register handler v1.
    let sg_v1 = simple_subgraph("h1", "v1");
    let cid_v1 = engine.register_subgraph(sg_v1).unwrap();

    // First call → parses + caches.
    engine.call_for_test("h1", "run", Value::unit()).unwrap();
    let parse_count_after_v1_first_call = engine.testing_parse_counter();

    // Second call at same CID → cache hit, parse count unchanged.
    engine.call_for_test("h1", "run", Value::unit()).unwrap();
    assert_eq!(
        engine.testing_parse_counter(),
        parse_count_after_v1_first_call,
        "second call at same subgraph_cid must hit cache"
    );

    // Re-register h1 with a semantically-different subgraph → new CID.
    let sg_v2 = {
        let mut sb = SubgraphBuilder::new("h1");
        let r = sb.read("v2");
        let t = sb.transform(r, "identity");
        sb.respond(t);
        sb.build_validated().unwrap()
    };
    let cid_v2 = engine.register_subgraph(sg_v2).unwrap();
    assert_ne!(cid_v1, cid_v2, "new CID expected for changed subgraph");

    // Call after re-registration → cache miss on the new key, parse count
    // advances.
    engine.call_for_test("h1", "run", Value::unit()).unwrap();
    assert!(
        engine.testing_parse_counter() > parse_count_after_v1_first_call,
        "re-registration must invalidate cached AST and cause a re-parse"
    );
}

#[test]
fn ast_cache_noop_on_reregister_with_identical_cid() {
    // Re-registering the exact same bytes yields the same CID; the cache
    // must NOT be invalidated, parse counter stays.
    let (_dir, engine) = engine();
    let sg = simple_subgraph("h1", "same");
    let cid = engine.register_subgraph(sg.clone()).unwrap();

    engine.call_for_test("h1", "run", Value::unit()).unwrap();
    let parse_count = engine.testing_parse_counter();

    // Re-register same content.
    let cid_again = engine.register_subgraph(sg).unwrap();
    assert_eq!(
        cid, cid_again,
        "identical content must produce identical CID"
    );

    engine.call_for_test("h1", "run", Value::unit()).unwrap();
    assert_eq!(
        engine.testing_parse_counter(),
        parse_count,
        "identical-CID re-registration must not invalidate the cache"
    );
}

#[test]
fn old_subgraph_cid_not_reachable_through_cached_call_after_reregister() {
    // After re-registration, the old subgraph is gone from the cache path.
    // The engine may still keep the old bytes around for audit, but a call
    // via `(handler_id, op)` must resolve to the NEW CID.
    let (_dir, engine) = engine();

    let v1 = simple_subgraph("h1", "v1");
    let old_cid = engine.register_subgraph(v1).unwrap();

    let v2 = {
        let mut sb = SubgraphBuilder::new("h1");
        let r = sb.read("v2");
        sb.respond(r);
        sb.build_validated().unwrap()
    };
    let new_cid = engine.register_subgraph(v2).unwrap();

    let resolved = engine
        .resolve_subgraph_cid_for_test("h1", "run")
        .expect("handler must resolve");
    assert_eq!(
        resolved, new_cid,
        "post-reregister call path must resolve to new CID, not old"
    );
    assert_ne!(resolved, old_cid);
}

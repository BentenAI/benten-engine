//! R6FP-tail (NEW-1) BLOCKER regression test.
//!
//! Pre-NEW-1 the user-facing `Engine::read_view*` healthy-view path
//! unconditionally returned `Outcome { list: Some(Vec::new()), .. }`
//! (pre-NEW-1 shape of `engine_views.rs::read_view_with`; the body
//! moved + grew during the NEW-1 fix).
//! `Subscriber::read_view` exists + works + is consumed by
//! `resolve_list_via_view_or_backend` for handler-internal listings, but
//! the user-facing API never called it; subgraphs composing READ_VIEW
//! against a registered view silently saw empty results.
//!
//! This test pins the post-NEW-1 wire-through: writes data through the
//! crud handler (which feeds the `content_listing_post` IVM view via the
//! engine's auto-registered subscriber) + asserts that
//! `engine.read_view("content_listing_post")` returns a non-empty list
//! whose contents match the writes.
//!
//! See `.addl/phase-2b/r6-round-1-deep-sweep-engine-eval-asymmetry.md`
//! NEW-1 for the full failure narrative.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::{Engine, ReadViewOptions};
use std::collections::BTreeMap;

fn post(n: u32) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(format!("post-{n:02}")));
    Node::new(vec!["post".into()], props)
}

/// `engine_read_view_returns_view_current_state_post_writes` — NEW-1
/// regression pin. Writes 3 posts via the crud handler; calls
/// `engine.read_view("content_listing")` (the canonical id under which
/// the auto-registered ContentListingView lives — the view's `id()` is
/// the constant `"content_listing"` regardless of the label it watches);
/// asserts the returned `Outcome.list` carries 3 Nodes (NOT the
/// pre-fix empty-list stub).
#[test]
fn engine_read_view_returns_view_current_state_post_writes() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();

    // 3 writes; each routes through the IVM subscriber + lands in the
    // auto-registered content_listing view.
    for i in 0..3u32 {
        engine.call(&handler_id, "post:create", post(i)).unwrap();
    }

    // Pre-NEW-1: this returns `Outcome { list: Some(Vec::new()) }` regardless
    // of the writes above. Post-NEW-1: it routes through
    // `Subscriber::read_view` and returns the view's current state.
    let outcome = engine
        .read_view("content_listing")
        .expect("healthy registered view returns Ok");
    let list = outcome
        .as_list()
        .expect("read_view populates Outcome.list when the view is healthy");
    assert_eq!(
        list.len(),
        3,
        "Engine::read_view must reflect the 3 writes through the live IVM subscriber, \
         not the pre-NEW-1 empty-list stub"
    );

    // Spot-check: the listed Nodes carry the write's `post` label.
    for node in list {
        assert!(
            node.labels.iter().any(|l| l == "post"),
            "listed Node must carry the 'post' label; got labels={:?}",
            node.labels
        );
    }
}

/// `engine_read_view_with_explicit_strict_returns_state` — NEW-1 sibling
/// pin against the explicit `read_view_with(..., strict)` variant. The
/// pre-NEW-1 stub bug applied to all four entry points
/// (`read_view`, `read_view_with`, `read_view_strict`,
/// `read_view_allow_stale`) — assert the strict variant also returns
/// real contents, not empty.
#[test]
fn engine_read_view_with_explicit_strict_returns_state() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();
    engine.call(&handler_id, "post:create", post(1)).unwrap();

    let outcome = engine
        .read_view_with("content_listing", ReadViewOptions::strict())
        .expect("healthy view + strict opts: Ok");
    let list = outcome.as_list().expect("strict read populates list");
    assert_eq!(
        list.len(),
        1,
        "read_view_with(strict) must reflect the 1 write through the live IVM subscriber"
    );
}

/// `engine_read_view_namespaced_alias_routes_through_subscriber` — NEW-1
/// pin for the `system:ivm:<id>` namespaced alias path. The
/// `engine_views.rs::read_view_with` body normalises this prefix before
/// consulting the subscriber; assert the projection still fires for the
/// alias form.
#[test]
fn engine_read_view_namespaced_alias_routes_through_subscriber() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let handler_id = engine.register_crud("post").unwrap();
    engine.call(&handler_id, "post:create", post(7)).unwrap();
    engine.call(&handler_id, "post:create", post(8)).unwrap();

    let outcome = engine
        .read_view("system:ivm:content_listing")
        .expect("namespaced alias routes through subscriber");
    let list = outcome.as_list().expect("alias path populates list");
    assert_eq!(
        list.len(),
        2,
        "namespaced alias must reflect the 2 writes after id normalisation"
    );
}

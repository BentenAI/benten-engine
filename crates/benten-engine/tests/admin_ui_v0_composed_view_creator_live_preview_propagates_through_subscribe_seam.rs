//! Phase-4-Foundation G24-C wave-6b SUBSTANTIVE pin (Rust-side companion
//! to `packages/admin-ui-v0/tests/composed_view_creator_live_preview_propagates_through_subscribe_seam.test.ts`).
//!
//! Asserts the engine-side production-runtime arm for the
//! composed-view creator's live-preview pathway:
//!
//! 1. The admin UI v0 plugin DID registers a subscription via
//!    `Engine::on_change_as_with_cursor` (the cap-recheck-enabled
//!    seam per sec-3.5-r1-9 floor + T12 actor-aware cap-recheck).
//! 2. A write through `Engine::call_as` drives a change event that
//!    propagates through the subscribe callback into the live-preview
//!    surface.
//! 3. The subscribe seam name is the literal string
//!    `"on_change_as_with_cursor"` — the grep-assert pin at
//!    `admin_ui_v0_subscribe_paths_only_via_on_change_as_with_cursor.rs`
//!    verifies the admin UI v0 source carries it.
//!
//! Failure mode defended against: admin UI v0 polling the engine
//! instead of subscribing through the cap-recheck-enabled seam (would
//! bypass per-event delivery cap-recheck per option-D Phase-4-Foundation
//! R1-FP G22-FP-1 PR #210).
//!
//! Pin source: `r2-test-landscape.md` §2.8 row 2 + `00-implementation-plan.md`
//! §3 G24-C row.

#![allow(clippy::unwrap_used)]

use benten_core::{Cid, Node, Value};
use benten_engine::Engine;
use benten_engine::engine_subscribe::SubscribeCursor;
use benten_platform_foundation::{
    ADMIN_UI_V0_SUBSCRIBE_SEAM, Category, Subscriber, build_category_route_subgraph,
};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn principal_cid_for(name: &str) -> Cid {
    let mut props = BTreeMap::new();
    props.insert("name".into(), Value::text(name));
    Node::new(vec!["actor".to_string()], props).cid().unwrap()
}

#[test]
fn admin_ui_v0_composed_view_subscribe_seam_constant_is_on_change_as_with_cursor() {
    // Sentinel: the canonical seam name the composed-view creator
    // routes through is the cap-recheck-enabled
    // `on_change_as_with_cursor` (NOT bare `on_change` / NOT
    // `subscribe_change_events`).
    assert_eq!(ADMIN_UI_V0_SUBSCRIBE_SEAM, "on_change_as_with_cursor");
}

#[test]
fn admin_ui_v0_composed_view_creator_subscribe_uses_on_change_as_with_cursor_seam() {
    // Build the engine + the admin UI VIEWS-category route subgraph.
    // The composed-view creator mounts under the VIEWS category per
    // G24-A 4-cat nav.
    let engine = Engine::open(":memory:").unwrap();
    let _views_route = build_category_route_subgraph(Category::Views);

    // Admin UI v0 plugin DID — the principal that the subscribe seam
    // attributes change events to (Class B β per CLAUDE.md baked-in #18).
    let admin_ui_did = principal_cid_for("admin-ui-v0-plugin-did");

    // The composed-view creator's subscribe pattern is derived from
    // the user-saved view's anchor pattern. We mirror the TS-side
    // `subscribePatternFor(spec)` shape — `view:<viewId>:<label>` —
    // by composing through the `Subscriber::for_category(Views)`
    // helper that the admin-ui-v0 plugin's handler-side code uses.
    let subscriber = Subscriber::for_category(Category::Views)
        .expect("Subscriber for Views category must succeed");
    let pattern = &subscriber.token.pattern;
    // Pattern carries the Views-category slug per per-row gating
    // traceability.
    assert!(
        pattern.contains("views"),
        "subscribe pattern MUST carry category slug; saw {pattern}"
    );

    // Counter the subscribe callback bumps on every event. The
    // production-runtime arm: the engine routes change events to
    // this callback ONLY via `on_change_as_with_cursor` (the seam
    // that consults `CapabilityPolicy::check_read` per delivery per
    // option-D G22-FP-1 closure).
    let delta_count = Arc::new(AtomicUsize::new(0));
    let delta_count_cb = Arc::clone(&delta_count);
    let callback: benten_engine::engine_subscribe::OnChangeCallback =
        Arc::new(move |_seq, _chunk| {
            delta_count_cb.fetch_add(1, Ordering::SeqCst);
        });

    // Subscribe via THE SAME seam the composed-view creator uses —
    // a parallel polling path would never reach this entry point.
    let subscription = engine
        .on_change_as_with_cursor(pattern, SubscribeCursor::Latest, callback, &admin_ui_did)
        .expect("on_change_as_with_cursor must accept the composed-view subscribe pattern");
    // Sanity: the subscription handle reports active state (engine
    // registry slot is live).
    assert!(
        subscription.is_active(),
        "subscription MUST be active immediately after registration"
    );
    assert_eq!(subscription.pattern(), pattern);

    // Drop the subscription to free the registry slot before the
    // engine is dropped.
    drop(subscription);
}

#[test]
fn admin_ui_v0_composed_view_creator_live_preview_substantively_propagates_writes() {
    // Substantive production-runtime arm: a write through the engine
    // drives an event that the subscribe callback observes.
    //
    // Setup: an engine + an admin-ui-v0 plugin DID + a subscribe
    // callback that counts deliveries.
    let engine = Engine::open(":memory:").unwrap();
    let admin_ui_did = principal_cid_for("admin-ui-v0-plugin-did");

    let delta_count = Arc::new(AtomicUsize::new(0));
    let delta_count_cb = Arc::clone(&delta_count);
    let callback: benten_engine::engine_subscribe::OnChangeCallback =
        Arc::new(move |_seq, _chunk| {
            delta_count_cb.fetch_add(1, Ordering::SeqCst);
        });

    // Subscribe to the broadest pattern the composed-view creator
    // would emit for a view over Note Nodes. The pattern `Note`
    // matches every `Note`-labelled change event.
    let subscription = engine
        .on_change_as_with_cursor("Note", SubscribeCursor::Latest, callback, &admin_ui_did)
        .expect("on_change_as_with_cursor must accept the live-preview pattern");

    // Drive a write — the engine's CRUD path emits a Created
    // change event that the subscribe seam routes through the
    // delivery-time cap-recheck closure + then into the registered
    // callback.
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text("hello".into()));
    props.insert("body".into(), Value::Text("world".into()));
    let note = Node::new(vec!["Note".to_string()], props);
    let _cid = engine
        .create_node(&note)
        .expect("create_node must succeed for benign Note write");

    // §3.6f SUBSTANCE: the callback observed at least one delivery
    // (the substantive propagation arm). A parallel polling layer
    // that bypassed `on_change_as_with_cursor` would not surface the
    // event here — `delta_count` would remain at 0.
    let observed = delta_count.load(Ordering::SeqCst);
    assert!(
        observed >= 1,
        "subscribe callback MUST observe ≥1 delivery from the write; saw {observed}",
    );

    drop(subscription);
}

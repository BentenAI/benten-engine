//! GREEN-PHASE pins for G19-C1 UserView.snapshot() + onUpdate()
//! (wave-7 parallel; §7.1.3).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.7 G19-C1 +
//! `.addl/phase-3/00-implementation-plan.md` §3 G19-C1 must-pass column):
//!
//! - `tests/user_view_snapshot_returns_current_materialized_rows` — §7.1.3
//! - `tests/user_view_on_update_yields_incremental_deltas` — §7.1.3
//!
//! ## What G19-C1 establishes (§7.1.3)
//!
//! `crates/benten-engine/src/engine_views.rs` adds:
//! - `user_view_snapshot(view_id)` returning the current materialized
//!   row set
//! - `user_view_on_update(view_id)` returning a `ChangeProbe` whose
//!   `drain()` yields incremental deltas (not full re-snapshots)
//!
//! Per cross-wave-file-touch note (seq-minor-1): post-G15-B
//! `engine_views.rs` already carries `PrefixMatcher` selector type
//! landed; G19-C1's user_view_snapshot/on_update is additive.

#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;

use benten_core::{Node, Value};
use benten_engine::{Engine, UserViewInputPattern, UserViewSpec};

/// Helper: build a CRUD-style `post` handler so post:create writes Nodes
/// with label `"post"`. Mirrors the engine_api_surface.rs / register_crud
/// pattern so the test exercises the production WRITE path.
fn open_engine_with_post_handler() -> (tempfile::TempDir, Engine, String) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let handler_id = engine.register_crud("post").unwrap();
    (dir, engine, handler_id)
}

#[test]
fn user_view_snapshot_returns_current_materialized_rows() {
    // §7.1.3 pin. G19-C1 wires `Engine::user_view_snapshot(view_id)`
    // returning the current materialized rows for a registered view.
    let (_dir, engine, handler_id) = open_engine_with_post_handler();

    let spec = UserViewSpec::builder()
        .id("custom:posts_snapshot")
        .input_pattern(UserViewInputPattern::Label("post".into()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    // Drive WRITEs through the production-grade entry point so the
    // ChangeBroadcast actually fires + the IVM subscriber materialises.
    let mut props1 = BTreeMap::new();
    props1.insert("title".to_string(), Value::Text("first".into()));
    engine
        .call(
            &handler_id,
            "post:create",
            Node::new(vec!["post".into()], props1),
        )
        .unwrap();
    let mut props2 = BTreeMap::new();
    props2.insert("title".to_string(), Value::Text("second".into()));
    engine
        .call(
            &handler_id,
            "post:create",
            Node::new(vec!["post".into()], props2),
        )
        .unwrap();

    // OBSERVABLE consequence: callers receive a point-in-time snapshot
    // of the materialized view without rolling their own materialization
    // walk via the lower-level IVM subscriber API.
    let rows = engine
        .user_view_snapshot("custom:posts_snapshot")
        .unwrap()
        .expect("registered user view must surface a snapshot");
    assert_eq!(
        rows.len(),
        2,
        "user_view_snapshot must return all materialized rows"
    );
}

#[test]
fn user_view_snapshot_unknown_view_id_returns_none() {
    // Fail-soft contract: unknown view ids resolve to Ok(None) so the
    // napi bridge can lift to a typed `E_UNKNOWN_VIEW` without a panic.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let result = engine.user_view_snapshot("custom:does_not_exist").unwrap();
    assert!(
        result.is_none(),
        "unknown view id MUST resolve to Ok(None), not error"
    );
}

#[test]
fn user_view_on_update_yields_incremental_deltas() {
    // §7.1.3 pin. G19-C1 wires `Engine::user_view_on_update(view_id)`
    // returning a `ChangeProbe` filtered to the view's input-pattern
    // label — drain() yields incremental ChangeEvents, not full
    // re-snapshots. Defends against the failure shape where on_update
    // returns full materialisations (would inflate per-write cost from
    // O(delta) to O(view-size)).
    let (_dir, engine, handler_id) = open_engine_with_post_handler();

    let spec = UserViewSpec::builder()
        .id("custom:posts_updates")
        .input_pattern(UserViewInputPattern::Label("post".into()))
        .build()
        .unwrap();
    engine.register_user_view(spec).unwrap();

    // Open the probe AFTER the registration so the start_offset captures
    // the post-registration state of the change stream.
    let probe = engine
        .user_view_on_update("custom:posts_updates")
        .unwrap()
        .expect("registered view yields a probe handle");

    // Drive a single WRITE — observable consequence is a single delta.
    let mut props = BTreeMap::new();
    props.insert("title".to_string(), Value::Text("new".into()));
    engine
        .call(
            &handler_id,
            "post:create",
            Node::new(vec!["post".into()], props),
        )
        .unwrap();

    let events = probe.drain();
    assert_eq!(
        events.len(),
        1,
        "on_update must fire ONCE per write to a matching label, not return a full snapshot"
    );
    // Verify the delta carries the matching label (label-filtered probe).
    assert!(
        events[0].labels.iter().any(|l| l == "post"),
        "filtered probe MUST only yield events whose labels include the view's input label; got {:?}",
        events[0].labels
    );
}

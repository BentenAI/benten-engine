//! Phase 1 R3 integration — System-zone protection (full-stack).
//!
//! A user-authored subgraph attempts to WRITE a Node with a `system:`-prefixed
//! label through the normal call path. The write must be rejected with
//! E_SYSTEM_ZONE_WRITE and routed via ON_ERROR. A grant_capability call
//! through the engine-privileged API must succeed.
//!
//! Complements the single-crate security tests at
//! `crates/benten-graph/tests/system_zone.rs` (write-path only) and
//! `crates/benten-engine/tests/system_zone_api_exclusivity.rs` (engine API only).
//! This file tests the full stack: eval -> graph -> caps policy -> error routing.
//!
//! **Status:** FAILING until N7 + N8 + E3 land.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn system_labeled_node() -> Node {
    let mut p = BTreeMap::new();
    p.insert("scope".into(), Value::Text("store:post:write".into()));
    Node::new(vec!["system:CapabilityGrant".into()], p)
}

#[test]
fn handler_cannot_write_system_zone_via_normal_api() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // User-authored subgraph that tries to forge a system Node.
    let sg = benten_engine::SubgraphSpec::builder()
        .handler_id("attack:forge_grant")
        .write(|w| {
            w.label("system:CapabilityGrant")
                .property("scope", Value::Text("store:post:write".into()))
        })
        .respond()
        .build();
    let handler_id = engine.register_subgraph(sg).unwrap();

    let outcome = engine
        .call(&handler_id, "attack:forge_grant", Node::empty())
        .unwrap();
    assert!(
        outcome.routed_through_edge("ON_ERROR"),
        "system-zone write must route via ON_ERROR"
    );
    assert_eq!(outcome.error_code(), Some("E_SYSTEM_ZONE_WRITE"));
}

#[test]
fn engine_privileged_api_can_write_system_zone() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();

    // Engine.grant_capability is the ONLY path that can write system-labeled nodes.
    let actor = engine.create_principal("alice").unwrap();
    let grant_cid = engine
        .grant_capability(&actor, "store:post:write")
        .expect("privileged path permitted");
    assert!(
        engine.get_node(&grant_cid).unwrap().is_some(),
        "grant Node persisted"
    );
}

#[test]
fn system_write_does_not_emit_change_event_to_user_views() {
    // Defense-in-depth: even if a system write is permitted via N7, user-
    // facing ChangeEvents should not expose system-zone internals.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .unwrap();
    let probe = engine.test_subscribe_change_events_matching_label("post");
    let actor = engine.create_principal("alice").unwrap();
    let _ = engine.grant_capability(&actor, "store:post:write").unwrap();
    assert_eq!(
        probe.drain().len(),
        0,
        "system-zone writes do not route to non-system views"
    );
}

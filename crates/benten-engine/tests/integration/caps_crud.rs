//! Phase 1 R3 integration — Capability-gated CRUD round-trip (Exit-criterion #3).
//!
//! Registers `crud('post')`, grants a WRITE capability, issues `post:create` and
//! asserts success. Revokes the grant, reissues `post:create`, and asserts the
//! response routed through the handler's `ON_DENIED` edge with error code
//! `E_CAP_DENIED`.
//!
//! This is the load-bearing integration test proving the end-to-end path:
//! benten-eval `requires` property recognition → benten-caps pre-write hook →
//! benten-graph transaction primitive aborts commit → typed error edge routing.
//!
//! **Status:** FAILING until G2–G8 land. Compile fails until Engine exposes
//! `register_subgraph` / `call` / `grant_capability` / `revoke_capability`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "integration tests exercise panics explicitly"
)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

/// Helper: build a minimal `post` Node (labels = ["post"]).
fn post_node(title: &str, body: &str) -> Node {
    let mut props = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    props.insert("body".into(), Value::Text(body.into()));
    Node::new(vec!["post".into()], props)
}

#[test]
fn revoked_cap_routes_to_on_denied() {
    // GIVEN: an engine with the NoAuth default swapped for a grant-backed policy
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed() // Phase 1 placeholder for GrantPolicy
        .build()
        .expect("engine opens");

    // WHEN: register crud('post')
    let handler_id = engine
        .register_crud("post")
        .expect("registration succeeds — invariants 1/2/3/5/6/9/10/12 pass");

    // WHEN: grant store:post:write, call post:create with input
    let actor = engine.create_principal("alice").unwrap();
    let _grant = engine
        .grant_capability(&actor, "store:post:write")
        .expect("grant succeeds via engine-privileged path");

    let created = engine
        .call(&handler_id, "post:create", post_node("first", "hello"))
        .expect("post:create under a live grant returns Ok");
    assert!(created.is_ok_edge(), "must route through success edge");

    // WHEN: revoke, then call again
    engine
        .revoke_capability(&actor, "store:post:write")
        .unwrap();

    let denied = engine
        .call(&handler_id, "post:create", post_node("second", "world"))
        .expect("call itself returns Ok even when routing to ON_DENIED");

    // THEN: response arrives through ON_DENIED with E_CAP_DENIED
    assert!(
        denied.routed_through_edge("ON_DENIED"),
        "expected route via ON_DENIED; got {:?}",
        denied.edge_taken()
    );
    assert_eq!(
        denied.error_code(),
        Some("E_CAP_DENIED"),
        "error must map to the stable catalog code"
    );
}

#[test]
fn no_auth_default_permits_all_crud_operations() {
    // Regression for the thinness-test path: engine with NoAuthBackend default
    // permits every WRITE without a grant. This protects the "Phase 1 DX
    // requires zero-config" property — if NoAuth accidentally denies anything,
    // `npx create-benten-app` fails on first `npm test`.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .expect("engine opens with NoAuth default");

    let handler_id = engine.register_crud("post").unwrap();

    for i in 0..3 {
        let outcome = engine
            .call(
                &handler_id,
                "post:create",
                post_node(&format!("title-{i}"), "body"),
            )
            .unwrap();
        assert!(outcome.is_ok_edge(), "NoAuth must not deny");
    }
}

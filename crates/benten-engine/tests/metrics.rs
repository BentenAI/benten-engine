//! Named compromise #5 regression: per-capability write metrics.
//!
//! SECURITY-POSTURE.md compromise #5 promises:
//!   *"What IS recorded: per-engine write counters surface in
//!    `engine.metrics_snapshot()` under `benten.writes.committed` and
//!    `benten.writes.denied`. Operators can detect abnormal rates
//!    out-of-band."*
//!
//! This test pins that claim. It also locks in the per-scope fan-out
//! (`benten.writes.committed.<scope>` keys) so Phase-3's rate-limit
//! enforcement can read the per-scope counter without re-deriving the
//! scope layout.
//!
//! Related: `crates/benten-engine/tests/integration/caps_crud.rs`
//! exercises the grant-backed policy path; this file exercises the
//! metric surface on top of it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::{Node, Value};
use benten_engine::Engine;
use std::collections::BTreeMap;

fn post_node(title: &str, body: &str) -> Node {
    let mut props: BTreeMap<String, Value> = BTreeMap::new();
    props.insert("title".into(), Value::Text(title.into()));
    props.insert("body".into(), Value::Text(body.into()));
    Node::new(vec!["post".into()], props)
}

#[test]
fn writes_committed_metric_is_recorded() {
    // Under the zero-config NoAuth path the committed counter still
    // increments — the metric is operational, not gated on a policy being
    // plumbed in. Five CRUD creates against the `post` label should leave
    // five increments on the aggregate counter and five increments on the
    // per-scope `store:post:write` key.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .expect("engine opens");

    let handler_id = engine.register_crud("post").unwrap();

    for i in 0..5 {
        let outcome = engine
            .call(&handler_id, "post:create", post_node(&format!("t{i}"), "b"))
            .expect("call under NoAuth is Ok");
        assert!(outcome.is_ok_edge(), "must route through OK edge");
    }

    let metrics = engine.metrics_snapshot();
    let committed = metrics
        .get("benten.writes.committed")
        .copied()
        .expect("benten.writes.committed key is recorded");
    assert!(
        (committed - 5.0).abs() < f64::EPSILON,
        "aggregate committed counter should read 5, got {committed}"
    );
    let per_scope = metrics
        .get("benten.writes.committed.store:post:write")
        .copied()
        .expect("per-scope committed counter surfaces under the flattened key");
    assert!(
        (per_scope - 5.0).abs() < f64::EPSILON,
        "per-scope committed counter should read 5, got {per_scope}"
    );
}

#[test]
fn per_capability_write_metrics_increment() {
    // Direct check against the typed accessor — callers that don't want to
    // parse the flattened string-keyed snapshot can read the per-scope map
    // as a `BTreeMap<String, u64>`.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .expect("engine opens");

    let handler_id = engine.register_crud("post").unwrap();
    for i in 0..3 {
        engine
            .call(&handler_id, "post:create", post_node(&format!("t{i}"), "b"))
            .unwrap();
    }

    let per_scope = engine.capability_writes_committed();
    assert_eq!(
        per_scope.get("store:post:write").copied(),
        Some(3),
        "three create calls must tally three commits under the post scope; got {per_scope:?}",
    );
    assert!(
        engine.capability_writes_denied().is_empty(),
        "no denials under NoAuth",
    );
}

#[test]
fn denied_writes_surface_on_denied_metric() {
    // Under GrantBackedPolicy, revoking the grant and then attempting a
    // write must bump `benten.writes.denied` (aggregate) plus the
    // per-scope denied counter.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .capability_policy_grant_backed()
        .build()
        .expect("engine opens");

    let handler_id = engine.register_crud("post").unwrap();
    let actor = engine.create_principal("alice").unwrap();
    let _grant = engine
        .grant_capability(&actor, "store:post:write")
        .expect("grant succeeds");

    // One committed write under the live grant.
    engine
        .call(&handler_id, "post:create", post_node("first", "hello"))
        .unwrap();

    // Revoke, then the next call should route through ON_DENIED and bump
    // the denied counter.
    engine
        .revoke_capability(&actor, "store:post:write")
        .unwrap();
    let denied_outcome = engine
        .call(&handler_id, "post:create", post_node("second", "world"))
        .expect("call returns Ok even when routed to ON_DENIED");
    assert!(
        denied_outcome.routed_through_edge("ON_DENIED"),
        "expected ON_DENIED; got {:?}",
        denied_outcome.edge_taken()
    );

    let metrics = engine.metrics_snapshot();
    let committed = metrics.get("benten.writes.committed").copied().unwrap();
    let denied = metrics.get("benten.writes.denied").copied().unwrap();
    assert!(
        (committed - 1.0).abs() < f64::EPSILON,
        "one committed write before revocation; got {committed}"
    );
    assert!(
        (denied - 1.0).abs() < f64::EPSILON,
        "one denied write after revocation; got {denied}"
    );
    let per_scope_denied = metrics
        .get("benten.writes.denied.store:post:write")
        .copied()
        .expect("per-scope denied counter should surface");
    assert!(
        (per_scope_denied - 1.0).abs() < f64::EPSILON,
        "per-scope denied should read 1, got {per_scope_denied}"
    );
}

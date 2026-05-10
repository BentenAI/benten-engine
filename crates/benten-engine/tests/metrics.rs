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

/// R6 fp Wave C2 (closes obs-r6r1-2 MAJOR): assert every Phase-3
/// observability counter surfaces in the canonical `metrics_snapshot`
/// key/value bag. Operator dashboards / Phase-4+ telemetry collectors
/// consume `metrics_snapshot` as the dispatch surface; pre-Wave-C2 the
/// per-handler SANDBOX high-water + on_change_registration_count +
/// emit_subscriber_count + sync_replica_cap_recheck_calls accessors
/// existed but were NEVER lifted into the map, so the dashboard surface
/// silently missed every Phase-3-added observable.
///
/// The test asserts the keys are present (would-FAIL-if-no-op'd per
/// pim-2 §3.6b) — adding the metric record without lifting it would
/// fail this pin even though the per-handler accessor still works.
#[test]
fn metrics_snapshot_lifts_phase_3_observability_counters() {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::builder()
        .path(dir.path().join("benten.redb"))
        .build()
        .expect("engine opens");

    let metrics = engine.metrics_snapshot();

    // SUBSCRIBE on_change registration count — value can be zero, key
    // MUST surface even when no subscribers are registered (Phase-3
    // operator-dashboard contract: structural-shape stable).
    assert!(
        metrics.contains_key("benten.subscribe.on_change_registration_count"),
        "metrics_snapshot must surface SUBSCRIBE on_change_registration_count; \
         keys present: {:?}",
        metrics.keys().collect::<Vec<_>>(),
    );

    // EMIT subscriber count — same shape contract: surfaces zero when
    // no broadcast subscribers are registered.
    assert!(
        metrics.contains_key("benten.emit.subscriber_count"),
        "metrics_snapshot must surface EMIT subscriber_count; \
         keys present: {:?}",
        metrics.keys().collect::<Vec<_>>(),
    );

    // Sync-replica per-row cap-recheck call count — G16-B-F sec-r4r1-2
    // closure observability surface. Value is zero pre-merge; key still
    // surfaces.
    assert!(
        metrics.contains_key("benten.sync_replica.cap_recheck_calls"),
        "metrics_snapshot must surface sync_replica.cap_recheck_calls; \
         keys present: {:?}",
        metrics.keys().collect::<Vec<_>>(),
    );

    // STREAM active-count — R6 R2 obs-r6-r2-1 partial-close lift
    // (engine_diagnostics.rs ~L312-316). Phase-2b stream surface; same
    // structural-shape contract as the SUBSCRIBE/EMIT/sync_replica arms
    // above (key surfaces zero pre-stream-open, value increments on
    // call_stream + decrements on Drop). Defends against silent
    // regression of the lift via per-family split / feature-gate /
    // refactor — pim-2 §3.6b sub-rule 4 closure-pin per-finding
    // granularity. Closes obs-final-1.
    assert!(
        metrics.contains_key("benten.stream.active_count"),
        "metrics_snapshot must surface STREAM active_count; \
         keys present: {:?}",
        metrics.keys().collect::<Vec<_>>(),
    );

    // SANDBOX per-handler high-water keys are populated lazily on first
    // SANDBOX invocation — without exercising the SANDBOX surface here
    // (which requires registering a wasm module, out of scope for this
    // pin), the per-handler keys may be absent. The accessor surface
    // is asserted by `tests/describe_sandbox_node_returns_diagnostic_shape.rs`
    // + `tests/metrics_snapshot_includes_sandbox_high_water.rs`.
}

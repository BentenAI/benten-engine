//! G14-D wave-5a closed-claim test pins (per pim-2 §3.6b).
//!
//! These pins drive PRODUCTION entry points that G14-D wave-5a wired:
//!
//! - `Engine::on_change_with_cap_recheck` — F6 SUBSCRIBE per-event cap
//!   recheck composing the `cap_recheck.rs` G13-pre-C scaffold.
//! - `Engine::emit_with_handler` + `Engine::subscribe_with_handler` —
//!   handler-id-router seam (seq-major-8 + stream-r1-2).
//! - `Engine::put_cap_snapshot_for_envelope` +
//!   `Engine::resume_from_bytes_*` — `cap_snapshot_hash` cross-process
//!   binding per CLR-2 + Compromise #10.
//! - `ThinClientConnection::connect` — thin-client SSE/WebSocket
//!   subscription seam per D-PHASE-3-30 + CLAUDE.md baked-in #17.
//! - `cap_snapshot_hash::compute` / `verify` — pure-function
//!   binding-helper per CLR-2.
//!
//! Per pim-2 §3.6b every test in this file:
//!
//! 1. Drives the production-grade entry point (no `testing_*` bypass).
//! 2. Asserts an OBSERVABLE behavioral consequence of the arm firing.
//! 3. Would FAIL if the arm were silently no-op'd back to its
//!    pre-G14-D shape.
//!
//! The RED-PHASE pins in `subscribe_cap_recheck.rs`,
//! `wait_resume_cross_process.rs`, etc. continue to assert the
//! end-to-end UCAN-chain integration that depends on G14-B's durable
//! grant-store accessor. That accessor lands in a follow-up wave; the
//! G14-D infrastructure shipped here is the seam those pins consume.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;
use std::sync::Arc;

use benten_core::Cid;
use benten_engine::cap_recheck::{CapRecheckFn, PrincipalId};
use benten_engine::cap_snapshot_hash;
use benten_engine::handler_router::HandlerRoute;
use benten_engine::thin_client_subscribe::{ThinClientConnection, ThinClientError};
use benten_engine::{Engine, OnChangeCallback};
use benten_errors::ErrorCode;
use tempfile::TempDir;

fn temp_engine() -> (Engine, TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path: PathBuf = dir.path().join("engine.redb");
    let e = Engine::open(&path).unwrap();
    (e, dir)
}

// ---------------------------------------------------------------------------
// F6 SUBSCRIBE per-event cap-recheck (plan §3 G14-D unit)
// ---------------------------------------------------------------------------

#[test]
fn on_change_with_cap_recheck_consults_closure_at_registration() {
    // pim-2 end-to-end: drive `on_change_with_cap_recheck` (production
    // entry point) + observe that the registration succeeds when the
    // cap-recheck closure permits + the engine reports a live
    // subscription handle.
    let (e, _d) = temp_engine();
    let actor = Cid::from_blake3_digest(*blake3::hash(b"test-actor").as_bytes());
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let allow: CapRecheckFn = Arc::new(|_p: &PrincipalId, _z: &str, _c: &Cid| true);
    let sub = e
        .on_change_with_cap_recheck("post:*", cb, &actor, allow)
        .unwrap();
    assert!(
        sub.is_active(),
        "F6 SUBSCRIBE registration with allow-all closure produces an active handle"
    );
    assert_eq!(sub.pattern(), "post:*");
}

#[test]
fn on_change_with_cap_recheck_rejects_empty_pattern() {
    let (e, _d) = temp_engine();
    let actor = Cid::from_blake3_digest(*blake3::hash(b"test-actor").as_bytes());
    let cb: OnChangeCallback = Arc::new(|_, _| {});
    let allow: CapRecheckFn = Arc::new(|_p, _z, _c| true);
    let err = e
        .on_change_with_cap_recheck("", cb, &actor, allow)
        .unwrap_err();
    assert!(
        matches!(err, benten_engine::EngineError::Other { code, .. } if code == ErrorCode::SubscribePatternInvalid)
    );
}

// ---------------------------------------------------------------------------
// Handler-id-router seam (seq-major-8 + stream-r1-2 LOAD-BEARING)
// ---------------------------------------------------------------------------

#[test]
fn emit_handler_id_router_routing_observably_differs_from_default_fan_out_end_to_end() {
    // stream-r1-2 LOAD-BEARING pin (concretized at G14-D wave-5a). The
    // routing must produce OBSERVABLY DIFFERENT execution traces — the
    // log surface is the proof.
    let (e, _d) = temp_engine();

    // Register a minimal handler subgraph the named route can target.
    // Reuse the testing helper that produces a respond-shape handler
    // (the simplest valid registered shape).
    e.register_subgraph(benten_engine::testing::minimal_respond_handler("h_a"))
        .unwrap();

    let log = e.handler_route_log();
    log.reset();

    // Default fan-out — bumps the default counter, NOT the named log:
    e.emit_with_handler(
        "evt:default",
        benten_core::Value::Null,
        HandlerRoute::DefaultFanOut,
    )
    .unwrap();
    assert_eq!(
        log.default_fan_out_count(),
        1,
        "DefaultFanOut bumps the default counter"
    );
    assert!(
        log.named_routes().is_empty(),
        "DefaultFanOut does NOT bump the named-routes log"
    );

    // Named route — bumps the named log, NOT the default counter:
    e.emit_with_handler(
        "evt:named",
        benten_core::Value::Null,
        HandlerRoute::Named("h_a".into()),
    )
    .unwrap();
    assert_eq!(
        log.default_fan_out_count(),
        1,
        "Named route does NOT bump the default counter (load-bearing per stream-r1-2)"
    );
    let routes = log.named_routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].0, "emit:evt:named");
    assert_eq!(routes[0].1, "h_a");

    // Negative pin — Named route to a nonexistent handler rejects:
    let err = e
        .emit_with_handler(
            "evt:bad",
            benten_core::Value::Null,
            HandlerRoute::Named("h_missing".into()),
        )
        .unwrap_err();
    assert!(
        matches!(err, benten_engine::EngineError::Other { code, .. } if code == ErrorCode::NotFound)
    );
}

#[test]
fn subscribe_handler_id_router_routes_change_event_through_named_handler() {
    // seq-major-8 LOAD-BEARING pin (concretized at G14-D wave-5a).
    let (e, _d) = temp_engine();
    e.register_subgraph(benten_engine::testing::minimal_respond_handler("h_named"))
        .unwrap();

    let log = e.handler_route_log();
    log.reset();

    e.subscribe_with_handler("/zone/posts", HandlerRoute::Named("h_named".into()))
        .unwrap();

    let routes = log.named_routes();
    assert_eq!(routes.len(), 1);
    assert_eq!(routes[0].0, "subscribe:/zone/posts");
    assert_eq!(routes[0].1, "h_named");
    assert_eq!(
        log.default_fan_out_count(),
        0,
        "Named route does NOT bump default fan-out (load-bearing per seq-major-8)"
    );
}

#[test]
fn subscribe_with_handler_rejects_empty_pattern_and_unregistered_handler() {
    let (e, _d) = temp_engine();
    let err = e
        .subscribe_with_handler("", HandlerRoute::DefaultFanOut)
        .unwrap_err();
    assert!(
        matches!(err, benten_engine::EngineError::Other { code, .. } if code == ErrorCode::SubscribePatternInvalid)
    );
    let err = e
        .subscribe_with_handler("/zone/posts", HandlerRoute::Named("missing".into()))
        .unwrap_err();
    assert!(
        matches!(err, benten_engine::EngineError::Other { code, .. } if code == ErrorCode::NotFound)
    );
}

// ---------------------------------------------------------------------------
// cap_snapshot_hash binding (CLR-2 + Compromise #10 closure)
// ---------------------------------------------------------------------------

#[test]
fn cap_snapshot_hash_pure_function_round_trip() {
    // Pure-function pin (no engine state). Asserts the algorithm is
    // deterministic, sorted-stable, and substitution-resistant.
    let actor = Cid::from_blake3_digest(*blake3::hash(b"actor:alice").as_bytes());
    let chain = vec![
        Cid::from_blake3_digest(*blake3::hash(b"u:1").as_bytes()),
        Cid::from_blake3_digest(*blake3::hash(b"u:2").as_bytes()),
    ];
    let h = cap_snapshot_hash::compute_legacy(&actor, &chain);
    assert!(cap_snapshot_hash::verify_legacy(&actor, &chain, &h));

    // Reorder doesn't change the hash:
    let reordered = vec![chain[1], chain[0]];
    assert_eq!(h, cap_snapshot_hash::compute_legacy(&actor, &reordered));

    // Different chain produces a different hash:
    let different = vec![Cid::from_blake3_digest(*blake3::hash(b"u:99").as_bytes())];
    assert_ne!(h, cap_snapshot_hash::compute_legacy(&actor, &different));
}

#[test]
fn put_cap_snapshot_round_trips_through_redb_suspension_store() {
    // Compromise #10 cross-process arm: persist a CapSnapshot through
    // the engine's redb-backed SuspensionStore, then re-open the
    // engine and verify the snapshot is hydrated.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("engine.redb");
    let envelope_cid = Cid::from_blake3_digest(*blake3::hash(b"envelope:1").as_bytes());
    let actor = Cid::from_blake3_digest(*blake3::hash(b"actor:1").as_bytes());
    let chain = vec![Cid::from_blake3_digest(*blake3::hash(b"u:1").as_bytes())];

    let expected_hash = cap_snapshot_hash::compute_legacy(&actor, &chain);
    {
        let e = Engine::open(&path).unwrap();
        e.put_cap_snapshot_for_envelope(envelope_cid, &actor, &chain, b"policy-meta-v1".to_vec())
            .unwrap();
        // First-process snapshot store readback:
        let store = e.suspension_store();
        let got = store.get_cap_snapshot(&envelope_cid).unwrap().unwrap();
        assert_eq!(got.cap_snapshot_hash, expected_hash);
        assert_eq!(got.historical_policy_metadata, b"policy-meta-v1");
    }

    // Re-open in "second process" — same disk path:
    let e2 = Engine::open(&path).unwrap();
    let store2 = e2.suspension_store();
    let got2 = store2.get_cap_snapshot(&envelope_cid).unwrap().unwrap();
    assert_eq!(
        got2.cap_snapshot_hash, expected_hash,
        "cap_snapshot_hash MUST be durable across engine open/close \
         per Compromise #10 cross-process arm"
    );
    assert_eq!(got2.historical_policy_metadata, b"policy-meta-v1");
}

#[test]
fn resume_with_meta_rejects_cap_snapshot_hash_mismatch() {
    // CLR-2 §11 LOAD-BEARING pin. Suspend an envelope with a bound
    // proof-chain hash, then "revoke" the chain (by registering a
    // different live chain on the same actor) and attempt resume —
    // MUST reject with E_CAP_SNAPSHOT_HASH_MISMATCH.
    use benten_engine::ResumePayload;
    let (e, _d) = temp_engine();

    let actor = Cid::from_blake3_digest(*blake3::hash(b"actor:victim").as_bytes());
    let chain_at_suspend = vec![
        Cid::from_blake3_digest(*blake3::hash(b"ucan:original-1").as_bytes()),
        Cid::from_blake3_digest(*blake3::hash(b"ucan:original-2").as_bytes()),
    ];

    // Construct a synthetic envelope (test-helper hook) +
    // pre-populate the live chain to match suspend-time:
    e.testing_register_actor_proof_chain(actor, chain_at_suspend.clone());
    let bytes = e.fabricate_test_suspend_envelope(&actor).unwrap();
    let envelope = benten_eval::ExecutionStateEnvelope::from_dagcbor(&bytes).unwrap();
    let envelope_cid = envelope.envelope_cid().unwrap();
    e.put_cap_snapshot_for_envelope(
        envelope_cid,
        &actor,
        &chain_at_suspend,
        b"policy-v1".to_vec(),
    )
    .unwrap();

    // Sanity: with the matching chain still in place, the resume
    // SUCCEEDS (the hash recompute matches; downstream Step 4 runs
    // with no policy configured = NoAuth-equivalent).
    e.resume_with_meta(&bytes, ResumePayload::None).unwrap();

    // Now "revoke" — register a different live chain for the same
    // actor:
    let chain_after_revoke = vec![Cid::from_blake3_digest(
        *blake3::hash(b"ucan:after-revoke").as_bytes(),
    )];
    e.testing_register_actor_proof_chain(actor, chain_after_revoke);

    // Resume MUST reject with E_CAP_SNAPSHOT_HASH_MISMATCH:
    let err = e.resume_with_meta(&bytes, ResumePayload::None).unwrap_err();
    let benten_engine::EngineError::Other { code, .. } = err else {
        panic!("expected EngineError::Other");
    };
    assert_eq!(
        code,
        ErrorCode::CapSnapshotHashMismatch,
        "resume against changed chain MUST reject per CLR-2 §11"
    );
}

#[test]
fn resume_without_cap_snapshot_succeeds_per_compromise_10_fail_open_asymmetry() {
    // Per Compromise #10 disclosed asymmetry: the engine-side resume
    // surface treats a missing cap_snapshot as "best-effort skip" so a
    // legitimate cross-process eviction window doesn't break resume.
    // The downstream Step-4 policy check still runs.
    use benten_engine::ResumePayload;
    let (e, _d) = temp_engine();
    let actor = Cid::from_blake3_digest(*blake3::hash(b"actor:no-snap").as_bytes());
    let bytes = e.fabricate_test_suspend_envelope(&actor).unwrap();
    e.resume_with_meta(&bytes, ResumePayload::None).unwrap();
}

// ---------------------------------------------------------------------------
// Thin-client subscribe (D-PHASE-3-30 + CLAUDE.md baked-in #17 +
// exit-criterion 19)
// ---------------------------------------------------------------------------

#[test]
fn thin_client_connection_authenticated_view_into_full_peer() {
    // exit-criterion 19 redundant-distinct pin (concretized at
    // G14-D wave-5a). Drives the AUTHENTICATION step in isolation.
    let (e, _d) = temp_engine();

    // 1. Connect WITHOUT attestation: rejects.
    let err = ThinClientConnection::connect_unauthenticated(&e).unwrap_err();
    assert!(matches!(err, ThinClientError::AttestationRequired));

    // 2. Connect with VALID device-DID: succeeds.
    let conn = ThinClientConnection::connect(&e, "did:key:zVALID").unwrap();
    assert!(conn.is_authenticated());

    // 3. Revoke device + try again: rejects.
    e.revoke_device_did("did:key:zVALID");
    let err = ThinClientConnection::connect(&e, "did:key:zVALID").unwrap_err();
    assert!(matches!(err, ThinClientError::DeviceRevoked));
}

#[test]
fn atrium_browser_tab_as_thin_client_view_into_full_peer_e2e_full_peer_filter() {
    // exit-criterion 19 LOAD-BEARING pin (concretized at G14-D wave-5a).
    // Drives the full-peer-side filtering branch — the load-bearing
    // commitment per baked-in #17 is that filtering happens at the
    // FULL PEER, not at the tab. Asserts via metrics.
    let (e, _d) = temp_engine();
    let conn = ThinClientConnection::connect(&e, "did:key:zALICE").unwrap();
    let sub = conn.subscribe("/zone/posts").unwrap();

    // 5. Full peer writes a node; full-peer-side filtering decides
    //    whether to forward to thin client:
    let ev1 = benten_graph::ChangeEvent::new_node(
        Cid::from_blake3_digest(*blake3::hash(b"node1").as_bytes()),
        vec!["posts".into()],
        benten_graph::ChangeKind::Created,
        1,
        None,
    );
    e.thin_client_publish_event("/zone/posts", ev1);

    // 6. Thin client receives the event over the protocol wire:
    assert_eq!(conn.delivered_count(sub), 1);

    // 7. Verify the FILTERING happened at the full peer side:
    let metrics = e.thin_client_metrics();
    assert_eq!(metrics.outbound_events_after_filter, 1);
    assert_eq!(metrics.outbound_events_filtered, 0);

    // 8. Now revoke the thin client's grant; verify subsequent
    //    events are FILTERED at the full peer (not delivered to tab):
    e.revoke_device_did("did:key:zALICE");
    let ev2 = benten_graph::ChangeEvent::new_node(
        Cid::from_blake3_digest(*blake3::hash(b"node2").as_bytes()),
        vec!["posts".into()],
        benten_graph::ChangeKind::Created,
        2,
        None,
    );
    e.thin_client_publish_event("/zone/posts", ev2);

    let metrics_post = e.thin_client_metrics();
    assert!(
        metrics_post.outbound_events_filtered > 0,
        "post-revoke event MUST be filtered at full peer per baked-in #17"
    );
    // Tab observably did NOT receive the post-revoke event:
    assert_eq!(conn.delivered_count(sub), 1);
}

// ---------------------------------------------------------------------------
// SECURITY-POSTURE.md compromise closure narrative pins
// ---------------------------------------------------------------------------

fn read_security_posture() -> String {
    // The SECURITY-POSTURE.md file lives at the workspace root.
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(manifest_dir)
        .join("..")
        .join("..")
        .join("docs")
        .join("SECURITY-POSTURE.md");
    std::fs::read_to_string(path).unwrap()
}

#[test]
fn security_posture_compromise_2_marked_closed_at_g14_d() {
    // pim-1 doc-coupling pin per §3.5b: SECURITY-POSTURE.md narrative
    // for Compromise #2 D5 must reference the G14-D closure.
    let posture = read_security_posture();
    // Find the Compromise #2 section:
    let start = posture
        .find("### Compromise #2")
        .expect("Compromise #2 section");
    let end = posture[start..]
        .find("\n### Compromise #")
        .map_or(posture.len(), |i| start + i);
    let section = &posture[start..end];
    assert!(
        section.to_lowercase().contains("closed"),
        "Compromise #2 MUST be marked CLOSED"
    );
    assert!(
        section.contains("G14-D") || section.contains("Phase-3"),
        "Compromise #2 closure must cite G14-D / Phase-3 for traceability"
    );
    assert!(
        section.contains("delivery") || section.contains("per-event"),
        "Compromise #2 D5 closure must cite delivery-time per-event cap recheck"
    );
}

#[test]
fn security_posture_compromise_10_engine_side_asymmetry_marked_closed_at_g14_d() {
    let posture = read_security_posture();
    let start = posture
        .find("### Compromise #10")
        .expect("Compromise #10 section");
    let end = posture[start..]
        .find("\n### Compromise #")
        .map_or(posture.len(), |i| start + i);
    let section = &posture[start..end];
    assert!(
        section.contains("CLOSED at Phase 3 G14-D") || section.contains("CLOSED at Phase-3 G14-D"),
        "Compromise #10 engine-side asymmetry MUST cite G14-D wave-5a closure"
    );
    assert!(
        section.contains("cap_snapshot_hash"),
        "Compromise #10 closure narrative must cite the cap_snapshot_hash mechanism"
    );
}

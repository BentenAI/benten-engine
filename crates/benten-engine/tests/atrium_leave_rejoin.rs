//! Phase-3 §6.12 item 7 LANDED — `AtriumHandle::leave()` + `rejoin()`
//! peer-churn lifecycle pin.
//!
//! ## Pin source
//!
//! - `docs/future/phase-3-backlog.md` §6.12 item 7 (R4b dist-systems
//!   sub-item C carry; Phase-3-close-blocking per Ben ratification
//!   2026-05-09).
//! - plan §1 exit-criterion 16 (multi-device support for a single
//!   identity) — peers leaving / rejoining is normal lifecycle.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `AtriumHandle::leave` / `rejoin` API
//! end-to-end through the engine's apex `apply_atrium_merge`
//! orchestrator. Without the wiring (no flag check at the merge seam,
//! or trust-store wiped on `leave()`), the post-rejoin merge would
//! either silently fail (no AttributionFrame continuity) or refuse the
//! repeated apply (Loro CRDT idempotency violated). The assertions
//! below trip on each failure mode.

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::too_many_lines)]

use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::AtriumHandle;

/// §6.12 item 7 load-bearing pin: peer-B leaves, peer-A keeps writing,
/// peer-B rejoins, and the post-rejoin merge reconciles via Loro CRDT
/// merge.
///
/// Sequence:
/// 1. Two peers A + B; both register zone `/zone/posts`. Peer A writes.
/// 2. Peer B exports + applies via `apply_atrium_merge` (BEFORE
///    leave, so peer B's trust-store + Loro state observe peer A's
///    contribution).
/// 3. Peer B calls `leave()`; flag flips to inactive; sync surfaces
///    return `InvalidState` while inactive.
/// 4. Peer A continues writing (representing the pre-leave window's
///    contributions that peer B missed).
/// 5. Peer B calls `rejoin()`; flag flips to active.
/// 6. Peer A's NEW Loro export is applied at peer B via
///    `apply_atrium_merge` (post-rejoin).
/// 7. Re-apply the SAME bytes a second time (replay safety — post
///    #615/#617 Inv-13 Row-1 close, the identical-content re-persist
///    is now a hard immutability refusal, not a silent REPLACE; the
///    chain stays uncorrupted + the handle stays active).
///
/// Observable consequences asserted:
/// - (i) post-rejoin `apply_atrium_merge` succeeds + advances peer B's
///   CURRENT pointer to the newly-minted merge Version Node.
/// - (ii) the merged Version's `AttributionFrame` carries
///   `peer_did_set` + `sync_hop_depth` slots populated (continuity of
///   peer-DID provenance across the leave-rejoin window via the
///   surviving trust-store).
/// - (iii) re-applying the same bytes a second time is refused by
///   Inv-13 Row-1 (replay safety via hard refusal; chain uncorrupted;
///   handle still active — tail-of-pin replay-safety anchor).
#[tokio::test]
async fn peer_leave_then_rejoin_reconciles_state_via_loro_merge() {
    let dir = tempfile::tempdir().unwrap();
    let engine_b = Engine::open(dir.path().join("benten-b.redb")).unwrap();

    // Two atrium peers: peer A is the remote producer; peer B is the
    // receiving engine that we exercise `leave()` / `rejoin()` against.
    let peer_b = engine_b
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();

    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;

    let peer_a_node_id = peer_a.hlc_node_id();
    let peer_b_node_id = peer_b.hlc_node_id();

    // Pre-leave window: peer A writes a first contribution; peer B
    // observes it via apply_atrium_merge (the trust-store + Loro state
    // populated here MUST survive across the leave-rejoin window).
    peer_a
        .with_zone(zone, |doc| {
            doc.set_property("title", "pre-leave", BentenHlc::new(100, 0, peer_a_node_id))
                .unwrap();
        })
        .await
        .unwrap();

    peer_b
        .register_peer_did(peer_a_node_id, format!("did:key:peer-a:{peer_a_node_id}"))
        .await;

    let pre_leave_bytes = peer_a
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let anchor = engine_b.create_anchor("post:p1").unwrap();
    let pre_leave_merge_cid = engine_b
        .apply_atrium_merge(&peer_b, &anchor, zone, &pre_leave_bytes, 0)
        .await
        .expect("pre-leave apply_atrium_merge must succeed");

    // (a) Peer B is initially active.
    assert!(
        peer_b.is_active(),
        "post-open AtriumHandle MUST report is_active=true"
    );

    // (b) Peer B leaves: flag flips to inactive; sync surfaces refuse
    //     merge attempts while inactive.
    peer_b.leave().await.expect("leave() is infallible");
    assert!(
        !peer_b.is_active(),
        "post-leave AtriumHandle MUST report is_active=false"
    );

    // While inactive, an apply_atrium_merge attempt MUST fail with
    // an InvalidState surface — defending against orphaned ChangeEvent
    // fan-out per the R4b dist-systems lens carry.
    let mid_leave_bytes = peer_a
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let mid_leave_attempt = engine_b
        .apply_atrium_merge(&peer_b, &anchor, zone, &mid_leave_bytes, 0)
        .await;
    assert!(
        mid_leave_attempt.is_err(),
        "merge against inactive (post-leave) handle MUST refuse, got: {mid_leave_attempt:?}"
    );

    // (c) Peer A keeps writing during peer B's leave window.
    peer_a
        .with_zone(zone, |doc| {
            doc.set_property(
                "title",
                "during-leave",
                BentenHlc::new(200, 0, peer_a_node_id),
            )
            .unwrap();
            doc.set_property("body", "added", BentenHlc::new(250, 0, peer_a_node_id))
                .unwrap();
        })
        .await
        .unwrap();

    // (d) Idempotent leave: a second `leave()` is a no-op.
    peer_b
        .leave()
        .await
        .expect("leave() on already-inactive handle MUST be a no-op");
    assert!(!peer_b.is_active(), "double-leave MUST stay inactive");

    // (e) Peer B rejoins: flag flips back to active.
    peer_b.rejoin().await.expect("rejoin() is infallible");
    assert!(
        peer_b.is_active(),
        "post-rejoin AtriumHandle MUST report is_active=true"
    );

    // (f) Idempotent rejoin: a second `rejoin()` is a no-op.
    peer_b
        .rejoin()
        .await
        .expect("rejoin() on already-active handle MUST be a no-op");
    assert!(peer_b.is_active(), "double-rejoin MUST stay active");

    // (g) Post-rejoin: apply peer A's accumulated state via the engine
    //     orchestrator. This is the load-bearing reconciliation step.
    let post_rejoin_bytes = peer_a
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let post_rejoin_merge_cid = engine_b
        .apply_atrium_merge(&peer_b, &anchor, zone, &post_rejoin_bytes, 0)
        .await
        .expect("post-rejoin apply_atrium_merge MUST succeed (Loro CRDT replay reconciles state)");

    // OBSERVABLE consequence (i): the post-rejoin merge minted a new
    // Version Node + advanced CURRENT.
    assert_ne!(
        post_rejoin_merge_cid, pre_leave_merge_cid,
        "post-rejoin merge MUST mint a new Version CID distinct from the pre-leave merge"
    );
    let current = engine_b.read_current_version(&anchor).unwrap().unwrap();
    assert_eq!(
        current, post_rejoin_merge_cid,
        "CURRENT pointer MUST advance to the post-rejoin merge Version Node"
    );

    // OBSERVABLE consequence (ii): the merged Version's
    // AttributionFrame carries `peer_did_set` + `sync_hop_depth` slots
    // populated. Continuity guarantee: the trust-store registration
    // for `peer_a_node_id` survived the leave-rejoin window so peer A's
    // DID resolves to a real `did:key:` shape (not the `node-id:NNN`
    // fallback) — proving the AttributionFrame.peer_did_set continuity
    // contract.
    let merged = engine_b
        .get_node(&post_rejoin_merge_cid)
        .unwrap()
        .expect("post-rejoin merge Version Node MUST be queryable");
    assert!(
        merged.properties.contains_key("attribution_frame_cid"),
        "post-rejoin merged Version MUST carry attribution_frame_cid slot per D-C \
         (continuity of peer-DID provenance across leave-rejoin window)"
    );
    assert!(
        merged.properties.contains_key("sync_hop_depth"),
        "post-rejoin merged Version MUST carry sync_hop_depth slot per D-PHASE-3-25"
    );

    // The trust-store entry registered pre-leave MUST survive: the
    // peer-DID resolution is non-fallback shape (`did:key:` prefix).
    let resolved = peer_b
        .resolve_peer_dids(&std::collections::BTreeSet::from([peer_a_node_id]))
        .await;
    assert!(
        resolved
            .iter()
            .any(|did| did.starts_with("did:key:peer-a:")),
        "trust-store MUST survive leave-rejoin window; expected did:key:peer-a:* \
         but resolve returned: {resolved:?}"
    );

    // OBSERVABLE consequence (iii): re-applying the SAME bytes a second
    // time. refinement-audit-2026-05 #615/#617 (ST-GRAPH Inv-13 Row-1
    // bypass close, §3.5l cross-crate-consumer class): the replay
    // re-mints the same post-rejoin merged Version Node CID; re-persisting
    // it under User authority is now an Inv-13 Row-1 immutability
    // violation (was a silent REPLACE under the closed bare `put_node`
    // bypass). The load-bearing property — replay does NOT poison further
    // sync activity / does NOT mint a divergent Version — is preserved by
    // the hard refusal (no chain corruption) + the post-cycle
    // `is_active()` guard below. Mirrors the engine-lane's
    // create_node_identical_content_second_put_is_inv13_refused precedent.
    let second_apply_err = engine_b
        .apply_atrium_merge(&peer_b, &anchor, zone, &post_rejoin_bytes, 0)
        .await
        .expect_err(
            "replay of identical post-rejoin bytes re-persists the \
             already-present merged Version Node under User authority — \
             must be refused by Inv-13 (Row 1), not silently REPLACE",
        );
    match second_apply_err {
        benten_engine::EngineError::Graph(g) => {
            let reason = g.to_string();
            assert!(
                reason.contains("immutability violation")
                    && reason.contains("attempted_authority: User"),
                "expected Inv-13 Row-1 immutability violation under User \
                 authority on identical-bytes replay, got: {reason}"
            );
        }
        other => panic!("expected EngineError::Graph (Inv-13 Row-1), got {other:?}"),
    }

    // Final guard: peer B is still active after the full leave →
    // rejoin → multi-merge cycle.
    assert!(
        peer_b.is_active(),
        "post-cycle AtriumHandle MUST remain active after rejoin + 2 merges"
    );
    let _ = peer_b_node_id;
}

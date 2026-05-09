//! G16-B-E LANDED — two-peer atrium bidirectional sync end-to-end pin
//! (exit-criterion 1 LOAD-BEARING).
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `integration/atrium_two_peer_bidirectional_sync` (file
//!   `tests/integration/atrium_two_peer.rs`).
//! - plan §1 exit-criterion 1 (Two full peer instances of `benten-engine`
//!   sync a shared subgraph bidirectionally over iroh transport —
//!   ChangeEvent fan-out + IVM materialisation on the receiver).
//! - plan §3 G16-B + G16-D rows.
//!
//! ## What this pins (G16-B-E LANDED)
//!
//! Two `benten-engine` peers, each with its own redb store, exchange
//! Loro CRDT updates across a real iroh transport stream. The
//! receiver-side `Engine::apply_atrium_merge` mints a Version Node +
//! advances the anchor CURRENT pointer + the receiver's
//! `subscribe_change_events` ChangeProbe drains the post-merge
//! ChangeEvent (closing Sub-item D — receiver-side fan-out).
//!
//! ## OBSERVABLE consequence
//!
//! If the iroh transport silently no-op'd at any leg, OR if the
//! receiver-side `apply_atrium_merge` skipped the Version-Node mint,
//! OR if the ChangeBroadcast wire was broken, this pin fails: peer-B
//! would not observe peer-A's write, OR the ChangeProbe drain would
//! return zero events, OR the anchor CURRENT pointer would not
//! advance.

#![allow(clippy::unwrap_used, clippy::too_many_lines)]
#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;

const ZONE: &str = "/zone/two-peer-e2e";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn atrium_two_peer_bidirectional_sync() {
    // Plan §1 exit-criterion 1 LOAD-BEARING pin.

    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let engine_a = Engine::open(dir_a.path().join("benten.redb")).unwrap();
    let engine_b = Engine::open(dir_b.path().join("benten.redb")).unwrap();

    let atrium_a = engine_a
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    let atrium_b = engine_b
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();

    atrium_a.register_zone(ZONE).await;
    atrium_b.register_zone(ZONE).await;

    let hlc_a = atrium_a.hlc_node_id();
    let hlc_b = atrium_b.hlc_node_id();
    atrium_b.register_peer_did(hlc_a, "did:key:zPeerA").await;
    atrium_a.register_peer_did(hlc_b, "did:key:zPeerB").await;

    atrium_a
        .with_zone(ZONE, |doc| {
            doc.set_property("title", "from_peer_a", BentenHlc::new(100, 0, hlc_a))
                .unwrap();
        })
        .await
        .unwrap();
    atrium_b
        .with_zone(ZONE, |doc| {
            doc.set_property("body", "from_peer_b", BentenHlc::new(200, 0, hlc_b))
                .unwrap();
        })
        .await
        .unwrap();

    let anchor_a = engine_a.create_anchor("two-peer-anchor-a").unwrap();
    let anchor_b = engine_b.create_anchor("two-peer-anchor-b").unwrap();
    let probe_a = engine_a.subscribe_change_events();
    let probe_b = engine_b.subscribe_change_events();

    // Bidirectional sync over real iroh transport.
    let b_addr = atrium_b.loopback_addr().unwrap();
    let atrium_b_clone = atrium_b.clone();
    let zone_owned = ZONE.to_string();
    let accept_task =
        tokio::spawn(async move { atrium_b_clone.accept_sync_subgraph(&zone_owned).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    atrium_a.sync_subgraph(ZONE, b_addr).await.unwrap();
    accept_task.await.unwrap().unwrap();

    // Apply merged state through each engine's apply_atrium_merge,
    // exercising the Version-Node mint + anchor advance + ChangeEvent
    // fan-out path on each side.
    let bytes_a = atrium_a
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let bytes_b = atrium_b
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let cid_on_b = engine_b
        .apply_atrium_merge(&atrium_b, &anchor_b, ZONE, &bytes_a, 0)
        .await
        .unwrap();
    let cid_on_a = engine_a
        .apply_atrium_merge(&atrium_a, &anchor_a, ZONE, &bytes_b, 0)
        .await
        .unwrap();

    // Convergence: both peers' Loro docs carry both keys.
    for (label, atrium) in [("a", &atrium_a), ("b", &atrium_b)] {
        let title = atrium
            .with_zone(ZONE, |doc| doc.get_property("title"))
            .await
            .unwrap();
        let body = atrium
            .with_zone(ZONE, |doc| doc.get_property("body"))
            .await
            .unwrap();
        assert_eq!(
            title.as_deref(),
            Some("from_peer_a"),
            "peer {label} must observe peer-A's title after bidirectional iroh sync"
        );
        assert_eq!(
            body.as_deref(),
            Some("from_peer_b"),
            "peer {label} must observe peer-B's body after bidirectional iroh sync"
        );
    }

    // Each peer's anchor advanced to its merge-Version CID.
    let current_a = engine_a.read_current_version(&anchor_a).unwrap().unwrap();
    let current_b = engine_b.read_current_version(&anchor_b).unwrap().unwrap();
    assert_eq!(current_a, cid_on_a);
    assert_eq!(current_b, cid_on_b);

    // Sub-item D: each peer's ChangeProbe drains a ChangeEvent keyed
    // on the merge-Version CID.
    let events_a = probe_a.drain();
    assert!(
        events_a.iter().any(|e| e.cid == cid_on_a),
        "peer-A ChangeProbe must drain ChangeEvent keyed on cid_on_a {cid_on_a}; got {events_a:?}"
    );
    let events_b = probe_b.drain();
    assert!(
        events_b.iter().any(|e| e.cid == cid_on_b),
        "peer-B ChangeProbe must drain ChangeEvent keyed on cid_on_b {cid_on_b}; got {events_b:?}"
    );
}

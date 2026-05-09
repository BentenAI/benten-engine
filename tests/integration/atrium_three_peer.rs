//! G16-B-E LANDED — three-peer atrium Loro convergence under
//! concurrent writes, exit-criterion 15 LOAD-BEARING.
//!
//! ## Pin source
//!
//! - r2-test-landscape §2.4 G16-B row
//!   `integration/atrium_three_peer_loro_convergence_under_concurrent_writes`.
//! - plan §3 G16-B row line "per C-10 — multi-peer 3+-peer
//!   convergence pin".
//! - `C-10` (3+-peer Loro convergence under concurrent writes).
//! - exit-criterion 15 (atrium as sociotechnical unit;
//!   3+-peer membership + propagation per FULL-ROADMAP.md exit
//!   criterion sentence 3).
//! - `ds-r4-1` (R4 large-council Round 1 distributed-systems lens —
//!   Byzantine-class 3+-peer concurrent-writes-AND-revoke proptest
//!   sibling pin landed here at R4-FP/R3-C; remains RED-PHASE for
//!   G14-D wave-6b pairing).

#![allow(clippy::unwrap_used, clippy::too_many_lines)]
#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::AtriumHandle;

const ZONE: &str = "/zone/three-peer-e2e";

async fn iroh_bidirectional_sync(zone: &str, dialer: &AtriumHandle, accepter: &AtriumHandle) {
    let accepter_addr = accepter.loopback_addr().expect("accepter loopback addr");
    let accepter_clone = accepter.clone();
    let zone_owned = zone.to_string();
    let accept_task =
        tokio::spawn(async move { accepter_clone.accept_sync_subgraph(&zone_owned).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    dialer
        .sync_subgraph(zone, accepter_addr)
        .await
        .expect("dialer sync_subgraph");
    accept_task.await.expect("accept join").expect("accept");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn atrium_three_peer_loro_convergence_under_concurrent_writes() {
    // C-10 + exit-criterion 15 LOAD-BEARING. Three full-peer
    // benten-engine instances; concurrent writes from each peer to
    // distinct keys; bidirectional iroh-transport syncs across all
    // pair directions converge every peer onto the merged state.

    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();
    let dir_c = tempfile::tempdir().unwrap();
    let engine_a = Engine::open(dir_a.path().join("benten.redb")).unwrap();
    let engine_b = Engine::open(dir_b.path().join("benten.redb")).unwrap();
    let engine_c = Engine::open(dir_c.path().join("benten.redb")).unwrap();

    let atrium_a = engine_a
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    let atrium_b = engine_b
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    let atrium_c = engine_c
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();

    for atrium in [&atrium_a, &atrium_b, &atrium_c] {
        atrium.register_zone(ZONE).await;
    }

    let dids = [
        (atrium_a.hlc_node_id(), "did:key:zPeerA"),
        (atrium_b.hlc_node_id(), "did:key:zPeerB"),
        (atrium_c.hlc_node_id(), "did:key:zPeerC"),
    ];
    for atrium in [&atrium_a, &atrium_b, &atrium_c] {
        for (nid, did) in &dids {
            atrium.register_peer_did(*nid, *did).await;
        }
    }

    atrium_a
        .with_zone(ZONE, |doc| {
            doc.set_property("k_a", "from_a", BentenHlc::new(100, 0, dids[0].0))
                .unwrap();
        })
        .await
        .unwrap();
    atrium_b
        .with_zone(ZONE, |doc| {
            doc.set_property("k_b", "from_b", BentenHlc::new(200, 0, dids[1].0))
                .unwrap();
        })
        .await
        .unwrap();
    atrium_c
        .with_zone(ZONE, |doc| {
            doc.set_property("k_c", "from_c", BentenHlc::new(300, 0, dids[2].0))
                .unwrap();
        })
        .await
        .unwrap();

    for _ in 0..2 {
        iroh_bidirectional_sync(ZONE, &atrium_a, &atrium_b).await;
        iroh_bidirectional_sync(ZONE, &atrium_b, &atrium_c).await;
        iroh_bidirectional_sync(ZONE, &atrium_a, &atrium_c).await;
    }

    for (label, atrium) in [("a", &atrium_a), ("b", &atrium_b), ("c", &atrium_c)] {
        let val_a = atrium
            .with_zone(ZONE, |doc| doc.get_property("k_a"))
            .await
            .unwrap();
        let val_b = atrium
            .with_zone(ZONE, |doc| doc.get_property("k_b"))
            .await
            .unwrap();
        let val_c = atrium
            .with_zone(ZONE, |doc| doc.get_property("k_c"))
            .await
            .unwrap();
        assert_eq!(val_a.as_deref(), Some("from_a"), "peer {label} k_a");
        assert_eq!(val_b.as_deref(), Some("from_b"), "peer {label} k_b");
        assert_eq!(val_c.as_deref(), Some("from_c"), "peer {label} k_c");
    }

    for (label, atrium) in [("a", &atrium_a), ("b", &atrium_b), ("c", &atrium_c)] {
        let attr = atrium
            .with_zone(ZONE, |doc| doc.winning_attribution())
            .await
            .unwrap();
        for (nid, _) in &dids {
            assert!(
                attr.contains(nid),
                "peer {label} must surface contributing node-id {nid}; got {attr:?}"
            );
        }
    }

    // Engine-anchor layer: each peer applies its zone export through
    // apply_atrium_merge, mints a Version Node, advances anchor
    // CURRENT, fires ChangeEvents observable via
    // subscribe_change_events.
    let anchor_a = engine_a.create_anchor("three-peer-anchor-a").unwrap();
    let anchor_b = engine_b.create_anchor("three-peer-anchor-b").unwrap();
    let anchor_c = engine_c.create_anchor("three-peer-anchor-c").unwrap();
    let probe_a = engine_a.subscribe_change_events();
    let probe_b = engine_b.subscribe_change_events();
    let probe_c = engine_c.subscribe_change_events();

    let bytes_a = atrium_a
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let bytes_b = atrium_b
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let bytes_c = atrium_c
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let cid_a = engine_a
        .apply_atrium_merge(&atrium_a, &anchor_a, ZONE, &bytes_a, 0)
        .await
        .unwrap();
    let cid_b = engine_b
        .apply_atrium_merge(&atrium_b, &anchor_b, ZONE, &bytes_b, 0)
        .await
        .unwrap();
    let cid_c = engine_c
        .apply_atrium_merge(&atrium_c, &anchor_c, ZONE, &bytes_c, 0)
        .await
        .unwrap();

    for (label, probe, expected_cid) in [
        ("a", &probe_a, cid_a),
        ("b", &probe_b, cid_b),
        ("c", &probe_c, cid_c),
    ] {
        let events = probe.drain();
        assert!(
            events.iter().any(|e| e.cid == expected_cid),
            "peer {label} ChangeProbe must drain ChangeEvent on expected CID {expected_cid}; got {events:?}"
        );
    }
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — Byzantine 3+-peer concurrent-writes-AND-partial-revoke-AND-offline-reconnect proptest. G16-B + G14-D + G16-B-E PR #160 shipped multi-peer iroh sync substantive close; test body pins 3+-peer Byzantine proptest contract that needs 3-peer test infrastructure authoring; un-ignore at next Phase-3-close orchestrator-direct fix-pass batch per Wave-E rationale-only sweep."]
fn atrium_three_peer_concurrent_writes_under_partial_revoke_with_offline_reconnect_converges() {
    // ds-r4-1 stays RED-PHASE — pairs with G14-D wave-6b partial-
    // revoke + G16-C MST-diff-on-reconnect surfaces. G16-B-E ships
    // the happy-path 3-peer iroh convergence (above); the
    // Byzantine-revoke composite is unblocked by the G14-D + G16-C
    // surfaces.
    unimplemented!(
        "G16-B + G14-D wire ds-r4-1 Byzantine 3-peer concurrent-write + partial-revoke + offline-reconnect proptest (10k iterations)"
    );
}

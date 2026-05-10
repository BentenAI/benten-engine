//! G16-B wave-6b LANDED — Atrium open/close lifecycle + two-peer
//! bidirectional sync (load-bearing exit-criterion-1 pin).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `atrium_open_close_lifecycle` +
//!   `atrium_sync_subgraph_two_peer_bidirectional`.
//! - plan §3 G16-B row.
//! - plan §"What success looks like" exit-criterion 1.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! Drives the production `Engine`-side `AtriumHandle::open` /
//! `sync_subgraph` / `accept_sync_subgraph` API end-to-end via the
//! real `benten_sync::transport::Endpoint` loopback canary path. The
//! tests would FAIL if the sync arm silently no-op'd (e.g. wired
//! `LoroDoc::merge` directly without bytes-over-the-wire transport).

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used)]

use benten_core::hlc::BentenHlc;
use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::AtriumHandle;

#[tokio::test]
async fn atrium_open_close_lifecycle() {
    let atrium = AtriumHandle::open(AtriumConfig::for_test())
        .await
        .expect("open atrium");
    let status = atrium.atrium_status().await;
    assert!(
        status.is_healthy,
        "freshly-opened atrium must report healthy, got: {status:?}"
    );
    drop(atrium);

    let atrium = AtriumHandle::open(AtriumConfig::for_test())
        .await
        .expect("re-open atrium");
    atrium.close().await;
}

#[tokio::test]
async fn atrium_sync_subgraph_two_peer_bidirectional() {
    let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();

    peer_a.register_zone("/zone/posts").await;
    peer_b.register_zone("/zone/posts").await;
    peer_a
        .with_zone("/zone/posts", |doc| {
            doc.set_property("title_a", "alpha", BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    peer_b
        .with_zone("/zone/posts", |doc| {
            doc.set_property("title_b", "beta", BentenHlc::new(200, 0, 0xBBBB))
                .unwrap();
        })
        .await
        .unwrap();

    let peer_b_addr = peer_b.loopback_addr().expect("peer_b loopback addr");
    let peer_b_clone = peer_b.clone();
    let accept_task =
        tokio::spawn(async move { peer_b_clone.accept_sync_subgraph("/zone/posts").await });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    peer_a
        .sync_subgraph("/zone/posts", peer_b_addr)
        .await
        .expect("peer_a sync_subgraph");
    let _conn_b = accept_task
        .await
        .expect("accept join")
        .expect("peer_b accept_sync_subgraph");

    let a_title_a = peer_a
        .with_zone("/zone/posts", |doc| doc.get_property("title_a"))
        .await
        .unwrap();
    let a_title_b = peer_a
        .with_zone("/zone/posts", |doc| doc.get_property("title_b"))
        .await
        .unwrap();
    let b_title_a = peer_b
        .with_zone("/zone/posts", |doc| doc.get_property("title_a"))
        .await
        .unwrap();
    let b_title_b = peer_b
        .with_zone("/zone/posts", |doc| doc.get_property("title_b"))
        .await
        .unwrap();
    assert_eq!(a_title_a.as_deref(), Some("alpha"));
    assert_eq!(a_title_b.as_deref(), Some("beta"));
    assert_eq!(b_title_a.as_deref(), Some("alpha"));
    assert_eq!(b_title_b.as_deref(), Some("beta"));
}

#[tokio::test]
async fn inv_13_row_4b_system_zone_anchor_immutable_divergent_cid_rejects_with_e_sync_divergent_cid_rejected()
 {
    let atrium = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let result = atrium
        .merge_remote_change("system:HandlerVersion/foo", b"any-bytes")
        .await;
    let err = result.expect_err("system-zone target must reject");
    assert_eq!(
        err.code(),
        benten_errors::ErrorCode::SyncDivergentCidRejected
    );
}

#[tokio::test]
async fn inv_13_row_4a_loro_merge_applicable_user_data_resolves_via_d_c_version_chain() {
    let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();

    peer_a.register_zone("/zone/data").await;
    peer_b.register_zone("/zone/data").await;
    peer_a
        .with_zone("/zone/data", |doc| {
            doc.set_property("k", "from_a", BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    peer_b
        .with_zone("/zone/data", |doc| {
            doc.set_property("k", "from_b", BentenHlc::new(200, 0, 0xBBBB))
                .unwrap();
        })
        .await
        .unwrap();

    let b_bytes = peer_b
        .with_zone("/zone/data", |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    peer_a
        .merge_remote_change("/zone/data", &b_bytes)
        .await
        .expect("user-data zone merge applies");

    let merged = peer_a
        .with_zone("/zone/data", |doc| doc.get_property("k"))
        .await
        .unwrap();
    assert_eq!(merged.as_deref(), Some("from_b"));

    let attr = peer_a
        .with_zone("/zone/data", |doc| doc.winning_attribution())
        .await
        .unwrap();
    assert!(attr.contains(&0xAAAA));
    assert!(attr.contains(&0xBBBB));
}

#[tokio::test]
async fn loro_merge_attribution_frame_captures_contributing_peer_dids() {
    let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let peer_c = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();

    for (peer, hlc, value) in [
        (&peer_a, BentenHlc::new(100, 0, 0xAAAA), "a"),
        (&peer_b, BentenHlc::new(200, 0, 0xBBBB), "b"),
        (&peer_c, BentenHlc::new(150, 0, 0xCCCC), "c"),
    ] {
        peer.register_zone("/zone/contrib").await;
        peer.with_zone("/zone/contrib", |doc| {
            doc.set_property("k", value, hlc).unwrap();
        })
        .await
        .unwrap();
    }

    for (donor_label, donor) in [("b", &peer_b), ("c", &peer_c)] {
        let bytes = donor
            .with_zone("/zone/contrib", |doc| doc.export_update().unwrap())
            .await
            .unwrap();
        peer_a
            .merge_remote_change("/zone/contrib", &bytes)
            .await
            .unwrap_or_else(|e| panic!("merge from {donor_label}: {e:?}"));
    }

    let attr = peer_a
        .with_zone("/zone/contrib", |doc| doc.winning_attribution())
        .await
        .unwrap();
    assert!(attr.contains(&0xAAAA));
    assert!(attr.contains(&0xBBBB));
    assert!(attr.contains(&0xCCCC));
}

#[tokio::test]
async fn loro_merged_node_is_graph_encoded_not_opaque_crdt_blob() {
    let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    peer_a.register_zone("/zone/g").await;
    peer_b.register_zone("/zone/g").await;
    peer_a
        .with_zone("/zone/g", |doc| {
            doc.set_property("title", "from_a", BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    peer_b
        .with_zone("/zone/g", |doc| {
            doc.set_property("body", "from_b", BentenHlc::new(150, 0, 0xBBBB))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone("/zone/g", |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    peer_a.merge_remote_change("/zone/g", &bytes).await.unwrap();
    let title = peer_a
        .with_zone("/zone/g", |doc| doc.get_property("title"))
        .await
        .unwrap();
    let body = peer_a
        .with_zone("/zone/g", |doc| doc.get_property("body"))
        .await
        .unwrap();
    assert_eq!(title.as_deref(), Some("from_a"));
    assert_eq!(body.as_deref(), Some("from_b"));
}

#[tokio::test]
async fn loro_merge_produces_attribution_frame_seed_for_anchor_version_chain() {
    // D-C / D-PHASE-3-22 / arch-r1-4 pin: the AttributionFrame SEED
    // (contributing peer-`node_id`s after sync-merge) is the
    // load-bearing CRDT-layer exit. The Anchor + Version + CURRENT
    // mint that consumes this seed lives in the engine's
    // version-chain mint path (Phase-1 shipped); the wire-up of
    // sync-merge → new-Version-Node mint shipped at G16-D PR #163
    // (criterion 16 cryptographic closure). G16-B canary scope is the
    // SEED via `LoroDoc::winning_attribution`.
    let peer_a = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    peer_a.register_zone("/zone/v").await;
    peer_b.register_zone("/zone/v").await;
    peer_a
        .with_zone("/zone/v", |doc| {
            doc.set_property("title", "v_a", BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    peer_b
        .with_zone("/zone/v", |doc| {
            doc.set_property("title", "v_b", BentenHlc::new(200, 0, 0xBBBB))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone("/zone/v", |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    peer_a.merge_remote_change("/zone/v", &bytes).await.unwrap();
    let attr = peer_a
        .with_zone("/zone/v", |doc| doc.winning_attribution())
        .await
        .unwrap();
    assert_eq!(
        attr.len(),
        2,
        "two peers contributed; seed must surface both"
    );
    assert!(attr.contains(&0xAAAA));
    assert!(attr.contains(&0xBBBB));
}

#[tokio::test]
async fn three_peer_loro_convergence_under_concurrent_writes() {
    let peers: Vec<AtriumHandle> = {
        let mut v = Vec::new();
        for _ in 0..3 {
            v.push(AtriumHandle::open(AtriumConfig::for_test()).await.unwrap());
        }
        v
    };
    for (i, peer) in peers.iter().enumerate() {
        peer.register_zone("/zone/conv").await;
        let hlc = BentenHlc::new(100 * (i as u64 + 1), 0, 0xA000 + i as u64);
        let value = format!("from_p{i}");
        peer.with_zone("/zone/conv", |doc| {
            doc.set_property("k", value, hlc).unwrap();
        })
        .await
        .unwrap();
    }

    for _ in 0..2 {
        let mut snapshots: Vec<Vec<u8>> = Vec::new();
        for peer in &peers {
            snapshots.push(
                peer.with_zone("/zone/conv", |doc| doc.export_update().unwrap())
                    .await
                    .unwrap(),
            );
        }
        for (i, peer) in peers.iter().enumerate() {
            for (j, snap) in snapshots.iter().enumerate() {
                if i != j {
                    peer.merge_remote_change("/zone/conv", snap).await.unwrap();
                }
            }
        }
    }

    let v0 = peers[0]
        .with_zone("/zone/conv", |doc| doc.get_property("k"))
        .await
        .unwrap();
    let v1 = peers[1]
        .with_zone("/zone/conv", |doc| doc.get_property("k"))
        .await
        .unwrap();
    let v2 = peers[2]
        .with_zone("/zone/conv", |doc| doc.get_property("k"))
        .await
        .unwrap();
    assert_eq!(v0, v1);
    assert_eq!(v1, v2);
    // peer_2's HLC is the highest (300) — peer_2's value wins.
    assert_eq!(v0.as_deref(), Some("from_p2"));
}

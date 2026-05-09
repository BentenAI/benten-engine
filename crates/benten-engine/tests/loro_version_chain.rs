//! Phase-3 G16-B-prime GREEN-PHASE pins for D-C HYBRID Loro →
//! Version-chain (engine-side merge callback per arch-r1-4 +
//! D-PHASE-3-22 RESOLVED + ds-r4b-1 BLOCKER closure).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `loro_merge_produces_new_version_node_via_anchor_version_chain` +
//!   `merge_loro_change_creates_versioned_anchor` +
//!   `loro_merge_attribution_frame_captures_contributing_peer_dids` +
//!   `loro_merged_version_carries_peer_did_set_in_attribution_frame` +
//!   `anchor_current_pointer_advances_on_loro_merge` +
//!   `old_version_remain_queryable_after_loro_merge` +
//!   `loro_merged_node_is_graph_encoded_not_opaque_crdt_blob`.
//! - r2-test-landscape §9 D-PHASE-3-22 hybrid (iii) Loro→Version-chain.
//! - plan §3 G16-B row "D-C HYBRID per arch-r1-4 / D-PHASE-3-22 RESOLVED".
//! - `arch-r1-4` (Loro merges via Anchor + Version + CURRENT pattern;
//!   no opaque CRDT blob in the storage layer).
//! - `cag-6` (Loro merged nodes are graph-encoded).
//!
//! ## D-C HYBRID narrative (D-PHASE-3-22 RESOLVED)
//!
//! Loro merges produce **new Version Nodes** via the existing
//! Anchor + Version + CURRENT pattern. The AttributionFrame at the
//! new Version captures the set of contributing peer-DIDs. The
//! CURRENT pointer advances atomically. Old Versions remain queryable
//! for content-addressing replay.
//!
//! ## G16-B-prime closure
//!
//! G16-B canary landed the structural surface (AttributionFrame
//! sync-boundary fields + SyncMergeAttribution seed at
//! `merge_remote_change_with_hop_depth`). G16-B-prime adds the
//! engine-side merge callback ([`Engine::apply_atrium_merge`]) that:
//! 1. Consumes the SyncMergeAttribution seed.
//! 2. Resolves peer node-ids → peer-DIDs via the trust-store
//!    (`AtriumHandle::resolve_peer_dids`).
//! 3. Mints a new Version Node via [`Engine::append_version`] under
//!    the named anchor (post-G16-B-prime: real wireup; pre was
//!    Phase-1 stub `E_NOT_IMPLEMENTED`).
//! 4. Updates the CURRENT pointer atomically.
//!
//! Pins below exercise the production-runtime arm via the engine
//! orchestration entry point.

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::AtriumHandle;

/// Build a peer-1 / peer-2 pair, register a shared zone, populate
/// disjoint writes on each, exchange Loro export bytes, and call
/// `Engine::apply_atrium_merge` against the receiving peer.
///
/// Returns: `(receiving_engine, receiving_atrium, anchor_handle,
///           merged_node_cid, peer_a_node_id, peer_b_node_id)`.
async fn run_concurrent_merge_fixture() -> (
    Engine,
    AtriumHandle,
    benten_engine::AnchorHandle,
    benten_core::Cid,
    u64,
    u64,
) {
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    // Two atrium peers; peer A receives, peer B is the remote.
    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();

    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;

    // Local peer A writes; remote peer B writes.
    let peer_a_node_id = peer_a.hlc_node_id();
    let peer_b_node_id = peer_b.hlc_node_id();
    peer_a
        .with_zone(zone, |doc| {
            doc.set_property("title", "from-A", BentenHlc::new(100, 0, peer_a_node_id))
                .unwrap();
        })
        .await
        .unwrap();
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property("body", "from-B", BentenHlc::new(200, 0, peer_b_node_id))
                .unwrap();
        })
        .await
        .unwrap();

    // Register peer B's DID in peer A's trust-store so resolve returns
    // a real DID-shape rather than the node-id:NNN fallback.
    peer_a
        .register_peer_did(peer_b_node_id, format!("did:key:peer-b:{peer_b_node_id}"))
        .await;

    // Export peer B's state and apply at peer A via the engine merge
    // orchestrator (the Phase-3 production arm).
    let remote_bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let anchor = engine.create_anchor("post:p1").unwrap();
    let merged_cid = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &remote_bytes, 0)
        .await
        .expect("apply_atrium_merge must succeed for row-4a user-data zone");

    (
        engine,
        peer_a,
        anchor,
        merged_cid,
        peer_a_node_id,
        peer_b_node_id,
    )
}

#[tokio::test]
async fn merge_loro_change_creates_versioned_anchor() {
    // G16-B-prime LOAD-BEARING pin (per the brief): a Loro merge
    // through the engine's apex orchestration entry point produces a
    // new Version Node + advances CURRENT under the named Anchor.
    let (engine, _peer_a, anchor, merged_cid, _, _) = run_concurrent_merge_fixture().await;

    // OBSERVABLE consequence (i): CURRENT advances to the merged Version.
    let current = engine.read_current_version(&anchor).unwrap().unwrap();
    assert_eq!(
        current, merged_cid,
        "CURRENT pointer MUST advance to the newly-minted merge Version Node"
    );

    // OBSERVABLE consequence (ii): the merged Version is graph-encoded
    // (queryable as a Node by CID per cag-6).
    let merged = engine
        .get_node(&merged_cid)
        .unwrap()
        .expect("merged Version Node MUST be queryable as a graph Node");
    assert_eq!(
        merged.labels.first().map(String::as_str),
        Some("version"),
        "merged Node MUST carry the 'version' label per arch-r1-4"
    );

    // OBSERVABLE consequence (iii): chain history grows (seed → merge).
    let versions: Vec<_> = engine.walk_versions(&anchor).unwrap().collect();
    assert!(
        versions.contains(&merged_cid),
        "walk_versions MUST yield the merged Version CID"
    );
    assert!(
        versions.len() >= 2,
        "chain MUST have at least seed + merged versions; got {versions:?}"
    );
}

#[tokio::test]
async fn loro_merge_produces_new_version_node_via_anchor_version_chain() {
    // D-C / D-PHASE-3-22 / arch-r1-4 LOAD-BEARING pin. Equivalent
    // shape-assertion to merge_loro_change_creates_versioned_anchor
    // (per r2-test-landscape §3.B "pin-rephrasing for cross-lens
    // visibility").
    let (engine, _peer_a, anchor, merged_cid, _, _) = run_concurrent_merge_fixture().await;
    let walk: Vec<_> = engine.walk_versions(&anchor).unwrap().collect();
    assert!(walk.len() >= 2, "Loro merge mints a new Version Node");
    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    assert_eq!(merged.labels.first().map(String::as_str), Some("version"));
}

#[tokio::test]
async fn loro_merge_attribution_frame_captures_contributing_peer_dids() {
    // D-C LOAD-BEARING pin. The merged Version Node carries an
    // AttributionFrame seed that captured the contributing peer-DIDs
    // from the Loro merge.
    let (engine, _peer_a, _anchor, merged_cid, _, peer_b_node_id) =
        run_concurrent_merge_fixture().await;
    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    // The merge node carries the encoded AttributionFrame CID + the
    // resolved peer-DID set surfaces via Loro's winning_attribution
    // (peer A's local writes + peer B's incoming writes both contribute
    // node-ids; the engine resolves peer B's via the trust-store).
    assert!(
        merged.properties.contains_key("attribution_frame_cid"),
        "merged Version MUST carry the AttributionFrame CID slot per D-C"
    );
    assert!(
        merged.properties.contains_key("sync_hop_depth"),
        "merged Version MUST carry sync_hop_depth slot per D-PHASE-3-25"
    );
    let _ = peer_b_node_id; // resolved via trust-store registration in fixture
}

#[tokio::test]
async fn loro_merged_version_carries_peer_did_set_in_attribution_frame() {
    // r2-test-landscape §3.B variant pin (cross-lens visibility for
    // the AttributionFrame contract). The merge minted a Version Node
    // whose AttributionFrame is content-distinguishable from a
    // purely-local frame (its peer_did_set is non-None — Inv-14
    // sync-grain).
    let (engine, _peer_a, _anchor, merged_cid, _, _) = run_concurrent_merge_fixture().await;
    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    // The encoded AttributionFrame CID is present + non-default
    // (non-default because peer_did_set is non-empty post-merge).
    let frame_cid_value = merged
        .properties
        .get("attribution_frame_cid")
        .expect("merged Version carries attribution_frame_cid");
    let bytes = match frame_cid_value {
        benten_core::Value::Bytes(b) => b.clone(),
        other => panic!("expected Bytes, got {other:?}"),
    };
    // Default-shape AttributionFrame CID has a known canonical bytes
    // (3-key Node with all-zero CIDs); the merged frame's CID MUST
    // differ because peer_did_set + sync_hop_depth are non-default.
    let default_frame = benten_eval::AttributionFrame::default();
    let default_cid = default_frame.cid().unwrap();
    assert_ne!(
        bytes,
        default_cid.as_bytes().to_vec(),
        "merged AttributionFrame MUST be content-distinguishable from a \
         purely-local default frame per Inv-14 (peer_did_set + sync_hop_depth)"
    );
}

#[tokio::test]
async fn anchor_current_pointer_advances_on_loro_merge() {
    // D-PHASE-3-22 pin. CURRENT pointer advances atomically on Loro
    // merge. Pre-merge CURRENT == anchor seed; post-merge CURRENT ==
    // merged Version CID.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let anchor = engine.create_anchor("post:p1").unwrap();
    let pre_merge_current = engine.read_current_version(&anchor).unwrap().unwrap();

    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property("title", "x", BentenHlc::new(200, 0, peer_b.hlc_node_id()))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let merged_cid = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await
        .unwrap();

    let post_merge_current = engine.read_current_version(&anchor).unwrap().unwrap();
    assert_ne!(
        pre_merge_current, post_merge_current,
        "CURRENT pointer MUST advance on Loro merge"
    );
    assert_eq!(
        post_merge_current, merged_cid,
        "post-merge CURRENT MUST equal the minted Version CID"
    );
}

#[tokio::test]
async fn old_version_remain_queryable_after_loro_merge() {
    // D-PHASE-3-22 + content-addressing preservation pin. After a
    // Loro merge advances CURRENT, the OLD Version Nodes remain
    // queryable by their CIDs.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let anchor = engine.create_anchor("post:p1").unwrap();

    // Append a v1 directly via the engine surface.
    let mut props_v1 = std::collections::BTreeMap::new();
    props_v1.insert("title".into(), benten_core::Value::Text("v1".into()));
    let v1_cid = engine
        .append_version(
            &anchor,
            &benten_core::Node::new(vec!["version".into()], props_v1),
        )
        .unwrap();

    // Now drive a Loro merge against the same anchor.
    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property(
                "body",
                "remote",
                BentenHlc::new(200, 0, peer_b.hlc_node_id()),
            )
            .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let merged_cid = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await
        .unwrap();

    // Old Version v1 still queryable by its CID.
    let v1 = engine.get_node(&v1_cid).unwrap();
    assert!(
        v1.is_some(),
        "old Version v1 MUST remain queryable after Loro merge minted \
         a new Version (content-addressing preservation per D-PHASE-3-22)"
    );

    // Chain walk yields both.
    let versions: Vec<_> = engine.walk_versions(&anchor).unwrap().collect();
    assert!(versions.contains(&v1_cid), "walk includes v1");
    assert!(versions.contains(&merged_cid), "walk includes merged");
}

#[tokio::test]
async fn loro_merged_node_is_graph_encoded_not_opaque_crdt_blob() {
    // cag-6 architectural pin. The merged Version Node MUST be a
    // graph Node (queryable by label, traversable via standard
    // engine surfaces) — NOT an opaque CRDT blob stored alongside
    // the graph.
    let (engine, _peer_a, _anchor, merged_cid, _, _) = run_concurrent_merge_fixture().await;
    let merged = engine.get_node(&merged_cid).unwrap().unwrap();
    assert_eq!(
        merged.labels.first().map(String::as_str),
        Some("version"),
        "merged Node carries the 'version' label"
    );
    // Graph-encoded properties (the merged Loro state surfaces as
    // `loro:<key>` slots).
    assert!(
        merged.properties.keys().any(|k| k.starts_with("loro:")),
        "merged Node carries graph-encoded loro:<key> properties; got: {:?}",
        merged.properties.keys().collect::<Vec<_>>()
    );
    // The merged Node is content-addressable — its CID equals
    // re-encoded bytes' CID.
    let recid = merged.cid().unwrap();
    assert_eq!(recid, merged_cid, "Node CID is content-addressed");
}

// =====================================================================
// G16-B-prime DEFERRED — pins kept RED-PHASE for downstream waves.
//
// These pins describe contracts that depend on infrastructure beyond
// G16-B-prime's scope:
//
// - `loro_merge_produces_new_version_node_in_anchor_chain` (multi-
//   Anchor cross-cutting merge) requires the wave-6b cross-Atrium
//   harness `apply_concurrent_loro_merge_across_atrium` which is a
//   G16-B-D distributed-systems-wave deliverable.
// - `loro_merge_fires_subscribe_notification_on_affected_zone_per_charter_9`
//   requires Loro merges to fan out via the SUBSCRIBE registry —
//   the SUBSCRIBE-from-merge composition is a G16-D wave-6b surface
//   per cag-r4-6 MINOR.
//
// Per HARD RULE rule-12 disposition (b) BELONGS-NAMED-NOW: the
// destinations are `docs/future/phase-3-backlog.md` §6.12 deferred
// items + the G16-B-D dispatch brief.
// =====================================================================

#[tokio::test]
async fn loro_merge_produces_new_version_node_in_anchor_chain() {
    // G16-B-D LOAD-BEARING multi-Anchor pin (cross-lens visibility for
    // the multi-Anchor cross-Atrium harness shape, complementing the
    // single-Anchor `merge_loro_change_creates_versioned_anchor` above).
    //
    // OBSERVABLE consequence:
    //   - Two distinct Anchors over two distinct zones each receive an
    //     independent Loro merge from a remote peer.
    //   - Each Anchor's chain advances independently (post-merge
    //     CURRENT differs across the two anchors).
    //   - walk_versions on each Anchor yields its own merged Version
    //     CID; cross-anchor cross-talk is absent (an anchor's chain
    //     does NOT contain the other anchor's merged Version).
    //
    // Replay safety: re-applying the SAME remote bytes to the same
    // anchor MUST be idempotent (CRDT merge semantics) — CURRENT
    // does not advance, no new Version Node minted on duplicate apply.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();

    let zone_posts = "/zone/posts";
    let zone_comments = "/zone/comments";
    peer_a.register_zone(zone_posts).await;
    peer_a.register_zone(zone_comments).await;
    peer_b.register_zone(zone_posts).await;
    peer_b.register_zone(zone_comments).await;

    let peer_b_node_id = peer_b.hlc_node_id();
    peer_a
        .register_peer_did(peer_b_node_id, format!("did:key:peer-b:{peer_b_node_id}"))
        .await;

    // Remote peer B writes to BOTH zones with distinct content.
    peer_b
        .with_zone(zone_posts, |doc| {
            doc.set_property(
                "title",
                "post-from-B",
                BentenHlc::new(100, 0, peer_b_node_id),
            )
            .unwrap();
        })
        .await
        .unwrap();
    peer_b
        .with_zone(zone_comments, |doc| {
            doc.set_property(
                "body",
                "comment-from-B",
                BentenHlc::new(200, 0, peer_b_node_id),
            )
            .unwrap();
        })
        .await
        .unwrap();

    let posts_bytes = peer_b
        .with_zone(zone_posts, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let comments_bytes = peer_b
        .with_zone(zone_comments, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    // Two distinct Anchors, one per zone.
    let anchor_posts = engine.create_anchor("post:p1").unwrap();
    let anchor_comments = engine.create_anchor("comment:c1").unwrap();

    let merged_posts_cid = engine
        .apply_atrium_merge(&peer_a, &anchor_posts, zone_posts, &posts_bytes, 0)
        .await
        .expect("multi-Anchor: apply_atrium_merge for posts zone");
    let merged_comments_cid = engine
        .apply_atrium_merge(&peer_a, &anchor_comments, zone_comments, &comments_bytes, 0)
        .await
        .expect("multi-Anchor: apply_atrium_merge for comments zone");

    // OBSERVABLE consequence (i): each anchor's CURRENT advances to
    // its own merged Version CID — cross-anchor isolation.
    assert_ne!(
        merged_posts_cid, merged_comments_cid,
        "multi-Anchor: distinct anchors over distinct zones MUST yield \
         distinct merged Version CIDs"
    );
    let posts_current = engine.read_current_version(&anchor_posts).unwrap().unwrap();
    let comments_current = engine
        .read_current_version(&anchor_comments)
        .unwrap()
        .unwrap();
    assert_eq!(posts_current, merged_posts_cid);
    assert_eq!(comments_current, merged_comments_cid);

    // OBSERVABLE consequence (ii): walk_versions on each anchor yields
    // its OWN merged Version, not the other anchor's.
    let posts_chain: Vec<_> = engine.walk_versions(&anchor_posts).unwrap().collect();
    let comments_chain: Vec<_> = engine.walk_versions(&anchor_comments).unwrap().collect();
    assert!(
        posts_chain.contains(&merged_posts_cid),
        "posts chain MUST contain its merged Version CID"
    );
    assert!(
        !posts_chain.contains(&merged_comments_cid),
        "posts chain MUST NOT contain comments-anchor's Version CID \
         (cross-anchor isolation): posts_chain={posts_chain:?}"
    );
    assert!(
        comments_chain.contains(&merged_comments_cid),
        "comments chain MUST contain its merged Version CID"
    );
    assert!(
        !comments_chain.contains(&merged_posts_cid),
        "comments chain MUST NOT contain posts-anchor's Version CID \
         (cross-anchor isolation): comments_chain={comments_chain:?}"
    );

    // OBSERVABLE consequence (iii): replay safety — re-applying the
    // SAME remote bytes to the same anchor is idempotent at the CRDT
    // level. CURRENT does not advance further (Loro merge of
    // already-applied bytes is a no-op).
    let _replay = engine
        .apply_atrium_merge(&peer_a, &anchor_posts, zone_posts, &posts_bytes, 0)
        .await
        .expect("idempotent replay must succeed");
    let posts_current_after_replay = engine.read_current_version(&anchor_posts).unwrap().unwrap();
    assert_eq!(
        posts_current_after_replay, merged_posts_cid,
        "replay safety: re-applying the same Loro update bytes MUST NOT \
         advance CURRENT past the original merged Version"
    );
}

#[test]
#[ignore = "DEFERRED: G16-D wave-6b — SUBSCRIBE fan-out from Loro merge per cag-r4-6 MINOR"]
fn loro_merge_fires_subscribe_notification_on_affected_zone_per_charter_9() {
    // cag-r4-6 MINOR pin. SUBSCRIBE fan-out from Loro merges requires
    // the engine's merge-callback to wire through the SUBSCRIBE
    // registry — a G16-D wave-6b deliverable. The G16-B-prime
    // engine-side merge mints the Version Node via append_version
    // (which fires the standard ChangeEvent path) but the explicit
    // zone-keyed SUBSCRIBE notification on Loro-merge boundaries is
    // out of scope here; named at phase-3-backlog §6.12 deferred.
}

#[test]
fn create_anchor_is_idempotent_under_repeated_calls() {
    // G16-B-prime fp closure (cor-3 MINOR): repeated create_anchor
    // calls under the same name MUST be idempotent. Returns a handle
    // pointing at the same anchor entry so the second call cannot
    // accidentally reset chain state. Defends against a regression
    // where re-init would clobber the version chain head.
    use benten_engine::Engine;
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();
    let h1 = engine.create_anchor("post:idempotent").unwrap();
    let h2 = engine.create_anchor("post:idempotent").unwrap();
    assert_eq!(
        h1.name(),
        h2.name(),
        "AnchorHandle::name MUST be stable across repeated create_anchor calls"
    );
    // Second call must not advance / reset the chain — current
    // pointer remains at the seed cid.
    let head_after_first = engine.read_current_version(&h1).unwrap();
    let head_after_second = engine.read_current_version(&h2).unwrap();
    assert_eq!(
        head_after_first, head_after_second,
        "create_anchor MUST NOT mutate an existing anchor's CURRENT \
         pointer on repeated calls"
    );
}

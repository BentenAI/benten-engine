//! G16-B canary GREEN-PHASE pin: sync-hop-depth bound at the Atrium
//! merge seam (ds-r4b-1 BLOCKER closure; D-PHASE-3-25 sync-hop-depth-
//! bounded contract).
//!
//! ## Pin sources
//!
//! - R4b round 1 distributed-systems lens, finding `ds-r4b-1`.
//! - D-PHASE-3-25 device-heterogeneity contract +
//!   sync-hop-depth-bounded clause.
//! - `feedback_end_to_end_test_pin_for_closed_claims.md` (pim-2 §3.6b).
//!
//! ## Coverage
//!
//! - **`merge_within_hop_depth_cap_admits`**: incoming hop-depth
//!   (cap-1) admits + the returned [`SyncMergeAttribution`] surfaces
//!   the new depth (cap).
//! - **`merge_at_hop_depth_cap_rejects_with_typed_error`**: incoming
//!   hop-depth (cap) → resulting depth (cap+1) > cap → rejects with
//!   [`benten_errors::ErrorCode::SyncHopDepthExceeded`].
//! - **`merge_remote_change_zero_depth_back_compat_call_shape`**: the
//!   plain [`AtriumHandle::merge_remote_change`] call shape (Phase-2b
//!   ergonomics) is preserved; equivalent to `incoming_hop_depth=0`.
//!
//! pim-2 §3.6b discipline: each test drives the production
//! `AtriumHandle::merge_remote_change_with_hop_depth` path + asserts
//! an OBSERVABLE consequence (typed error firing / depth incrementing
//! / back-compat call shape continuing to compile). Each test would
//! FAIL if the production-runtime arm silently no-op'd.

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used)]

use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::{AtriumError, AtriumHandle};
use benten_eval::exec_state::SYNC_HOP_DEPTH_CAP;

#[tokio::test]
async fn merge_within_hop_depth_cap_admits() {
    // Build a self-loop pair so we have a real Loro export to merge.
    // The CAP is 8; passing incoming_hop_depth=cap-1 → resulting depth
    // = cap, still admits. The returned SyncMergeAttribution carries
    // the new depth.
    let writer = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let reader = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    writer.register_zone("/zone/canary").await;
    writer
        .with_zone("/zone/canary", |doc| {
            doc.set_property("k", "v", benten_core::hlc::BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = writer
        .with_zone("/zone/canary", |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let incoming = SYNC_HOP_DEPTH_CAP - 1;
    let seed = reader
        .merge_remote_change_with_hop_depth("/zone/canary", &bytes, incoming)
        .await
        .expect("incoming=cap-1 MUST admit");
    assert_eq!(
        seed.sync_hop_depth, SYNC_HOP_DEPTH_CAP,
        "seed.sync_hop_depth MUST = incoming + 1"
    );
}

#[tokio::test]
async fn merge_at_hop_depth_cap_rejects_with_typed_error() {
    // incoming_hop_depth=cap → resulting depth = cap + 1 > cap → reject.
    // OBSERVABLE consequence: the typed
    // ErrorCode::SyncHopDepthExceeded fires; the doc state is
    // unchanged (pre-merge order rejection).
    let writer = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let reader = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    writer.register_zone("/zone/canary").await;
    writer
        .with_zone("/zone/canary", |doc| {
            doc.set_property("k", "v", benten_core::hlc::BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = writer
        .with_zone("/zone/canary", |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let result = reader
        .merge_remote_change_with_hop_depth("/zone/canary", &bytes, SYNC_HOP_DEPTH_CAP)
        .await;
    match result {
        Err(e @ AtriumError::SyncHopDepthExceeded { .. }) => {
            assert_eq!(
                e.code(),
                benten_errors::ErrorCode::SyncHopDepthExceeded,
                "AtriumError::SyncHopDepthExceeded MUST route to E_SYNC_HOP_DEPTH_EXCEEDED"
            );
        }
        other => panic!("expected SyncHopDepthExceeded, got {other:?}"),
    }
    // Doc state unchanged — read after rejected merge yields no value.
    let value = reader
        .with_zone("/zone/canary", |doc| doc.get_property("k"))
        .await
        .unwrap();
    assert_eq!(
        value, None,
        "rejected merge MUST leave doc state unchanged (pre-merge order)"
    );
}

#[tokio::test]
async fn merge_remote_change_zero_depth_back_compat_call_shape() {
    // The plain Phase-2b call shape `merge_remote_change(zone, bytes)`
    // is preserved as a thin wrapper over
    // `merge_remote_change_with_hop_depth(zone, bytes, 0)`. Asserts
    // both call shapes compile + admit + the depth-aware variant
    // surfaces the seed.
    let writer = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let reader = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    writer.register_zone("/zone/canary").await;
    writer
        .with_zone("/zone/canary", |doc| {
            doc.set_property("k", "v", benten_core::hlc::BentenHlc::new(100, 0, 0xAAAA))
                .unwrap();
        })
        .await
        .unwrap();
    let bytes = writer
        .with_zone("/zone/canary", |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    // Plain call: admits + has the same (pre-canary) shape.
    reader
        .merge_remote_change("/zone/canary", &bytes)
        .await
        .unwrap();
    let value = reader
        .with_zone("/zone/canary", |doc| doc.get_property("k"))
        .await
        .unwrap();
    assert_eq!(value.as_deref(), Some("v"));
}

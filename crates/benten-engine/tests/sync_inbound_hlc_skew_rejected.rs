//! R6-FP Wave-C1 (ds-r6-1 / hlc-r6-r1-1 closure) — production-runtime
//! end-to-end pin for the inbound-sync-frame HLC skew classifier
//! wired into [`benten_engine::Engine::apply_atrium_merge`]'s per-row
//! [`benten_core::hlc::Hlc::update`] verification loop.
//!
//! ## What this defends against
//!
//! sec-r4r2-1 attack-vector cluster (HLC skew injection biasing LWW +
//! revocation-vs-data ordering). An adversarial peer manipulating its
//! local HLC to inject future-timestamped writes:
//!
//! - **Biases LWW resolution**: a future-HLC write retroactively
//!   defeats any honest write at the same property.
//! - **Forges revocation-vs-data ordering**: data writes at
//!   `HLC=now+24h` win against revocations issued at `HLC=now+1m`.
//!
//! Defense: per-row inbound HLC skew classifier inside
//! `apply_atrium_merge` calls [`Hlc::update`] against each row's wire
//! HLC and rejects on `CoreError::HlcSkewExceeded`. The merge is
//! atomic — a single skew-exceeding row vetoes the entire merge.
//!
//! ## pim-2 §3.6b end-to-end discipline
//!
//! - drives the production receive path (`apply_atrium_merge` row-loop
//!   calling `Hlc::update`);
//! - asserts an OBSERVABLE behavioral consequence (typed-error
//!   `EngineError::Other { code: SyncHlcDrift }` mapping to
//!   `E_SYNC_HLC_DRIFT` + observability counter increments via
//!   `inbound_hlc_skew_classifier_calls()`);
//! - would FAIL if the inbound-skew classifier were silently no-op'd.

#![cfg(all(not(target_arch = "wasm32"), not(feature = "browser-backend")))]
#![allow(clippy::unwrap_used)]

use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::AtriumHandle;
use benten_errors::ErrorCode;

#[tokio::test]
async fn apply_atrium_merge_rejects_inbound_row_with_future_skewed_hlc() {
    // sec-r4r2-1 attack-vector pin (R6-FP Wave-C1 closure).
    //
    // Construct a 2-peer scenario:
    //   - peer_a is the legitimate receiver (engine-side merge boundary).
    //   - peer_b is the adversary; it crafts a write whose HLC
    //     `physical_ms = u64::MAX/2` (well past peer_a's local clock +
    //     the default 5-minute skew tolerance window).
    //
    // The adversarial bytes are exported via Loro and applied at peer_a
    // via `apply_atrium_merge`. The per-row HLC verification loop fires
    // `Hlc::update` against the future-skewed stamp and rejects the
    // entire merge with `EngineError::Other { code: SyncHlcDrift }`.

    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;

    // Adversarial write: HLC physical_ms = u64::MAX / 2.
    let adversarial_physical_ms: u64 = u64::MAX / 2;
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property(
                "title",
                "attacker-substitute",
                BentenHlc::new(adversarial_physical_ms, 0, peer_b.hlc_node_id()),
            )
            .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    peer_a
        .register_peer_did(peer_b.hlc_node_id(), "did:key:adversarial-peer-b")
        .await;

    let pre_attack_classifier_calls = peer_a.inbound_hlc_skew_classifier_calls();

    let anchor = engine.create_anchor("post:p1").unwrap();
    let result = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await;

    // OBSERVABLE consequence #1: typed error fires with
    // `ErrorCode::SyncHlcDrift` (`E_SYNC_HLC_DRIFT`).
    match result {
        Err(err) => {
            let code = err.code();
            assert_eq!(
                code,
                ErrorCode::SyncHlcDrift,
                "expected SyncHlcDrift; got {code:?} — inbound HLC skew classifier was \
                 silently no-op'd or fired the wrong typed error"
            );
            assert_eq!(code.as_str(), "E_SYNC_HLC_DRIFT");
        }
        Ok(_) => panic!(
            "attack succeeded — future-skewed HLC was applied at apply_atrium_merge; \
             LWW resolution + revocation-vs-data ordering are open to HLC-skew injection"
        ),
    }

    // OBSERVABLE consequence #2: the per-row HLC classifier counter
    // observably incremented at least once. Defends against the
    // silent-no-op failure shape per pim-2 §3.6b end-to-end discipline.
    let post_attack_classifier_calls = peer_a.inbound_hlc_skew_classifier_calls();
    assert!(
        post_attack_classifier_calls > pre_attack_classifier_calls,
        "inbound_hlc_skew_classifier_calls did not increment — \
         the per-row Hlc::update verification loop was never invoked"
    );
}

#[tokio::test]
async fn apply_atrium_merge_accepts_inbound_row_with_in_tolerance_hlc() {
    // Companion-positive pin: rows whose HLC `physical_ms` is within
    // the default 5-minute tolerance window apply cleanly without
    // tripping the skew classifier. Pairs with the adversarial pin
    // above so the classifier is asymmetric — it rejects future-skew
    // exceeding tolerance but does not over-reject legitimate rows
    // whose HLC may be slightly ahead/behind local wall-clock by
    // network-queue latency.
    let dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(dir.path().join("benten.redb")).unwrap();

    let peer_a = engine.open_atrium(AtriumConfig::for_test()).await.unwrap();
    let peer_b = AtriumHandle::open(AtriumConfig::for_test()).await.unwrap();
    let zone = "/zone/posts";
    peer_a.register_zone(zone).await;
    peer_b.register_zone(zone).await;

    // Legitimate write: HLC physical_ms = 100 (well in the past
    // relative to wall-clock — past-skew is always accepted per the
    // Hlc::update Kulkarni-Demirbas algorithm).
    peer_b
        .with_zone(zone, |doc| {
            doc.set_property(
                "title",
                "legitimate-content",
                BentenHlc::new(100, 0, peer_b.hlc_node_id()),
            )
            .unwrap();
        })
        .await
        .unwrap();
    let bytes = peer_b
        .with_zone(zone, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    peer_a
        .register_peer_did(peer_b.hlc_node_id(), "did:key:legitimate-peer-b")
        .await;

    let anchor = engine.create_anchor("post:p2").unwrap();
    let result = engine
        .apply_atrium_merge(&peer_a, &anchor, zone, &bytes, 0)
        .await;
    result.expect("legitimate in-tolerance HLC should NOT fire skew classifier");
}

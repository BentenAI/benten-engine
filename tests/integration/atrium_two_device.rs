//! G16-D wave-6b GREEN: two-device same-identity selective zone sync
//! end-to-end pin (plan §1 exit-criterion 16 closure).
//!
//! ## Pin source
//!
//! - r2-test-landscape §3.F multi-device sync row
//!   `integration/atrium_two_device_same_identity_selective_zone_sync`.
//! - plan §1 deliverable 16 (multi-device support per FULL-ROADMAP.md
//!   amendment 2026-05-04).
//! - exit-criterion 16 (multi-device support; full peers under
//!   shared identity + heterogeneous capability envelopes).
//! - `.addl/phase-3/exploration-device-mesh.md` (D-PHASE-3 multi-device
//!   resolution).
//!
//! ## What this pins (G16-D wave-6b LANDED)
//!
//! Two FULL PEER instances under the SAME identity (e.g., user's
//! laptop + user's phone-OS-app) sync a SHARED ZONE bidirectionally
//! over real iroh transport. Both engines are bound to the SAME
//! actor_cid (single principal); each engine has a DISTINCT
//! device_cid + device-DID (laptop vs phone). Post-sync the
//! AttributionFrame on each side preserves the ORIGINATING device's
//! device-DID — defending against the failure shape where multi-
//! device merges silently lose device-grain provenance and a
//! compromised device cannot be quarantined surgically.
//!
//! ## Scope vs original RED-PHASE pin steps 1-7
//!
//! G16-D wave-6b LANDED scope: steps 1-2 (two engines under same
//! identity, distinct device-DIDs, both join atrium), steps 4-5 (the
//! shared zone /zone/notifications carries writes from BOTH sides;
//! both sides observe the other's write post-sync), step 7
//! (AttributionFrame on each side carries BOTH peer-DID + originating
//! device-DID).
//!
//! Steps 3 + 6 (heterogeneous-envelope per-zone capability filtering —
//! laptop=full envelope, phone=notifications-only envelope; phone
//! CANNOT write to /zone/notes via cap denial) compose with G14-D
//! per-subscriber capability-filtering at the sync-replica boundary.
//! G14-D wires the per-zone-cap envelope filter; G16-D wave-6b ships
//! the on-the-wire device-DID-attestation envelope that lets G14-D
//! key its filter on device-DID. The cap-denial halves are pinned as
//! BELONGS-NAMED-NOW carry to phase-3-backlog §6.12 item 5
//! follow-up (G14-D heterogeneous-cap-envelope wave) — the floor
//! that G16-D wave-6b lands here is the load-bearing
//! exit-criterion-16 closure for the device-DID attestation +
//! multi-device-bidirectional-sync surface.
//!
//! ## OBSERVABLE consequence
//!
//! If the on-the-wire device-DID-attestation envelope silently no-op'd
//! at any leg, OR if the receiver-side `apply_atrium_merge` defaulted
//! to the receiver's own device_cid for the AttributionFrame's
//! device_did slot, this pin fails: the laptop-side AttributionFrame
//! would NOT carry phone's device-DID after merging the phone's write,
//! losing device-grain provenance per Inv-14.

#![allow(clippy::unwrap_used, clippy::too_many_lines)]
#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use benten_core::Cid;
use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;

const ZONE: &str = "/zone/notifications";
const ACTOR_BYTES: &[u8] = b"actor:alice-shared-identity";
const LAPTOP_DEVICE_BYTES: &[u8] = b"device:alice-laptop";
const PHONE_DEVICE_BYTES: &[u8] = b"device:alice-phone";
const LAPTOP_DEVICE_DID: &str = "did:key:zAliceLaptopDevice";
const PHONE_DEVICE_DID: &str = "did:key:zAlicePhoneDevice";

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn atrium_two_device_same_identity_selective_zone_sync() {
    // Plan §1 exit-criterion 16 LOAD-BEARING pin (G16-D wave-6b).

    // Step 1: Spin up two engines under the SAME actor_cid (account
    // identity) but DIFFERENT device_cids (laptop + phone).
    let actor_cid = Cid::from_blake3_digest(*blake3::hash(ACTOR_BYTES).as_bytes());
    let laptop_device_cid =
        Cid::from_blake3_digest(*blake3::hash(LAPTOP_DEVICE_BYTES).as_bytes());
    let phone_device_cid =
        Cid::from_blake3_digest(*blake3::hash(PHONE_DEVICE_BYTES).as_bytes());
    assert_ne!(
        laptop_device_cid, phone_device_cid,
        "two-device pin requires distinct device CIDs"
    );

    let dir_laptop = tempfile::tempdir().unwrap();
    let dir_phone = tempfile::tempdir().unwrap();
    let engine_laptop = Engine::open(dir_laptop.path().join("benten.redb")).unwrap();
    let engine_phone = Engine::open(dir_phone.path().join("benten.redb")).unwrap();

    // Same actor (account identity) on both devices.
    engine_laptop.set_actor_cid(Some(actor_cid));
    engine_phone.set_actor_cid(Some(actor_cid));
    // Distinct device identities.
    engine_laptop.set_device_cid(Some(laptop_device_cid));
    engine_phone.set_device_cid(Some(phone_device_cid));

    // Step 2: Both devices join the same Atrium under per-device
    // device-DID-attestation envelopes emitted on the wire.
    let atrium_laptop = engine_laptop
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    let atrium_phone = engine_phone
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    atrium_laptop
        .set_local_device_did(Some(LAPTOP_DEVICE_DID.into()))
        .await;
    atrium_phone
        .set_local_device_did(Some(PHONE_DEVICE_DID.into()))
        .await;

    atrium_laptop.register_zone(ZONE).await;
    atrium_phone.register_zone(ZONE).await;

    // Cross-trust-store registration (peer-DID resolution at merge
    // time). Both devices belong to the SAME account-DID so the
    // peer_did set on each side names the shared identity.
    let hlc_laptop = atrium_laptop.hlc_node_id();
    let hlc_phone = atrium_phone.hlc_node_id();
    atrium_laptop
        .register_peer_did(hlc_phone, "did:key:zAliceAccount")
        .await;
    atrium_phone
        .register_peer_did(hlc_laptop, "did:key:zAliceAccount")
        .await;

    // Step 4: laptop writes to /zone/notifications/n1 — both devices
    // share the zone so the write is in scope for both.
    atrium_laptop
        .with_zone(ZONE, |doc| {
            doc.set_property("n1", "from-laptop", BentenHlc::new(100, 0, hlc_laptop))
                .unwrap();
        })
        .await
        .unwrap();
    // Step 5: phone writes to /zone/notifications/n2.
    atrium_phone
        .with_zone(ZONE, |doc| {
            doc.set_property("n2", "from-phone", BentenHlc::new(200, 0, hlc_phone))
                .unwrap();
        })
        .await
        .unwrap();

    let anchor_laptop = engine_laptop.create_anchor("alice-laptop-anchor").unwrap();
    let anchor_phone = engine_phone.create_anchor("alice-phone-anchor").unwrap();

    // Bidirectional sync over real iroh transport. The on-the-wire
    // DeviceAttestationEnvelope precedes the Loro export on each leg;
    // each side stashes the remote's device-DID into the per-zone
    // last_received_remote_device_did slot so apply_atrium_merge can
    // populate AttributionFrame.device_did from the ORIGINATING device.
    let phone_addr = atrium_phone.loopback_addr().unwrap();
    let atrium_phone_clone = atrium_phone.clone();
    let zone_owned = ZONE.to_string();
    let accept_task =
        tokio::spawn(async move { atrium_phone_clone.accept_sync_subgraph(&zone_owned).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    atrium_laptop.sync_subgraph(ZONE, phone_addr).await.unwrap();
    accept_task.await.unwrap().unwrap();

    // Apply the merged Loro state through each engine's
    // apply_atrium_merge to mint a Version Node carrying the
    // post-merge AttributionFrame. The slot populated by
    // sync_subgraph above carries the ORIGINATING device's DID — for
    // each engine, the originating device on the OTHER side.
    let bytes_from_laptop = atrium_laptop
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();
    let bytes_from_phone = atrium_phone
        .with_zone(ZONE, |doc| doc.export_update().unwrap())
        .await
        .unwrap();

    let cid_on_phone = engine_phone
        .apply_atrium_merge(&atrium_phone, &anchor_phone, ZONE, &bytes_from_laptop, 0)
        .await
        .unwrap();
    let cid_on_laptop = engine_laptop
        .apply_atrium_merge(&atrium_laptop, &anchor_laptop, ZONE, &bytes_from_phone, 0)
        .await
        .unwrap();

    // Convergence: both engines' Loro docs carry both keys.
    for (label, atrium) in [("laptop", &atrium_laptop), ("phone", &atrium_phone)] {
        let n1 = atrium
            .with_zone(ZONE, |doc| doc.get_property("n1"))
            .await
            .unwrap();
        let n2 = atrium
            .with_zone(ZONE, |doc| doc.get_property("n2"))
            .await
            .unwrap();
        assert_eq!(
            n1.as_deref(),
            Some("from-laptop"),
            "device {label} must observe laptop's n1 after bidirectional iroh sync"
        );
        assert_eq!(
            n2.as_deref(),
            Some("from-phone"),
            "device {label} must observe phone's n2 after bidirectional iroh sync"
        );
    }

    // Step 7 LOAD-BEARING (exit-criterion 16): each side's
    // AttributionFrame carries the ORIGINATING device-DID — phone's
    // merge of laptop-bytes mints a frame carrying LAPTOP_DEVICE_DID;
    // laptop's merge of phone-bytes mints a frame carrying
    // PHONE_DEVICE_DID. If the engine had defaulted to the receiver's
    // own device_cid, the device_did would carry the LOCAL device's
    // identity instead — losing originating-device provenance.
    let phone_merged_node = engine_phone.get_node(&cid_on_phone).unwrap().unwrap();
    let laptop_merged_node = engine_laptop.get_node(&cid_on_laptop).unwrap().unwrap();
    let phone_frame_cid = match phone_merged_node.properties.get("attribution_frame_cid") {
        Some(benten_core::Value::Bytes(b)) => b.clone(),
        other => panic!("phone: expected Bytes for attribution_frame_cid, got {other:?}"),
    };
    let laptop_frame_cid = match laptop_merged_node.properties.get("attribution_frame_cid") {
        Some(benten_core::Value::Bytes(b)) => b.clone(),
        other => panic!("laptop: expected Bytes for attribution_frame_cid, got {other:?}"),
    };

    // Reconstruct the expected AttributionFrame on each side. Both
    // share the SAME actor_cid (single principal); each side carries
    // the OTHER device's device-DID. The peer_did_set carries the
    // shared account-DID for the registered remote-peer + a synthetic
    // `node-id:NNN` fallback for the local engine's own writes that
    // arrived in the merged Loro state (the local peer-id is not
    // registered in its own trust-store — that mapping is for
    // OBSERVED remote peers).
    let phone_peer_did_set = std::collections::BTreeSet::from([
        "did:key:zAliceAccount".to_string(),
        format!("node-id:{}", hlc_phone),
    ]);
    let laptop_peer_did_set = std::collections::BTreeSet::from([
        "did:key:zAliceAccount".to_string(),
        format!("node-id:{}", hlc_laptop),
    ]);
    let expected_phone_frame = benten_eval::AttributionFrame {
        actor_cid,
        handler_cid: Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        peer_did_set: Some(phone_peer_did_set),
        device_did: Some(LAPTOP_DEVICE_DID.into()),
        sync_hop_depth: 1,
    };
    let expected_laptop_frame = benten_eval::AttributionFrame {
        actor_cid,
        handler_cid: Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        peer_did_set: Some(laptop_peer_did_set),
        device_did: Some(PHONE_DEVICE_DID.into()),
        sync_hop_depth: 1,
    };
    let expected_phone_cid = expected_phone_frame.cid().unwrap();
    let expected_laptop_cid = expected_laptop_frame.cid().unwrap();
    assert_eq!(
        phone_frame_cid,
        expected_phone_cid.as_bytes().to_vec(),
        "phone-side AttributionFrame.device_did MUST carry LAPTOP_DEVICE_DID \
         (the ORIGINATING device for the phone's merge of laptop-bytes), \
         NOT the phone's own device_cid; defends against multi-device merges \
         losing device-grain provenance per Inv-14"
    );
    assert_eq!(
        laptop_frame_cid,
        expected_laptop_cid.as_bytes().to_vec(),
        "laptop-side AttributionFrame.device_did MUST carry PHONE_DEVICE_DID \
         (the ORIGINATING device for the laptop's merge of phone-bytes), \
         NOT the laptop's own device_cid; defends against multi-device merges \
         losing device-grain provenance per Inv-14"
    );

    // Continuity: each engine's anchor advanced to its merge-Version CID.
    let current_laptop = engine_laptop
        .read_current_version(&anchor_laptop)
        .unwrap()
        .unwrap();
    let current_phone = engine_phone
        .read_current_version(&anchor_phone)
        .unwrap()
        .unwrap();
    assert_eq!(current_laptop, cid_on_laptop);
    assert_eq!(current_phone, cid_on_phone);
}

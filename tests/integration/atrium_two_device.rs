//! G16-D wave-6b GREEN: two-device same-identity selective zone sync
//! end-to-end pin (plan §1 exit-criterion 16 closure) +
//! cryptographic-attestation closure for criterion 16 (G16-D wave-6b
//! fix-pass per Ben ratification 2026-05-09).
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
//! ## What this pins (G16-D wave-6b LANDED + fix-pass)
//!
//! Two FULL PEER instances under the SAME identity (e.g., user's
//! laptop + user's phone-OS-app) sync a SHARED ZONE bidirectionally
//! over real iroh transport. Both engines are bound to the SAME
//! actor_cid (single principal); each engine has a DISTINCT
//! device_cid + REAL signed `benten_id::DeviceAttestation` (parent →
//! device-DID binding). Post-sync the AttributionFrame on each side
//! preserves the ORIGINATING device's device-DID — defending against
//! the failure shapes:
//!
//! 1. **Multi-device merges silently lose device-grain provenance** —
//!    receiver-side AttributionFrame.device_did reflects originating
//!    device, NOT receiver.
//! 2. **DID forgery** — a peer cannot impersonate another device's
//!    DID without holding that device's secret key (envelope-signature
//!    verification at receive against the public key resolved from
//!    `attestation.device_did`).
//! 3. **Replay** — a captured envelope cannot be replayed verbatim
//!    against a different sync session (parent-issued attestation
//!    nonce consumed by the receiver's `Acceptor::accept_at`
//!    nonce-store).
//! 4. **Frame-pair binding violation** — a MITM cannot swap the
//!    Loro payload while preserving the envelope (the envelope's
//!    signed `payload_hash` is BLAKE3 of the upcoming payload).
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
//! CANNOT write to /zone/notes via cap denial) are scope-distinct
//! from criterion 16 cryptographic closure: they BELONG-NAMED-NOW
//! carry to phase-3-backlog §6.12 item 8 (heterogeneous-cap-envelope
//! per-device write filter at sync-replica boundary). G16-D wave-6b
//! lands the on-the-wire signed device-DID-attestation envelope that
//! lets §6.12 item 8 key its filter on a verified device-DID.
//!
//! ## OBSERVABLE consequence
//!
//! If the on-the-wire DeviceAttestationEnvelope silently no-op'd at
//! any leg, OR if the receiver-side `apply_atrium_merge` defaulted
//! to the receiver's own device_cid for the AttributionFrame's
//! device_did slot, the GREEN-path pin fails: the laptop-side
//! AttributionFrame would NOT carry phone's device-DID after merging
//! the phone's write, losing device-grain provenance per Inv-14.
//!
//! The forgery / replay / frame-pair-binding pins ALSO observably
//! fail-fast: a forged envelope (mismatched device-DID vs
//! envelope-signing keypair) rejects with
//! `E_DEVICE_ATTESTATION_FORGED`; a replayed parent-issued nonce
//! rejects via the receiver's nonce-store; a swapped payload rejects
//! via constant-time BLAKE3 comparison.

#![allow(clippy::unwrap_used, clippy::too_many_lines)]
#![cfg(not(target_arch = "wasm32"))]

use std::time::Duration;

use benten_core::Cid;
use benten_core::hlc::BentenHlc;
use benten_engine::Engine;
use benten_engine::atrium_api::AtriumConfig;
use benten_engine::engine_sync::DeviceAttestationEnvelope;
use benten_id::device_attestation::{
    Acceptor, CapabilityEnvelope, DeviceAttestation, FreshnessPolicy, UptimePolicy, ZoneScope,
};
use benten_id::did::Did;
use benten_id::keypair::Keypair;

const ZONE: &str = "/zone/notifications";
const ACTOR_BYTES: &[u8] = b"actor:alice-shared-identity";
const LAPTOP_DEVICE_BYTES: &[u8] = b"device:alice-laptop";
const PHONE_DEVICE_BYTES: &[u8] = b"device:alice-phone";

/// Issue a fresh full-peer capability envelope for a device-DID under
/// the supplied parent (user-identity) keypair.
fn issue_full_peer_attestation(parent_kp: &Keypair, device_did: Did) -> DeviceAttestation {
    DeviceAttestation::issue(
        parent_kp,
        device_did,
        CapabilityEnvelope {
            runs_sandbox: true,
            holds_zones: ZoneScope::Full,
            online_uptime: UptimePolicy::AlwaysOn,
            runs_atrium_peer: true,
        },
    )
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn atrium_two_device_same_identity_selective_zone_sync() {
    // Plan §1 exit-criterion 16 LOAD-BEARING pin (G16-D wave-6b + fp).
    //
    // The fix-pass (G16-D wave-6b fp) replaces the prior fake-DID
    // strings ("did:key:zAlice{Laptop,Phone}Device") with REAL signed
    // DeviceAttestation envelopes — closing the cryptographic gaps
    // surfaced at the post-PR-#163 mini-review (cryptography lens
    // findings g16d6b-crypto-1/2/3/4 + correctness g16d6b-corr-2/3).

    // Step 1: Spin up two engines under the SAME actor_cid (account
    // identity) but DIFFERENT device_cids (laptop + phone).
    let actor_cid = Cid::from_blake3_digest(*blake3::hash(ACTOR_BYTES).as_bytes());
    let laptop_device_cid = Cid::from_blake3_digest(*blake3::hash(LAPTOP_DEVICE_BYTES).as_bytes());
    let phone_device_cid = Cid::from_blake3_digest(*blake3::hash(PHONE_DEVICE_BYTES).as_bytes());
    assert_ne!(
        laptop_device_cid, phone_device_cid,
        "two-device pin requires distinct device CIDs"
    );

    // Real Ed25519 keypairs: parent (user identity), laptop device,
    // phone device. The parent keypair signs each device's
    // attestation; each device-keypair signs its own outbound wire
    // envelope.
    let parent_kp = Keypair::generate();
    let laptop_kp = Keypair::generate();
    let phone_kp = Keypair::generate();
    let laptop_device_did = Did::from_public_key(laptop_kp.public_key());
    let phone_device_did = Did::from_public_key(phone_kp.public_key());
    let laptop_attestation = issue_full_peer_attestation(&parent_kp, laptop_device_did.clone());
    let phone_attestation = issue_full_peer_attestation(&parent_kp, phone_device_did.clone());

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

    // Step 2: Both devices join the same Atrium. Bind the SIGNED
    // attestation + device-keypair so outbound envelopes are V2 shape
    // (signed; payload-hash bound; session-nonce replay-defended).
    // Install permissive Acceptor on each side (test scope: parent-DID
    // is the trusted issuer; freshness window is u64::MAX so the
    // issued_at=0 attestations don't expire under wall-clock).
    let atrium_laptop = engine_laptop
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    let atrium_phone = engine_phone
        .open_atrium(AtriumConfig::for_test())
        .await
        .unwrap();
    atrium_laptop
        .set_local_device_attestation(Some(laptop_attestation.clone()))
        .await;
    atrium_laptop
        .set_local_device_keypair(Some(seed_clone(&laptop_kp)))
        .await;
    atrium_phone
        .set_local_device_attestation(Some(phone_attestation.clone()))
        .await;
    atrium_phone
        .set_local_device_keypair(Some(seed_clone(&phone_kp)))
        .await;
    // Each side accepts attestations from the parent identity.
    atrium_laptop
        .set_acceptor(Acceptor::new(FreshnessPolicy::seconds(u64::MAX)))
        .await;
    atrium_phone
        .set_acceptor(Acceptor::new(FreshnessPolicy::seconds(u64::MAX)))
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
    // Per fix-pass, the envelope is signed (V2 shape); receiver
    // verifies signature + parent-chain Acceptor + payload-hash before
    // populating the slot.
    let phone_addr = atrium_phone.loopback_addr().unwrap();
    let atrium_phone_clone = atrium_phone.clone();
    let zone_owned = ZONE.to_string();
    let accept_task =
        tokio::spawn(async move { atrium_phone_clone.accept_sync_subgraph(&zone_owned).await });
    tokio::time::sleep(Duration::from_millis(50)).await;
    atrium_laptop.sync_subgraph(ZONE, phone_addr).await.unwrap();
    accept_task.await.unwrap().unwrap();

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
    // merge of laptop-bytes mints a frame carrying laptop's device-DID;
    // laptop's merge of phone-bytes mints a frame carrying phone's
    // device-DID. If the engine had defaulted to the receiver's own
    // device_cid, the device_did would carry the LOCAL device's
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
    // share the SAME actor_cid; each side carries the OTHER device's
    // SIGNED device-DID (the resolved did:key:z<base58> form, NOT a
    // synthetic placeholder).
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
        device_did: Some(laptop_device_did.as_str().to_string()),
        sync_hop_depth: 1,
    };
    let expected_laptop_frame = benten_eval::AttributionFrame {
        actor_cid,
        handler_cid: Cid::from_blake3_digest([0u8; 32]),
        capability_grant_cid: Cid::from_blake3_digest([0u8; 32]),
        sandbox_depth: 0,
        peer_did_set: Some(laptop_peer_did_set),
        device_did: Some(phone_device_did.as_str().to_string()),
        sync_hop_depth: 1,
    };
    let expected_phone_cid = expected_phone_frame.cid().unwrap();
    let expected_laptop_cid = expected_laptop_frame.cid().unwrap();
    assert_eq!(
        phone_frame_cid,
        expected_phone_cid.as_bytes().to_vec(),
        "phone-side AttributionFrame.device_did MUST carry laptop's REAL signed device-DID \
         (the ORIGINATING device for the phone's merge of laptop-bytes), NOT the phone's \
         own device_cid; defends against multi-device merges losing device-grain provenance \
         per Inv-14"
    );
    assert_eq!(
        laptop_frame_cid,
        expected_laptop_cid.as_bytes().to_vec(),
        "laptop-side AttributionFrame.device_did MUST carry phone's REAL signed device-DID \
         (the ORIGINATING device for the laptop's merge of phone-bytes), NOT the laptop's \
         own device_cid; defends against multi-device merges losing device-grain provenance \
         per Inv-14"
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

/// G16-D wave-6b fp — DID forgery rejection at the wire boundary.
///
/// A bad-faith peer constructs an envelope claiming the victim's
/// device-DID but signed by an attacker's keypair. The receiver's
/// envelope-signature check (against the public key resolved from
/// `attestation.device_did`) rejects with
/// `E_DEVICE_ATTESTATION_FORGED`. This is the load-bearing
/// observable-consequence pin for cryptography MAJOR-1: an unsigned
/// peer cannot impersonate another device's identity over the wire.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn forged_device_did_rejected_at_envelope_verify() {
    let parent_kp = Keypair::generate();
    let victim_kp = Keypair::generate();
    let attacker_kp = Keypair::generate();
    let victim_did = Did::from_public_key(victim_kp.public_key());
    // Parent issues an attestation for the victim's device-DID.
    let victim_attestation = issue_full_peer_attestation(&parent_kp, victim_did.clone());

    // Attacker holds the victim's parent-signed attestation but tries
    // to sign the wire envelope with their OWN keypair — DID forgery.
    let loro_payload = b"forged-payload-bytes";
    let envelope =
        DeviceAttestationEnvelope::new_signed(victim_attestation, loro_payload, &attacker_kp)
            .unwrap();
    // Receiver verifies — must reject. (Acceptor configured permissive
    // so the failure is unambiguously the envelope-signature check,
    // not a freshness-window edge case.)
    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    let now_secs = 0u64;
    let result = envelope.verify(loro_payload, &acceptor, now_secs);
    let err = result.expect_err(
        "forged envelope MUST reject — attacker keypair cannot sign for victim's device-DID",
    );
    assert!(
        format!("{err}").contains("envelope signature does not verify")
            || format!("{err}").contains("DID forgery"),
        "expected DID-forgery rejection, got {err}"
    );
    let code = err.code();
    assert_eq!(
        code,
        benten_engine::ErrorCode::DeviceAttestationForged,
        "DID forgery MUST surface E_DEVICE_ATTESTATION_FORGED typed code; got {code:?}"
    );
}

/// G16-D wave-6b fp — replay rejection at the wire boundary.
///
/// A captured envelope from session A is replayed against the same
/// receiver in session B. The receiver's `Acceptor::accept_at`
/// nonce-store insertion fails on the second call (the parent-issued
/// attestation nonce is already in the store) so the replayed
/// envelope rejects with `E_DEVICE_ATTESTATION_FORGED`. Load-bearing
/// observable-consequence pin for cryptography MAJOR-2.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn replayed_envelope_rejected_by_acceptor_nonce_store() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_full_peer_attestation(&parent_kp, device_did);

    let loro_payload = b"replay-test-payload";
    // The same attestation is reused across two envelope constructions
    // (production: two consecutive sync_subgraph calls with the same
    // bound attestation). Each envelope carries a fresh session_nonce
    // and a fresh signature, but the parent-issued attestation nonce
    // is shared — that's what the Acceptor's nonce-store catches.
    let envelope_1 =
        DeviceAttestationEnvelope::new_signed(attestation.clone(), loro_payload, &device_kp)
            .unwrap();
    let envelope_2 =
        DeviceAttestationEnvelope::new_signed(attestation, loro_payload, &device_kp).unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    let now_secs = 0u64;
    // First envelope verifies (consumes the attestation nonce).
    envelope_1
        .verify(loro_payload, &acceptor, now_secs)
        .expect("first envelope must verify");
    // Second envelope replays the same parent-issued attestation
    // nonce — must reject.
    let err = envelope_2
        .verify(loro_payload, &acceptor, now_secs)
        .expect_err("replayed envelope MUST reject via Acceptor nonce-store");
    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "replay rejection MUST surface E_DEVICE_ATTESTATION_FORGED"
    );
    assert!(
        format!("{err}").contains("attestation chain rejected"),
        "expected attestation-chain rejection (NonceReplay), got {err}"
    );
}

/// G16-D wave-6b fp — frame-pair payload-binding rejection.
///
/// A MITM constructs a legitimate envelope for payload A but delivers
/// payload B to the receiver. The receiver's BLAKE3 comparison
/// (`payload_hash != BLAKE3(received_payload)`) rejects with
/// `E_DEVICE_ATTESTATION_FORGED`. Load-bearing observable-consequence
/// pin for cryptography MAJOR-3.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn frame_pair_payload_swap_rejected_by_payload_hash_binding() {
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_full_peer_attestation(&parent_kp, device_did);

    let payload_a = b"original-payload-bytes-A";
    let payload_b = b"swapped-payload-bytes-B-different-length";
    // Envelope signs over BLAKE3(payload_a).
    let envelope =
        DeviceAttestationEnvelope::new_signed(attestation, payload_a, &device_kp).unwrap();

    let acceptor = Acceptor::new(FreshnessPolicy::seconds(u64::MAX));
    let now_secs = 0u64;
    // Verifying against the original payload succeeds.
    envelope
        .verify(payload_a, &acceptor, now_secs)
        .expect("envelope must verify against the payload it signed over");
    // Verifying against a SWAPPED payload must reject (the BLAKE3
    // mismatch is the load-bearing assertion).
    //
    // Note: build a SECOND envelope so the Acceptor's nonce-store
    // doesn't reject for replay reasons (we want the failure to be
    // unambiguously the payload-hash binding, not the nonce store).
    let parent_kp2 = Keypair::generate();
    let device_kp2 = Keypair::generate();
    let device_did2 = Did::from_public_key(device_kp2.public_key());
    let attestation2 = issue_full_peer_attestation(&parent_kp2, device_did2);
    let envelope2 =
        DeviceAttestationEnvelope::new_signed(attestation2, payload_a, &device_kp2).unwrap();
    let err = envelope2
        .verify(payload_b, &acceptor, now_secs)
        .expect_err("swapped payload MUST reject via payload_hash binding");
    assert_eq!(
        err.code(),
        benten_engine::ErrorCode::DeviceAttestationForged,
        "frame-pair binding violation MUST surface E_DEVICE_ATTESTATION_FORGED"
    );
    assert!(
        format!("{err}").contains("frame-pair binding")
            || format!("{err}").contains("payload_hash"),
        "expected frame-pair-binding rejection, got {err}"
    );
}

/// G16-D wave-6b fp — version validation in `from_canonical_bytes`.
///
/// Closes cryptography MINOR-5: a future-version envelope (v=255)
/// must reject at decode time so a newer peer's envelope shape doesn't
/// silently surface as v1/v2 fields with possibly different semantics.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn future_wire_version_rejected_at_decode() {
    // Construct a forward-version envelope by hand-rolling the bytes:
    // start from a valid V2 envelope, decode it as a serde_ipld_dagcbor
    // Value, mutate the version field to 255, re-encode, and feed
    // through from_canonical_bytes.
    let parent_kp = Keypair::generate();
    let device_kp = Keypair::generate();
    let device_did = Did::from_public_key(device_kp.public_key());
    let attestation = issue_full_peer_attestation(&parent_kp, device_did);
    let envelope_v2 =
        DeviceAttestationEnvelope::new_signed(attestation, b"payload", &device_kp).unwrap();
    // Mutate the in-memory struct's version field to a future shape +
    // re-encode. (DAG-CBOR canonical encoding is deterministic so the
    // resulting bytes are exactly what a future v255 peer would emit
    // for this struct shape, sans semantic interpretation.)
    let mut future_envelope = envelope_v2;
    future_envelope.version = 255;
    let future_bytes = future_envelope.to_canonical_bytes().unwrap();
    let result = DeviceAttestationEnvelope::from_canonical_bytes(&future_bytes);
    let err = result.expect_err("future-version envelope MUST reject at decode");
    let s = format!("{err}");
    assert!(
        s.contains("version 255") && s.contains("MAX_WIRE_VERSION"),
        "expected version-mismatch rejection naming MAX_WIRE_VERSION, got {s}"
    );
}

/// Helper: clone a keypair via the export/import envelope path.
/// `Keypair` does NOT implement `Clone` per crypto-blocker-1; tests
/// that need to bind the same logical device-keypair to multiple
/// surfaces (e.g. `set_local_device_keypair` here while the same
/// keypair signs the test's pre-issued attestation) round-trip
/// through `export_seed_envelope` + `from_dag_cbor_envelope` per the
/// production-audit-shaped clone path.
fn seed_clone(kp: &Keypair) -> Keypair {
    let envelope = kp.export_seed_envelope();
    Keypair::from_dag_cbor_envelope(&envelope).unwrap()
}

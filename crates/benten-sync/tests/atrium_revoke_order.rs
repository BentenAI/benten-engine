//! R3-C RED-PHASE pins for atrium revocation-order at reconnect
//! (G16-B + G16-C wave-6b; per r2-test-landscape §2.4 G16-B + plan
//! §3 G16-B row + device-mesh exploration brief-edits + net-blocker-3
//! + crypto-major-6).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-B rows
//!   `atrium_revoke_propagates_before_data_to_offline_then_reconnect_peer` +
//!   `atrium_revocation_message_kind_ordered_before_data_at_handshake` +
//!   `atrium_device_did_revocation_propagates_before_data_to_offline_then_reconnect_peer`.
//! - plan §3 G16-B row line "revocation-order pin per device-mesh
//!   exploration brief-edit 2026-05-04 — when peer-A revokes peer-B's
//!   grant while B is offline, then both come back online, B MUST
//!   receive the revocation event BEFORE any subsequent data writes
//!   A makes; otherwise B continues acting under stale grant".
//! - `net-blocker-3` BLOCKER (revocation-message-kind ordered before
//!   data at handshake + MST diff drain).
//! - `crypto-major-6` (device-DID revocation propagation).
//! - `.addl/phase-3/exploration-device-mesh.md` brief-edits 2026-05-04.
//!
//! ## Revocation-order narrative
//!
//! When peer-A revokes peer-B's grant while B is offline, then both
//! come back online, B MUST receive the revocation event BEFORE any
//! subsequent data writes A makes. Otherwise B continues acting under
//! a stale grant — a security failure shape per net-blocker-3 +
//! crypto-major-6.
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale pointing to phase-3-backlog §7.3.D STALE-RATIONALE sweep #2 (Phase-3 R6 R1 fix-pass Wave E 2026-05-09); destination §6.12 G16-B post-canary residuals (v1-assessment-window).

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "phase-3-backlog §7.3.D — revocation propagates before data to offline-then-reconnect peer. G16-B + G16-C wave-6b shipped MST diff + Loro CRDT integration; test body pins specific revocation-ordering defensive contract that composes with §6.12 G16-B post-canary residuals + §3.1-followup multi-peer iroh sync (CLOSED at G16-B-E PR #160). Body un-ignore at §6.12 v1-assessment-window landing per Wave-E rationale-only sweep."]
fn atrium_revoke_propagates_before_data_to_offline_then_reconnect_peer() {
    // device-mesh exploration brief-edit 2026-05-04 + net-blocker-3
    // BLOCKER pin. G16-B implementer wires this:
    //
    //   let mut peer_a = test_peer(peer_a_did);
    //   let mut peer_b = test_peer(peer_b_did);
    //   peer_a.atrium_join(shared_atrium()).await.unwrap();
    //   peer_b.atrium_join(shared_atrium()).await.unwrap();
    //
    //   // peer_a grants peer_b read on /zone/secrets:
    //   peer_a.atrium_grant(peer_b_did, ucan_grant("/zone/secrets", "read")).await.unwrap();
    //   wait_for_sync(&[&peer_a, &peer_b]).await;
    //
    //   // peer_b goes offline:
    //   peer_b.disconnect();
    //
    //   // peer_a revokes peer_b's grant + writes new data:
    //   peer_a.atrium_revoke(peer_b_did, "/zone/secrets/*").await.unwrap();
    //   peer_a.write_node_in_zone("/zone/secrets", make_secret("s1")).await.unwrap();
    //   peer_a.write_node_in_zone("/zone/secrets", make_secret("s2")).await.unwrap();
    //
    //   // peer_b reconnects + drains pending atrium events:
    //   peer_b.reconnect().await.unwrap();
    //   peer_b.atrium_drain_pending().await.unwrap();
    //
    //   // BEFORE ANY data write reaches peer_b, the revocation arrived:
    //   let drain_log = peer_b.drain_log();
    //   let revoke_idx = drain_log.iter().position(|e| matches!(e.kind(), MessageKind::Revocation)).unwrap();
    //   let first_data_idx = drain_log.iter().position(|e| matches!(e.kind(), MessageKind::Data)).unwrap_or(usize::MAX);
    //   assert!(revoke_idx < first_data_idx,
    //       "revocation MUST be drained before any subsequent data write per net-blocker-3");
    //
    //   // peer_b's effective cap-set no longer includes /zone/secrets:
    //   let effective_caps = peer_b.atrium_effective_cap_set();
    //   assert!(!effective_caps.includes_path("/zone/secrets/*"));
    //
    // OBSERVABLE consequence: revocation always arrives ahead of
    // subsequent data writes; peer_b never observes data under a
    // stale grant. Defends against the net-blocker-3 BLOCKER attack
    // class.
    unimplemented!("G16-B wires revocation-before-data ordering at offline-reconnect");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — revocation message-kind ordered before data at handshake. G16-B wave-6b shipped Atrium API; test body pins revocation-message-ordering defensive contract; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn atrium_revocation_message_kind_ordered_before_data_at_handshake() {
    // net-blocker-3 BLOCKER pin. The handshake protocol explicitly
    // names `Revocation` as a typed message-kind drained BEFORE
    // `Data` message-kinds at peer reconnect.
    //
    //   use benten_sync::handshake::MessageKind;
    //   use benten_sync::handshake::HandshakeStateMachine;
    //
    //   let mut hs = HandshakeStateMachine::new(peer_b_handle);
    //   // Inject a queue with mixed message kinds:
    //   hs.enqueue_pending_message(MessageKind::Data, data_msg_1);
    //   hs.enqueue_pending_message(MessageKind::Revocation, revoke_msg);
    //   hs.enqueue_pending_message(MessageKind::Data, data_msg_2);
    //
    //   // Drain order is determined by message kind, not arrival order:
    //   let drain: Vec<_> = hs.drain_pending().collect();
    //   assert_eq!(drain[0].kind(), MessageKind::Revocation);
    //   // Data messages come after:
    //   assert!(drain[1..].iter().all(|m| m.kind() == MessageKind::Data));
    //
    // OBSERVABLE consequence: drain order guarantees revocation is
    // processed first; defends against arrival-order ambiguity.
    unimplemented!("G16-B wires MessageKind::Revocation drain-priority assertion");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — device-DID revocation propagates before data. G16-D wave-6b PR #163 shipped on-the-wire device-DID-attestation envelope; test body pins device-DID-revocation propagation order; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn atrium_device_did_revocation_propagates_before_data_to_offline_then_reconnect_peer() {
    // crypto-major-6 pin. Companion to the peer-DID revocation pin
    // above, but at the DEVICE-DID grain. When peer-A revokes a
    // SPECIFIC DEVICE-DID under peer-B's account (e.g., B's lost
    // phone), the device-DID revocation propagates before subsequent
    // data writes reach the affected device.
    //
    //   peer_a.atrium_revoke_device_did(peer_b_account, peer_b_lost_phone_did).await.unwrap();
    //   peer_a.write_node_in_zone("/zone/sensitive", ...).await.unwrap();
    //
    //   // peer_b's other devices reconnect; the revoked device's
    //   // device-DID is in the revocation set BEFORE any data write
    //   // reaches it (or any of B's other devices that share the
    //   // same Atrium membership).
    //   peer_b_phone.reconnect().await.unwrap();
    //   let drain_log = peer_b_phone.drain_log();
    //   let revoke_idx = drain_log.iter().position(|e| {
    //       matches!(e.kind(), MessageKind::Revocation)
    //           && e.target_device_did() == Some(peer_b_lost_phone_did)
    //   }).unwrap();
    //   let first_data_idx = drain_log.iter().position(|e| matches!(e.kind(), MessageKind::Data)).unwrap_or(usize::MAX);
    //   assert!(revoke_idx < first_data_idx);
    //
    // OBSERVABLE consequence: device-DID revocation is observable
    // at the device-grain across peer reconnects.
    unimplemented!("G16-B wires device-DID revocation-before-data ordering");
}

#[test]
#[ignore = "phase-3-backlog §7.3.D — MST diff preserves temporal ordering of grants/revocations interleaved with data writes during offline-reconnect. G16-C wave-6b shipped MST diff (PR #124); test body pins offline-reconnect temporal-ordering defensive contract; un-ignore at §6.12 G16-B post-canary residuals landing (v1-assessment-window) per Wave-E rationale-only sweep."]
fn mst_diff_preserves_temporal_ordering_of_grants_and_revocations_relative_to_data_writes_under_offline_reconnect()
 {
    // ds-r4-2 (R4 large-council Round 1 distributed-systems lens) pin.
    // R1 ds-9 was triaged into 'distribute across G16 row briefs' but
    // the substantive content (interleaved-during-offline-window
    // temporal-ordering preservation) was lost.
    //
    // The simple revoke-then-data ordering case is covered by
    // `atrium_revoke_propagates_before_data_to_offline_then_reconnect_peer`
    // above. THIS pin covers the INTERLEAVED case where during the
    // phone-offline window peer-A wrote N1, revoked phone's grant,
    // then wrote N2 — phone reconnects and MST diff offers
    // {N1, revoke-event, N2} as a SET. Phone needs to apply
    // revoke-event BETWEEN N1 and N2 to correctly enforce the
    // partial-revoke (otherwise phone may briefly observe N2 before
    // the revocation lands locally).
    //
    // MST diff converges on a SET-equality basis, not a
    // temporal-ordering basis by default. The HLC infrastructure
    // landed at G14-pre-D supports the temporal-ordering observation;
    // this pin asserts the consumer-side preservation.
    //
    //   use benten_sync::mst_diff::MstDiff;
    //   use benten_sync::handshake::MessageKind;
    //
    //   let mut peer_a = test_peer(peer_a_did);
    //   let mut peer_b_phone = test_peer(peer_b_phone_did);
    //   peer_a.atrium_join(shared_atrium()).await.unwrap();
    //   peer_b_phone.atrium_join(shared_atrium()).await.unwrap();
    //
    //   // peer_a grants peer_b read on /zone/sub:
    //   peer_a.atrium_grant(peer_b_phone_did, ucan_grant("/zone/sub/*", "read")).await.unwrap();
    //   wait_for_sync(&[&peer_a, &peer_b_phone]).await;
    //
    //   // peer_b_phone goes offline:
    //   peer_b_phone.disconnect();
    //
    //   // Interleaved sequence at peer_a (with HLC-staggered timestamps):
    //   let n1 = peer_a.write_node_in_zone("/zone/sub", make_node("n1")).await.unwrap();  // T1
    //   peer_a.atrium_revoke(peer_b_phone_did, "/zone/sub/*").await.unwrap();              // T2
    //   let n2 = peer_a.write_node_in_zone("/zone/sub", make_node("n2")).await.unwrap();  // T3
    //
    //   // peer_b_phone reconnects + drains MST diff:
    //   peer_b_phone.reconnect().await.unwrap();
    //   let diff = MstDiff::compute(&peer_a.mst_root(), &peer_b_phone.mst_root());
    //   assert!(diff.contains_node(&n1));
    //   assert!(diff.contains_revocation_for(peer_b_phone_did, "/zone/sub/*"));
    //   assert!(diff.contains_node(&n2));
    //
    //   // Apply diff in HLC-temporal order (NOT raw set order):
    //   let ordered = diff.into_hlc_ordered_events();
    //   peer_b_phone.apply_ordered(ordered.clone()).await.unwrap();
    //
    //   // ASSERTION: ordered application places revoke-event BETWEEN
    //   // n1 and n2 (per HLC timestamps T1 < T2 < T3):
    //   let event_kinds: Vec<_> = ordered.iter().map(|e| e.kind()).collect();
    //   let revoke_idx = event_kinds.iter().position(|k|
    //       matches!(k, MessageKind::Revocation)).unwrap();
    //   let n1_idx = event_kinds.iter().position(|k|
    //       matches!(k, MessageKind::Data) && /*n1 cid*/).unwrap();
    //   let n2_idx = event_kinds.iter().position(|k|
    //       matches!(k, MessageKind::Data) && /*n2 cid*/).unwrap();
    //   assert!(n1_idx < revoke_idx,
    //       "n1 (T1) must apply BEFORE revoke (T2) per HLC ordering");
    //   assert!(revoke_idx < n2_idx,
    //       "revoke (T2) must apply BEFORE n2 (T3) per HLC ordering — \
    //        otherwise phone briefly observes n2 under stale grant");
    //
    //   // peer_b_phone never saw n2 under stale grant:
    //   let observation_log = peer_b_phone.observation_log_for("/zone/sub");
    //   for event in &observation_log {
    //       if event.cid() == n2.cid() {
    //           assert!(!event.under_grant_for_did(peer_b_phone_did),
    //               "n2 must never be observable under the (revoked) grant");
    //       }
    //   }
    //
    // OBSERVABLE consequence: MST-diff drainage preserves HLC-temporal
    // ordering of grants/revocations interleaved with data writes
    // during the offline window; phone never briefly observes data
    // under a (later-revoked) stale grant. Composes G14-pre-D HLC +
    // G16-B Loro merges + G16-C MST-diff. Defends against the
    // SET-vs-ORDER attack class that R1 ds-9 named.
    unimplemented!(
        "G16-B + G16-C wire MST-diff HLC-temporal-ordering preservation for interleaved revoke+data offline-reconnect"
    );
}

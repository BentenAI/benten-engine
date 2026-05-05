//! R3-B + R3-D RED-PHASE pins: atrium browser tab as thin-client view
//! into a full peer (G14-D + G18-A; D-PHASE-3-N + baked-in #17 +
//! exit-criterion 19).
//!
//! Pin sources (per `.addl/phase-3/r2-test-landscape.md` §2.2 G14-D +
//! §3.E thin-client cluster):
//!
//! - `integration/atrium_browser_tab_as_thin_client_view_into_full_peer_e2e` — D-PHASE-3-N + baked-in #17 + exit-criterion 19
//! - `browser_tab_thin_client_authenticated_view_into_full_peer` — exit-criterion 19 (redundant-distinct)
//!
//! ## Ownership (per r2-test-landscape §13 ambiguous-ownership pre-emption)
//!
//! - **R3-B** (this dispatch): authors the full-peer-side filtering
//!   pin (`atrium_browser_tab_as_thin_client_view_into_full_peer_e2e`).
//!   Asserts the full peer applies per-subscriber filtering when a
//!   thin-client tab connects via D-PHASE-3-N protocol.
//!
//! - **R3-D** (subsequent dispatch): extends this file with the
//!   browser-side IndexedDB cache assertion. Both pins share the file
//!   with disjoint test-fn ownership.
//!
//! ## Architectural intent (CLAUDE.md baked-in #17)
//!
//! Browser tabs are NOT full atrium peers (they are wasm32, no
//! native iroh / Loro). They are AUTHENTICATED THIN-CLIENT VIEWS into
//! a full peer running on the user's hardware (laptop / phone OS app).
//! The thin-client protocol (D-PHASE-3-N) carries an SSE / WebSocket
//! subscription wire from full peer to browser tab; the full peer
//! does the cap recheck + filtering BEFORE forwarding events to the
//! tab.
//!
//! Per exit-criterion 19, this is the load-bearing pin demonstrating
//! the thin-client commitment ships end-to-end.
//!
//! ## RED-PHASE discipline
//!
//! Per R3-A canary precedent. Stays `#[ignore]`'d until G14-D + G18-A
//! implementer un-ignores. Per §3.6b pim-2 the test must drive a
//! SIMULATED browser tab (likely via headless wasm32 runner) +
//! observe the production filtering at the full-peer side.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G14-D + G18-A — D-PHASE-3-N + baked-in #17 + exit-criterion 19 — browser thin-client e2e"]
fn atrium_browser_tab_as_thin_client_view_into_full_peer_e2e() {
    // R3-B owns this pin. G14-D + G18-A implementer wires this:
    //
    //   // 1. Spin up a full peer (native engine):
    //   let full_peer = benten_engine::Engine::open(full_peer_store.path()).unwrap();
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp_browser = benten_id::keypair::Keypair::generate();
    //
    //   // 2. Browser tab declares thin-client envelope:
    //   let browser_envelope = benten_id::device_attestation::CapabilityEnvelope {
    //       runs_sandbox: false,
    //       holds_zones: benten_id::device_attestation::ZoneScope::CacheOnly,
    //       online_uptime: benten_id::device_attestation::UptimePolicy::SessionBounded,
    //       runs_atrium_peer: false, // KEY: thin-client, not full peer
    //   };
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp_browser.public_key().to_did(), browser_envelope).unwrap();
    //
    //   // 3. Browser tab connects via thin-client protocol:
    //   let thin_client = benten_engine::ThinClientConnection::connect(
    //       &full_peer.thin_client_endpoint(), attestation).unwrap();
    //
    //   // 4. Thin client subscribes to a zone:
    //   let sub_id = thin_client.subscribe("/zone/posts").unwrap();
    //
    //   // 5. Full peer writes a node; full-peer-side filtering decides
    //   //    whether to forward to thin client:
    //   full_peer.write_node(&node_in_zone_posts).unwrap();
    //
    //   // 6. Thin client receives the event over the protocol wire:
    //   let evt = thin_client.next_event(Duration::from_millis(500)).unwrap();
    //   assert_eq!(evt.subscription_id(), sub_id);
    //   assert_eq!(evt.zone(), "/zone/posts");
    //
    //   // 7. Critically — the FILTERING happened at the full peer side
    //   //    (the browser tab cannot run cap policy itself). Verify by
    //   //    inspecting full-peer's outbound metrics:
    //   let metrics = full_peer.thin_client_metrics();
    //   assert!(metrics.outbound_events_after_filter > 0);
    //   assert_eq!(metrics.outbound_events_filtered, 0); // none filtered for this allowed cap
    //
    //   // 8. Now revoke the thin client's grant; verify subsequent
    //   //    events are FILTERED at the full peer (not delivered to tab):
    //   full_peer.caps().revoke_for_device(&device_kp_browser.public_key().to_did()).unwrap();
    //   full_peer.write_node(&another_node).unwrap();
    //   assert!(thin_client.try_next_event(Duration::from_millis(200)).is_none());
    //
    //   let metrics_post = full_peer.thin_client_metrics();
    //   assert!(metrics_post.outbound_events_filtered > 0,
    //       "post-revoke event MUST be filtered at full peer per baked-in #17");
    //
    // OBSERVABLE consequence: the browser tab observably receives
    // events (proves the protocol wire works) AND the full peer
    // observably filters post-revocation events (proves the auth
    // gate is at the full peer, not the tab). Closes
    // exit-criterion 19.
    unimplemented!(
        "G14-D + G18-A wires browser-thin-client e2e: full-peer-side filtering + protocol wire delivery"
    );
}

#[test]
#[ignore = "RED-PHASE: G14-D — exit-criterion 19 — redundant-distinct: thin client is authenticated view"]
fn browser_tab_thin_client_authenticated_view_into_full_peer() {
    // exit-criterion 19 redundant-distinct pin (CLR-2-style). Composes
    // with `atrium_browser_tab_as_thin_client_view_into_full_peer_e2e`
    // but tests the AUTHENTICATION step in isolation — a thin client
    // that fails to present a valid device attestation cannot connect.
    //
    // R3-B owns this pin too (both R3-B pins are full-peer-side).
    // R3-D will extend with browser-IndexedDB-cache assertions in a
    // SEPARATE test fn within this same file.
    //
    // Implementer wires:
    //
    //   let full_peer = benten_engine::Engine::open(full_peer_store.path()).unwrap();
    //
    //   // 1. Connect WITHOUT attestation: rejects.
    //   let err = benten_engine::ThinClientConnection::connect_unauthenticated(
    //       &full_peer.thin_client_endpoint()).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::ThinClientError::AttestationRequired));
    //
    //   // 2. Connect with VALID attestation: succeeds.
    //   let parent_kp = benten_id::keypair::Keypair::generate();
    //   let device_kp = benten_id::keypair::Keypair::generate();
    //   let attestation = benten_id::device_attestation::DeviceAttestation::issue(
    //       &parent_kp, device_kp.public_key().to_did(),
    //       benten_id::device_attestation::CapabilityEnvelope::default()).unwrap();
    //   let conn = benten_engine::ThinClientConnection::connect(
    //       &full_peer.thin_client_endpoint(), attestation).unwrap();
    //   assert!(conn.is_authenticated());
    //
    //   // 3. Connect with REVOKED-device attestation: rejects.
    //   let revocation = benten_id::device_attestation::DeviceRevocation::issue(
    //       &parent_kp, device_kp.public_key().to_did(),
    //       benten_id::device_attestation::RevocationReason::DeviceLoss).unwrap();
    //   full_peer.consume_device_revocation(&revocation).unwrap();
    //   let err = benten_engine::ThinClientConnection::connect(
    //       &full_peer.thin_client_endpoint(), attestation).unwrap_err();
    //   assert!(matches!(err,
    //       benten_engine::ThinClientError::DeviceRevoked));
    //
    // OBSERVABLE consequence: thin-client connection requires a
    // valid + non-revoked device attestation; the full peer is the
    // auth boundary. Closes the "anyone can connect to my engine"
    // failure shape per baked-in #17.
    unimplemented!("G14-D wires thin-client authentication contract at full-peer connection seam");
}

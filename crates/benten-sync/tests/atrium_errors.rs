//! R3-C RED-PHASE pins for typed atrium-transport errors (G16-A
//! wave-6 canary; per r2-test-landscape §2.4 G16-A + plan §3 G16-A
//! row + net-blocker-2 + net-blocker-4).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A rows
//!   `atrium_relay_unreachable_fires_typed_error_no_panic` +
//!   `atrium_transport_degraded_explicit_state_observable_via_engine_atrium_status` +
//!   `atrium_handshake_wire_format_carries_peer_did_and_device_did`.
//! - plan §3 G16-A row.
//! - `net-blocker-2` BLOCKER (typed `E_ATRIUM_RELAY_UNREACHABLE` +
//!   `E_ATRIUM_TRANSPORT_DEGRADED`).
//! - `net-blocker-4` BLOCKER (peer-handshake metadata carries
//!   peer-DID AND device-DID).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A wave-6 canary lands typed atrium-transport errors"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — net-blocker-2 — relay-unreachable typed error"]
fn atrium_relay_unreachable_fires_typed_error_no_panic() {
    // net-blocker-2 BLOCKER pin. When the relay is unreachable,
    // the transport surface returns `E_ATRIUM_RELAY_UNREACHABLE`
    // (typed; no panic; no untyped String).
    //
    // G16-A implementer wires this:
    //   use benten_sync::errors::AtriumTransportError;
    //   let endpoint = Endpoint::builder()
    //       .relay_url("https://nonexistent.invalid".parse().unwrap())
    //       .build().await.unwrap();
    //   let result = endpoint.connect(remote_peer_id()).await;
    //   match result {
    //       Err(AtriumTransportError::RelayUnreachable { url, .. }) => {
    //           assert!(url.contains("nonexistent"));
    //       }
    //       Err(other) => panic!("expected RelayUnreachable, got {other:?}"),
    //       Ok(_) => panic!("expected RelayUnreachable error, got Ok"),
    //   }
    //   // The error must map to the stable error code:
    //   assert_eq!(result.unwrap_err().code(), ErrorCode::E_ATRIUM_RELAY_UNREACHABLE);
    //
    // OBSERVABLE consequence: a relay-unreachable failure surfaces
    // as a typed error variant with the stable error code; never as
    // a panic, never as an untyped String error.
    unimplemented!(
        "G16-A wires E_ATRIUM_RELAY_UNREACHABLE typed error variant + error-code stability"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — net-blocker-2 — transport-degraded observable"]
fn atrium_transport_degraded_explicit_state_observable_via_engine_atrium_status() {
    // net-blocker-2 BLOCKER pin. When the transport degrades (e.g.
    // direct connection lost, relay slow, packet loss above
    // threshold), the engine's atrium-status surface reports
    // `TransportDegraded` as an explicit state — not as a missing
    // value, not as a panic.
    //
    // G16-A implementer wires this:
    //   use benten_sync::transport::{TransportStatus, TransportKind};
    //   let endpoint = Endpoint::bind_loopback().await.unwrap();
    //   simulate_packet_loss(&endpoint, 0.30);  // 30% loss; above degrade threshold
    //   let status = endpoint.transport_status();
    //   assert!(matches!(status, TransportStatus::Degraded { .. }));
    //   // Status must be observable via the engine-side atrium-status surface:
    //   let atrium_status = engine.atrium_status();
    //   assert!(atrium_status.transport_state == TransportStatus::Degraded);
    //   // The error code maps to E_ATRIUM_TRANSPORT_DEGRADED:
    //   assert_eq!(atrium_status.error_code(), Some(ErrorCode::E_ATRIUM_TRANSPORT_DEGRADED));
    //
    // OBSERVABLE consequence: a degraded transport state is
    // explicitly visible to the engine + propagates to the public
    // atrium-status surface. Defends against the silent-degradation
    // failure shape that net-blocker-2 named as BLOCKER.
    unimplemented!(
        "G16-A wires TransportDegraded explicit state + engine-side atrium_status observability"
    );
}

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — net-blocker-4 — handshake wire-format carries peer-DID AND device-DID"]
fn atrium_handshake_wire_format_carries_peer_did_and_device_did() {
    // net-blocker-4 BLOCKER pin. Per device-mesh exploration
    // brief-edits + sec-r1-6 + Inv-14 device-grain attribution: the
    // peer-handshake wire-format MUST carry BOTH the peer-DID
    // (account identity) AND the device-DID (device identity under
    // that account). Phase-3 multi-device support relies on the
    // device-DID being observable end-to-end.
    //
    // G16-A implementer wires this against the handshake serializer
    // + a wire-format test fixture:
    //   use benten_sync::handshake::HandshakeFrame;
    //   let frame = HandshakeFrame::new(peer_did, device_did, ...);
    //   let bytes = frame.to_canonical_bytes();
    //   let decoded = HandshakeFrame::from_canonical_bytes(&bytes).unwrap();
    //   assert_eq!(decoded.peer_did(), peer_did);
    //   assert_eq!(decoded.device_did(), device_did);
    //   // Both DIDs must be REQUIRED (not Optional) at the wire-format level:
    //   let frame_missing_device = HandshakeFrame::builder()
    //       .peer_did(peer_did)
    //       .build();
    //   assert!(frame_missing_device.is_err(), "device_did must be required at the handshake wire format");
    //
    // OBSERVABLE consequence: a handshake without device-DID fails
    // construction (not silently coerced to a default device-DID).
    // Defends against the multi-device attribution failure shape
    // (where Inv-14 attribution would lose device-grain).
    unimplemented!("G16-A wires HandshakeFrame wire format with required peer-DID + device-DID");
}

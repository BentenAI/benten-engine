//! R3-C RED-PHASE pins for iroh transport loopback + relay-fallback +
//! holepunch (G16-A wave-6 canary; per r2-test-landscape §2.4 G16-A +
//! plan §3 G16-A row).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A rows
//!   `iroh_transport_two_peer_loopback_round_trip` +
//!   `iroh_transport_relay_fallback_when_holepunch_fails` +
//!   `iroh_transport_holepunch_smoke`.
//! - plan §3 G16-A row.
//! - `net-minor-1` (single-process two-Endpoint loopback round-trip
//!   via in-process iroh test fixture).
//! - `D-PHASE-3-3` RESOLVED-at-R1 (iroh QUIC + holepunch +
//!   relay-default; peer-list bootstrap as opt-in fallback).
//! - `scope-real-10` (CI-conditional gating: holepunch smoke gated to
//!   a specific runner cell; loopback + relay-fallback required-on-every-PR).
//!
//! ## RED-PHASE discipline
//!
//! `#[ignore]`'d with rationale `"RED-PHASE: G16-A wave-6 canary lands iroh transport"`.

#![allow(clippy::unwrap_used)]

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — net-minor-1 — two-peer loopback round-trip"]
fn iroh_transport_two_peer_loopback_round_trip() {
    // net-minor-1 + plan §3 G16-A pin. G16-A implementer wires this:
    //
    //   use benten_sync::transport::Endpoint;
    //   let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    //   rt.block_on(async {
    //       let a = Endpoint::bind_loopback().await.unwrap();
    //       let b = Endpoint::bind_loopback().await.unwrap();
    //       let conn = a.connect(b.peer_id()).await.unwrap();
    //       conn.send_bytes(b"hello").await.unwrap();
    //       let received = b.next_message().await.unwrap();
    //       assert_eq!(received, b"hello");
    //       // Reply leg:
    //       let reply_conn = b.connect(a.peer_id()).await.unwrap();
    //       reply_conn.send_bytes(b"hello back").await.unwrap();
    //       let reply = a.next_message().await.unwrap();
    //       assert_eq!(reply, b"hello back");
    //   });
    //
    // OBSERVABLE consequence: two iroh Endpoints in a single process
    // (no relay needed; loopback) successfully round-trip bytes
    // bidirectionally. This is the canary smoke test that gates G16-A
    // landing per Q7 RESOLVED.
    unimplemented!("G16-A wires iroh two-Endpoint loopback round-trip via in-process fixture");
}

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — plan §3 G16-A — relay fallback"]
fn iroh_transport_relay_fallback_when_holepunch_fails() {
    // plan §3 G16-A pin. When holepunch fails (simulated by binding
    // both peers behind synthetic NATs), iroh falls back to the
    // relay default; the round-trip succeeds via relay.
    //
    // G16-A implementer wires this against the iroh relay test
    // fixtures (or a custom synthetic NAT harness):
    //   let relay_url = test_relay_endpoint();
    //   let a = Endpoint::builder().relay_url(relay_url.clone()).build().await.unwrap();
    //   let b = Endpoint::builder().relay_url(relay_url).build().await.unwrap();
    //   simulate_holepunch_failure(&a, &b);
    //   let conn = a.connect(b.peer_id()).await.unwrap();
    //   assert_eq!(conn.transport_kind(), TransportKind::Relay);
    //   conn.send_bytes(b"via relay").await.unwrap();
    //   assert_eq!(b.next_message().await.unwrap(), b"via relay");
    //
    // OBSERVABLE consequence: under simulated holepunch failure,
    // the relay path delivers the message; transport_kind() reports
    // Relay (not Direct).
    unimplemented!("G16-A wires iroh relay-fallback under simulated holepunch failure");
}

#[test]
#[ignore = "RED-PHASE: G16-A wave-6 — scope-real-10 — holepunch smoke (CI-conditional)"]
fn iroh_transport_holepunch_smoke() {
    // scope-real-10 pin. Holepunch smoke: two endpoints behind
    // simulated NATs negotiate a direct connection via iroh's
    // holepunch protocol (when relay infrastructure is available
    // for STUN-style coordination). Per scope-real-10, this test is
    // CI-conditional: gated to a specific runner cell (e.g.
    // ubuntu-22.04 with relay infrastructure available); not
    // blocking on macos-latest / windows-latest cells.
    //
    // G16-A implementer wires this against the iroh test
    // infrastructure + a #[cfg(test_holepunch_available)] gate (or
    // similar runner-cell-conditional discriminator).
    //
    // OBSERVABLE consequence: under runner cells with relay
    // infrastructure, holepunch succeeds and transport_kind() reports
    // Direct (not Relay). Informational at G16-A launch; promoted to
    // required after CI cell stabilization per scope-real-10.
    unimplemented!(
        "G16-A wires iroh holepunch smoke under simulated NAT scenario (CI-conditional)"
    );
}

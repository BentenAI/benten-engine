//! G16-A LANDED pins for iroh transport loopback round-trip +
//! relay-fallback (CI-conditional) + holepunch (CI-conditional)
//! per r2-test-landscape §2.4 G16-A + plan §3 G16-A row.
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-A rows
//!   `iroh_transport_two_peer_loopback_round_trip` +
//!   `iroh_transport_relay_fallback_when_holepunch_fails` +
//!   `iroh_transport_holepunch_smoke`.
//! - plan §3 G16-A row.
//! - `net-minor-1` (single-process two-Endpoint loopback round-trip
//!   via in-process iroh test fixture). Required-on-every-PR.
//! - `D-PHASE-3-3` RESOLVED-at-R1 (iroh QUIC + holepunch +
//!   relay-default; peer-list bootstrap as opt-in fallback).
//! - `scope-real-10` (CI-conditional gating: holepunch smoke gated to
//!   a specific runner cell; loopback + relay-fallback required-on-
//!   every-PR. Holepunch + relay-fallback against a real
//!   relay-fixture stay `#[ignore]`'d at G16-A landing pending iroh
//!   test-fixture stabilization on benten-engine's CI matrix; G16-D
//!   wave-6b unblocks once the handshake protocol body lands and the
//!   wave-6b mini-review verifies the iroh test-fixture wiring).
//!
//! ## CI gating shape per scope-real-10
//!
//! - `iroh_transport_two_peer_loopback_round_trip` is `#[test]` (not
//!   `#[ignore]`'d) — the load-bearing canary that gates G16-A
//!   landing per Q7 RESOLVED. Required-on-every-PR.
//! - `iroh_transport_relay_fallback_when_holepunch_fails` and
//!   `iroh_transport_holepunch_smoke` stay `#[ignore]`'d at G16-A
//!   landing because they require iroh test-fixture infrastructure
//!   (a synthetic NAT harness + a relay endpoint) that the G16-A
//!   canary's "transport core only" scope does not ship. G16-D
//!   wave-6b lands the handshake protocol body + un-ignores these
//!   pins per pim-4 §3.10 wave-paired closure pattern. Tracked in
//!   `docs/future/phase-3-backlog.md` (G16-D row).

#![allow(clippy::unwrap_used)]

use std::time::Duration;

use benten_sync::transport::{Endpoint, TransportKind, TransportStatus};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn iroh_transport_two_peer_loopback_round_trip() {
    // net-minor-1 + plan §3 G16-A canary pin: two iroh Endpoints in
    // a single process round-trip bytes via the local-network path
    // (no relay infrastructure required).
    //
    // OBSERVABLE consequence: peer_a sends bytes via `send_bytes()`,
    // peer_b's `accept_next() + recv_bytes()` returns the same bytes
    // byte-for-byte. This is the load-bearing canary that gates G16-A
    // landing per Q7 RESOLVED.

    let peer_a = Endpoint::bind_loopback().await.expect("bind a");
    let peer_b = Endpoint::bind_loopback().await.expect("bind b");

    // Resolve peer_b's full EndpointAddr (with bound socket addresses)
    // for peer_a to dial. Construction is direct from
    // bound_sockets() — no DNS / pkarr / relay watcher required.
    let peer_b_addr = peer_b.loopback_addr().expect("peer_b loopback_addr");

    // Spawn peer_b's accept loop FIRST so the connect from peer_a
    // resolves immediately. The accept task drains one inbound
    // message + signals the test via a oneshot.
    let (tx, rx) = tokio::sync::oneshot::channel();
    let accept_task = tokio::spawn(async move {
        let conn = peer_b.accept_next().await.expect("accept_next");
        let received = conn.recv_bytes().await.expect("recv_bytes");
        let _ = tx.send(received);
        // Hold the connection alive briefly so peer_a's send finishes
        // its stopped() handshake before the connection drops.
        tokio::time::sleep(Duration::from_millis(50)).await;
        conn.close();
    });

    // peer_a connects to peer_b's loopback address + sends.
    let conn_a_to_b =
        tokio::time::timeout(Duration::from_secs(15), peer_a.connect_to_addr(peer_b_addr))
            .await
            .expect("connect did not time out")
            .expect("connect a→b");

    conn_a_to_b
        .send_bytes(b"hello from a")
        .await
        .expect("send a→b");

    let received_at_b = tokio::time::timeout(Duration::from_secs(15), rx)
        .await
        .expect("recv did not time out")
        .expect("recv side ran");
    assert_eq!(received_at_b, b"hello from a");

    accept_task.await.expect("accept-task join");
    conn_a_to_b.close();
    peer_a.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn external_keypair_loopback_round_trip_uses_caller_provided_keypair() {
    // g16a-mr-minor-1 closure pin: exercises `Endpoint::bind_loopback_with_keypair`
    // + `Connection::remote_peer()` end-to-end so the public symbols
    // ship with real consumer call sites (not just zero-call-site
    // declarations). Mirrors the canonical loopback round-trip but
    // with caller-managed keypairs — production scenarios where the
    // keypair lives on benten-id's secure store, not auto-generated.
    //
    // OBSERVABLE consequence: the connection's `remote_peer()` returns
    // peer_b's PeerId derived from peer_b's externally-provided
    // keypair. If `bind_loopback_with_keypair` regressed to ignoring
    // the caller's keypair (e.g., generated a fresh one internally),
    // this test would fail because the observed remote peer-id would
    // not match the keypair's pubkey.
    let kp_a = benten_id::keypair::Keypair::generate();
    let kp_b = benten_id::keypair::Keypair::generate();
    let expected_peer_b = benten_sync::peer_id::PeerId::from_public_key(kp_b.public_key());

    let peer_a = Endpoint::bind_loopback_with_keypair(&kp_a)
        .await
        .expect("bind a with external keypair");
    let peer_b = Endpoint::bind_loopback_with_keypair(&kp_b)
        .await
        .expect("bind b with external keypair");

    let peer_b_addr = peer_b.loopback_addr().expect("peer_b loopback_addr");

    let (tx, rx) = tokio::sync::oneshot::channel();
    let accept_task = tokio::spawn(async move {
        let conn = peer_b.accept_next().await.expect("accept_next");
        let received = conn.recv_bytes().await.expect("recv_bytes");
        let _ = tx.send(received);
        tokio::time::sleep(Duration::from_millis(50)).await;
        conn.close();
    });

    let conn_a_to_b =
        tokio::time::timeout(Duration::from_secs(15), peer_a.connect_to_addr(peer_b_addr))
            .await
            .expect("connect did not time out")
            .expect("connect a→b");

    // Load-bearing assertion: remote_peer() returns peer_b's id
    // derived from kp_b. If bind_loopback_with_keypair silently
    // ignored kp_b (or remote_peer() were unwired), this would not
    // hold.
    assert_eq!(
        conn_a_to_b.remote_peer(),
        expected_peer_b,
        "Connection::remote_peer() must return peer_b's id derived from the externally-provided keypair"
    );

    conn_a_to_b
        .send_bytes(b"hello with external keypair")
        .await
        .expect("send a→b");

    let received_at_b = tokio::time::timeout(Duration::from_secs(15), rx)
        .await
        .expect("recv did not time out")
        .expect("recv side ran");
    assert_eq!(received_at_b, b"hello with external keypair");

    accept_task.await.expect("accept-task join");
    conn_a_to_b.close();
    peer_a.close().await;
}

#[tokio::test]
async fn bind_with_keypair_non_loopback_returns_endpoint_with_caller_keypair() {
    // g16a-mr-minor-1 closure pin: exercises `Endpoint::bind_with_keypair`
    // (non-loopback bind path). The endpoint is the production-shape
    // bind that wave-6b's relay-mode wires through; G16-A canary scope
    // returns a bound endpoint whose status reports a non-loopback
    // kind. If `bind_with_keypair` regressed to invoking the loopback
    // path internally, this test would fail.
    let kp = benten_id::keypair::Keypair::generate();
    let ep = Endpoint::bind_with_keypair(&kp)
        .await
        .expect("bind with external keypair (non-loopback)");
    match ep.transport_status().await {
        TransportStatus::Healthy {
            kind: TransportKind::Direct | TransportKind::Relay,
        } => {}
        other => panic!("expected non-loopback Healthy kind from bind_with_keypair, got {other:?}"),
    }
    ep.close().await;
}

#[tokio::test]
async fn iroh_transport_loopback_reports_loopback_status() {
    // net-minor-1 companion pin: the loopback canary's status surface
    // reports `TransportKind::Loopback` per net-blocker-2 observability
    // contract. G16-B/D's `engine.atrium_status()` consumes this so
    // operators can distinguish loopback (test) vs production paths.
    let peer = Endpoint::bind_loopback().await.expect("bind");
    match peer.transport_status().await {
        TransportStatus::Healthy {
            kind: TransportKind::Loopback,
        } => {}
        other => panic!("expected Healthy/Loopback, got {other:?}"),
    }
    peer.close().await;
}

#[tokio::test]
#[ignore = "G16-D wave-6b — scope-real-10 — relay-fallback gated on iroh-test-fixture availability"]
async fn iroh_transport_relay_fallback_when_holepunch_fails() {
    // scope-real-10 + plan §3 G16-A pin. When holepunch fails (simulated
    // by binding both peers behind synthetic NATs), iroh falls back
    // to the relay default; the round-trip succeeds via relay.
    //
    // G16-A canary scope ships only the loopback round-trip; the
    // relay-fallback assertion requires a relay test fixture +
    // synthetic NAT harness that lives in iroh's test-utils feature
    // (or a custom in-tree harness G16-D wave-6b ships). Per
    // scope-real-10, this test is CI-conditional pending the
    // synthetic-NAT fixture; it is NOT a load-bearing closure pin
    // for G16-A's canary landing.
    //
    // pim-4 §3.10 wave-paired closure: G16-D wave-6b un-ignores this
    // pin alongside the handshake protocol body wiring + the iroh
    // test-fixture for synthetic NAT.
    //
    // OBSERVABLE consequence (when un-ignored): under simulated
    // holepunch failure, `Connection::transport_kind()` reports
    // `Relay` (not `Direct`); bytes still round-trip successfully
    // through the relay path.
    panic!(
        "G16-D wave-6b — un-ignore once iroh test-fixture (synthetic NAT + relay endpoint) wires"
    );
}

#[tokio::test]
#[ignore = "G16-D wave-6b — scope-real-10 — holepunch smoke CI-conditional, gated on relay infrastructure"]
async fn iroh_transport_holepunch_smoke() {
    // scope-real-10 pin. Holepunch smoke: two endpoints behind
    // simulated NATs negotiate a direct connection via iroh's
    // holepunch protocol. Per scope-real-10, this test is CI-conditional
    // — gated to specific runner cells (e.g. ubuntu-22.04 with relay
    // infrastructure available); not blocking on macos-latest /
    // windows-latest cells.
    //
    // G16-D wave-6b un-ignores per pim-4 §3.10 wave-paired closure
    // alongside the handshake protocol body + iroh test-fixture wiring.
    //
    // OBSERVABLE consequence (when un-ignored, on a runner cell with
    // relay infrastructure): under simulated NAT scenario, holepunch
    // succeeds + `Connection::transport_kind()` reports `Direct` (not
    // `Relay`). Informational at G16-A launch; promoted to required
    // after CI cell stabilization per scope-real-10.
    panic!(
        "G16-D wave-6b — un-ignore once iroh test-fixture (synthetic NAT + relay endpoint) wires"
    );
}

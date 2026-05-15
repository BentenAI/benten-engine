//! Boundary test pin for the `Transport` trait abstraction.
//!
//! ## RATIFIED 2026-05-15 (Ben §15.3 #1 / umbrella #1176) — Surf-1 #889
//!
//! This pin asserts the `benten_sync::transport_trait::{Transport,
//! TransportEndpoint, TransportConnection}` traits are **implementable
//! in isolation** — i.e. a concrete impl can be authored that has NO
//! iroh dependency anywhere in its signatures. This is the
//! compile-fence against accidental iroh-leak: if a future edit
//! reintroduces an `iroh::`-concrete type into a trait signature
//! (instead of routing it through the `TransportEndpoint::Addr`
//! associated-type seam), the `MockTransport` impl below STOPS
//! COMPILING and this test fails to build.
//!
//! The mock is a fully in-memory, channel-backed transport. It uses
//! only `std` + `tokio::sync` (a dev-dep already present for
//! `#[tokio::test]`). It does NOT depend on `iroh` — that is the whole
//! point. The Surf-1 #889 finding was that iroh-concrete types leaked
//! across the crate boundary; this pin proves the trait surface itself
//! is iroh-free (the iroh dependency lives only inside the
//! `IrohTransport` impl, not the trait contract).
//!
//! ## Coupling
//!
//! - Sibling #1081 (iroh-EndpointId ↔ PeerId byte-equivalence) — the
//!   mock derives its `PeerId` from raw bytes, exercising the same
//!   byte-construction path without iroh.
//! - `transport_trait.rs` module doc — names this file as the fence.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::VecDeque;
use std::sync::Arc;

use benten_sync::errors::{AtriumTransportError, AtriumTransportResult};
use benten_sync::peer_id::PeerId;
use benten_sync::transport::{TransportKind, TransportStatus};
use benten_sync::transport_trait::{Transport, TransportConnection, TransportEndpoint};
use tokio::sync::Mutex;

/// An iroh-FREE in-memory transport. The whole module imports nothing
/// from `iroh` — if the trait signatures required an iroh-concrete
/// type, this impl would not compile.
struct MockTransport;

/// The mock's address type: a plain byte vector. This is the seam that
/// proves `TransportEndpoint::Addr` contains the iroh-`EndpointAddr`
/// leak — a non-iroh transport supplies its own `Addr` type.
#[derive(Clone)]
struct MockAddr(Vec<u8>);

struct MockEndpoint {
    peer: PeerId,
    /// Shared in-memory mailbox the two mock peers round-trip through.
    inbox: Arc<Mutex<VecDeque<Vec<u8>>>>,
    status: TransportStatus,
}

struct MockConnection {
    remote: PeerId,
    kind: TransportKind,
    inbox: Arc<Mutex<VecDeque<Vec<u8>>>>,
}

impl Transport for MockTransport {
    type Endpoint = MockEndpoint;
    type Connection = MockConnection;
}

impl TransportEndpoint for MockEndpoint {
    type Connection = MockConnection;
    type Addr = MockAddr;

    fn peer_id(&self) -> PeerId {
        self.peer
    }

    fn loopback_addr(&self) -> AtriumTransportResult<Self::Addr> {
        Ok(MockAddr(self.peer.as_bytes().to_vec()))
    }

    async fn transport_status(&self) -> TransportStatus {
        self.status.clone()
    }

    async fn connect(&self, remote: PeerId) -> AtriumTransportResult<Self::Connection> {
        Ok(MockConnection {
            remote,
            kind: TransportKind::Loopback,
            inbox: Arc::clone(&self.inbox),
        })
    }

    async fn connect_to_addr(
        &self,
        remote_addr: Self::Addr,
    ) -> AtriumTransportResult<Self::Connection> {
        if remote_addr.0.len() != 32 {
            return Err(AtriumTransportError::TransportDegraded {
                reason: "mock addr must be a 32-byte peer id".into(),
            });
        }
        let mut b = [0u8; 32];
        b.copy_from_slice(&remote_addr.0);
        Ok(MockConnection {
            remote: PeerId::from_bytes(b),
            kind: TransportKind::Loopback,
            inbox: Arc::clone(&self.inbox),
        })
    }

    async fn accept_next(&self) -> AtriumTransportResult<Self::Connection> {
        Ok(MockConnection {
            remote: self.peer,
            kind: TransportKind::Loopback,
            inbox: Arc::clone(&self.inbox),
        })
    }

    async fn close(self) {}
}

impl TransportConnection for MockConnection {
    fn transport_kind(&self) -> TransportKind {
        self.kind
    }

    fn remote_peer(&self) -> PeerId {
        self.remote
    }

    async fn send_bytes(&self, payload: &[u8]) -> AtriumTransportResult<()> {
        self.inbox.lock().await.push_back(payload.to_vec());
        Ok(())
    }

    async fn recv_bytes(&self) -> AtriumTransportResult<Vec<u8>> {
        self.inbox
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| AtriumTransportError::TransportDegraded {
                reason: "mock inbox empty".into(),
            })
    }

    fn close(self) {}
}

/// Generic helper: drives a bytes round-trip over ANY `T: Transport`
/// whose connection is `TransportConnection`. Proves the sync runtime
/// CAN be written generic over the trait (the post-v1 migration path
/// per CLAUDE.md #19) without touching iroh.
async fn round_trip<C: TransportConnection>(conn: &C, payload: &[u8]) -> Vec<u8> {
    conn.send_bytes(payload).await.expect("send");
    conn.recv_bytes().await.expect("recv")
}

#[tokio::test]
async fn mock_transport_implements_boundary_in_isolation_without_iroh() {
    // The whole point: this impl exists + compiles with zero iroh
    // imports. Reaching this line means the trait surface is iroh-free.
    let peer = PeerId::from_bytes([7u8; 32]);
    let inbox = Arc::new(Mutex::new(VecDeque::new()));
    let ep = MockEndpoint {
        peer,
        inbox: Arc::clone(&inbox),
        status: TransportStatus::Healthy {
            kind: TransportKind::Loopback,
        },
    };

    // Exercise the abstracted endpoint surface.
    assert_eq!(TransportEndpoint::peer_id(&ep), peer);
    let addr = ep.loopback_addr().expect("mock loopback addr");
    assert!(matches!(
        ep.transport_status().await,
        TransportStatus::Healthy {
            kind: TransportKind::Loopback
        }
    ));

    // Connect via both the PeerId path and the (non-iroh) Addr path.
    let conn_by_peer = ep.connect(peer).await.expect("connect by peer");
    assert_eq!(conn_by_peer.remote_peer(), peer);
    assert_eq!(conn_by_peer.transport_kind(), TransportKind::Loopback);

    let conn_by_addr = ep.connect_to_addr(addr).await.expect("connect by addr");
    assert_eq!(conn_by_addr.remote_peer(), peer);

    // Round-trip bytes through the generic helper — proves the runtime
    // can be written `<C: TransportConnection>` (post-v1 migration
    // shape) without referencing iroh.
    let echoed = round_trip(&conn_by_peer, b"hello-boundary").await;
    assert_eq!(echoed, b"hello-boundary");

    conn_by_peer.close();
    conn_by_addr.close();
    ep.close().await;
}

/// Compile-time assertion that `MockTransport` satisfies the
/// `Transport` bound — the same bound the post-v1 generic sync runtime
/// will require. If the trait gained an iroh-concrete super-bound or
/// associated-type default, this would fail to compile.
#[test]
fn mock_satisfies_transport_bound() {
    fn assert_transport<T: Transport>() {}
    assert_transport::<MockTransport>();
}

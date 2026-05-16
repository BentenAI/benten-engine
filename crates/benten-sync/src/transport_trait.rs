//! Transport abstraction boundary — `trait Transport` + companions.
//!
//! ## RATIFIED 2026-05-15 (Ben §15.3 #1 / umbrella #1176)
//!
//! `benten-sync` ships an iroh-concrete transport in [`crate::transport`]
//! ([`crate::transport::Endpoint`] / [`crate::transport::Connection`]).
//! Until this module, there was NO trait abstraction over the
//! connection layer. CLAUDE.md baked-in **#19** explicitly contemplates
//! engine-level extensions for **alternate transports** (post-iroh —
//! Tor / Nostr-relay / shaped relay). This module introduces the
//! abstraction boundary that lets a future compile-time engine
//! extension swap the transport implementation WITHOUT a SemVer-major
//! break propagating through `benten-engine` + `bindings/napi`.
//!
//! Mirrors the `Renderer`-trait swappability pattern named in CLAUDE.md
//! baked-in **#17**: a trait at the boundary, multiple concrete impls
//! shipping as compile-time engine extensions per #19. The first (and,
//! pre-v1, only) concrete impl is [`IrohTransport`], which delegates to
//! the existing iroh-backed [`crate::transport::Endpoint`] /
//! [`crate::transport::Connection`] with **zero behavioral change**.
//!
//! ## Why this is additive (no behavioral regression)
//!
//! Per INTERNALS.md §7's pre-existing mitigation note: a `Transport`
//! trait can be introduced WITHOUT changing the engine-facing API
//! because the engine (`engine_sync.rs`) only ever calls
//! [`crate::transport::Endpoint`] / [`crate::transport::Connection`]
//! newtype methods — never iroh types through a generic. This module
//! therefore adds the trait surface + an `IrohTransport` impl that
//! forwards to the existing concrete methods. No existing call path
//! changes; the loopback canary + every sync-runtime test stays green.
//!
//! Post-v1 alternate-transport work (the CLAUDE.md #19 use cases) is the
//! consumer of this boundary: a `TorTransport` / `NostrRelayTransport`
//! / `ShapedRelayTransport` implements these three traits + the engine
//! is migrated to be generic over `T: Transport` at that point. That
//! migration is out of scope for the pre-v1 fix-pass (engine-facing API
//! is intentionally left unchanged here to guarantee no regression);
//! the boundary existing pre-v1 is the load-bearing deliverable so the
//! v1 surface contract names the abstraction rather than the concrete.
//!
//! ## The abstracted surface
//!
//! The four-to-five methods iroh actually exercises across the sync
//! runtime, abstracted behind associated types so no iroh-concrete type
//! (`iroh::EndpointAddr` / `iroh::Endpoint` / `iroh::endpoint::Connection`)
//! appears in the trait signature:
//!
//! - [`Transport::Endpoint`] — bound local endpoint handle.
//! - [`Transport::Connection`] — an established peer connection.
//! - [`TransportEndpoint::Addr`] — the transport's address type
//!   (iroh's is `iroh::EndpointAddr`; a relay transport's would be a
//!   relay-URL + topic). **This associated type is the seam that
//!   contains the iroh-`EndpointAddr` leak** flagged by Surf-1 #889.
//! - `connect` / `connect_to_addr` / `accept_next` / `loopback_addr`
//!   / `transport_status` / `peer_id` / `close` on the endpoint.
//! - `send_bytes` / `recv_bytes` / `transport_kind` / `remote_peer` /
//!   `close` on the connection.
//!
//! ## Boundary test pin
//!
//! `crates/benten-sync/tests/transport_trait_boundary.rs` implements
//! all three traits with an in-memory channel-backed mock that has NO
//! iroh dependency. It compile-fences against accidental iroh-leak: if
//! a future edit reintroduces an `iroh::`-concrete type into a trait
//! signature, the mock impl stops compiling.

use crate::errors::AtriumTransportResult;
use crate::peer_id::PeerId;
use crate::transport::{Connection, Endpoint, TransportKind, TransportStatus};

/// The transport abstraction boundary.
///
/// A `Transport` is a factory + namespace for a concrete connection
/// layer. It names the [`TransportEndpoint`] and [`TransportConnection`]
/// associated types the rest of the sync runtime is (eventually)
/// generic over. The pre-v1 concrete impl is [`IrohTransport`]; post-v1
/// alternate transports (Tor / Nostr-relay / shaped relay per CLAUDE.md
/// baked-in #19) implement this trait as compile-time engine
/// extensions.
///
/// `Send + Sync` because the sync runtime drives the transport from
/// tokio tasks; the engine holds the endpoint inside an `Arc`-shared
/// `AtriumHandle`.
pub trait Transport: Send + Sync + 'static {
    /// The bound local-endpoint handle for this transport.
    type Endpoint: TransportEndpoint<Connection = Self::Connection>;

    /// An established connection to a remote peer for this transport.
    type Connection: TransportConnection;
}

/// A bound local endpoint — the per-peer-process handle that connects
/// to + accepts connections from other peers.
///
/// Abstracts [`crate::transport::Endpoint`]'s engine-exercised method
/// surface. The [`TransportEndpoint::Addr`] associated type is the seam
/// that contains the iroh-`EndpointAddr` leak (Surf-1 #889): the trait
/// signature names `Self::Addr`, never `iroh::EndpointAddr`.
#[allow(
    async_fn_in_trait,
    reason = "\
    Pre-v1 boundary introduction (RATIFIED §15.3 #1 / umbrella #1176). \
    The trait is consumed in-crate by `IrohTransport` + the boundary \
    test mock; no `dyn TransportEndpoint` object is constructed (the \
    engine-facing API is intentionally left on the concrete newtypes \
    per the no-behavioral-regression constraint), so RPITIT \
    auto-trait-leakage across a `dyn` boundary is not a concern here. \
    Post-v1 generic-over-`T: Transport` migration (CLAUDE.md #19 \
    alternate-transport work) can add `+ Send` bounds at that point."
)]
pub trait TransportEndpoint: Send + Sync {
    /// The established-connection type this endpoint produces.
    type Connection: TransportConnection;

    /// This transport's address type.
    ///
    /// iroh's is `iroh::EndpointAddr`; a relay transport's would be a
    /// relay-URL + topic tuple. **This is the abstraction seam that
    /// contains the iroh-`EndpointAddr` leak** flagged by Surf-1 #889 —
    /// the trait names `Self::Addr`, never the concrete iroh type.
    type Addr: Send + 'static;

    /// This endpoint's stable peer identity (Ed25519 pubkey == iroh
    /// EndpointId per crypto-minor-4).
    fn peer_id(&self) -> PeerId;

    /// The local endpoint address for in-process two-peer test
    /// fixtures (loopback canary). Production peers discover each other
    /// via the transport's native discovery path.
    ///
    /// # Errors
    ///
    /// Transport-specific failure (e.g. iroh: no bound sockets).
    fn loopback_addr(&self) -> AtriumTransportResult<Self::Addr>;

    /// Current observable transport status per net-blocker-2.
    async fn transport_status(&self) -> TransportStatus;

    /// Connect to a remote peer by [`PeerId`] (native discovery path).
    ///
    /// # Errors
    ///
    /// Transport-specific connection failure.
    async fn connect(&self, remote: PeerId) -> AtriumTransportResult<Self::Connection>;

    /// Connect to a remote peer by full transport address.
    ///
    /// # Errors
    ///
    /// Transport-specific connection failure.
    async fn connect_to_addr(
        &self,
        remote_addr: Self::Addr,
    ) -> AtriumTransportResult<Self::Connection>;

    /// Accept the next inbound connection.
    ///
    /// # Errors
    ///
    /// Transport-specific accept-loop failure.
    async fn accept_next(&self) -> AtriumTransportResult<Self::Connection>;

    /// Close the endpoint and tear down all connections.
    async fn close(self)
    where
        Self: Sized;
}

/// An established connection between two peers.
///
/// Abstracts [`crate::transport::Connection`]'s engine-exercised method
/// surface (the minimum-viable bytes round-trip + observability
/// accessors). No iroh-concrete type appears in any signature.
#[allow(
    async_fn_in_trait,
    reason = "\
    Pre-v1 boundary introduction (RATIFIED §15.3 #1 / umbrella #1176); \
    same rationale as TransportEndpoint — consumed in-crate only, no \
    `dyn` object constructed."
)]
pub trait TransportConnection: Send + Sync {
    /// Path-discriminator (Direct / Relay / Loopback) at
    /// connection-establishment time, for net-blocker-2 observability.
    fn transport_kind(&self) -> TransportKind;

    /// Remote peer identity.
    fn remote_peer(&self) -> PeerId;

    /// Send a bytes payload to the remote peer.
    ///
    /// # Errors
    ///
    /// Transport-specific stream open/write failure.
    async fn send_bytes(&self, payload: &[u8]) -> AtriumTransportResult<()>;

    /// Receive a bytes payload from the remote peer.
    ///
    /// # Errors
    ///
    /// Transport-specific stream accept/read failure.
    async fn recv_bytes(&self) -> AtriumTransportResult<Vec<u8>>;

    /// Close the connection.
    fn close(self)
    where
        Self: Sized;
}

/// The pre-v1 concrete [`Transport`] — iroh-backed QUIC.
///
/// This is a zero-sized marker type: it names the iroh-backed
/// [`Endpoint`] / [`Connection`] as the [`Transport`] associated types.
/// All behavior lives in the existing [`crate::transport`] module; the
/// trait impls below forward to the existing concrete methods with **no
/// behavioral change** (verified by the unchanged sync-runtime test
/// suite + the loopback canary staying green).
///
/// Post-v1, a `TorTransport` / `NostrRelayTransport` /
/// `ShapedRelayTransport` would sit alongside this as additional
/// compile-time engine extensions per CLAUDE.md baked-in #19.
#[derive(Debug, Clone, Copy, Default)]
pub struct IrohTransport;

/// The pre-v1 concrete transport address type, surfaced through the
/// [`TransportEndpoint::Addr`] seam (Surf-1 #889 / residual #1232).
///
/// `benten-engine`'s public sync API (`AtriumHandle::loopback_addr` /
/// `AtriumHandle::sync_subgraph`) names THIS alias instead of
/// `iroh::EndpointAddr` directly, so the engine's public surface no
/// longer leaks an `iroh::`-concrete type two crates outward. The
/// alias is *defined as* `<Endpoint as TransportEndpoint>::Addr` — it
/// is the same type the abstraction boundary already names via
/// `Self::Addr`, just given a transport-neutral public name at the
/// crate root. The full `<T: Transport>` engine-generic migration
/// (which would make this an associated type the engine is generic
/// over) remains the genuinely-post-v1 CLAUDE.md #19 alternate-
/// transport work; this alias is the bounded pre-v1 closure that
/// stops the leak without that migration.
pub type TransportAddr = <Endpoint as TransportEndpoint>::Addr;

impl Transport for IrohTransport {
    type Endpoint = Endpoint;
    type Connection = Connection;
}

impl TransportEndpoint for Endpoint {
    type Connection = Connection;
    type Addr = iroh::EndpointAddr;

    fn peer_id(&self) -> PeerId {
        Endpoint::peer_id(self)
    }

    fn loopback_addr(&self) -> AtriumTransportResult<Self::Addr> {
        Endpoint::loopback_addr(self)
    }

    async fn transport_status(&self) -> TransportStatus {
        Endpoint::transport_status(self).await
    }

    async fn connect(&self, remote: PeerId) -> AtriumTransportResult<Self::Connection> {
        Endpoint::connect(self, remote).await
    }

    async fn connect_to_addr(
        &self,
        remote_addr: Self::Addr,
    ) -> AtriumTransportResult<Self::Connection> {
        Endpoint::connect_to_addr(self, remote_addr).await
    }

    async fn accept_next(&self) -> AtriumTransportResult<Self::Connection> {
        Endpoint::accept_next(self).await
    }

    async fn close(self) {
        Endpoint::close(self).await;
    }
}

impl TransportConnection for Connection {
    fn transport_kind(&self) -> TransportKind {
        Connection::transport_kind(self)
    }

    fn remote_peer(&self) -> PeerId {
        Connection::remote_peer(self)
    }

    async fn send_bytes(&self, payload: &[u8]) -> AtriumTransportResult<()> {
        Connection::send_bytes(self, payload).await
    }

    async fn recv_bytes(&self) -> AtriumTransportResult<Vec<u8>> {
        Connection::recv_bytes(self).await
    }

    fn close(self) {
        Connection::close(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `IrohTransport` names the iroh-backed concrete types as its
    /// `Transport` associated types. This is a compile-time assertion
    /// that the impl is wired; the substantive behavioral coverage is
    /// the existing loopback canary + sync-runtime suite (unchanged by
    /// this additive boundary).
    #[test]
    fn iroh_transport_names_concrete_associated_types() {
        fn assert_transport<T: Transport>() {}
        assert_transport::<IrohTransport>();
        // `IrohTransport` is the zero-sized marker. Assert the `Copy`
        // + `Default` derives are wired (a future edit dropping them
        // would break the post-v1 generic-construction ergonomics).
        fn assert_copy_default<T: Copy + Default>() {}
        assert_copy_default::<IrohTransport>();
    }
}

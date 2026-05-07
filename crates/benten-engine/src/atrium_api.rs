//! Atrium public API surface (engine-side wrapper).
//!
//! Phase-3 G16-B wave-6b. Native-only — gated to non-wasm32 targets
//! per CLAUDE.md baked-in #17 (browser tabs participate via
//! authenticated thin-client views, NOT as full Atrium peers).
//!
//! ## Surface shape (per arch-r1-7 + Ben's D1 session-handle B-prime)
//!
//! [`AtriumConfig`] is the configuration carrier consumed at
//! [`crate::Engine::open_atrium`] construction time. [`AtriumHandle`] is the
//! Atrium session handle returned to callers; it carries the iroh
//! transport + the per-zone Loro docs + the merge dispatch surface.
//!
//! Per Ben's D1 (2026-05-05), the call shape is:
//!
//! ```ignore
//! let atrium = engine.open_atrium(AtriumConfig::for_test()).await?;
//! atrium.sync_subgraph("/zone/posts", remote_peer).await?;
//! ```
//!
//! The session handle holds the iroh `Endpoint` + per-zone CRDT
//! state; dropping it tears down the Atrium connection cleanly.
//!
//! ## What this module ships (G16-B canary scope)
//!
//! - [`AtriumConfig`] — configuration type.
//! - [`AtriumHandle`] — session handle re-export from
//!   [`crate::engine_sync`].
//! - [`SyncStatus`] — observable state surface for
//!   `engine.atrium_status()`.
//!
//! ## What this module does NOT ship at G16-B
//!
//! - Full handshake protocol body (G16-D wave-6b).
//! - MST diff sync driver (G16-C wave-6b).
//! - UCAN-grant exchange + revocation propagation (G16-D wave-6b +
//!   G14-D wave-5a integration).
//! - Light-client verification (G16-C wave-6b).
//!
//! These surfaces are pinned in `crates/benten-sync/tests/` with
//! BELONGS-NAMED-NOW per HARD RULE rule-12 dispositions to their
//! wave-6b implementer destinations.
//!
//! ## Pin sources
//!
//! - plan §3 G16-B row.
//! - r2-test-landscape §2.4 G16-B rows
//!   `atrium_open_close_lifecycle` +
//!   `atrium_sync_subgraph_two_peer_bidirectional`.
//! - arch-r1-7 (Atrium API surface ~100-200 LOC).
//! - D1 (Ben's 2026-05-05 ratification: session-handle B-prime shape).

use benten_sync::transport::TransportKind;

pub use crate::engine_sync::{AtriumError, AtriumHandle};

/// Configuration for [`crate::Engine::open_atrium`].
///
/// G16-B canary scope: minimum-viable carrier. [`AtriumConfig::for_test`]
/// constructs the loopback-mode config used by integration tests;
/// production wires arrive at G16-D wave-6b alongside the handshake
/// protocol body.
#[derive(Clone, Debug)]
pub struct AtriumConfig {
    /// The transport-binding mode. `Loopback` for in-process integration
    /// tests; `Production` for peer-to-peer connections via iroh's
    /// relay-default + holepunch path per D-PHASE-3-3.
    pub mode: AtriumMode,
}

impl AtriumConfig {
    /// Construct a config suitable for in-process integration tests.
    ///
    /// Binds the iroh `Endpoint` in loopback-mode (no relay
    /// infrastructure) so two-peer round-trips work in CI without
    /// network access.
    #[must_use]
    pub fn for_test() -> Self {
        Self {
            mode: AtriumMode::Loopback,
        }
    }

    /// Construct a config suitable for production peer-to-peer
    /// connections. G16-B canary scope: the iroh production preset
    /// plumbing wires at G16-D wave-6b alongside the handshake
    /// protocol body — until then [`AtriumMode::Production`] falls
    /// back to Loopback on construction.
    #[must_use]
    pub fn production() -> Self {
        Self {
            mode: AtriumMode::Production,
        }
    }
}

impl Default for AtriumConfig {
    fn default() -> Self {
        Self::for_test()
    }
}

/// The Atrium-binding mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AtriumMode {
    /// Loopback — for in-process integration tests + the load-bearing
    /// `atrium_sync_subgraph_two_peer_bidirectional` exit-criterion-1
    /// pin.
    Loopback,
    /// Production — relay-default + holepunch via iroh per
    /// D-PHASE-3-3. G16-B canary scope: falls back to Loopback until
    /// G16-D wave-6b lands the production preset binding.
    Production,
}

/// Observable Atrium status surface for `engine.atrium_status()`.
///
/// Per net-blocker-2 BLOCKER (typed errors + observability), every
/// Atrium connection's transport state surfaces through this type.
/// Operators consume the [`SyncStatus::transport_kind`] +
/// [`SyncStatus::is_healthy`] discriminators to route observability
/// alerts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncStatus {
    /// The active transport-path kind (Direct / Relay / Loopback).
    pub transport_kind: TransportKind,
    /// Whether the connection is healthy (per the underlying
    /// `transport::TransportStatus` surface).
    pub is_healthy: bool,
    /// Operator-readable reason if degraded; empty if healthy.
    pub reason: String,
}

impl SyncStatus {
    /// Construct a healthy status carrying the named transport-kind.
    #[must_use]
    pub fn healthy(kind: TransportKind) -> Self {
        Self {
            transport_kind: kind,
            is_healthy: true,
            reason: String::new(),
        }
    }

    /// Construct a degraded status carrying the operator-readable
    /// reason.
    #[must_use]
    pub fn degraded(reason: impl Into<String>) -> Self {
        Self {
            transport_kind: TransportKind::Loopback,
            is_healthy: false,
            reason: reason.into(),
        }
    }
}

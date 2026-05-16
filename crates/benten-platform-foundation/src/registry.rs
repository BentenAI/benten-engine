//! Phase-4-Foundation G24-D — decentralized registry reserved shapes.
//!
//! Per post-R1-triage ratification #3: **decentralized registry →
//! Phase 4-Meta**. Phase 4-Foundation v0 uses direct content-addressed-
//! share over Atriums (out-of-band handshake; user pulls from peer
//! they trust). This module declares the *concrete data shapes*
//! ([`RegistryEntry`], [`DiscoveryQuery`], [`DiscoveryResult`]) +
//! reserves the `E_REGISTRY_DISCOVERY_TIMEOUT` ErrorCode via
//! [`timeout_error_code`] so Phase 4-Meta's Atrium-substrate wiring has
//! stable anchors.
//!
//! **No `Registry` trait at v1 (Fwd-2 #1014 RATIFIED Path A,
//! 2026-05-15).** The earlier paper-only `trait Registry` was a
//! forward-architecture abstraction with zero v1 consumers; it locked
//! a publish-of-body / closed-enum / single-error-code shape pre-Atrium-
//! substrate that would conflict with the Phase-8 decentralized-
//! discovery trajectory named in `docs/VISION.md` (CID-keyed announce,
//! trust-graph-keyed discovery, signed publish receipts). Per Path A
//! ("match docs to code"): the trait is retracted and NOT part of the
//! v1 public API surface. Phase 4-Meta / Phase-5+ introduces a trait
//! *if and when* a real second impl materializes (e.g. an Atrium-
//! substrate registry whose shape genuinely differs from an in-memory
//! one) — tracked at `docs/future/phase-4-backlog.md §3.1`.
//!
//! At G24-D the module has zero production call sites — only test
//! pins enumerate the reserved surface (per
//! `tests/registry_phase_4_meta_reserved_no_production_callsites.rs`).

use crate::plugin_manifest::PluginManifest;
use benten_core::Cid;
use benten_errors::ErrorCode;
use benten_id::did::Did;

/// Registry entry — what gets published to the decentralized registry.
///
/// **Phase 4-Foundation: this type exists but is NOT wired to any
/// production publish surface.** Phase 4-Meta fills the substrate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    /// CID of the published manifest.
    pub manifest_cid: Cid,
    /// Peer-DID of the publisher (typically equals
    /// `manifest.peer_did`; sync intermediaries may differ).
    pub publisher_did: Did,
    /// Manifest body (cached on the registry; receiver still verifies
    /// peer-DID signature + content-CID match independently).
    pub manifest: PluginManifest,
}

/// Discovery query — what kind of plugins to surface.
#[derive(Debug, Clone)]
pub enum DiscoveryQuery {
    /// All published plugins by a specific peer-DID.
    ByAuthor(Did),
    /// Plugins by name (exact match).
    ByName(String),
    /// Plugins that accept a specific content-CID (e.g., a schema CID).
    AcceptingContent(Cid),
}

/// Discovery result.
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Matching registry entries.
    pub matches: Vec<RegistryEntry>,
}

// NOTE (Fwd-2 #1014 RATIFIED Path A, 2026-05-15): there is intentionally
// NO `trait Registry` here. The paper-only forward-architecture trait
// was retracted to keep the v1 public API surface honest — the
// concrete publish/discover wiring (and the decision of whether a trait
// abstraction is warranted, given Phase-8's CID-keyed-announce shape)
// lands at Phase 4-Meta. See `docs/future/phase-4-backlog.md §3.1`.

/// Reserved registry-discovery-timeout error helper.
///
/// **Phase 4-Foundation reserved-but-not-emitted.** The variant is
/// minted in `benten-errors` at G24-D so future Phase 4-Meta call
/// sites have a stable code; no fires here.
#[must_use]
pub fn timeout_error_code() -> ErrorCode {
    ErrorCode::RegistryDiscoveryTimeout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_error_code_returns_reserved_variant() {
        assert_eq!(
            timeout_error_code().as_static_str(),
            "E_REGISTRY_DISCOVERY_TIMEOUT"
        );
    }
}

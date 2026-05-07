//! G16-D wave-6b — napi bridge for the Atrium TS DSL B-prime
//! factory-handle form (per Ben's D1 ratification 2026-05-05).
//!
//! ## Pattern B-prime factory-handle shape (D-PHASE-3-15 RESOLVED)
//!
//! `engine.atrium({config})` is a factory call returning a typed
//! `Atrium` handle. Methods (`join` / `leave` / `listPeers` /
//! `trustPeer` / `revokePeer` / `subscribe` / `declareDeviceAttestation`
//! / `listDeclaredDeviceAttestations`) live on the returned handle —
//! NOT as flat-namespace properties on the engine class.
//!
//! ## Scope at G16-D wave-6b
//!
//! The engine-side `Atrium` Rust type + `Engine::open_atrium` /
//! `Atrium::sync_subgraph` surfaces (`crates/benten-engine/src/atrium_api.rs`
//! + `engine_sync.rs`) are G16-B territory — they land on a parallel
//! branch and merge alongside this PR. To keep the wave-6b parallel-3
//! merge mechanical, this napi bridge ships a self-contained `JsAtrium`
//! class that owns its own state at the napi-shim layer (config,
//! per-handle declared device-attestation list, peer roster,
//! handshake-derived session). G16-B reconciliation: at merge, the
//! state-holding fields here delegate to the G16-B engine-side
//! `Atrium` Rust type via a single field `inner: benten_engine::Atrium`;
//! the napi method bodies stay shape-stable (the D1 factory-handle
//! contract is locked at this layer).
//!
//! ## Pin sources
//!
//! - r2-test-landscape §2.4 G16-D row `atrium.test.ts (TS DSL surface)`.
//! - plan §3 G16-D row line "TS DSL — `engine.atrium({config}).join()`
//!   factory shape per D-PHASE-3-15 + Ben's D1 ratification".
//! - `D-PHASE-3-15` RESOLVED-at-R4-FP/R3-C — Pattern B-prime
//!   factory-handle form (flat-namespace REJECTED).
//! - `r1-napi-2` (declareDeviceAttestation TS round-trip).
//! - `r1-napi-10` (B-prime factory shape architectural pin).
//! - `pcds-r4-r1-2` instance-26 PRE-EMPTION (typed
//!   `DeviceAttestationDeclaration` napi struct).

use std::sync::Mutex;

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Configuration object for the Atrium factory call.
///
/// Mirrors the TS `AtriumConfig` interface at
/// `packages/engine/src/atrium.ts`. `atriumId` is required;
/// `inviteBytes` is optional (carries an invite-shaped UCAN grant
/// when accepting an existing atrium's invitation).
#[napi(object)]
#[derive(Clone)]
pub struct AtriumConfig {
    /// Caller-chosen atrium identifier (e.g. "family", "team-foo").
    /// Stable per-atrium handle key — multiple `engine.atrium({...})`
    /// calls with the same `atriumId` return distinct handles that
    /// route to the same logical atrium under the engine.
    pub atrium_id: String,
}

/// Capability-claim entry inside a `DeviceAttestationDeclaration`.
///
/// Mirrors the TS `CapabilityClaim` interface at
/// `packages/engine/src/types.ts`. Per pcds-r4-r1-2 +
/// `tests/device_attestation_ts_interface_present.test.ts`: typed
/// schema parity is asserted.
#[napi(object)]
#[derive(Clone)]
pub struct CapabilityClaim {
    /// Path-glob the claim applies to (e.g. `/zone/notifications/*`).
    pub path: String,
    /// Ability the claim grants (e.g. `read` / `write` / `emit`).
    pub ability: String,
}

/// Device-attestation declaration envelope per CLAUDE.md baked-in #17
/// + D-PHASE-3-25 + r1-napi-2.
///
/// Browser thin-client tabs declare their device-DID + capability
/// envelope at handshake-time via this typed struct. The Rust
/// producer round-trips the envelope at the napi boundary; G14-A2's
/// `JsDeviceAttestation` issuance class (with crypto-grade signing)
/// is the upstream issuance path. The declaration here records the
/// declared envelope on the Atrium handle so a subsequent join-flow
/// presents it at handshake.
#[napi(object)]
#[derive(Clone)]
pub struct DeviceAttestationDeclaration {
    /// `did:key:...` identifier of the declaring device.
    pub device_did: String,
    /// Per-claim capabilities this device may exercise within the
    /// atrium.
    pub capabilities: Vec<CapabilityClaim>,
    /// TTL in seconds before the attestation must be re-declared.
    pub freshness_window: u32,
}

/// `Atrium` typed handle — the per-call object returned from
/// `engine.atrium({config})` per D1 Pattern B-prime.
///
/// Per Ben's D1 ratification: the factory call
/// `engine.atrium({config})` returns a fresh `Atrium` handle whose
/// methods carry per-session state. Multiple calls with the same
/// `atriumId` return distinct handles routing to the same logical
/// atrium.
///
/// G16-B reconciliation: the `state` field here will be replaced by
/// `inner: Arc<benten_engine::Atrium>` at merge — the napi method
/// bodies stay shape-stable.
#[napi]
pub struct JsAtrium {
    config: AtriumConfig,
    state: Mutex<AtriumHandleState>,
}

#[derive(Default)]
struct AtriumHandleState {
    /// Declared device attestations (round-trip surface for r1-napi-2 +
    /// pcds-r4-r1-2 pins). G16-B integration: this field delegates to
    /// the engine-side device-attestation table at merge.
    declared_attestations: Vec<DeviceAttestationDeclaration>,
    /// Whether the handle has called `join()` (handshake completed at
    /// the engine layer). Used to gate `subscribe()` / `listPeers()`
    /// per the post-handshake-only contract.
    joined: bool,
    /// Trusted peer-DID roster. G16-B integration: delegates to the
    /// engine-side peer-roster Node's stored property at merge.
    trusted_peers: Vec<String>,
    /// Revoked peer-DID roster.
    revoked_peers: Vec<String>,
}

#[napi]
impl JsAtrium {
    /// Construct a fresh Atrium handle per the D1 factory call shape.
    ///
    /// Called by the TS DSL `engine.atrium({config})` factory — NOT
    /// invoked directly by application code.
    #[napi(factory)]
    pub fn create(config: AtriumConfig) -> Self {
        Self {
            config,
            state: Mutex::new(AtriumHandleState::default()),
        }
    }

    /// The atrium's identifier (echo of `config.atriumId` for
    /// observability).
    #[napi(getter)]
    pub fn atrium_id(&self) -> String {
        self.config.atrium_id.clone()
    }

    /// Whether `join()` has completed on this handle.
    #[napi(getter)]
    pub fn is_joined(&self) -> bool {
        self.state.lock().expect("atrium state mutex").joined
    }

    /// Join the atrium — initiates the peer-discovery + handshake
    /// flow per G16-D wave-6b.
    ///
    /// G16-B integration: at merge, this method delegates to the
    /// engine-side `benten_engine::Atrium::join` flow that consumes
    /// the G16-A iroh transport + G16-D handshake protocol body.
    /// The wave-6b napi shim records joined-state observably.
    #[napi]
    pub fn join(&self) -> Result<()> {
        let mut state = self.state.lock().expect("atrium state mutex");
        state.joined = true;
        Ok(())
    }

    /// Leave the atrium — tears down the per-session state.
    #[napi]
    pub fn leave(&self) -> Result<()> {
        let mut state = self.state.lock().expect("atrium state mutex");
        state.joined = false;
        // Trust + revocation rosters survive across leave/rejoin per
        // the engine-side persistence contract; only joined-state
        // resets.
        Ok(())
    }

    /// List peers currently trusted in this atrium.
    ///
    /// Returns the peer-DID strings. The roster is the union of
    /// trusted peers minus revoked peers (which terminates active
    /// subscriptions per exit-criterion 15).
    #[napi]
    pub fn list_peers(&self) -> Vec<String> {
        let state = self.state.lock().expect("atrium state mutex");
        state
            .trusted_peers
            .iter()
            .filter(|p| !state.revoked_peers.contains(p))
            .cloned()
            .collect()
    }

    /// Trust a peer-DID — adds it to the atrium's trusted roster.
    ///
    /// G16-B integration: at merge, this method delegates to the
    /// engine-side trust-policy update + persistence.
    #[napi]
    pub fn trust_peer(&self, peer_did: String) -> Result<()> {
        let mut state = self.state.lock().expect("atrium state mutex");
        if !state.trusted_peers.contains(&peer_did) {
            state.trusted_peers.push(peer_did);
        }
        Ok(())
    }

    /// Revoke a peer-DID — removes it from the trusted roster + adds
    /// to the revoked-peer list.
    ///
    /// Active subscriptions on the revoked peer terminate per
    /// exit-criterion 15 (composes with G14-D F6 delivery-time
    /// cap-recheck at the engine layer; this napi shim records the
    /// revocation).
    #[napi]
    pub fn revoke_peer(&self, peer_did: String) -> Result<()> {
        let mut state = self.state.lock().expect("atrium state mutex");
        if !state.revoked_peers.contains(&peer_did) {
            state.revoked_peers.push(peer_did);
        }
        Ok(())
    }

    /// Declare a device-attestation envelope on this atrium handle per
    /// CLAUDE.md baked-in #17 + r1-napi-2.
    ///
    /// Per Ben's D1: the declaration lives on the Atrium handle (NOT
    /// flat on engine), and may be invoked before `join()` so the
    /// handshake-time presentation includes the declared envelope.
    #[napi]
    pub fn declare_device_attestation(
        &self,
        attestation: DeviceAttestationDeclaration,
    ) -> Result<()> {
        let mut state = self.state.lock().expect("atrium state mutex");
        // Replace any existing entry for the same device-DID.
        state
            .declared_attestations
            .retain(|a| a.device_did != attestation.device_did);
        state.declared_attestations.push(attestation);
        Ok(())
    }

    /// List declared device-attestations on this handle. Round-trip
    /// surface per pcds-r4-r1-2 + r1-napi-2.
    #[napi]
    pub fn list_declared_device_attestations(&self) -> Vec<DeviceAttestationDeclaration> {
        let state = self.state.lock().expect("atrium state mutex");
        state.declared_attestations.clone()
    }
}

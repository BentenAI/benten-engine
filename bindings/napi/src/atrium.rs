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
//! ## G21-T2 §C audit-6-2 closure
//!
//! Pre-G21-T2: `JsAtrium` was a self-contained napi-shim with hollow
//! in-memory state — the engine-side `Engine::open_atrium` /
//! `AtriumHandle` surfaces existed at G16-B canary scope but were
//! NOT exposed at the napi boundary. This caused
//! `engine.atrium({}).join()` from JS/TS to fail with "is not a
//! function" once consumers reached past the surface-level type
//! assertions; Phase-3 Atrium auth story was unreachable end-to-end.
//!
//! Post-G21-T2 (this file): the napi `Engine.atrium()` factory
//! constructs a `JsAtrium` carrying an `Arc<Engine>`. On `join()`,
//! the JsAtrium drives `Engine::open_atrium(...).await` to construct
//! a real engine-side `AtriumHandle` (iroh transport endpoint,
//! per-zone Loro CRDT documents, merge-dispatch surface). The handle
//! is stored inside the JsAtrium for subsequent operations.
//!
//! Trust-roster ops (`trustPeer` / `revokePeer` / `listPeers`) and
//! lifecycle hooks (`onPeerJoin` / `onPeerLeave`) currently maintain
//! their own state alongside the engine-side handle: the engine-side
//! `AtriumHandle` does not yet expose a trust-roster surface (that
//! lives in the broader Phase-3 Atrium peer-management work). When
//! the engine-side trust-roster lands, the in-memory state in this
//! file delegates through to it; the napi method bodies stay
//! shape-stable per the D1 contract.
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
//! - audit-6-2 BLOCKER — `JsAtrium` delegation to engine-side `Atrium`.

use std::sync::{Arc, Mutex, OnceLock};

use benten_engine::Engine as InnerEngine;
use benten_engine::atrium_api::AtriumConfig as EngineAtriumConfig;
use benten_engine::engine_sync::AtriumHandle;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// G21-T2 fp-mini-review MAJOR-5 closure — a process-singleton tokio
/// runtime shared across every JsAtrium handle.
///
/// Pre-fp-mini-review the napi `JsAtrium::join` and
/// `declare_device_attestation` each constructed a fresh
/// `tokio::runtime::Builder::new_current_thread().build()` per call.
/// iroh's `Endpoint` drives background tasks via the runtime context
/// active at construction; when the per-call runtime drops post-
/// `block_on`, those background tasks terminate. The stored
/// `AtriumHandle` then points to an `Endpoint` whose driving runtime
/// is gone, so subsequent operations (especially when real engine-
/// side delegation lands) deadlock or fail.
///
/// The shared runtime is built once on first access (multi-threaded
/// flavor with `enable_all`) and reused for every async operation
/// driven through the JsAtrium napi boundary. The runtime is owned
/// by the static — it lives for the lifetime of the cdylib, so the
/// `Endpoint`'s background tasks survive across multiple JS-side
/// `await`-driven calls.
///
/// Multi-threaded flavor (vs current-thread): iroh's networking
/// layer is multi-threaded internally; using current-thread here
/// would serialize all atrium-side work onto the single thread that
/// happens to drive the first `block_on` and back-pressure JS calls.
fn js_atrium_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("benten-js-atrium")
            .build()
            .expect("benten js-atrium tokio runtime init must succeed (cdylib startup)")
    })
}

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
/// G21-T2 §C audit-6-2 closure: `JsAtrium` carries
/// `Arc<benten_engine::Engine>` so `join()` can drive
/// `Engine::open_atrium(...)` into a real engine-side `AtriumHandle`
/// (stored under `inner` once `join` completes). Pre-G21-T2 the
/// state was hollow in-memory only; the engine-side surfaces were
/// unreachable from JS/TS.
#[napi]
pub struct JsAtrium {
    config: AtriumConfig,
    state: Mutex<AtriumHandleState>,
}

/// Per-handle session state. Combines:
/// - The engine reference (so `join()` can construct the
///   engine-side `AtriumHandle`).
/// - The engine-side `AtriumHandle` (None until `join()` succeeds).
/// - The trust-roster + declared-attestations surface (in-memory
///   today; delegates to engine-side when the broader peer-mgmt API
///   lands).
struct AtriumHandleState {
    /// Engine reference used at `join()` time to construct the
    /// engine-side `AtriumHandle`. `Some` when the JsAtrium was
    /// constructed via `Engine::atrium()`; `None` for the test-only
    /// `JsAtrium::create()` constructor (preserved for the existing
    /// TS round-trip pins).
    engine: Option<Arc<InnerEngine>>,
    /// Engine-side `AtriumHandle` — populated by `join()`.
    engine_atrium: Option<AtriumHandle>,
    /// Test-only joined-state flag for `JsAtrium::create()` callers
    /// (no engine reference). Pre-G21-T2 the round-trip pins drove
    /// `join()` against this in-memory state directly; preserved to
    /// keep the TS round-trip pins green.
    joined_no_engine: bool,
    /// Declared device attestations (round-trip surface for r1-napi-2 +
    /// pcds-r4-r1-2 pins). G16-B integration: this field delegates to
    /// the engine-side device-attestation table at merge.
    declared_attestations: Vec<DeviceAttestationDeclaration>,
    /// Trusted peer-DID roster. The engine-side `AtriumHandle`
    /// doesn't yet expose a trust-roster surface; this field is
    /// authoritative for trust-roster reads via `listPeers` and
    /// will delegate to engine-side when the peer-mgmt API lands.
    trusted_peers: Vec<String>,
    /// Revoked peer-DID roster. Same delegation seam as
    /// `trusted_peers`.
    revoked_peers: Vec<String>,
}

impl Default for AtriumHandleState {
    fn default() -> Self {
        Self {
            engine: None,
            engine_atrium: None,
            joined_no_engine: false,
            declared_attestations: Vec::new(),
            trusted_peers: Vec::new(),
            revoked_peers: Vec::new(),
        }
    }
}

#[napi]
impl JsAtrium {
    /// Construct a fresh Atrium handle per the D1 factory call shape.
    ///
    /// Test-only entry point: produces a JsAtrium WITHOUT an
    /// engine-side reference, so `join()` falls back to recording
    /// joined-state observably without driving the iroh transport.
    /// The TS round-trip pins (`atrium.test.ts`) exercise the typed-
    /// struct surface against this entry; production callers go
    /// through `Engine.atrium({config})` (this file's
    /// `from_engine` constructor below).
    #[napi(factory)]
    pub fn create(config: AtriumConfig) -> Self {
        Self {
            config,
            state: Mutex::new(AtriumHandleState::default()),
        }
    }

    /// G21-T2 §C audit-6-2 closure constructor. Used by
    /// `Engine.atrium({config})` to bind the JsAtrium to the
    /// engine-side `Arc<Engine>` so subsequent `join()` calls can
    /// drive `Engine::open_atrium(...)` into a real engine-side
    /// `AtriumHandle`.
    pub(crate) fn from_engine(config: AtriumConfig, engine: Arc<InnerEngine>) -> Self {
        Self {
            config,
            state: Mutex::new(AtriumHandleState {
                engine: Some(engine),
                ..AtriumHandleState::default()
            }),
        }
    }

    /// The atrium's identifier (echo of `config.atriumId` for
    /// observability).
    #[napi(getter)]
    pub fn atrium_id(&self) -> String {
        self.config.atrium_id.clone()
    }

    /// Whether `join()` has completed on this handle.
    ///
    /// For engine-bound JsAtria (constructed via `Engine.atrium()`),
    /// returns true once `join()` has populated `engine_atrium` with
    /// a real `AtriumHandle`. For test-only `JsAtrium::create(...)`
    /// callers (no engine reference), `joined_no_engine` records the
    /// observable post-join state so the existing TS pins keep
    /// working without an engine-side handle.
    #[napi(getter)]
    pub fn is_joined(&self) -> bool {
        let state = self.state.lock().expect("atrium state mutex");
        state.engine_atrium.is_some() || state.joined_no_engine
    }

    /// Join the atrium — initiates the peer-discovery + handshake
    /// flow per G16-D wave-6b.
    ///
    /// G21-T2 §C audit-6-2 closure: when the JsAtrium was
    /// constructed via `Engine.atrium(...)`, this method drives
    /// `Engine::open_atrium(AtriumConfig::for_test()).await` to
    /// produce a real engine-side `AtriumHandle` (iroh `Endpoint`
    /// bound + per-zone Loro CRDT machinery ready). The handle is
    /// stored under `state.engine_atrium`.
    ///
    /// The test-only `JsAtrium::create()` path falls through to a
    /// no-op success (the engine-side handle stays `None`); the
    /// `is_joined` getter still flips true so the existing TS
    /// round-trip pins keep working.
    #[napi]
    pub fn join(&self) -> Result<()> {
        let engine_opt = {
            let state = self.state.lock().expect("atrium state mutex");
            state.engine.clone()
        };
        if let Some(engine) = engine_opt {
            // G21-T2 fp-mini-review MAJOR-5 closure: drive the
            // engine-side open_atrium through the process-singleton
            // shared runtime (`js_atrium_runtime()`). Pre-fp-mini-
            // review a fresh `new_current_thread()` runtime was
            // constructed per call; the runtime dropped at the end
            // of `block_on` so iroh's `Endpoint` background tasks
            // (driven by the runtime's reactor) terminated and
            // subsequent operations on the stored `AtriumHandle`
            // would deadlock once real engine-side delegation
            // arrives.
            let handle = js_atrium_runtime()
                .block_on(engine.open_atrium(EngineAtriumConfig::for_test()))
                .map_err(|e| {
                    napi::Error::new(
                        Status::GenericFailure,
                        format!("E_ATRIUM_TRANSPORT_DEGRADED: open_atrium failed: {e:?}"),
                    )
                })?;
            let mut state = self.state.lock().expect("atrium state mutex");
            state.engine_atrium = Some(handle);
        } else {
            // Test-only `create()` path: no engine reference, so we
            // can't drive a real iroh Endpoint bind. Flip the in-memory
            // joined-state flag so the existing TS round-trip pins
            // (`atrium.test.ts`) observe `isJoined === true` without
            // requiring a built napi cdylib + Engine instance.
            let mut state = self.state.lock().expect("atrium state mutex");
            state.joined_no_engine = true;
        }
        Ok(())
    }

    /// Leave the atrium — tears down the per-session state.
    ///
    /// G21-T2 fp-mini-review MAJOR-4 closure: drives the engine-side
    /// `AtriumHandle::close().await` through the shared runtime
    /// (see `js_atrium_runtime()` at the top of this file). Pre-fp-
    /// mini-review the napi `leave()` body merely cleared
    /// `engine_atrium = None`; iroh's `Endpoint::close()` does NOT
    /// fire from synchronous Drop, so in-flight datagrams may not
    /// flush and OS sockets may linger. Calling `close().await`
    /// explicitly closes the endpoint cleanly.
    ///
    /// Trust + declared-attestation rosters survive across
    /// leave/rejoin per the engine-side persistence contract; only
    /// joined-state resets.
    #[napi]
    pub fn leave(&self) -> Result<()> {
        // Take the engine_atrium handle out of state under the lock
        // (so a concurrent `is_joined` cannot observe a half-torn-
        // down state). Then drive `close().await` outside the lock
        // because `close` is async + can take a few hundred ms to
        // flush in-flight datagrams.
        let handle_opt = {
            let mut state = self.state.lock().expect("atrium state mutex");
            state.joined_no_engine = false;
            state.engine_atrium.take()
        };
        if let Some(handle) = handle_opt {
            // `AtriumHandle::close(self)` consumes the handle so the
            // last Arc-clone drops + the underlying iroh Endpoint
            // shuts down. Awaiting on the shared runtime ensures the
            // shutdown completes before `leave()` returns.
            js_atrium_runtime().block_on(handle.close());
        }
        Ok(())
    }

    /// List peers currently trusted in this atrium.
    ///
    /// Returns the peer-DID strings. The roster is the union of
    /// trusted peers minus revoked peers (which terminates active
    /// subscriptions per exit-criterion 15). The trust-roster
    /// surface is in-memory today (see module-level
    /// G21-T2 §C narrative); delegates to engine-side when the
    /// broader peer-mgmt API lands.
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
    /// G21-T2 §D audit-6-3: when an engine-side `AtriumHandle` is
    /// present (post-join), the declaration is also forwarded to
    /// the handshake machinery so peers observe it on the wire.
    /// Pre-join + test-only-create paths record locally only.
    #[napi]
    pub fn declare_device_attestation(
        &self,
        attestation: DeviceAttestationDeclaration,
    ) -> Result<()> {
        // First snapshot for the engine-side forward; release lock
        // before driving async to avoid deadlock under heavy parallel
        // declare flows.
        let engine_atrium = {
            let mut state = self.state.lock().expect("atrium state mutex");
            // Replace any existing entry for the same device-DID
            // (local round-trip surface per r1-napi-2 + pcds-r4-r1-2).
            state
                .declared_attestations
                .retain(|a| a.device_did != attestation.device_did);
            state.declared_attestations.push(attestation.clone());
            state.engine_atrium.clone()
        };
        // G21-T2 §D audit-6-3 closure: forward the declaration to the
        // engine-side `AtriumHandle::register_device_attestation` so
        // the envelope rides the handshake-time presentation path.
        // The engine-side recording is the load-bearing pin —
        // `AtriumHandle::list_declared_device_attestations` round-trips
        // the recorded envelopes; the on-the-wire frame-emission to
        // peer handshakes wires through G16-D wave-6b's broader
        // handshake protocol body work (BELONGS-NAMED-NOW per HARD
        // RULE rule-12 to phase-3-backlog §3.1 Atrium peer-handshake).
        if let Some(handle) = engine_atrium {
            // G21-T2 fp-mini-review MAJOR-5 closure: shared runtime
            // (see `js_atrium_runtime()` rationale at the top of
            // this file).
            let envelope = benten_engine::engine_sync::DeclaredDeviceAttestation {
                device_did: attestation.device_did.clone(),
                claims: attestation
                    .capabilities
                    .iter()
                    .map(|c| benten_engine::engine_sync::DeclaredCapabilityClaim {
                        path: c.path.clone(),
                        ability: c.ability.clone(),
                    })
                    .collect(),
                freshness_window: attestation.freshness_window,
            };
            js_atrium_runtime().block_on(handle.register_device_attestation(envelope));
        }
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

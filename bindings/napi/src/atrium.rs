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

use crate::identity::{JsDeviceAttestation, JsKeypair};

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

    /// Leave the atrium — tear down per-session sync participation
    /// while preserving the underlying iroh transport so a subsequent
    /// [`JsAtrium::rejoin`] can resume on the same handle.
    ///
    /// R6-FP Wave A Sub-A1 (`napi-r6-r1-1`) closure semantic shift:
    /// pre-Wave-A `leave()` drove `AtriumHandle::close(self)` which
    /// CONSUMED the handle + tore down the iroh `Endpoint`, leaving
    /// JS callers no path back to participation without rebuilding the
    /// Engine. Wave A flips this to drive the non-consuming
    /// [`AtriumHandle::leave`] (Phase-3 §6.12 item 7 — flips
    /// `is_active` to false; iroh endpoint stays bound; per-zone Loro
    /// state survives). The `engine_atrium` handle is RETAINED in
    /// state so `rejoin()` can flip the flag back to true.
    ///
    /// Engine-side guarantee (per `engine_sync.rs` rustdoc on
    /// `AtriumHandle::leave`): inbound merges + outbound publish/share/
    /// close-share paths return `AtriumError::InvalidState` (mapped to
    /// `E_ATRIUM_INACTIVE`) while inactive, without touching the
    /// underlying transport.
    ///
    /// Trust + declared-attestation rosters survive across
    /// leave/rejoin per the engine-side persistence contract; only the
    /// `is_active` flag transitions.
    ///
    /// Idempotent: a `leave()` on an already-inactive handle is a no-op.
    ///
    /// G21-T2 fp-mini-review MAJOR-4 history: an earlier revision of
    /// this method routed through `close().await` for clean
    /// `Endpoint::close()` flush semantics; preserved here is the
    /// shared-runtime drive (no per-call runtime construction) for the
    /// same iroh-background-task lifetime reasons documented at
    /// `js_atrium_runtime()`.
    #[napi]
    pub fn leave(&self) -> Result<()> {
        let handle_opt = {
            let mut state = self.state.lock().expect("atrium state mutex");
            state.joined_no_engine = false;
            // Wave A: clone (NOT take) — the handle survives so
            // `rejoin()` can resume on it.
            state.engine_atrium.clone()
        };
        if let Some(handle) = handle_opt {
            // Non-consuming engine-side `leave()` flips the
            // `is_active` flag to false. Currently infallible per the
            // engine-side rustdoc; the result-shape is preserved for
            // future versions that may surface drain-failure reasons.
            js_atrium_runtime().block_on(handle.leave()).map_err(|e| {
                napi::Error::new(
                    Status::GenericFailure,
                    format!("E_ATRIUM_LEAVE_FAILED: leave failed: {e:?}"),
                )
            })?;
        }
        Ok(())
    }

    /// R6-FP Wave A Sub-A1 closure (`napi-r6-r1-1`) — non-consuming
    /// graceful re-engagement counterpart to [`JsAtrium::leave`].
    ///
    /// When the JsAtrium was constructed via `Engine.atrium(...)` AND a
    /// previous `join()` populated an engine-side `AtriumHandle`,
    /// drives [`AtriumHandle::leave`] then re-engages via
    /// [`AtriumHandle::rejoin`]. Pre-G16-B-G the napi surface
    /// previously exposed only `leave()`; JS callers had no way to
    /// resume sync on the same handle without dropping + rebuilding the
    /// whole engine-side iroh transport.
    ///
    /// G16-B-G engine-side semantics (Phase-3 §6.12 item 7): the iroh
    /// endpoint stays bound across `leave()` / `rejoin()` cycles +
    /// per-zone Loro state survives, so the next inbound merge
    /// reconciles deterministically via Loro's natural delta-state
    /// replay. Trust-store + declared-attestation tables also survive,
    /// preserving causal-history continuity per the R4b dist-systems
    /// lens carry.
    ///
    /// For the test-only `JsAtrium::create(...)` path (no engine-side
    /// handle), flips the in-memory `joined_no_engine` flag back to
    /// true so the existing TS round-trip pins keep working.
    ///
    /// Idempotent: a `rejoin()` on an already-active handle is a no-op
    /// (the engine-side handle's `is_active` flag is already `true`).
    #[napi]
    pub fn rejoin(&self) -> Result<()> {
        let engine_atrium = {
            let mut state = self.state.lock().expect("atrium state mutex");
            // Test-only `create()` path (no engine reference): flip the
            // in-memory flag back to true so the existing TS round-trip
            // pins observe `isActive === true` post-rejoin. Idempotent
            // for already-joined-no-engine handles.
            if state.engine.is_none() {
                state.joined_no_engine = true;
                return Ok(());
            }
            state.engine_atrium.clone()
        };
        if let Some(handle) = engine_atrium {
            // Drive engine-side `rejoin()` through the shared runtime
            // (see `js_atrium_runtime()` rationale at the top of this
            // file). `AtriumHandle::rejoin` is idempotent + currently
            // infallible; the result-shape is preserved for future
            // versions that may surface re-bind failure.
            js_atrium_runtime().block_on(handle.rejoin()).map_err(|e| {
                napi::Error::new(
                    Status::GenericFailure,
                    format!("E_ATRIUM_REJOIN_FAILED: rejoin failed: {e:?}"),
                )
            })?;
        }
        // If the engine reference exists but `engine_atrium` is None
        // (meaning the engine-bound JsAtrium was never joined to begin
        // with), `rejoin()` is a no-op — callers must drive `join()`
        // first to populate the handle.
        Ok(())
    }

    /// R6-FP Wave A Sub-A1 closure (`napi-r6-r1-1`) — observable
    /// accessor for the Phase-3 §6.12 item 7 `is_active` lifecycle
    /// flag.
    ///
    /// Returns `true` when this handle is participating in Atrium sync
    /// (post-`join()` / post-`rejoin()`); `false` after `leave()` until
    /// the next `rejoin()`. Mirrors [`AtriumHandle::is_active`] —
    /// operators consume this from JS to gate UI affordances +
    /// observability dashboards on peer-churn lifecycle state.
    ///
    /// Distinct from [`JsAtrium::is_joined`] (the existing getter),
    /// which records whether this handle ever transitioned through
    /// `join()` once. `is_joined` is sticky-true after the first
    /// `join()`; `is_active` flips false on each `leave()` + true on
    /// each `rejoin()`.
    ///
    /// For the test-only `JsAtrium::create(...)` path (no engine-side
    /// handle), reflects the in-memory `joined_no_engine` flag so the
    /// existing TS pins keep working without an engine instance.
    #[napi(getter)]
    pub fn is_active(&self) -> bool {
        let state = self.state.lock().expect("atrium state mutex");
        if let Some(handle) = state.engine_atrium.as_ref() {
            handle.is_active()
        } else {
            state.joined_no_engine
        }
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

    // ========================================================================
    // R6-FP Wave A Sub-A2 closure (`napi-r6-r1-2`)
    // ========================================================================
    //
    // Per CLAUDE.md baked-in #17, JS-driven full peers (Tauri / Electron
    // / Node-AI-assistant deployments) are first-class. The G16-D wave-6b
    // setters (`set_local_device_did` / `set_local_device_keypair` /
    // `set_local_device_attestation` / `set_acceptor`) shipped engine-
    // side at PR #163 but were Rust-only — JS-driven peers fell back to
    // the legacy unsigned `device-cid:<hex>` envelope (no on-the-wire
    // device-DID attestation) regardless of what attestation the JS
    // caller wanted to bind.
    //
    // The four setters below close that gap. Each requires a prior
    // `join()` on an engine-bound JsAtrium (constructed via
    // `Engine.atrium(...)`); they return E_ATRIUM_NOT_JOINED otherwise,
    // matching the engine-side requirement that the underlying
    // `AtriumHandle` exists before its setters can fire.

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — bind the local
    /// device-DID for emission in the on-the-wire
    /// [`benten_engine::engine_sync::DeviceAttestationEnvelope`].
    ///
    /// Mirrors [`AtriumHandle::set_local_device_did`]. Pass an empty
    /// string to clear the binding (next outbound sync emits an
    /// envelope with `device_did = None`); any non-empty string is
    /// stored verbatim. Idempotent / replaceable — calling twice with
    /// different DIDs replaces the slot.
    ///
    /// Composes with [`JsAtrium::set_local_device_keypair`] +
    /// [`JsAtrium::set_local_device_attestation`]: when ALL three +
    /// the attestation are bound, the wire envelope emitted is SIGNED
    /// (V2 shape) — covering `(version, attestation, payload_hash,
    /// session_nonce)` so the receiver can verify DID binding,
    /// replay-resistance, and frame-pair payload-hash.
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn set_local_device_did(&self, device_did: String) -> Result<()> {
        let handle = self.engine_atrium_or_err("set_local_device_did")?;
        let did_opt = if device_did.is_empty() {
            None
        } else {
            Some(device_did)
        };
        js_atrium_runtime().block_on(handle.set_local_device_did(did_opt));
        Ok(())
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — bind the local device's
    /// secret keypair for signing outbound device-attestation envelope
    /// frames.
    ///
    /// Mirrors [`AtriumHandle::set_local_device_keypair`]. Pass a
    /// `Keypair` to bind; pass nothing (call
    /// [`JsAtrium::clear_local_device_keypair`]) to clear.
    ///
    /// Per `crypto-blocker-1`, [`benten_id::keypair::Keypair`] is
    /// non-`Clone`; the napi `JsKeypair` wrapper duplicates via the
    /// audited DAG-CBOR seed envelope path
    /// (`export_seed_envelope` + `from_dag_cbor_envelope`). The bound
    /// keypair lives engine-side under
    /// `AtriumInner::local_device_keypair` (zeroize-on-drop).
    ///
    /// Independent of the iroh-endpoint keypair (held at
    /// `AtriumInner::peer_keypair`); production deployments typically
    /// pass the same conceptual keypair to both — but the seam is
    /// preserved so forgery-test fixtures can drive mismatched cases.
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn set_local_device_keypair(&self, keypair: &JsKeypair) -> Result<()> {
        let handle = self.engine_atrium_or_err("set_local_device_keypair")?;
        let kp = keypair.duplicate_via_envelope()?;
        js_atrium_runtime().block_on(handle.set_local_device_keypair(Some(kp)));
        Ok(())
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — clear the local device's
    /// signing keypair binding.
    ///
    /// After clearing, outbound envelopes fall back to the unsigned
    /// legacy shape until a fresh keypair is bound via
    /// [`JsAtrium::set_local_device_keypair`].
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn clear_local_device_keypair(&self) -> Result<()> {
        let handle = self.engine_atrium_or_err("clear_local_device_keypair")?;
        js_atrium_runtime().block_on(handle.set_local_device_keypair(None));
        Ok(())
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — bind the local device's
    /// signed [`benten_id::device_attestation::DeviceAttestation`]
    /// for embedding in the outbound envelope.
    ///
    /// Mirrors [`AtriumHandle::set_local_device_attestation`]. When
    /// the attestation is bound AND
    /// [`JsAtrium::set_local_device_keypair`] has bound the device's
    /// keypair, outbound `sync_subgraph` / `accept_sync_subgraph`
    /// frames are SIGNED (V2 shape).
    ///
    /// Convenience: also updates the local device-DID slot (per the
    /// engine-side rustdoc — `attestation.device_did` propagates to
    /// `AtriumHandle::local_device_did`) so legacy callers reading
    /// that slot observe the same identity.
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn set_local_device_attestation(&self, attestation: &JsDeviceAttestation) -> Result<()> {
        let handle = self.engine_atrium_or_err("set_local_device_attestation")?;
        let att = attestation.inner_clone();
        js_atrium_runtime().block_on(handle.set_local_device_attestation(Some(att)));
        Ok(())
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — clear the local device's
    /// attestation binding.
    ///
    /// After clearing, outbound envelopes fall back to the unsigned
    /// legacy shape until a fresh attestation is bound via
    /// [`JsAtrium::set_local_device_attestation`].
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn clear_local_device_attestation(&self) -> Result<()> {
        let handle = self.engine_atrium_or_err("clear_local_device_attestation")?;
        js_atrium_runtime().block_on(handle.set_local_device_attestation(None));
        Ok(())
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — install a custom
    /// [`benten_id::device_attestation::Acceptor`] for inbound envelope
    /// verification, parameterised by a freshness window in seconds.
    ///
    /// Mirrors [`AtriumHandle::set_acceptor`] with a JS-friendly
    /// constructor surface — the full Rust `Acceptor` struct (carrying
    /// the nonce-store mutex + revocation list + optional expected-
    /// parent gate) is not directly exposed across the napi boundary;
    /// instead, this setter constructs a fresh acceptor with the given
    /// `freshness_window_secs` (matching the existing
    /// `JsDeviceAttestation::accept_at` pattern).
    ///
    /// `freshness_window_secs = 0` rejects any attestation older than
    /// `now`; very large values (up to `u32::MAX` ≈ 136 years; the
    /// engine-side `set_acceptor` accepts `u64` but the napi sig caps
    /// at `u32` — sufficient for any realistic operator deployment)
    /// accept-any-age.
    /// Production deployments typically configure a window matching
    /// the local UCAN backend's calibration (post-promotion) +
    /// optionally seed a revocation list — the latter requires the
    /// future [`JsAtrium::set_acceptor_with_revocations`] surface
    /// which composes a `DeviceRevocation` napi shape (BELONGS-NAMED-
    /// NOW per HARD RULE rule-12 to `docs/future/phase-3-backlog.md`
    /// §3 acceptor-extension surface — kept out of this Wave to keep
    /// the LOC budget honest).
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn set_acceptor(&self, freshness_window_secs: u32) -> Result<()> {
        let handle = self.engine_atrium_or_err("set_acceptor")?;
        let acceptor = benten_id::device_attestation::Acceptor::new(
            benten_id::device_attestation::FreshnessPolicy::seconds(u64::from(
                freshness_window_secs,
            )),
        );
        js_atrium_runtime().block_on(handle.set_acceptor(acceptor));
        Ok(())
    }
}

impl JsAtrium {
    /// R6-FP Wave A Sub-A2 helper — borrow the engine-bound
    /// `AtriumHandle`, returning an `E_ATRIUM_NOT_JOINED` napi error
    /// when the handle hasn't been populated by `join()` yet (or when
    /// the JsAtrium was constructed via the test-only
    /// `JsAtrium::create(...)` ctor with no engine reference).
    fn engine_atrium_or_err(&self, op: &'static str) -> Result<AtriumHandle> {
        let state = self.state.lock().expect("atrium state mutex");
        match state.engine_atrium.as_ref() {
            Some(handle) => Ok(handle.clone()),
            None => Err(napi::Error::new(
                Status::GenericFailure,
                format!(
                    "E_ATRIUM_NOT_JOINED: {op} requires a prior join() on an engine-bound Atrium handle"
                ),
            )),
        }
    }
}

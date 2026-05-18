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
///
/// ## Safe-4-async / META #744 PR-A — AsyncTask migration
///
/// The 9 `block_on`-driving mutators (`join` / `leave` / `rejoin` /
/// `declare_device_attestation` / `set_local_device_did` /
/// `set_local_device_keypair` / `clear_local_device_keypair` /
/// `set_local_device_attestation` / `clear_local_device_attestation`)
/// are Promise-returning `AsyncTask`s, NOT sync `#[napi]` methods.
/// Pre-PR-A every one of these mapped to a sync Rust fn that called
/// `js_atrium_runtime().block_on(async_op())` **on the libuv worker
/// thread the JS call arrived on** — parking that thread for the full
/// iroh round-trip. With the default `UV_THREADPOOL_SIZE=4`, four
/// concurrent `await atrium.*()` calls saturated the pool and every
/// unrelated fs / DNS / crypto op on the same Node process queued
/// behind them (META #744 headline).
///
/// Each migrated method clones the owned/`Arc` data it needs into a
/// `Send + 'static` `Task` struct and returns `AsyncTask<…>`. napi-rs
/// schedules `Task::compute()` onto a libuv worker via
/// `uv_queue_work`; the `block_on` still runs there (the iroh ops are
/// genuinely blocking from this layer's view), but the JS event loop
/// is freed and the Promise resolves on completion. The `block_on`
/// **always runs on the persistent `js_atrium_runtime()` static** —
/// never a per-task / napi-default runtime — so iroh `Endpoint`
/// background tasks survive across calls (the original
/// `js_atrium_runtime()` invariant; see its rustdoc).
///
/// `state` is `Arc<Mutex<…>>` (not a bare `Mutex`) so a `Task` can
/// clone the `Arc` in and briefly lock it inside `compute()` to read
/// the engine reference / write the resulting handle back — without
/// holding the `&self` borrow or any guard across the async boundary
/// (the #652 / #704 / #735 lock-across-await class). The std `Mutex`
/// guard is acquired + released around (never across) the `block_on`.
///
/// **NOT migrated (no `block_on`):** the sync setters
/// `set_envelope_freshness_window` (bare atomic store) +
/// `is_active` / `is_joined` getters + the in-memory roster ops
/// (`list_peers` / `trust_peer` / `revoke_peer` /
/// `list_declared_device_attestations`). `StreamHandleJs::next` is
/// **explicitly out of PR-A scope** (PR-B, #1203 — it is the only
/// public-SemVer-breaking surface and is handled there via a
/// lock-split, not by Promise-ifying the public sync getter).
///
/// **Cancellation contract:** an `AsyncTask` is NOT cancellable once
/// `compute()` has started running on a libuv worker (napi-rs only
/// supports cancelling tasks still queued, via `AbortSignal`, which
/// this binding does not wire). A `join()` whose `compute()` is
/// in-flight runs to completion even if the JS caller drops the
/// Promise. This is acceptable for the atrium mutators (bounded iroh
/// ops) and is load-bearing context for PR-B's `StreamHandleJs::next`
/// (an unbounded `recv_blocking()` MUST stay cancellable — hence
/// PR-B's lock-split rather than a naive AsyncTask conversion).
#[napi]
pub struct JsAtrium {
    config: AtriumConfig,
    state: Arc<Mutex<AtriumHandleState>>,
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
            state: Arc::new(Mutex::new(AtriumHandleState::default())),
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
            state: Arc::new(Mutex::new(AtriumHandleState {
                engine: Some(engine),
                ..AtriumHandleState::default()
            })),
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
    /// `Engine::open_atrium(AtriumConfig::production()).await` to
    /// produce a real engine-side `AtriumHandle` (iroh `Endpoint`
    /// bound + per-zone Loro CRDT machinery ready). The handle is
    /// stored under `state.engine_atrium`.
    ///
    /// #1187 closure (refinement-audit-2026-05): pre-fix this path
    /// unconditionally drove `AtriumConfig::for_test()`
    /// (= `AtriumMode::Loopback`) for engine-bound callers, so any
    /// production JS caller invoking `engine.atrium(id).join()`
    /// silently bound a loopback (no-relay, no-holepunch) transport
    /// regardless of intent — a deployed production-invariant
    /// violation invisible in default-feature builds. The original
    /// #869 finding prescribed threading `atriumId` through
    /// `EngineAtriumConfig::from_id(...)`, but post-COLLAPSE the
    /// engine-side `AtriumConfig` carries NO atrium-id field
    /// (`AtriumConfig { mode }` only — see
    /// `benten_engine::atrium_api::AtriumConfig`); atrium identity is
    /// tracked solely in this layer's `JsAtrium.config.atrium_id`.
    /// The genuine residual is the transport-mode mis-wire: the
    /// engine-bound path is the real-peer path and MUST drive
    /// `production()`. The test-only `create()` path (no engine
    /// reference) is unaffected.
    ///
    /// The test-only `JsAtrium::create()` path falls through to a
    /// no-op success (the engine-side handle stays `None`); the
    /// `is_joined` getter still flips true so the existing TS
    /// round-trip pins keep working.
    #[napi(ts_return_type = "Promise<void>")]
    pub fn join(&self) -> AsyncTask<JoinTask> {
        AsyncTask::new(JoinTask {
            state: Arc::clone(&self.state),
        })
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
    #[napi(ts_return_type = "Promise<void>")]
    pub fn leave(&self) -> AsyncTask<LeaveTask> {
        AsyncTask::new(LeaveTask {
            state: Arc::clone(&self.state),
        })
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
    #[napi(ts_return_type = "Promise<void>")]
    pub fn rejoin(&self) -> AsyncTask<RejoinTask> {
        AsyncTask::new(RejoinTask {
            state: Arc::clone(&self.state),
        })
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
    ///
    /// Safe-4 #704 closure (refinement-audit-2026-05): the engine-side
    /// `AtriumHandle` is cloned OUT of the sync `state` lock BEFORE
    /// `handle.is_active()` is called, rather than calling it while the
    /// lock is held. `AtriumHandle::is_active` is a bare atomic load
    /// today so holding the lock across it is presently safe, but the
    /// lock-held-across-handle-call shape is a latent re-entrancy /
    /// deadlock hazard if `is_active` ever evolves to acquire an
    /// engine-side `tokio::sync::Mutex` (or becomes `async`). This
    /// brings `is_active` into shape-conformance with the 6 R6-FP
    /// Wave A Sub-A2 setters + `engine_atrium_or_err`, all of which
    /// clone the handle out of the lock before driving any handle call.
    /// `Arc<AtriumInner>` clone is cheap (refcount bump); the lock is
    /// released before the (currently trivial) handle read.
    #[napi(getter)]
    pub fn is_active(&self) -> bool {
        // #704 invariant (refinement-audit-2026-05): the engine-side
        // `AtriumHandle` is cloned OUT of the `state` mutex BEFORE any
        // `handle.is_active()` dispatch — never called while the lock is
        // held. Reverting to a lock-held-across-handle-call shape
        // reintroduces the latent re-entrancy / deadlock hazard #704
        // closed. There is no inline `#[cfg(test)]` pin for this: the
        // whole `atrium` module is `#[cfg(all(feature = "napi-export",
        // not(target_arch = "wasm32")))]` (see `lib.rs`), so `JsAtrium`
        // exists ONLY on the `napi-export` cdylib surface. A plain
        // `cargo test` lib-unit-test binary that linked a `JsAtrium`
        // test would pull napi-rs's `Error<S>` Drop glue
        // (`napi_reference_unref` / `napi_delete_reference`) with no
        // Node runtime at link time and fail to link. Every other
        // `JsAtrium`-constructing test in this crate therefore lives in
        // the `bindings/napi/tests/` build-harness suite under the
        // no-`napi-export` `in-process-test` rlib path — and `JsAtrium`
        // is not reachable on that surface, so this invariant is
        // compile-proven by the clone-out shape below + was
        // substantively verified by mini-review-1288, not by a runtime
        // pin.
        let (handle_opt, joined_no_engine) = {
            let state = self.state.lock().expect("atrium state mutex");
            (state.engine_atrium.clone(), state.joined_no_engine)
        };
        match handle_opt {
            Some(handle) => handle.is_active(),
            None => joined_no_engine,
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
    /// #688 fix-2 (refinement-audit-2026-05): the engine-bound
    /// `AtriumHandle` is resolved through `engine_atrium_or_err`
    /// BEFORE any local recording, surfacing `E_ATRIUM_NOT_JOINED`
    /// when called pre-join. Pre-#688 this path did
    /// `state.engine_atrium.clone()` + `if let Some(handle)` so a
    /// pre-join caller silently recorded the attestation in the local
    /// `declared_attestations` Vec ONLY — the engine-side forward was
    /// skipped without error (the symmetric fail-OPEN flaw to
    /// Phase-3 §13.11). It now matches the 6 sibling R6-FP Wave A
    /// Sub-A2 setters which all gate through `engine_atrium_or_err`.
    /// The local round-trip Vec push moves into the `AsyncTask`
    /// `compute()` so the recorded-then-forwarded ordering is
    /// preserved (#688 fix-1: the engine-side
    /// `register_device_attestation` result is now `let _: () =`
    /// bound + error-mapped — fix-3 upgraded it to
    /// `AtriumResult<()>`).
    #[napi(ts_return_type = "Promise<void>")]
    pub fn declare_device_attestation(
        &self,
        attestation: DeviceAttestationDeclaration,
    ) -> Result<AsyncTask<DeclareDeviceAttestationTask>> {
        // #688 fix-2: gate on the engine-bound handle BEFORE recording
        // anything locally — surfaces `E_ATRIUM_NOT_JOINED` pre-join,
        // matching the 6 sibling setters. The test-only `create()`
        // path (no engine reference) likewise errors here, same as
        // every other engine-bound setter.
        let handle = self.engine_atrium_or_err("declare_device_attestation")?;
        Ok(AsyncTask::new(DeclareDeviceAttestationTask {
            state: Arc::clone(&self.state),
            handle,
            attestation,
        }))
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
    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_local_device_did(
        &self,
        device_did: String,
    ) -> Result<AsyncTask<SetLocalDeviceDidTask>> {
        let handle = self.engine_atrium_or_err("set_local_device_did")?;
        let did_opt = if device_did.is_empty() {
            None
        } else {
            Some(device_did)
        };
        Ok(AsyncTask::new(SetLocalDeviceDidTask { handle, did_opt }))
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
    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_local_device_keypair(
        &self,
        keypair: &JsKeypair,
    ) -> Result<AsyncTask<SetLocalDeviceKeypairTask>> {
        let handle = self.engine_atrium_or_err("set_local_device_keypair")?;
        // `Keypair` duplication (audited DAG-CBOR seed-envelope
        // round-trip) happens synchronously here, BEFORE the task is
        // scheduled, because `&JsKeypair` is a borrow that cannot
        // cross the `Send + 'static` task boundary. The owned
        // duplicate moves into the task.
        let kp = keypair.duplicate_via_envelope()?;
        Ok(AsyncTask::new(SetLocalDeviceKeypairTask {
            handle,
            keypair: Some(kp),
        }))
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — clear the local device's
    /// signing keypair binding.
    ///
    /// After clearing, outbound envelopes fall back to the unsigned
    /// legacy shape until a fresh keypair is bound via
    /// [`JsAtrium::set_local_device_keypair`].
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi(ts_return_type = "Promise<void>")]
    pub fn clear_local_device_keypair(&self) -> Result<AsyncTask<SetLocalDeviceKeypairTask>> {
        let handle = self.engine_atrium_or_err("clear_local_device_keypair")?;
        Ok(AsyncTask::new(SetLocalDeviceKeypairTask {
            handle,
            keypair: None,
        }))
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
    #[napi(ts_return_type = "Promise<void>")]
    pub fn set_local_device_attestation(
        &self,
        attestation: &JsDeviceAttestation,
    ) -> Result<AsyncTask<SetLocalDeviceAttestationTask>> {
        let handle = self.engine_atrium_or_err("set_local_device_attestation")?;
        // `inner_clone()` runs synchronously here — `&JsDeviceAttestation`
        // is a borrow that cannot cross the task boundary; the owned
        // clone moves into the task.
        let att = attestation.inner_clone();
        Ok(AsyncTask::new(SetLocalDeviceAttestationTask {
            handle,
            attestation: Some(att),
        }))
    }

    /// R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — clear the local device's
    /// attestation binding.
    ///
    /// After clearing, outbound envelopes fall back to the unsigned
    /// legacy shape until a fresh attestation is bound via
    /// [`JsAtrium::set_local_device_attestation`].
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi(ts_return_type = "Promise<void>")]
    pub fn clear_local_device_attestation(
        &self,
    ) -> Result<AsyncTask<SetLocalDeviceAttestationTask>> {
        let handle = self.engine_atrium_or_err("clear_local_device_attestation")?;
        Ok(AsyncTask::new(SetLocalDeviceAttestationTask {
            handle,
            attestation: None,
        }))
    }

    /// COLLAPSE (P3) — set the freshness window (seconds) applied to
    /// inbound signed device-attestation envelopes.
    ///
    /// **Replaces the deleted `set_acceptor`.** Under COLLAPSE
    /// (DECISION-RECORD §4 RATIFIED) the device-attestation
    /// *acceptance* pipe (`benten_id::Acceptor` — nonce-store /
    /// revocation-list / expected-parent gate) is DELETED: the device
    /// envelope is no longer a distinct trust-root. Revocation
    /// collapses to user-root UCAN revocation at the durable seam; the
    /// J8 envelope-ceiling is ANDed once at the engine's single
    /// inbound-sync recheck seam. The freshness window (the one
    /// generic anti-replay control the operator still tunes) survives,
    /// mirroring the engine-side
    /// [`AtriumHandle::set_envelope_freshness_window`].
    ///
    /// `freshness_window_secs = 0` rejects any attestation older than
    /// `now`; large values (up to `u32::MAX` ≈ 136 years) accept-any-
    /// age. The wire-envelope signature + payload-binding +
    /// embedded-attestation→user-root signature defenses remain
    /// load-bearing regardless of the window.
    ///
    /// Errors with `E_ATRIUM_NOT_JOINED` if called before `join()`.
    #[napi]
    pub fn set_envelope_freshness_window(&self, freshness_window_secs: u32) -> Result<()> {
        let handle = self.engine_atrium_or_err("set_envelope_freshness_window")?;
        handle.set_envelope_freshness_window(u64::from(freshness_window_secs));
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

// ============================================================================
// Safe-4-async / META #744 PR-A — AsyncTask migration
// ============================================================================
//
// Each `Task` below carries owned / `Arc` data only (`Send + 'static`);
// no `&self` borrow or `MutexGuard` crosses into `compute()`. napi-rs
// schedules `compute()` onto a libuv worker thread via `uv_queue_work`,
// so the JS event loop is freed while the (genuinely blocking from this
// layer's view) iroh round-trip runs. The `block_on` ALWAYS targets the
// persistent process-singleton `js_atrium_runtime()` static — never a
// per-task / napi-default runtime — so iroh `Endpoint` background tasks
// survive across calls (see `js_atrium_runtime()` rustdoc + the
// `JsAtrium` doc-comment cancellation/runtime contract).
//
// Error-mapping strings are byte-for-byte preserved from the pre-PR-A
// sync bodies so downstream string-matching consumers + the ErrorCode
// 4-surface mirror (§3.5g) stay stable.

/// `AsyncTask` backing [`JsAtrium::join`]. Holds the shared
/// `Arc<Mutex<AtriumHandleState>>`; reads the engine ref + writes the
/// resulting `AtriumHandle` back under brief std-`Mutex` sections that
/// never span the `block_on`.
pub struct JoinTask {
    state: Arc<Mutex<AtriumHandleState>>,
}

impl Task for JoinTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        // Brief lock to read the engine ref; released before block_on.
        let engine_opt = {
            let state = self.state.lock().expect("atrium state mutex");
            state.engine.clone()
        };
        if let Some(engine) = engine_opt {
            // Drive engine-side open_atrium on the PERSISTENT shared
            // runtime (`js_atrium_runtime()`). A per-call / per-task
            // runtime would drop iroh's `Endpoint` background tasks
            // when it fell out of scope post-`block_on`.
            let handle = js_atrium_runtime()
                .block_on(engine.open_atrium(EngineAtriumConfig::production()))
                .map_err(|e| {
                    napi::Error::new(
                        Status::GenericFailure,
                        format!("E_ATRIUM_TRANSPORT_DEGRADED: open_atrium failed: {e:?}"),
                    )
                })?;
            // Brief lock to write the handle back; no await held.
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

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

/// `AsyncTask` backing [`JsAtrium::leave`].
pub struct LeaveTask {
    state: Arc<Mutex<AtriumHandleState>>,
}

impl Task for LeaveTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
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

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

/// `AsyncTask` backing [`JsAtrium::rejoin`].
pub struct RejoinTask {
    state: Arc<Mutex<AtriumHandleState>>,
}

impl Task for RejoinTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
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

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

/// `AsyncTask` backing [`JsAtrium::declare_device_attestation`].
///
/// The engine-bound `AtriumHandle` is resolved + the
/// `E_ATRIUM_NOT_JOINED` gate (#688 fix-2) is applied SYNCHRONOUSLY in
/// the `#[napi]` method before this task is constructed (matching the
/// 6 sibling setters). This task does the local round-trip Vec push
/// (r1-napi-2 + pcds-r4-r1-2 surface) THEN forwards to the engine —
/// preserving the recorded-then-forwarded ordering.
pub struct DeclareDeviceAttestationTask {
    state: Arc<Mutex<AtriumHandleState>>,
    handle: AtriumHandle,
    attestation: DeviceAttestationDeclaration,
}

impl Task for DeclareDeviceAttestationTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        // Local round-trip surface (r1-napi-2 + pcds-r4-r1-2): replace
        // any existing entry for the same device-DID. Brief lock; no
        // await held across it.
        {
            let mut state = self.state.lock().expect("atrium state mutex");
            state
                .declared_attestations
                .retain(|a| a.device_did != self.attestation.device_did);
            state.declared_attestations.push(self.attestation.clone());
        }
        // G21-T2 §D audit-6-3 closure: forward to the engine-side
        // `AtriumHandle::register_device_attestation` so the envelope
        // rides the handshake-time presentation path. The on-the-wire
        // frame-emission wires through G16-D wave-6b's broader
        // handshake protocol body work (BELONGS-NAMED-NOW per HARD
        // RULE rule-12 to phase-3-backlog §3.1 Atrium peer-handshake).
        let envelope = benten_engine::engine_sync::DeclaredDeviceAttestation {
            device_did: self.attestation.device_did.clone(),
            claims: self
                .attestation
                .capabilities
                .iter()
                .map(|c| benten_engine::engine_sync::DeclaredCapabilityClaim {
                    path: c.path.clone(),
                    ability: c.ability.clone(),
                })
                .collect(),
            freshness_window: self.attestation.freshness_window,
        };
        // #688 fix-1 + fix-3: the result is BOUND (`let _: () =`) and
        // error-mapped. fix-3 upgraded the engine-side
        // `register_device_attestation` to `AtriumResult<()>` with an
        // `ensure_active` gate that returns `E_ATRIUM_INACTIVE` on a
        // mid-leave race; pre-#688 the return was silently discarded
        // (`;`-terminated) which would have dropped that error once
        // the engine-side return was upgraded.
        let _: () = js_atrium_runtime()
            .block_on(self.handle.register_device_attestation(envelope))
            .map_err(|e| {
                napi::Error::new(
                    Status::GenericFailure,
                    format!("E_ATRIUM_INACTIVE: register_device_attestation failed: {e:?}"),
                )
            })?;
        Ok(())
    }

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

/// `AsyncTask` backing [`JsAtrium::set_local_device_did`]. The
/// `E_ATRIUM_NOT_JOINED` gate is applied synchronously before this
/// task is constructed.
pub struct SetLocalDeviceDidTask {
    handle: AtriumHandle,
    did_opt: Option<String>,
}

impl Task for SetLocalDeviceDidTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        js_atrium_runtime().block_on(self.handle.set_local_device_did(self.did_opt.take()));
        Ok(())
    }

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

/// `AsyncTask` backing [`JsAtrium::set_local_device_keypair`] +
/// [`JsAtrium::clear_local_device_keypair`] (the latter passes
/// `keypair: None`). The owned `Keypair` duplicate / the
/// `E_ATRIUM_NOT_JOINED` gate are produced synchronously before this
/// task is constructed.
pub struct SetLocalDeviceKeypairTask {
    handle: AtriumHandle,
    keypair: Option<benten_id::keypair::Keypair>,
}

impl Task for SetLocalDeviceKeypairTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        js_atrium_runtime().block_on(self.handle.set_local_device_keypair(self.keypair.take()));
        Ok(())
    }

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

/// `AsyncTask` backing [`JsAtrium::set_local_device_attestation`] +
/// [`JsAtrium::clear_local_device_attestation`] (the latter passes
/// `attestation: None`). The owned attestation clone / the
/// `E_ATRIUM_NOT_JOINED` gate are produced synchronously before this
/// task is constructed.
pub struct SetLocalDeviceAttestationTask {
    handle: AtriumHandle,
    attestation: Option<benten_id::device_attestation::DeviceAttestation>,
}

impl Task for SetLocalDeviceAttestationTask {
    type Output = ();
    type JsValue = ();

    fn compute(&mut self) -> Result<Self::Output> {
        js_atrium_runtime().block_on(
            self.handle
                .set_local_device_attestation(self.attestation.take()),
        );
        Ok(())
    }

    fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
        Ok(())
    }
}

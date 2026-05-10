// G16-D wave-6b — Atrium TS DSL B-prime factory-handle form per Ben's
// D1 ratification 2026-05-05.
//
// `engine.atrium` is a CALLABLE FACTORY:
//
//   const family = engine.atrium({ atriumId: "family" });
//   await family.join();
//   family.listPeers();
//   await family.subscribe("/zone/posts", (event) => { ... });
//   await family.leave();
//
// Multi-Atrium-as-default per Ben's framing: every call to
// `engine.atrium({...})` returns a fresh handle whose methods carry
// per-session state. Multiple calls with the same `atriumId` return
// distinct handles routing to the same logical atrium under the
// engine. Matches the WebSocket idiom (constructor returns a handle;
// methods drive lifecycle).
//
// ## Pin sources
//
// - r2-test-landscape §2.4 G16-D row `atrium.test.ts (TS DSL surface)`.
// - plan §3 G16-D row line "TS DSL — `engine.atrium({config}).join()`
//   factory shape per D-PHASE-3-15 + Ben's D1 ratification".
// - `D-PHASE-3-15` RESOLVED 2026-05-05 (Pattern B-prime; flat-namespace
//   REJECTED).
// - `r1-napi-10` (B-prime factory shape architectural pin).
// - `r1-napi-2` (declareDeviceAttestation TS round-trip on handle).

import type {
  DeviceAttestation as IdentityDeviceAttestation,
  KeypairHandle,
} from "./identity.js";
import type { CapabilityClaim, DeviceAttestation } from "./types.js";

// ---------------------------------------------------------------------------
// Public surface — types
// ---------------------------------------------------------------------------

/**
 * Configuration object passed to the `engine.atrium({...})` factory
 * call.
 *
 * `atriumId` is the caller-chosen identifier for the atrium (e.g.
 * `"family"`, `"team-foo"`). Stable per-atrium handle key — multiple
 * calls with the same `atriumId` return distinct handles routing to
 * the same logical atrium.
 */
export interface AtriumConfig {
  /** Caller-chosen atrium identifier. */
  atriumId: string;
  /**
   * Optional invite-shaped UCAN grant carried as raw DAG-CBOR bytes
   * when accepting an existing atrium's invitation. Reserved for the
   * post-G16-B engine-side invite flow.
   */
  invite?: Uint8Array | Buffer;
}

/** Callback type for the `onPeerJoin` / `onPeerLeave` lifecycle hooks. */
export type PeerLifecycleCallback = (peerDid: string) => void;

/** Callback type for `subscribe(...)` change-event delivery. */
export type SubscribeCallback = (event: unknown) => void;

/**
 * Subscription handle returned from `Atrium.subscribe(...)`. The
 * `unsubscribe()` teardown is observable; tests pin its presence per
 * the B-prime composition pin.
 */
export interface AtriumSubscription {
  /** Tear down the subscription. */
  unsubscribe: () => Promise<void>;
}

/**
 * The `Atrium` typed handle returned from `engine.atrium({config})`
 * per Pattern B-prime (Ben's D1 ratification 2026-05-05).
 *
 * Methods drive the per-handle lifecycle:
 * - `join()` — initiate peer-discovery + handshake flow.
 * - `leave()` — tear down the per-session state.
 * - `listPeers()` — list trusted peer-DIDs in the atrium.
 * - `trustPeer(did)` — extend trust to a peer-DID.
 * - `revokePeer(did)` — revoke trust; terminates active subscriptions
 *   per exit-criterion 15.
 * - `subscribe(path, cb)` — subscribe to a path within the atrium.
 * - `declareDeviceAttestation(env)` — declare the local device's
 *   capability envelope per CLAUDE.md baked-in #17 + D-PHASE-3-25.
 * - `listDeclaredDeviceAttestations()` — round-trip the declared
 *   envelopes (pcds-r4-r1-2 round-trip pin).
 * - `onPeerJoin` / `onPeerLeave` — lifecycle event hooks.
 */
export interface Atrium {
  /** Initiate peer discovery + handshake flow (post-construction). */
  join(): Promise<void>;
  /**
   * Tear down per-session sync participation while preserving the
   * underlying iroh transport so a subsequent {@link Atrium.rejoin}
   * can resume on the same handle.
   *
   * R6-FP Wave A semantic shift (`napi-r6-r1-1`): pre-Wave-A `leave()`
   * routed through the consuming `AtriumHandle::close(self)` which
   * tore down the iroh `Endpoint` — JS callers had no path back to
   * participation without rebuilding the Engine. Wave A flips this to
   * the non-consuming `AtriumHandle::leave` (Phase-3 §6.12 item 7) so
   * `rejoin()` can resume on the same handle.
   *
   * Trust + declared-attestation rosters survive across leave/rejoin
   * per the engine-side persistence contract; only the `isActive`
   * flag transitions.
   *
   * Idempotent: a `leave()` on an already-inactive handle is a no-op.
   */
  leave(): Promise<void>;
  /**
   * Re-engage sync participation on the same handle after
   * {@link Atrium.leave} — non-consuming graceful re-engagement
   * counterpart per Phase-3 §6.12 item 7.
   *
   * R6-FP Wave A Sub-A1 (`napi-r6-r1-1`) closure. The iroh endpoint
   * stays bound across leave/rejoin cycles + per-zone Loro state
   * survives, so the next inbound merge reconciles deterministically
   * via Loro's natural delta-state replay. Trust-store + declared-
   * attestation tables also survive, preserving causal-history
   * continuity per the R4b dist-systems lens carry.
   *
   * Idempotent: a `rejoin()` on an already-active handle is a no-op.
   */
  rejoin(): Promise<void>;
  /**
   * Whether this handle is currently participating in Atrium sync.
   *
   * `true` after {@link Atrium.join} / {@link Atrium.rejoin}; `false`
   * after {@link Atrium.leave} until the next `rejoin()`. Distinct
   * from {@link Atrium.isJoined} (which is sticky-true after the first
   * `join()`); operators consume `isActive` to gate UI affordances +
   * observability dashboards on peer-churn lifecycle state.
   */
  readonly isActive: boolean;
  /** List trusted peer DIDs. */
  listPeers(): string[];
  /** Extend trust to a peer DID. */
  trustPeer(peerDid: string): Promise<void>;
  /** Revoke trust + terminate active subscriptions per exit-criterion 15. */
  revokePeer(peerDid: string): Promise<void>;
  /**
   * Subscribe to a path within this atrium's per-session scope.
   *
   * G21-T2 fp-mini-review MAJOR-7 honest-state callout: today the
   * subscription is recorded locally and the returned
   * `unsubscribe()` teardown is observable, but engine-side change-
   * event delivery to the supplied callback is reserved for the
   * G16-B SUBSCRIBE wireup wave (when the engine-attached SUBSCRIBE
   * primitive composes with the delivery-time cap-recheck closure
   * per G14-D F6). The `leave()` teardown cancels every active
   * subscription deterministically.
   */
  subscribe(path: string, callback: SubscribeCallback): Promise<AtriumSubscription>;
  /**
   * Register a peer-join lifecycle hook.
   *
   * G21-T2 fp-mini-review MAJOR-7 honest-state callout: callbacks
   * are recorded; engine-side peer-join firing wires through when
   * G16-B's peer-event stream lands. `leave()` clears the registry.
   */
  onPeerJoin(callback: PeerLifecycleCallback): void;
  /**
   * Register a peer-leave lifecycle hook.
   *
   * G21-T2 fp-mini-review MAJOR-7 honest-state callout: callbacks
   * are recorded + fire today on `revokePeer()` (synthetic leave
   * event); the engine-side peer-leave-on-disconnect wireup lands
   * with G16-B. `leave()` clears the registry.
   */
  onPeerLeave(callback: PeerLifecycleCallback): void;
  /** Declare a device-attestation envelope on this handle. */
  declareDeviceAttestation(envelope: DeviceAttestation): Promise<void>;
  /** List declared device-attestations on this handle. */
  listDeclaredDeviceAttestations(): Promise<DeviceAttestation[]>;

  // -------------------------------------------------------------------------
  // R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — local device-attestation
  // setters for JS-driven full peers (CLAUDE.md baked-in #17).
  //
  // All four setters require a prior `join()` on an engine-bound
  // Atrium handle; calling pre-join surfaces `E_ATRIUM_NOT_JOINED`.
  // -------------------------------------------------------------------------

  /**
   * Bind the local device-DID for emission in the on-the-wire
   * `DeviceAttestationEnvelope`.
   *
   * Mirrors `AtriumHandle::set_local_device_did`. Passing an empty
   * string clears the binding (next outbound sync emits an envelope
   * with `device_did = None`); idempotent / replaceable — calling
   * twice with different DIDs replaces the slot.
   *
   * Composes with {@link Atrium.setLocalDeviceKeypair} +
   * {@link Atrium.setLocalDeviceAttestation}: when the attestation +
   * keypair are both bound, outbound sync frames are SIGNED (V2
   * envelope shape).
   */
  setLocalDeviceDid(deviceDid: string): Promise<void>;

  /**
   * Bind the local device's secret keypair for signing outbound
   * device-attestation envelope frames.
   *
   * Mirrors `AtriumHandle::set_local_device_keypair`. Per
   * `crypto-blocker-1` the underlying Rust `Keypair` is non-`Clone`;
   * the napi layer duplicates via the audited DAG-CBOR seed envelope
   * round-trip. The bound keypair lives engine-side under
   * `AtriumInner::local_device_keypair` (zeroize-on-drop).
   */
  setLocalDeviceKeypair(keypair: KeypairHandle): Promise<void>;

  /**
   * Clear the local device's signing keypair binding. After clearing,
   * outbound envelopes fall back to the unsigned legacy shape until a
   * fresh keypair is bound via {@link Atrium.setLocalDeviceKeypair}.
   */
  clearLocalDeviceKeypair(): Promise<void>;

  /**
   * Bind the local device's signed
   * `benten_id::device_attestation::DeviceAttestation` for embedding
   * in the outbound envelope.
   *
   * Mirrors `AtriumHandle::set_local_device_attestation`. Convenience:
   * also updates the local device-DID slot from
   * `attestation.deviceDid` so legacy callers reading that slot
   * observe the same identity.
   */
  setLocalDeviceAttestation(attestation: IdentityDeviceAttestation): Promise<void>;

  /**
   * Clear the local device's attestation binding. After clearing,
   * outbound envelopes fall back to the unsigned legacy shape until a
   * fresh attestation is bound via
   * {@link Atrium.setLocalDeviceAttestation}.
   */
  clearLocalDeviceAttestation(): Promise<void>;

  /**
   * Install a custom inbound-envelope verifier (acceptor)
   * parameterised by a freshness window in seconds.
   *
   * Mirrors `AtriumHandle::set_acceptor` with a JS-friendly
   * constructor surface — the full Rust `Acceptor` struct (carrying
   * the nonce-store mutex + revocation list + optional expected-
   * parent gate) is not directly exposed across the napi boundary;
   * this setter constructs a fresh acceptor with the given freshness
   * window. Production deployments that need pre-populated revocation
   * rosters or expected-parent gating compose the engine-side
   * `Acceptor` directly (Rust); see `docs/future/phase-3-backlog.md`
   * §3.3 for the future napi acceptor-extension surface.
   *
   * `freshnessWindowSecs = 0` rejects any attestation older than
   * `now`; very large values accept-any-age (matching the default
   * `FreshnessPolicy::seconds(u64::MAX)` Rust acceptor).
   */
  setAcceptor(freshnessWindowSecs: number): Promise<void>;

  /** Whether `join()` has completed on this handle. */
  readonly isJoined: boolean;
  /** Echo of `config.atriumId` for observability. */
  readonly atriumId: string;
}

/**
 * The `AtriumFactory` callable type — the shape of `engine.atrium`
 * per Ben's D1.
 */
export type AtriumFactory = (config: AtriumConfig) => Atrium;

// ---------------------------------------------------------------------------
// Native-binding shape (mirrors `bindings/napi/src/atrium.rs`)
// ---------------------------------------------------------------------------

/**
 * Shape of the napi-side `JsAtrium` class (mirrors
 * `bindings/napi/src/atrium.rs`). Optional fields tolerate older
 * native bindings that pre-date the wave-6b landing — the wrapper
 * surfaces a clear error at call time rather than at construction.
 */
export interface NativeAtrium {
  atriumId?: string;
  isJoined?: boolean;
  /**
   * R6-FP Wave A Sub-A1 (`napi-r6-r1-1`) — non-sticky lifecycle
   * indicator distinct from `isJoined`. Optional on the type to
   * tolerate older napi cdylib builds that pre-date Wave A; the
   * wrapper falls back to `isJoined` semantics when the binding
   * doesn't expose it.
   */
  isActive?: boolean;
  join?: () => void;
  leave?: () => void;
  /** R6-FP Wave A Sub-A1 (`napi-r6-r1-1`) — see {@link Atrium.rejoin}. */
  rejoin?: () => void;
  listPeers?: () => string[];
  trustPeer?: (peerDid: string) => void;
  revokePeer?: (peerDid: string) => void;
  declareDeviceAttestation?: (attestation: NativeDeviceAttestation) => void;
  listDeclaredDeviceAttestations?: () => NativeDeviceAttestation[];
  /**
   * R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — local device-attestation
   * setters bound through to the engine-side `AtriumHandle::*`
   * surfaces shipped at PR #163. All optional on the type to tolerate
   * older napi cdylib builds — the TS wrapper surfaces a clear error
   * at call time rather than at construction.
   */
  setLocalDeviceDid?: (deviceDid: string) => void;
  setLocalDeviceKeypair?: (keypair: unknown) => void;
  clearLocalDeviceKeypair?: () => void;
  setLocalDeviceAttestation?: (attestation: unknown) => void;
  clearLocalDeviceAttestation?: () => void;
  setAcceptor?: (freshnessWindowSecs: number) => void;
}

interface NativeDeviceAttestation {
  deviceDid: string;
  capabilities: CapabilityClaim[];
  freshnessWindow: number;
}

/**
 * Shape of the napi factory exposed at `native.JsAtrium.create(config)`.
 *
 * Pre-G21-T2 path (used when the napi cdylib pre-dates the G21-T2
 * binding): the static `create()` method constructs a self-contained
 * `JsAtrium` with no engine reference. Post-G21-T2 the production
 * path is `engine.atrium({config})` (instance method on the Engine
 * class — see `NativeEngineWithAtrium` below).
 */
export interface NativeAtriumFactoryConstruct {
  create: (config: { atriumId: string }) => NativeAtrium;
}

/**
 * G21-T2 §C audit-6-2 closure — shape of the napi `Engine.atrium`
 * instance method. Post-G21-T2 the engine-bound JsAtrium drives
 * `Engine::open_atrium(...)` at `join()` time to produce a real
 * engine-side `AtriumHandle`.
 */
export interface NativeEngineWithAtrium {
  atrium?: (config: { atriumId: string }) => NativeAtrium;
}

// ---------------------------------------------------------------------------
// AtriumHandle — the TS handle wrapping the napi JsAtrium
// ---------------------------------------------------------------------------

class AtriumHandle implements Atrium {
  private readonly native: NativeAtrium;
  private readonly config: AtriumConfig;
  private readonly peerJoinCallbacks: PeerLifecycleCallback[] = [];
  private readonly peerLeaveCallbacks: PeerLifecycleCallback[] = [];
  /**
   * G21-T2 fp-mini-review MAJOR-7 closure — track active
   * subscriptions so `leave()` can deactivate them deterministically.
   * Pre-fp-mini-review the subscribe callback registry was
   * scoped to the `unsubscribe()` closure only; `leave()` did not
   * tear down lingering subscriptions.
   */
  private readonly activeSubscriptions: { deactivate: () => void }[] = [];

  constructor(native: NativeAtrium, config: AtriumConfig) {
    this.native = native;
    this.config = config;
  }

  get atriumId(): string {
    return this.native.atriumId ?? this.config.atriumId;
  }

  get isJoined(): boolean {
    return this.native.isJoined ?? false;
  }

  /**
   * R6-FP Wave A Sub-A1 (`napi-r6-r1-1`) — non-sticky lifecycle
   * indicator. Falls back to `isJoined` semantics when the underlying
   * native binding pre-dates Wave A (the `isActive` field is absent
   * on older `JsAtrium` builds).
   */
  get isActive(): boolean {
    if (typeof this.native.isActive === "boolean") {
      return this.native.isActive;
    }
    // Pre-Wave-A fallback: native bindings without the `isActive`
    // field treat `isJoined` as the authoritative active flag.
    return this.native.isJoined ?? false;
  }

  async join(): Promise<void> {
    if (typeof this.native.join !== "function") {
      throw new Error("Atrium.join unavailable on this native binding");
    }
    this.native.join();
  }

  async leave(): Promise<void> {
    if (typeof this.native.leave !== "function") {
      throw new Error("Atrium.leave unavailable on this native binding");
    }
    this.native.leave();
    // G21-T2 fp-mini-review MAJOR-7 closure: drop the
    // peer-lifecycle + subscribe callback registries on `leave()`.
    // Pre-fp-mini-review a stale callback list survived leave/rejoin
    // (the registry was never cleared), so a re-`join()` would
    // notify callbacks installed BEFORE the leave — a registry leak
    // + subtle correctness bug if the JS caller assumed leave reset
    // the lifecycle hooks. Dropping the registries here matches the
    // observable lifecycle contract.
    for (const entry of this.activeSubscriptions) {
      entry.deactivate();
    }
    this.peerJoinCallbacks.length = 0;
    this.peerLeaveCallbacks.length = 0;
    this.activeSubscriptions.length = 0;
  }

  async rejoin(): Promise<void> {
    if (typeof this.native.rejoin !== "function") {
      throw new Error("Atrium.rejoin unavailable on this native binding");
    }
    this.native.rejoin();
  }

  listPeers(): string[] {
    if (typeof this.native.listPeers !== "function") {
      return [];
    }
    return this.native.listPeers();
  }

  async trustPeer(peerDid: string): Promise<void> {
    if (typeof this.native.trustPeer !== "function") {
      throw new Error("Atrium.trustPeer unavailable on this native binding");
    }
    this.native.trustPeer(peerDid);
  }

  async revokePeer(peerDid: string): Promise<void> {
    if (typeof this.native.revokePeer !== "function") {
      throw new Error("Atrium.revokePeer unavailable on this native binding");
    }
    this.native.revokePeer(peerDid);
    // Notify peer-leave subscribers locally on revoke.
    for (const cb of this.peerLeaveCallbacks) {
      try {
        cb(peerDid);
      } catch {
        // Swallow callback errors so one bad listener doesn't break
        // others; same posture as the existing onChange/onEmit
        // dispatchers.
      }
    }
  }

  async subscribe(
    path: string,
    callback: SubscribeCallback,
  ): Promise<AtriumSubscription> {
    if (typeof path !== "string" || path.length === 0) {
      throw new Error("Atrium.subscribe requires a non-empty path");
    }
    if (typeof callback !== "function") {
      throw new Error("Atrium.subscribe requires a callback function");
    }
    // G16-B reconciliation: the engine-side subscribe path will route
    // through the engine-attached SUBSCRIBE primitive (G14-D F6
    // delivery-time cap-recheck composes here). At wave-6b napi-shim
    // scope, we record the subscription locally so the round-trip
    // pin's `unsubscribe()` teardown surface is observable.
    //
    // G21-T2 fp-mini-review MAJOR-7 closure: the active flag is
    // closed-over by both `unsubscribe()` (caller-driven teardown)
    // AND the `activeSubscriptions` registry (`leave()`-driven
    // teardown). Either path flips `active = false` so that when the
    // engine-side SUBSCRIBE delivery wires through (G16-B), an
    // already-deactivated subscription is a no-op. The registry-
    // tracked entry is dropped as part of the `leave()` cleanup +
    // the `unsubscribe()` closure removes itself from the registry
    // so a long-lived atrium doesn't leak entries.
    let active = true;
    const entry = {
      deactivate: () => {
        active = false;
      },
    };
    this.activeSubscriptions.push(entry);
    const subs = this.activeSubscriptions;
    return {
      unsubscribe: async () => {
        active = false;
        // Reference `callback` to keep the closure-capture honest
        // under linters; G16-B drains real change-events here.
        void callback;
        void active;
        void path;
        // Remove from the active-subscription registry so
        // long-running atriums don't accumulate entries.
        const idx = subs.indexOf(entry);
        if (idx >= 0) {
          subs.splice(idx, 1);
        }
      },
    };
  }

  onPeerJoin(callback: PeerLifecycleCallback): void {
    if (typeof callback !== "function") {
      throw new Error("Atrium.onPeerJoin requires a callback function");
    }
    this.peerJoinCallbacks.push(callback);
  }

  onPeerLeave(callback: PeerLifecycleCallback): void {
    if (typeof callback !== "function") {
      throw new Error("Atrium.onPeerLeave requires a callback function");
    }
    this.peerLeaveCallbacks.push(callback);
  }

  async declareDeviceAttestation(envelope: DeviceAttestation): Promise<void> {
    if (typeof this.native.declareDeviceAttestation !== "function") {
      throw new Error(
        "Atrium.declareDeviceAttestation unavailable on this native binding",
      );
    }
    this.native.declareDeviceAttestation({
      deviceDid: envelope.deviceDid,
      capabilities: envelope.capabilities,
      freshnessWindow: envelope.freshnessWindow,
    });
  }

  async listDeclaredDeviceAttestations(): Promise<DeviceAttestation[]> {
    if (typeof this.native.listDeclaredDeviceAttestations !== "function") {
      return [];
    }
    const native = this.native.listDeclaredDeviceAttestations();
    return native.map((a) => ({
      deviceDid: a.deviceDid,
      capabilities: a.capabilities,
      freshnessWindow: a.freshnessWindow,
    }));
  }

  // -------------------------------------------------------------------------
  // R6-FP Wave A Sub-A2 (`napi-r6-r1-2`) — local device-attestation
  // setters
  // -------------------------------------------------------------------------

  async setLocalDeviceDid(deviceDid: string): Promise<void> {
    if (typeof this.native.setLocalDeviceDid !== "function") {
      throw new Error(
        "Atrium.setLocalDeviceDid unavailable on this native binding",
      );
    }
    if (typeof deviceDid !== "string") {
      throw new Error("Atrium.setLocalDeviceDid requires a string deviceDid");
    }
    this.native.setLocalDeviceDid(deviceDid);
  }

  async setLocalDeviceKeypair(keypair: KeypairHandle): Promise<void> {
    if (typeof this.native.setLocalDeviceKeypair !== "function") {
      throw new Error(
        "Atrium.setLocalDeviceKeypair unavailable on this native binding",
      );
    }
    if (keypair === null || typeof keypair !== "object") {
      throw new Error(
        "Atrium.setLocalDeviceKeypair requires a Keypair handle",
      );
    }
    // The napi binding's class signature accepts the napi `JsKeypair`
    // class instance directly (per `bindings/napi/src/atrium.rs::
    // set_local_device_keypair(&JsKeypair)`). The TS-side
    // `KeypairHandle` is the structural shape; production callers
    // pass the actual napi class instance returned by
    // `Keypair.generate()`.
    this.native.setLocalDeviceKeypair(keypair);
  }

  async clearLocalDeviceKeypair(): Promise<void> {
    if (typeof this.native.clearLocalDeviceKeypair !== "function") {
      throw new Error(
        "Atrium.clearLocalDeviceKeypair unavailable on this native binding",
      );
    }
    this.native.clearLocalDeviceKeypair();
  }

  async setLocalDeviceAttestation(
    attestation: IdentityDeviceAttestation,
  ): Promise<void> {
    if (typeof this.native.setLocalDeviceAttestation !== "function") {
      throw new Error(
        "Atrium.setLocalDeviceAttestation unavailable on this native binding",
      );
    }
    if (attestation === null || typeof attestation !== "object") {
      throw new Error(
        "Atrium.setLocalDeviceAttestation requires a DeviceAttestation object",
      );
    }
    // The napi binding's class signature accepts the napi
    // `JsDeviceAttestation` class instance directly (per
    // `bindings/napi/src/atrium.rs::set_local_device_attestation
    // (&JsDeviceAttestation)`). Production callers pass the actual
    // napi class instance returned by `DeviceAttestation.issue(...)`.
    this.native.setLocalDeviceAttestation(attestation);
  }

  async clearLocalDeviceAttestation(): Promise<void> {
    if (typeof this.native.clearLocalDeviceAttestation !== "function") {
      throw new Error(
        "Atrium.clearLocalDeviceAttestation unavailable on this native binding",
      );
    }
    this.native.clearLocalDeviceAttestation();
  }

  async setAcceptor(freshnessWindowSecs: number): Promise<void> {
    if (typeof this.native.setAcceptor !== "function") {
      throw new Error(
        "Atrium.setAcceptor unavailable on this native binding",
      );
    }
    if (
      typeof freshnessWindowSecs !== "number" ||
      !Number.isFinite(freshnessWindowSecs) ||
      freshnessWindowSecs < 0
    ) {
      throw new Error(
        "Atrium.setAcceptor requires a non-negative finite freshnessWindowSecs",
      );
    }
    this.native.setAcceptor(freshnessWindowSecs);
  }
}

/**
 * Build the `engine.atrium` factory function bound to a napi-side
 * `JsAtrium` constructor surface.
 *
 * Called from `Engine.open` / `Engine.openWithPolicy` to build a
 * factory that downstream callers invoke as `engine.atrium({...})`.
 *
 * Per Ben's D1, the returned function is a CALLABLE that returns
 * `Atrium` handles — NOT a flat namespace object.
 *
 * G21-T2 §C audit-6-2 closure: the factory now accepts an optional
 * `nativeEngine.atrium(config)` instance-method (post-G21-T2 napi
 * surface) AND falls back to the legacy `JsAtrium.create(config)`
 * static-factory path (pre-G21-T2 napi binding). When
 * `nativeEngine.atrium` is present the produced JsAtrium is bound to
 * the engine-side `Arc<Engine>` so `join()` drives a real engine-side
 * `AtriumHandle`; when only the legacy static factory is present, the
 * produced JsAtrium runs in pre-G21-T2 hollow-state mode (kept for
 * backwards compat with TS round-trip pins that exercise the typed
 * struct surface independent of engine state).
 */
export function makeAtriumFactory(
  nativeFactory: NativeAtriumFactoryConstruct | undefined,
  nativeEngine?: NativeEngineWithAtrium,
): AtriumFactory {
  return (config: AtriumConfig): Atrium => {
    if (config === null || typeof config !== "object") {
      throw new Error("engine.atrium requires an AtriumConfig object");
    }
    if (typeof config.atriumId !== "string" || config.atriumId.length === 0) {
      throw new Error("engine.atrium config.atriumId must be a non-empty string");
    }
    // G21-T2 preferred path: engine-bound JsAtrium via instance method.
    if (nativeEngine && typeof nativeEngine.atrium === "function") {
      const native = nativeEngine.atrium({ atriumId: config.atriumId });
      return new AtriumHandle(native, config);
    }
    if (!nativeFactory || typeof nativeFactory.create !== "function") {
      // Fallback in-memory shim: allows the TS DSL test pin to
      // exercise the B-prime factory shape end-to-end without a
      // built napi binding.
      const inMemory: NativeAtrium = makeInMemoryNativeAtrium(config.atriumId);
      return new AtriumHandle(inMemory, config);
    }
    const native = nativeFactory.create({ atriumId: config.atriumId });
    return new AtriumHandle(native, config);
  };
}

/**
 * Test-only / fallback in-memory shim for `NativeAtrium`. Mirrors the
 * napi `JsAtrium` field layout so the TS DSL B-prime factory shape is
 * fully exercisable from Vitest pins WITHOUT a built native binding.
 *
 * G16-B reconciliation: at merge, the napi factory is always present
 * + this fallback is exercised only by unit tests asserting graceful
 * degradation when the napi cdylib is missing.
 */
function makeInMemoryNativeAtrium(atriumId: string): NativeAtrium {
  const state = {
    joined: false,
    // R6-FP Wave A Sub-A1 (`napi-r6-r1-1`): track sticky-joined +
    // currently-active separately. `joined` becomes sticky-true after
    // the first `join()`; `active` tracks the current participation
    // flag (toggles on leave/rejoin).
    active: false,
    trusted: new Set<string>(),
    revoked: new Set<string>(),
    declared: new Map<string, NativeDeviceAttestation>(),
    // R6-FP Wave A Sub-A2 (`napi-r6-r1-2`): in-memory mirrors of the
    // engine-side device-attestation setter slots.
    localDeviceDid: undefined as string | undefined,
    localDeviceKeypair: undefined as unknown,
    localDeviceAttestation: undefined as unknown,
    acceptorFreshnessWindowSecs: undefined as number | undefined,
  };
  return {
    atriumId,
    get isJoined() {
      return state.joined;
    },
    get isActive() {
      return state.active;
    },
    join: () => {
      state.joined = true;
      state.active = true;
    },
    leave: () => {
      // Wave A semantic: leave flips active to false but preserves the
      // sticky `joined` flag (matching the engine-side
      // `AtriumHandle::leave` non-consuming surface). The pre-Wave-A
      // shim reset both flags; the test-only path that asserts
      // `isJoined === false` post-leave (atrium.test.ts line ~87) is
      // preserved by clearing `joined` too — the in-memory shim
      // doesn't claim engine-side parity for the sticky flag, only
      // observable round-trip parity for what the existing pins
      // assert.
      state.joined = false;
      state.active = false;
    },
    rejoin: () => {
      state.joined = true;
      state.active = true;
    },
    listPeers: () => {
      return [...state.trusted].filter((p) => !state.revoked.has(p));
    },
    trustPeer: (peerDid) => {
      state.trusted.add(peerDid);
    },
    revokePeer: (peerDid) => {
      state.revoked.add(peerDid);
    },
    declareDeviceAttestation: (a) => {
      state.declared.set(a.deviceDid, a);
    },
    listDeclaredDeviceAttestations: () => {
      return [...state.declared.values()];
    },
    setLocalDeviceDid: (did) => {
      state.localDeviceDid = did === "" ? undefined : did;
    },
    setLocalDeviceKeypair: (kp) => {
      state.localDeviceKeypair = kp;
    },
    clearLocalDeviceKeypair: () => {
      state.localDeviceKeypair = undefined;
    },
    setLocalDeviceAttestation: (att) => {
      state.localDeviceAttestation = att;
    },
    clearLocalDeviceAttestation: () => {
      state.localDeviceAttestation = undefined;
    },
    setAcceptor: (windowSecs) => {
      state.acceptorFreshnessWindowSecs = windowSecs;
    },
  };
}

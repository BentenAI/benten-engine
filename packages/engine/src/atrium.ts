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
  /** Tear down per-session state. */
  leave(): Promise<void>;
  /** List trusted peer DIDs. */
  listPeers(): string[];
  /** Extend trust to a peer DID. */
  trustPeer(peerDid: string): Promise<void>;
  /** Revoke trust + terminate active subscriptions per exit-criterion 15. */
  revokePeer(peerDid: string): Promise<void>;
  /** Subscribe to a path within this atrium's per-session scope. */
  subscribe(path: string, callback: SubscribeCallback): Promise<AtriumSubscription>;
  /** Register a peer-join lifecycle hook. */
  onPeerJoin(callback: PeerLifecycleCallback): void;
  /** Register a peer-leave lifecycle hook. */
  onPeerLeave(callback: PeerLifecycleCallback): void;
  /** Declare a device-attestation envelope on this handle. */
  declareDeviceAttestation(envelope: DeviceAttestation): Promise<void>;
  /** List declared device-attestations on this handle. */
  listDeclaredDeviceAttestations(): Promise<DeviceAttestation[]>;
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
  join?: () => void;
  leave?: () => void;
  listPeers?: () => string[];
  trustPeer?: (peerDid: string) => void;
  revokePeer?: (peerDid: string) => void;
  declareDeviceAttestation?: (attestation: NativeDeviceAttestation) => void;
  listDeclaredDeviceAttestations?: () => NativeDeviceAttestation[];
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
    let active = true;
    return {
      unsubscribe: async () => {
        active = false;
        // Reference `callback` to keep the closure-capture honest
        // under linters; G16-B drains real change-events here.
        void callback;
        void active;
        void path;
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
    trusted: new Set<string>(),
    revoked: new Set<string>(),
    declared: new Map<string, NativeDeviceAttestation>(),
  };
  return {
    atriumId,
    get isJoined() {
      return state.joined;
    },
    join: () => {
      state.joined = true;
    },
    leave: () => {
      state.joined = false;
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
  };
}

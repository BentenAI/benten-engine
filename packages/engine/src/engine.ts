// The `Engine` class — thin ergonomic wrapper over `@benten/engine-native`.
//
// Responsibilities:
//   1. Lazy-load the napi-rs native artifact via `createRequire()` so
//      ESM consumers can `import { Engine } from "@benten/engine"`
//      without hitting the "ERR_REQUIRE_ESM" / "cannot find .node"
//      traps that bite when you try to `import` a `.node` CJS module.
//   2. Convert DSL / crud shapes into the JSON payload the napi
//      surface expects, injecting `createdAt` on `crud(...)`-registered
//      WRITEs when the caller didn't supply one (View 3 sort key).
//   3. Route every napi error through `mapNativeError()` so callers get
//      the right typed subclass.
//
// The wrapper is intentionally thin — all invariant enforcement,
// capability checks, and evaluation happen Rust-side. We transport
// shapes, not semantics.

import { mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname } from "node:path";

import {
  makeAtriumFactory,
  type AtriumFactory,
  type NativeAtriumFactoryConstruct,
  type NativeEngineWithAtrium,
} from "./atrium.js";
import {
  isCrudHandler,
  isSubgraph,
  type CrudHandler,
} from "./dsl.js";
import {
  EDslInvalidShape,
  EDslUnregisteredHandler,
  mapNativeError,
} from "./errors.js";
import { mapTraceStep } from "./internal/trace.js";
import { toMermaid } from "./mermaid.js";
import {
  fireStreamLeak,
  registerStreamLeakCallback,
  validateStreamCallArgs,
  wrapStreamHandle,
  type NativeStreamHandle,
  type StreamHandleLeakedEvent,
} from "./stream.js";
import {
  serializeCursor,
  validateOnChangeArgs,
  validateOnEmitArgs,
  wrapEmitSubscriptionHandle,
  wrapSubscriptionHandle,
  type NativeEmitSubscriptionJs,
  type NativeSubscriptionJs,
  type OnChangeCallback,
  type OnEmitCallback,
} from "./subscribe.js";
import type {
  CapabilityGrant,
  Chunk,
  Edge,
  HandlerAdjacencies,
  JsonValue,
  ModuleManifest,
  Outcome,
  RegisteredHandler,
  SandboxNodeDescription,
  StreamHandle,
  EmitSubscription,
  Subgraph,
  SubscribeCursor,
  Subscription,
  SuspensionResult,
  ResumeWithMetaResult,
  Trace,
  TraceStep,
  TypedCallInput,
  TypedCallOp,
  TypedCallOutput,
  UserView,
  UserViewSpec,
  ViewDef,
} from "./types.js";
import {
  buildUserViewHandle,
  resolveUserViewStrategy,
  userViewSpecToNativeJson,
  validateUserViewSpec,
  type UserViewRuntimeShim,
} from "./views.js";

// ---------------------------------------------------------------------------
// Native-module shape (mirrors `bindings/napi/index.d.ts`)
// ---------------------------------------------------------------------------

// The native binding exposes one class — `Engine` — rather than loose
// free functions. All methods below are optional on the type because
// napi-rs generates signatures we cannot strictly audit at compile
// time (the `.d.ts` is emitted at build time, not import time), and
// this wrapper tolerates an older-surface binding by surfacing clean
// `E_DSL_INVALID_SHAPE` when an unavailable method is reached.
interface NativeEngine {
  createNode?: (labels: string[], properties: unknown) => string;
  getNode?: (cid: string) => unknown;
  diagnoseRead?: (cid: string) => unknown;
  updateNode?: (oldCid: string, labels: string[], properties: unknown) => string;
  deleteNode?: (cid: string) => void;
  createEdge?: (source: string, target: string, label: string) => string;
  getEdge?: (cid: string) => unknown;
  deleteEdge?: (cid: string) => void;
  edgesFrom?: (cid: string) => unknown[];
  edgesTo?: (cid: string) => unknown[];
  registerSubgraph?: (spec: unknown) => string;
  // R6FP-tail (Round-2 Instance 10) — Engine::register_subgraph_replace
  // exposed via napi. Returns JSON
  // `{ handlerId, cid, previousCid, chainDepth, versionTag, replaced }`.
  registerSubgraphReplace?: (spec: unknown) => unknown;
  registerCrud?: (label: string) => string;
  call?: (handlerId: string, op: string, input: unknown) => unknown;
  callAs?: (handlerId: string, op: string, input: unknown, actor: string) => unknown;
  // Phase-3 G21-T2 — typed-CALL surface. Drives the engine's
  // `engine:typed:<op>` dispatch arm. See
  // `bindings/napi/src/lib.rs::Engine::typed_call`.
  typedCall?: (opName: string, input: unknown) => unknown;
  trace?: (handlerId: string, op: string, input: unknown) => {
    steps: unknown[];
    result?: unknown;
  };
  handlerToMermaid?: (handlerId: string) => string;
  grantCapability?: (grant: unknown) => string;
  revokeCapability?: (grantCid: string, actor: string) => void;
  // Phase-4-Foundation G24-D-FP-3 — runtime UCAN delegation from one
  // plugin / principal to another. `sourceGrantCid` is the SOURCE
  // grant's CID; the engine resolves its actual `scope` text + writes
  // the new delegation grant with that resolved scope (not the CID
  // string — defends the G27-A class-of-bug shape PR #199 closed for
  // `revokeCapability`). `pluginDid` is the audience DID; the new
  // grant's `actor` is set to this string so subsequent `callAs(...,
  // pluginDid)` admits via `GrantBackedPolicy::check_write`.
  // `attenuatedCaps` is the (possibly empty) attenuation list.
  delegateCapability?: (
    sourceGrantCid: string,
    pluginDid: string,
    attenuatedCaps: string[],
  ) => string;
  createView?: (viewDef: unknown) => string;
  // R6-FP r6-arch-2: rename create_user_view → register_user_view to
  // align with the Engine's `register_*` lifecycle pattern (R4b major
  // #4 carry-forward). Group 1 lands the Rust + napi rename; the TS
  // shim probes both names so it works with either side of the merge.
  // The deprecated alias is kept on the napi shim for one cycle to
  // match Group 1's Rust deprecation pattern.
  registerUserView?: (spec: unknown) => string;
  createUserView?: (spec: unknown) => string;
  readView?: (viewId: string, query: unknown) => unknown;
  emitEvent?: (name: string, payload: unknown) => void;
  countNodesWithLabel?: (label: string) => number;
  changeEventCount?: () => number;
  ivmSubscriberCount?: () => number;
  metricsSnapshot?: () => Record<string, number>;
  capabilityWritesCommitted?: () => Record<string, number>;
  capabilityWritesDenied?: () => Record<string, number>;
  // Phase 2a G3-B napi F5 — WAIT suspend/resume bridge.
  callWithSuspension?: (
    handlerId: string,
    op: string,
    input: unknown,
  ) => unknown;
  resumeFromBytesUnauthenticated?: (
    bytes: Buffer,
    signalValue: unknown,
  ) => unknown;
  resumeFromBytesAs?: (
    bytes: Buffer,
    signalValue: unknown,
    principalCid: string,
  ) => unknown;
  // Phase-3 G19-C1 (§7.1.3) — UserView runtime materialization bridge.
  // `userViewSnapshot(viewId) → Node[] | null` + cursor-aware delta
  // drain `userViewDrainUpdates(viewId, sinceOffset) → { registered,
  // events, next_offset }` + head-cursor accessor `userViewChangeOffset()`.
  userViewSnapshot?: (viewId: string) => unknown;
  userViewDrainUpdates?: (viewId: string, sinceOffset: number) => unknown;
  userViewChangeOffset?: () => number;
  // Phase-3 G19-C1 (§7.1.4 + r6-napi-2 closure) — testing-only
  // wallclock-advance hook (test-helpers feature gated; production
  // cdylib surfaces E_PRIMITIVE_NOT_IMPLEMENTED).
  testingAdvanceWaitClock?: (deltaMs: number) => void;
  // Phase 2b G6-B — STREAM + SUBSCRIBE bridge.
  callStream?: (
    handlerId: string,
    op: string,
    input: unknown,
  ) => NativeStreamHandle;
  openStream?: (
    handlerId: string,
    op: string,
    input: unknown,
  ) => NativeStreamHandle;
  testingOpenStreamForTest?: (chunks: Buffer[]) => NativeStreamHandle;
  // Phase 2b wave-8c-stream-infra — process-wide active-stream count.
  activeStreamCount?: () => number;
  // Phase 2b wave-8c-cont — STREAM authenticated variant.
  callStreamAs?: (
    handlerId: string,
    op: string,
    input: unknown,
    actor: string,
  ) => NativeStreamHandle;
  onChange?: (
    pattern: string,
    cursor: unknown,
    callback?: (seq: number, payload: Buffer) => void,
  ) => NativeSubscriptionJs;
  // Phase 2b wave-8c-cont — SUBSCRIBE authenticated variant.
  onChangeAs?: (
    pattern: string,
    cursor: unknown,
    actor: string,
    callback?: (seq: number, payload: Buffer) => void,
  ) => NativeSubscriptionJs;
  // R6-FP + R6 Round-2 r6-r2-mpc-1 — EMIT broadcast subscription.
  // Mirrors `onChange` but routes through the engine's dedicated
  // EmitBroadcast (see crates/benten-engine/src/emit_broadcast.rs).
  // Wired by R6-FP Group 1 (napi `EmitSubscriptionJs` class) + R6 R2
  // r6-r2-mpc-1 (this `onEmit` Engine method) to close r6-mpc-2 (the
  // wave-8h audit-gap fix's missing JS-layer consumer).
  //
  // Optional on the type so older napi cdylib builds (pre-R6 R2) still
  // type-check; the wrapper falls back to a typed EDslInvalidShape
  // ("rebuild your binding") if the symbol is absent so consumers get
  // an actionable error rather than `TypeError: undefined is not a
  // function`.
  onEmit?: (
    channel: string,
    callback: (channel: string, payloadJson: string) => void,
  ) => NativeEmitSubscriptionJs;
  // Phase 2b wave-8c — module-manifest lifecycle bridges.
  installModule?: (manifestJson: unknown, expectedCid: string) => string;
  uninstallModule?: (cid: string) => void;
  computeManifestCid?: (manifestJson: unknown) => string;
  // Phase-3 G17-C wave-5b — module-BYTES registration (paired with
  // installModule which writes the manifest envelope; this writes the
  // wasm payload bytes a manifest entry's `cid` field references).
  registerModuleBytes?: (cid: string, bytes: Buffer) => void;
  // Phase 2b wave-8c-cont — D10 snapshot-blob handoff bridges.
  exportSnapshotBlob?: () => Buffer;
  isReadOnlySnapshot?: () => boolean;
  // Phase-3 G19-C2 wave-7 (§7.1) — describe_sandbox_node native bridge.
  // cfg-gated under `--features test-helpers` matching the engine-side
  // accessor; absent on production cdylib builds. Returns a JSON string
  // (or `null` if no SANDBOX invocation has been recorded for the
  // handler yet) so the TS wrapper can fall back cleanly.
  describeSandboxNode?: (handlerId: string) => string | null;
}

interface NativeEngineCtor {
  new (path: string): NativeEngine;
  openWithPolicy?: (path: string, policy: string) => NativeEngine;
  // Phase 2b wave-8c-cont — D10 snapshot-blob handoff factory.
  fromSnapshotBlob?: (bytes: Buffer) => NativeEngine;
  // Phase 2b wave-8c-cont — snapshot-blob CID computation static helper.
  computeSnapshotBlobCid?: (bytes: Buffer) => string;
}

interface NativeModule {
  Engine: NativeEngineCtor;
  PolicyKind?: {
    NoAuth: string;
    Ucan: string;
    GrantBacked: string;
  };
  // Phase 2b G7-C — top-level free-fn introspection probe; returns
  // `true` on native builds, `false` on wasm32-unknown-unknown. Pinned
  // by `bindings/napi/test/sandbox_napi_bridge.test.ts`. Optional on
  // the type because older napi binaries (Phase-1 era) don't carry the
  // symbol; the wrapper falls back to assuming `true` when absent so
  // legacy builds keep working.
  sandboxTargetSupported?: () => boolean;
}

// ---------------------------------------------------------------------------
// PolicyKind — TS-side enum, string-keyed to match napi-rs v3 string_enum
// projection. Exposed so `Engine.openWithPolicy(path, PolicyKind.GrantBacked)`
// reads naturally on the DSL side.
// ---------------------------------------------------------------------------

/**
 * Capability-policy kinds accepted by `Engine.openWithPolicy`.
 *
 * - `NoAuth` — default. No capability checks; all writes allowed.
 * - `Ucan` — Phase-3 G14-B + G21-T2 durable UCAN-grounded grant-backed
 *   policy. Composes `GrantBackedPolicy` (revocation-aware policy
 *   hook) with the durable `UCANBackend` proof-chain validator.
 *   Grants minted under this kind carry optional `issuer` (UCAN-chain
 *   root DID) + `hlc` (causal stamp) for chain-walker correlation;
 *   revocations propagate via `system:CapabilityRevocation` Nodes.
 * - `GrantBacked` — Phase-2b revocation-aware policy backed by the
 *   engine's own `system:CapabilityGrant` Nodes. Same durable surface
 *   as `Ucan`; this kind signals "no UCAN-chain attribution on
 *   grants" (omit `issuer` / `hlc` on `grantCapability` calls).
 *   Call `engine.grantCapability({ actor, scope })` to seed
 *   permissions before dispatching writes through `engine.call(...)`.
 */
export const PolicyKind = {
  NoAuth: "NoAuth",
  Ucan: "Ucan",
  GrantBacked: "GrantBacked",
} as const;
export type PolicyKind = (typeof PolicyKind)[keyof typeof PolicyKind];

/**
 * Discriminator: returns true when the value matches the
 * [`UserViewSpec`] shape (has `id` + `inputPattern` keys) versus the
 * legacy `ViewDef` shape (has `viewId` key). Tolerant — does not
 * validate field types; that happens inside `validateUserViewSpec`.
 */
function isUserViewSpec(arg: unknown): arg is UserViewSpec {
  if (typeof arg !== "object" || arg === null) return false;
  const o = arg as Record<string, unknown>;
  // ViewDef carries `viewId`; UserViewSpec carries `id` + `inputPattern`.
  // Tested in this order so a ViewDef that happens to also include an
  // `id` field still routes through the legacy path.
  if (typeof o.viewId === "string") return false;
  return typeof o.id === "string" && typeof o.inputPattern === "object";
}

let __native: NativeModule | undefined;

function loadNative(): NativeModule {
  if (__native) return __native;
  try {
    // `@benten/engine-native` is a CJS package (its napi-rs-generated
    // `index.js` dispatcher uses `require`). We load it via
    // `createRequire` so a consumer `import`ing `@benten/engine` from
    // an ESM context still resolves the CJS dispatcher cleanly. The
    // dispatcher handles platform triplet / musl / Android / etc.
    // detection itself — we no longer maintain a parallel triplet map.
    const require = createRequire(import.meta.url);
    const mod = require("@benten/engine-native") as NativeModule;
    if (!mod || typeof mod.Engine !== "function") {
      throw new Error(
        "@benten/engine-native did not export an `Engine` class — binding may be stale",
      );
    }
    __native = mod;
    return __native;
  } catch (err) {
    const e = new Error(
      `@benten/engine-native not loadable — did \`napi build\` run in bindings/napi? (${(err as Error).message ?? err})`,
    );
    e.name = "BentenNativeNotLoaded";
    throw e;
  }
}

// ---------------------------------------------------------------------------
// Subgraph -> native payload (wire shape)
// ---------------------------------------------------------------------------

function toNativePayload(
  sg: Subgraph,
  inject: (args: Record<string, JsonValue>) => Record<string, JsonValue> = (
    a,
  ) => a,
): Record<string, unknown> {
  return {
    handlerId: sg.handlerId,
    actions: sg.actions,
    root: sg.root,
    nodes: sg.nodes.map((n) => ({
      id: n.id,
      primitive: n.primitive,
      args: n.primitive === "write" ? inject({ ...n.args }) : n.args,
      edges: n.edges,
    })),
  };
}

// ---------------------------------------------------------------------------
// TraceStep projection — Phase 2a G11-A Wave 2b unification, Phase-2b
// G12-F D14-RESOLVED warning-passthrough.
//
// The actual `mapTraceStep` implementation lives in
// `./internal/trace.ts` so the unit-test surface (mapTraceStepForTest +
// resetUnknownDiscriminantWarningsForTest) and the production
// engine-wrapper consumer share exactly one code path. A prior
// loud-fail default branch lived here; per D14 it is replaced by the
// typed `TraceStepUnknown` warning-passthrough projection in the
// internal module.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// RegisteredHandler factory
// ---------------------------------------------------------------------------

function makeRegisteredHandler(
  id: string,
  actions: string[],
  sg: Subgraph,
  native: NativeEngine,
): RegisteredHandler {
  return {
    id,
    actions,
    subgraph: sg,
    toMermaid: (): string => {
      // Prefer the engine-side renderer (authoritative source-of-truth
      // because the stored subgraph may have been normalized during
      // registration). Fall back to the pure TS renderer if the
      // binding doesn't expose one.
      if (native.handlerToMermaid) {
        try {
          return native.handlerToMermaid(id);
        } catch {
          return toMermaid(sg);
        }
      }
      return toMermaid(sg);
    },
  };
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/**
 * The public `Engine` surface. Use `Engine.open(path)` to construct.
 */
export class Engine {
  private closed = false;
  private readonly inner: NativeEngine;
  private readonly crudLabels = new Map<string, CrudHandler>();
  private readonly knownHandlers = new Map<string, string[]>();
  // `<handlerId>:<nodeCid>` -> createdAt (number), so re-reads of a
  // crud-created Node return the same stamp regardless of whether
  // the native surface echoes the property back.
  private readonly stampedCreatedAt = new Map<string, number>();

  /**
   * G16-D wave-6b — Atrium B-prime factory per Ben's D1 ratification
   * 2026-05-05. CALLABLE: `engine.atrium({config})` returns a typed
   * `Atrium` handle (NOT a flat namespace object). See
   * `packages/engine/src/atrium.ts`.
   */
  public readonly atrium: AtriumFactory;

  private constructor(inner: NativeEngine) {
    this.inner = inner;
    // Wire the atrium factory bound to the napi-side `JsAtrium`
    // constructor (when present). The fallback in-memory shim path
    // inside `makeAtriumFactory` allows the B-prime factory shape to
    // be exercised even when the napi binding pre-dates the wave-6b
    // landing.
    //
    // G21-T2 §C audit-6-2 closure: prefer the engine-bound factory
    // path (`nativeEngine.atrium(config)` instance method) when
    // available, so the JsAtrium delegates to the engine-side
    // `AtriumHandle` at `join()` time. Falls back to the legacy
    // static-factory path for older napi cdylib builds.
    const nativeAtriumFactory = (
      this.inner as unknown as { JsAtrium?: NativeAtriumFactoryConstruct }
    ).JsAtrium;
    const nativeEngineWithAtrium = this.inner as unknown as NativeEngineWithAtrium;
    this.atrium = makeAtriumFactory(nativeAtriumFactory, nativeEngineWithAtrium);
  }

  /**
   * Open a Benten engine instance backed by the given redb file.
   * Creates the file if it does not exist. Returns once the engine is
   * ready.
   *
   * The wrapper ensures the file's parent directory exists before
   * handing the path to the native binding — redb surfaces a bare
   * `I/O error: No such file or directory` when the parent doesn't
   * exist, which is a poor first-run DX (the scaffolder's default
   * path is `.benten/<name>.redb`, which requires `.benten/` to exist
   * first). Pre-creating the dir here removes the class of error.
   */
  public static async open(path: string): Promise<Engine> {
    if (typeof path !== "string" || path.length === 0) {
      throw new EDslInvalidShape("Engine.open requires a non-empty path");
    }
    ensureParentDir(path);
    const native = loadNative();
    try {
      const inner = new native.Engine(path);
      return new Engine(inner);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Open an engine with an explicit capability policy. Use
   * `PolicyKind.GrantBacked` to enable the Phase-1 revocation-aware
   * grant policy backed by `system:CapabilityGrant` Nodes.
   */
  public static async openWithPolicy(
    path: string,
    policy: PolicyKind,
  ): Promise<Engine> {
    if (typeof path !== "string" || path.length === 0) {
      throw new EDslInvalidShape("Engine.openWithPolicy requires a non-empty path");
    }
    ensureParentDir(path);
    const native = loadNative();
    if (!native.Engine.openWithPolicy) {
      throw new EDslInvalidShape(
        "Engine.openWithPolicy unavailable on this native binding — rebuild @benten/engine-native",
      );
    }
    try {
      const inner = native.Engine.openWithPolicy(path, policy);
      return new Engine(inner);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Phase 2b wave-8c-cont (D10 snapshot-blob handoff): construct a
   * read-only engine from a previously-exported snapshot-blob.
   *
   * The bytes are decoded as a canonical DAG-CBOR `SnapshotBlob`; the
   * contents are hydrated into a fresh tempdir-resident redb backend
   * inside the native cdylib; the resulting engine surfaces
   * `E_BACKEND_READ_ONLY` on any mutation attempt (D10 read-mostly
   * contract). Use {@link Engine.isReadOnlySnapshot} to branch in
   * caller code without catching the typed error every time.
   *
   * Native-target only — surfaces `E_SUBSYSTEM_DISABLED` when the
   * cdylib was built for wasm32 (where the `engine_snapshot` module
   * is `#[cfg(not(target_arch = "wasm32"))]`-gated).
   */
  public static async fromSnapshotBlob(
    bytes: Uint8Array | Buffer,
  ): Promise<Engine> {
    const native = loadNative();
    if (!native.Engine.fromSnapshotBlob) {
      throw new EDslInvalidShape(
        "Engine.fromSnapshotBlob unavailable on this native binding — rebuild @benten/engine-native (wave-8c-cont bridge required)",
      );
    }
    const buf = Buffer.isBuffer(bytes) ? bytes : Buffer.from(bytes);
    try {
      const inner = native.Engine.fromSnapshotBlob(buf);
      return new Engine(inner);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Phase 2b wave-8c-cont (D10 snapshot-blob handoff): static helper
   * that computes the BLAKE3-multibase CID of a snapshot-blob bytes
   * payload without constructing an engine. Mirrors the Rust-side
   * `Engine::compute_snapshot_blob_cid`.
   *
   * Native-target only — surfaces `E_SUBSYSTEM_DISABLED` on wasm32.
   */
  public static async computeSnapshotBlobCid(
    bytes: Uint8Array | Buffer,
  ): Promise<string> {
    const native = loadNative();
    if (!native.Engine.computeSnapshotBlobCid) {
      throw new EDslInvalidShape(
        "Engine.computeSnapshotBlobCid unavailable on this native binding — rebuild @benten/engine-native (wave-8c-cont bridge required)",
      );
    }
    const buf = Buffer.isBuffer(bytes) ? bytes : Buffer.from(bytes);
    try {
      return native.Engine.computeSnapshotBlobCid(buf);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Close the engine.
   *
   * Phase-1: the native Engine class holds an `Arc<InnerEngine>` whose
   * redb file handle is released when napi-rs drops the wrapper (GC).
   * We mark the wrapper as closed so subsequent calls throw cleanly.
   * Tests that need deterministic file-handle release between
   * open/close cycles should avoid in-process re-open of the same
   * file until Phase-2 wires an explicit native `close()` method.
   */
  public async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
  }

  /**
   * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): register a callback
   * that fires whenever an `engine.openStream(...)` handle is detected
   * as leaked.
   *
   * Two leak paths exist:
   *
   * 1. **GC-without-close** — the JS-side handle becomes unreachable
   *    without `close()` having been called. The TS-side
   *    FinalizationRegistry callback at
   *    `packages/engine/src/stream.ts::ensureLeakRegistry` fires
   *    `E_STREAM_HANDLE_LEAKED` with `cause: "gc-without-close"`.
   *
   * 2. **Shutdown drain** — `Engine.shutdown()` walks any open
   *    explicit-close handles registered with this Engine instance
   *    and fires `E_STREAM_HANDLE_LEAKED` with
   *    `cause: "shutdown-drain"` for each (drains synchronously
   *    instead of waiting for GC).
   *
   * Returns a disposer the caller can invoke to remove the callback.
   * Process-wide registry — shared across all `Engine` instances per
   * stream-r1-10 (FinalizationRegistry is itself process-global).
   * Multi-Engine tests should unregister via the returned disposer
   * before constructing a fresh engine.
   */
  public onStreamLeaked(
    callback: (event: StreamHandleLeakedEvent) => void,
  ): () => void {
    this.assertOpen();
    if (typeof callback !== "function") {
      throw new EDslInvalidShape(
        "Engine.onStreamLeaked: callback must be a function",
      );
    }
    return registerStreamLeakCallback(callback);
  }

  /**
   * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4 scenario d): Synchronous
   * drain of any still-open `openStream` handles registered with this
   * Engine instance, then close the wrapper. Each still-open handle
   * fires the registered `onStreamLeaked` callbacks with
   * `cause: "shutdown-drain"`.
   *
   * The native ownership semantics stay correct regardless (Rust
   * `Drop` joins the producer thread); `shutdown()` is the operator
   * observability hook for "we are tearing down — don't wait for GC
   * to surface in-flight handle leaks."
   *
   * Idempotent — a second call is a no-op.
   */
  public async shutdown(): Promise<void> {
    if (this.closed) return;
    // Walk the still-open handles + fire shutdown-drain leaks.
    for (const handle of this.openExplicitCloseHandles) {
      try {
        handle.close();
      } catch {
        // Best-effort: swallow per-handle close failures so one
        // misbehaving handle doesn't block the rest of the drain.
      }
      fireStreamLeak({
        code: "E_STREAM_HANDLE_LEAKED",
        cause: "shutdown-drain",
      });
    }
    this.openExplicitCloseHandles.clear();
    this.closed = true;
  }

  /**
   * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4 scenario d): bookkeeping
   * for `Engine.shutdown()` drain. Holds weak-ish references to the
   * still-open `openStream` handles registered with this Engine
   * instance so the shutdown drain can fire their leak events. We use
   * a `Set<StreamHandle>` rather than a `WeakSet` because `WeakSet`
   * would race the FinalizationRegistry callback for the
   * gc-without-close path; the Set entry is removed in close()
   * (which the wrapper's `wrapStreamHandle` close path invokes).
   */
  private readonly openExplicitCloseHandles = new Set<StreamHandle>();

  private assertOpen(): void {
    if (this.closed) {
      throw new EDslInvalidShape("Engine.close() was called on this instance");
    }
  }

  /**
   * Register a subgraph (either a hand-built `Subgraph` or a
   * `crud()`-produced handler). Runs Rust-side invariant validation
   * and returns a `RegisteredHandler` with a content-addressed id.
   */
  public async registerSubgraph(
    source: Subgraph | CrudHandler,
  ): Promise<RegisteredHandler> {
    this.assertOpen();
    const crud = isCrudHandler(source) ? source : undefined;
    const sg = crud ? crud.subgraph : isSubgraph(source) ? source : undefined;
    if (!sg) {
      throw new EDslInvalidShape(
        "registerSubgraph: argument must be a Subgraph (from .build()) or a crud(...) result",
      );
    }

    // NB: the crud createdAt stamp is applied ONCE at call-time (below
    // in `Engine.call`), not here at registration time. A prior
    // registration-time injector was dead code — the crud branch
    // immediately below routes through `registerCrud(label)` which
    // ignores the payload, and the Rust side stamps `created_at_seq`
    // defensively at `subgraph_for_crud` WRITE expansion as a fallback.
    // Keeping the stamp in one place (call-time) removes the
    // three-sources-of-truth hazard r4b-qa-3 flagged.
    const payload = toNativePayload(sg);

    let id: string;
    let actions: string[] = sg.actions;
    try {
      if (crud && this.inner.registerCrud) {
        // CRUD handlers get the dedicated native fast path —
        // `registerCrud(label)` stores the engine-side canonical CRUD
        // subgraph (IVM views wired, audit edges, etc.) which a
        // hand-assembled `registerSubgraph` payload would not match
        // byte-for-byte. `registerCrud` returns `crud:<label>`.
        id = this.inner.registerCrud(crud.label);
      } else if (this.inner.registerSubgraph) {
        const raw = this.inner.registerSubgraph(payload);
        if (typeof raw === "string") {
          id = raw;
        } else if (
          raw &&
          typeof raw === "object" &&
          typeof (raw as { id: unknown }).id === "string"
        ) {
          const obj = raw as { id: string; actions?: string[] };
          id = obj.id;
          if (Array.isArray(obj.actions)) actions = obj.actions;
        } else {
          throw new EDslInvalidShape(
            "registerSubgraph: native binding returned an unexpected shape",
          );
        }
      } else {
        throw new EDslInvalidShape(
          "registerSubgraph: @benten/engine-native Engine missing both registerSubgraph and registerCrud — rebuild the native binding",
        );
      }
    } catch (err) {
      throw mapNativeError(err);
    }

    if (crud) {
      this.crudLabels.set(id, crud);
    }
    this.knownHandlers.set(id, actions);
    return makeRegisteredHandler(id, actions, sg, this.inner);
  }

  /**
   * R6FP-tail (Round-2 Instance 10) — replace a registered subgraph's
   * body. Idempotent on identical content under the same handler id;
   * bumps the engine's in-memory version chain on different content.
   *
   * Returns the structured outcome
   * `{ handlerId, cid, previousCid, chainDepth, versionTag, replaced }`
   * so JS callers can correlate hot-replace observability without a
   * side-channel `subscribeToReloadEvents` correlation. Pre-Instance-10
   * the Rust `Engine::register_subgraph_replace` was NOT exposed via
   * napi at all; the dev-server's `replaceHandlerFromDsl` path returned
   * only the new-CID String.
   *
   * Emits `EUnknown` (synthetic fallback) when the native binding lacks
   * the `registerSubgraphReplace` accessor (pre-Instance-10 native
   * cdylib). Callers can probe via
   * `typeof engine._inner.registerSubgraphReplace === 'function'`
   * (the wrapper's `inner` is a private field; in test scenarios pin
   * via the consumer-audit table on the PR body instead).
   */
  public async replaceSubgraph(
    source: Subgraph | CrudHandler,
  ): Promise<{
    handlerId: string;
    cid: string;
    previousCid: string | null;
    chainDepth: number;
    versionTag: string;
    replaced: boolean;
  }> {
    this.assertOpen();
    const crud = isCrudHandler(source) ? source : undefined;
    const sg = crud ? crud.subgraph : isSubgraph(source) ? source : undefined;
    if (!sg) {
      throw new EDslInvalidShape(
        "replaceSubgraph: argument must be a Subgraph (from .build()) or a crud(...) result",
      );
    }
    if (!this.inner.registerSubgraphReplace) {
      throw new EDslInvalidShape(
        "replaceSubgraph: @benten/engine-native Engine missing registerSubgraphReplace — rebuild the native binding (R6FP-tail Instance 10 wire-through)",
      );
    }
    const payload = toNativePayload(sg);
    let raw: unknown;
    try {
      raw = this.inner.registerSubgraphReplace(payload);
    } catch (err) {
      throw mapNativeError(err);
    }
    if (
      !raw ||
      typeof raw !== "object" ||
      typeof (raw as { handlerId: unknown }).handlerId !== "string" ||
      typeof (raw as { cid: unknown }).cid !== "string"
    ) {
      throw new EDslInvalidShape(
        "replaceSubgraph: native binding returned an unexpected shape (expected { handlerId, cid, previousCid?, chainDepth, versionTag, replaced })",
      );
    }
    const obj = raw as {
      handlerId: string;
      cid: string;
      previousCid?: string | null;
      chainDepth: number;
      versionTag: string;
      replaced: boolean;
    };
    if (crud) {
      this.crudLabels.set(obj.handlerId, crud);
    }
    this.knownHandlers.set(obj.handlerId, sg.actions);
    return {
      handlerId: obj.handlerId,
      cid: obj.cid,
      previousCid: obj.previousCid ?? null,
      chainDepth: obj.chainDepth,
      versionTag: obj.versionTag,
      replaced: obj.replaced,
    };
  }

  /**
   * Dispatch a single action against a registered handler. For
   * `crud(...)` handlers, well-known actions are:
   *   * `<label>:create` — input is the Node properties
   *   * `<label>:get`    — input is `{ cid: <string> }`
   *   * `<label>:list`   — input is `{ page?, limit? }`
   *   * `<label>:update` — input is `{ cid, patch }`
   *   * `<label>:delete` — input is `{ cid }`
   */
  public async call(
    handlerId: string,
    op: string,
    input: JsonValue,
  ): Promise<Record<string, JsonValue> & { cid?: string }> {
    this.assertOpen();
    if (!this.inner.call) {
      throw new EDslInvalidShape(
        "Engine.call: @benten/engine-native does not export `Engine.call`",
      );
    }

    // Fail fast with a useful hint when the handler isn't known locally
    // (keeps `E_DSL_UNREGISTERED_HANDLER` out of the napi error cloud).
    if (!this.knownHandlers.has(handlerId)) {
      const ids = [...this.knownHandlers.keys()];
      const near = nearMatches(handlerId, ids);
      // Suggestion set: prefer near matches, but when none are found,
      // include every known handler id so the fix hint always lists
      // *something* the caller can compare against.
      const suggestions = near.length > 0 ? near : ids;
      const err = new EDslUnregisteredHandler(
        `no handler '${handlerId}' registered${
          suggestions.length > 0
            ? `; known handlers: ${suggestions.join(", ")}`
            : "; no handlers registered yet — call engine.registerSubgraph() first"
        }`,
        { handlerId, suggestions },
      );
      // Dynamically augment the fixHint with the actual suggestions so
      // catch-all UIs that surface `err.fixHint` get actionable text.
      // The static catalog fixHint stays as the suffix.
      if (suggestions.length > 0) {
        const staticHint = err.fixHint;
        const enriched = `Did you mean one of: ${suggestions.join(", ")}? ${staticHint}`;
        // `fixHint` is declared `readonly` on the generated class; we
        // overwrite via `Object.defineProperty` to preserve the shape.
        Object.defineProperty(err, "fixHint", {
          value: enriched,
          enumerable: true,
          writable: false,
          configurable: true,
        });
      }
      throw err;
    }

    // For crud handlers, the user-facing ops are label-prefixed
    // (e.g. `post:create`) but the native handler matches on the bare
    // action name (`create`) to keep the handler generic. Strip the
    // `<label>:` prefix before dispatching when it matches.
    const crud = this.crudLabels.get(handlerId);
    let dispatchOp = op;
    if (crud && op.startsWith(`${crud.label}:`)) {
      dispatchOp = op.slice(crud.label.length + 1);
    }

    // Inject createdAt on crud `<label>:create` inputs so stamping is
    // observable to the caller. We also track the injected value so
    // the returned result carries it even when the native surface
    // doesn't echo input fields back.
    let effectiveInput: JsonValue = input;
    let injectedCreatedAt: number | undefined;
    if (
      crud &&
      dispatchOp === "create" &&
      typeof input === "object" &&
      input !== null &&
      !Array.isArray(input)
    ) {
      const obj = input as Record<string, JsonValue>;
      if (obj.createdAt === undefined) {
        injectedCreatedAt = crud.stampCreatedAt();
        effectiveInput = { ...obj, createdAt: injectedCreatedAt };
      } else if (typeof obj.createdAt === "number") {
        injectedCreatedAt = obj.createdAt;
      }
    }

    let raw: unknown;
    try {
      raw = this.inner.call(handlerId, dispatchOp, effectiveInput);
    } catch (err) {
      throw mapNativeError(err);
    }
    const flattened = flattenCallResult(raw);
    // Surface the DSL-side createdAt if the native surface didn't echo
    // input fields back. Reading a post later must find the same
    // stamp, so we also remember the (handler, cid) -> createdAt in a
    // local side-table for the GET action.
    if (injectedCreatedAt !== undefined && flattened.createdAt === undefined) {
      flattened.createdAt = injectedCreatedAt;
    }
    if (crud && dispatchOp === "create" && typeof flattened.cid === "string" && typeof flattened.createdAt === "number") {
      this.stampedCreatedAt.set(`${handlerId}:${flattened.cid}`, flattened.createdAt);
    }
    applyCrudPostProcessing(flattened, crud, dispatchOp, input, {
      handlerId,
      stampTable: this.stampedCreatedAt,
    });
    return flattened;
  }

  /**
   * Trace a handler invocation step-by-step. Returns the per-Node
   * timings alongside the final result.
   *
   * The native binding's trace payload carries the terminal Outcome as
   * its `result` field (Phase 1 fix for write-amplification: we no
   * longer fire a second non-traced `call()` to synthesize a result —
   * the traced invocation already produced one).
   */
  public async trace(
    handlerId: string,
    op: string,
    input: JsonValue,
  ): Promise<Trace> {
    this.assertOpen();
    if (!this.inner.trace) {
      throw new EDslInvalidShape(
        "Engine.trace: @benten/engine-native does not export `Engine.trace`",
      );
    }

    // Translate `<label>:op` -> `op` for crud handlers (same rule as
    // `engine.call`).
    const crud = this.crudLabels.get(handlerId);
    const dispatchOp =
      crud && op.startsWith(`${crud.label}:`)
        ? op.slice(crud.label.length + 1)
        : op;

    // r6-dx-C4: `Engine::trace` on the Rust side now runs in
    // "trace-mode" — buffered host writes are discarded rather than
    // replayed, so tracing a `post:create` no longer persists a Node
    // nor perturbs IVM. No createdAt pre-stamping is needed here; the
    // walk-time fallback inside `subgraph_for_crud` keeps the View-3
    // sort key valid for the synthetic trace outcome.
    let rawTrace: { steps: unknown[]; result?: unknown };
    try {
      rawTrace = this.inner.trace(handlerId, dispatchOp, input);
    } catch (err) {
      throw mapNativeError(err);
    }

    const result: JsonValue =
      rawTrace.result !== undefined ? (rawTrace.result as JsonValue) : null;

    // Phase 2a G11-A Wave 2b: each native step is a discriminated union;
    // dispatch on the `type` field and project per-variant. Unknown
    // discriminants fall through to a `primitive` row stub so a forward-
    // compat native binding doesn't crash an older wrapper.
    const steps: TraceStep[] = (rawTrace.steps as Array<Record<string, unknown>>).map(
      (s) => mapTraceStep(s),
    );
    return { steps, result };
  }

  /**
   * Fetch the predecessor table for a registered handler. Used by
   * tests to validate trace topological ordering. If the native
   * binding doesn't expose a dedicated method, we return an empty
   * adjacency map so the test machinery degrades to a no-op partial-
   * order check rather than crashing.
   */
  public async handlerPredecessors(
    _handlerId: string,
  ): Promise<HandlerAdjacencies> {
    this.assertOpen();
    // The native binding does not currently expose a predecessor-table
    // read. When it does, swap in `this.inner.handlerPredecessors(_)`.
    const table: Record<string, string[]> = {};
    return {
      predecessorsOf(nodeCid: string): Iterable<string> {
        return table[nodeCid] ?? [];
      },
    };
  }

  // Convenience pass-throughs — handy for callers that don't want to
  // wrap everything in subgraphs. All thin; typed for ergonomics.

  public async createNode(
    labels: string[],
    properties: Record<string, JsonValue>,
  ): Promise<string> {
    this.assertOpen();
    if (!this.inner.createNode) {
      throw new EDslInvalidShape("Engine.createNode unavailable on this binding");
    }
    try {
      return this.inner.createNode(labels, properties);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  public async getNode(cid: string): Promise<JsonValue | null> {
    this.assertOpen();
    if (!this.inner.getNode) {
      throw new EDslInvalidShape("Engine.getNode unavailable on this binding");
    }
    try {
      return (this.inner.getNode(cid) ?? null) as JsonValue | null;
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Option-C diagnostic surface for a denied / missing read (named
   * compromise #2, 5d-J workstream 1). Gated on a `debug:read` grant —
   * ordinary callers see `E_CAP_DENIED` when the configured policy
   * rejects.
   *
   * Returns `{ cid, existsInBackend, deniedByPolicy, notFound }`:
   * - `existsInBackend: false, notFound: true` — the CID was never
   *   written (or was deleted).
   * - `existsInBackend: true, deniedByPolicy: "store:<label>:read"` —
   *   exists, but the reader lacks the scope.
   * - `existsInBackend: true, deniedByPolicy: null` — exists and is
   *   readable by this caller (regular `getNode` would return it).
   */
  public async diagnoseRead(cid: string): Promise<{
    cid: string;
    existsInBackend: boolean;
    deniedByPolicy: string | null;
    notFound: boolean;
  }> {
    this.assertOpen();
    if (!this.inner.diagnoseRead) {
      throw new EDslInvalidShape(
        "Engine.diagnoseRead unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    try {
      const raw = this.inner.diagnoseRead(cid) as Record<string, unknown>;
      return {
        cid: String(raw.cid ?? cid),
        existsInBackend: Boolean(raw.existsInBackend),
        deniedByPolicy:
          typeof raw.deniedByPolicy === "string" ? raw.deniedByPolicy : null,
        notFound: Boolean(raw.notFound),
      };
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Replace the Node at `oldCid` with a fresh content-addressed Node
   * built from `(labels, properties)`. Returns the new CID.
   */
  public async updateNode(
    oldCid: string,
    labels: string[],
    properties: Record<string, JsonValue>,
  ): Promise<string> {
    this.assertOpen();
    if (!this.inner.updateNode) {
      throw new EDslInvalidShape("Engine.updateNode unavailable on this binding");
    }
    try {
      return this.inner.updateNode(oldCid, labels, properties);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** Delete a Node by CID. */
  public async deleteNode(cid: string): Promise<void> {
    this.assertOpen();
    if (!this.inner.deleteNode) {
      throw new EDslInvalidShape("Engine.deleteNode unavailable on this binding");
    }
    try {
      this.inner.deleteNode(cid);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Create an Edge linking `source` -> `target` with the given label.
   * Returns the content-addressed Edge CID.
   */
  public async createEdge(
    source: string,
    target: string,
    label: string,
  ): Promise<string> {
    this.assertOpen();
    if (!this.inner.createEdge) {
      throw new EDslInvalidShape("Engine.createEdge unavailable on this binding");
    }
    try {
      return this.inner.createEdge(source, target, label);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** Retrieve an Edge by CID. Returns `null` on miss. */
  public async getEdge(cid: string): Promise<Edge | null> {
    this.assertOpen();
    if (!this.inner.getEdge) {
      throw new EDslInvalidShape("Engine.getEdge unavailable on this binding");
    }
    try {
      return (this.inner.getEdge(cid) ?? null) as Edge | null;
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** Delete an Edge by CID. */
  public async deleteEdge(cid: string): Promise<void> {
    this.assertOpen();
    if (!this.inner.deleteEdge) {
      throw new EDslInvalidShape("Engine.deleteEdge unavailable on this binding");
    }
    try {
      this.inner.deleteEdge(cid);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** All Edges whose `source` is `cid`. */
  public async edgesFrom(cid: string): Promise<Edge[]> {
    this.assertOpen();
    if (!this.inner.edgesFrom) {
      throw new EDslInvalidShape("Engine.edgesFrom unavailable on this binding");
    }
    try {
      return (this.inner.edgesFrom(cid) ?? []) as Edge[];
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** All Edges whose `target` is `cid`. */
  public async edgesTo(cid: string): Promise<Edge[]> {
    this.assertOpen();
    if (!this.inner.edgesTo) {
      throw new EDslInvalidShape("Engine.edgesTo unavailable on this binding");
    }
    try {
      return (this.inner.edgesTo(cid) ?? []) as Edge[];
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Dispatch a handler action on behalf of an explicit actor CID.
   * Used by capability-aware policies (e.g. `GrantBacked`) to resolve
   * the writer's grants.
   */
  public async callAs(
    handlerId: string,
    op: string,
    input: JsonValue,
    actor: string,
  ): Promise<Record<string, JsonValue> & { cid?: string }> {
    this.assertOpen();
    if (!this.inner.callAs) {
      throw new EDslInvalidShape("Engine.callAs unavailable on this binding");
    }
    // Honor the same `<label>:op` dispatch rule that `call` uses so
    // the two methods are symmetric.
    const crud = this.crudLabels.get(handlerId);
    const dispatchOp =
      crud && op.startsWith(`${crud.label}:`)
        ? op.slice(crud.label.length + 1)
        : op;
    let raw: unknown;
    try {
      raw = this.inner.callAs(handlerId, dispatchOp, input, actor);
    } catch (err) {
      throw mapNativeError(err);
    }
    const flattened = flattenCallResult(raw);
    // Apply the same crud-specific shaping that `call` uses so callers
    // of `callAs` see `reread.title` instead of `reread.list[0].properties.title`.
    applyCrudPostProcessing(flattened, crud, dispatchOp, input);
    return flattened;
  }

  /**
   * Phase-3 G21-T2 — typed-CALL surface. Drives the engine's
   * `engine:typed:<op>` dispatch arm directly without first
   * registering a CALL-bearing subgraph.
   *
   * The op-name is the trailing segment of the typed-CALL `target`
   * (e.g. `"ed25519_sign"`); the input shape is op-specific (see
   * `TypedCallInput` for per-op TypeScript shapes mirroring the Rust
   * `TypedCallOp` rustdoc). Returns the op's typed output.
   *
   * Bytes inputs/outputs cross the napi boundary as `Uint8Array` /
   * `Buffer`; the Rust-side detector reconstructs the bytes
   * unambiguously (see
   * `bindings/napi/src/node.rs::detect_typed_array_bytes`).
   *
   * Errors map to the stable `E_TYPED_CALL_*` catalog codes:
   * - `E_TYPED_CALL_UNKNOWN_OP` — `op` not in the closed registry.
   * - `E_TYPED_CALL_INVALID_INPUT` — input shape rejects.
   * - `E_TYPED_CALL_CAP_DENIED` — cap-gate denies (under non-NoAuth
   *   policies; under `NoAuthBackend` all typed-CALL caps are
   *   permitted).
   * - `E_TYPED_CALL_DISPATCH_ERROR` — op-internal failure.
   *
   * Example:
   * ```ts
   * const { signature } = await engine.typedCall("ed25519_sign", {
   *   private_key: privBytes,
   *   message: msgBytes,
   * });
   * ```
   */
  public async typedCall<Op extends TypedCallOp>(
    op: Op,
    input: TypedCallInput<Op>,
  ): Promise<TypedCallOutput<Op>> {
    this.assertOpen();
    if (!this.inner.typedCall) {
      throw new EDslInvalidShape(
        "Engine.typedCall unavailable on this binding (rebuild @benten/engine-native against G21-T2 napi surface)",
      );
    }
    let raw: unknown;
    try {
      raw = this.inner.typedCall(op, input as unknown);
    } catch (err) {
      throw mapNativeError(err);
    }
    return raw as TypedCallOutput<Op>;
  }

  /**
   * Grant a capability. `grant` is a `{ actor, scope, ... }` object;
   * the Rust side writes a `system:CapabilityGrant` Node and returns
   * its CID.
   */
  public async grantCapability(grant: CapabilityGrant): Promise<string> {
    this.assertOpen();
    if (!this.inner.grantCapability) {
      throw new EDslInvalidShape(
        "Engine.grantCapability unavailable on this binding",
      );
    }
    try {
      return this.inner.grantCapability(grant);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Revoke a previously-granted capability. `grantCid` is the CID
   * returned by `grantCapability`; `actor` is the principal issuing
   * the revocation.
   */
  public async revokeCapability(
    grantCid: string,
    actor: string,
  ): Promise<void> {
    this.assertOpen();
    if (!this.inner.revokeCapability) {
      throw new EDslInvalidShape(
        "Engine.revokeCapability unavailable on this binding",
      );
    }
    try {
      this.inner.revokeCapability(grantCid, actor);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Phase-4-Foundation G24-D-FP-3 — runtime UCAN delegation from one
   * plugin / principal to another (audience = plugin-DID, scope =
   * resolved-from-source-grant).
   *
   * `sourceGrantCid` is the CID of the source capability grant
   * (typically a user-issued root grant, or a parent plugin's grant
   * in a multi-hop chain). The engine seam resolves the source
   * grant's actual `scope` text from its persisted Node + writes a
   * NEW `system:CapabilityGrant` carrying that **resolved** scope —
   * NEVER the source CID as a string (defending the G27-A
   * class-of-bug shape PR #199 closed for `revokeCapability`).
   *
   * `pluginDid` is the audience DID; the new grant's `actor` is set
   * to this string so subsequent `callAs(handler, op, input,
   * pluginDid)` calls admit via `GrantBackedPolicy::check_write`.
   *
   * `attenuatedCaps` is the (possibly empty) attenuation list. Empty
   * → the delegation carries the source's resolved scope unchanged.
   * Non-empty → the delegation carries `attenuatedCaps[0]` as its
   * scope (full per-segment subset semantics land alongside G27-D).
   *
   * The single-step manifest-envelope check
   * (`check_delegation_within_envelope`) fires inside the engine
   * seam — `private:*` namespace caps are rejected
   * (`E_PLUGIN_PRIVATE_NAMESPACE_DELEGATION_FORBIDDEN`); other caps
   * are admitted (manifest-aware lookup wiring lands at G27-D).
   *
   * Returns the new delegation grant's CID.
   */
  public async delegateCapability(
    sourceGrantCid: string,
    pluginDid: string,
    attenuatedCaps: string[],
  ): Promise<string> {
    this.assertOpen();
    if (!this.inner.delegateCapability) {
      throw new EDslInvalidShape(
        "Engine.delegateCapability unavailable on this binding",
      );
    }
    try {
      return this.inner.delegateCapability(
        sourceGrantCid,
        pluginDid,
        attenuatedCaps,
      );
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Register a user-defined IVM view (Phase 2b G8-B).
   *
   * Pass `{ id, inputPattern, strategy?, project? }`. Returns a
   * [`UserView`] handle exposing `id`, `strategy`, `inputPattern`,
   * `snapshot()`, and `onUpdate()`. Strategy defaults to `'B'` per
   * D8-RESOLVED; `'A'` and `'C'` produce typed errors
   * (`E_VIEW_STRATEGY_A_REFUSED` / `E_VIEW_STRATEGY_C_RESERVED`).
   *
   * # Naming (r6-arch-2 closure)
   *
   * Renamed from `createUserView` to `registerUserView` to align with
   * the Engine's existing `register_*` lifecycle verbs
   * (`registerSubgraph`, `registerCrud`, `registerModule`). The
   * `register_*` verb introduces a runtime artifact; `create_*` (e.g.
   * `createPrincipal`, `createView`) instantiates from a registry /
   * factory; `install_*` (e.g. `installModule`) loads + verifies
   * external content. R6-FP Group 1 lands the matching Rust + napi
   * rename; this TS surface routes through the new `registerUserView`
   * napi symbol when present and falls back to the legacy
   * `createUserView` napi symbol on older bindings.
   *
   * The canonical-view-from-registry path stays on `createView(viewDef)`
   * (semantically distinct — instantiates a hand-written view from a
   * registry-of-strategy; doesn't introduce a new artifact).
   */
  public async registerUserView(spec: UserViewSpec): Promise<UserView> {
    this.assertOpen();
    const validationError = validateUserViewSpec(spec);
    if (validationError !== null) {
      throw new EDslInvalidShape(validationError);
    }
    const native =
      this.inner.registerUserView ?? this.inner.createUserView;
    if (!native) {
      throw new EDslInvalidShape(
        "Engine.registerUserView unavailable on this binding — rebuild the napi cdylib against benten-engine ≥ Phase-2b G8-B",
      );
    }
    const resolvedStrategy = resolveUserViewStrategy(spec);
    try {
      // The Rust side enforces the typed E_VIEW_STRATEGY_A_REFUSED /
      // E_VIEW_STRATEGY_C_RESERVED errors; we forward the strategy
      // string verbatim so the engine boundary owns the policy.
      // R6 Round-2 r6-r2-napi-2: bind `native` to `this.inner` —
      // napi-rs class methods require the napi class instance as
      // `this`; calling the bare reference throws "Illegal
      // invocation" before the typed-error guard fires.
      native.call(this.inner, userViewSpecToNativeJson(spec));
    } catch (err) {
      throw mapNativeError(err);
    }
    return buildUserViewHandle(
      spec,
      resolvedStrategy,
      this.userViewRuntimeShim(),
    );
  }

  /**
   * Phase-3 G19-C1 (§7.1.3) — construct a [`UserViewRuntimeShim`] that
   * forwards `view.snapshot()` / `view.onUpdate()` calls to the napi
   * cdylib's `userViewSnapshot` / `userViewDrainUpdates` /
   * `userViewChangeOffset` accessors. Returns `null` when the cdylib
   * lacks the G19-C1 surface (older builds) so [`buildUserViewHandle`]
   * falls back to the no-op shape.
   */
  private userViewRuntimeShim(): UserViewRuntimeShim | null {
    const snap = this.inner.userViewSnapshot;
    const drain = this.inner.userViewDrainUpdates;
    const head = this.inner.userViewChangeOffset;
    if (!snap || !drain || !head) {
      return null;
    }
    const native = this.inner;
    return {
      snapshotRows(viewId: string): unknown[] | null {
        const raw = snap.call(native, viewId);
        if (raw === null || raw === undefined) {
          return null;
        }
        if (!Array.isArray(raw)) {
          // Defensive — pre-G19-C1 napi shapes returned other types
          // for unknown views; treat as "no rows" so the caller sees
          // an empty iterable rather than a runtime panic.
          return [];
        }
        return raw;
      },
      currentChangeOffset(): number {
        const raw = head.call(native);
        return typeof raw === "number" ? raw : 0;
      },
      drainUpdates(
        viewId: string,
        sinceOffset: number,
      ): { registered: boolean; events: unknown[]; nextOffset: number } {
        const raw = drain.call(native, viewId, sinceOffset) as
          | { registered?: boolean; events?: unknown; next_offset?: number }
          | null
          | undefined;
        if (!raw || typeof raw !== "object") {
          return { registered: false, events: [], nextOffset: sinceOffset };
        }
        const registered = raw.registered === true;
        const events = Array.isArray(raw.events) ? raw.events : [];
        const nextOffset =
          typeof raw.next_offset === "number" ? raw.next_offset : sinceOffset;
        return { registered, events, nextOffset };
      },
    };
  }

  /**
   * Register / materialize an IVM view definition.
   *
   * Two call shapes:
   *
   * 1. **Legacy id-string form** (`viewDef: ViewDef`): the `viewDef`
   *    object carries a `viewId` string from the canonical id family
   *    (e.g. `"content_listing_post"`). Returns the view definition
   *    Node's CID as a string. This form is preserved for the 5
   *    Phase-1 hand-written views.
   *
   * 2. **User-view builder form** (Phase 2b G8-B; `spec: UserViewSpec`):
   *    pass `{ id, inputPattern, strategy?, project? }`. Returns a
   *    [`UserView`] handle.
   *
   *    @deprecated since R6-FP — use {@link Engine.registerUserView}
   *    for the user-view spec form. The `createView(spec)` overload
   *    forwards to `registerUserView` for one cycle. The legacy
   *    `createView(viewDef)` form for the 5 Phase-1 hand-written
   *    views is unchanged.
   */
  public async createView(viewDef: ViewDef): Promise<string>;
  public async createView(spec: UserViewSpec): Promise<UserView>;
  public async createView(
    arg: ViewDef | UserViewSpec,
  ): Promise<string | UserView> {
    this.assertOpen();
    if (isUserViewSpec(arg)) {
      // r6-arch-2 deprecation alias — forward to the canonical
      // registerUserView surface. One-cycle alias matching Group 1's
      // Rust deprecation pattern.
      return this.registerUserView(arg);
    }
    if (!this.inner.createView) {
      throw new EDslInvalidShape("Engine.createView unavailable on this binding");
    }
    try {
      return this.inner.createView(arg);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Read a materialized view. Phase-1 accepts a `query` argument for
   * forward-compatibility but does not consult it.
   */
  public async readView(
    viewId: string,
    query: JsonValue = {},
  ): Promise<JsonValue> {
    this.assertOpen();
    if (!this.inner.readView) {
      throw new EDslInvalidShape("Engine.readView unavailable on this binding");
    }
    try {
      return this.inner.readView(viewId, query) as JsonValue;
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Emit a named event with a JSON payload.
   *
   * Phase-3 G19-B (§7.8): wires through the engine's
   * `EmitBroadcast` bus directly. The event is delivered to every
   * `engine.onEmit(name, ...)` consumer (string-equality channel
   * match) without going through a handler dispatch. Mirrors the
   * in-handler EMIT primitive's publish path so standalone +
   * handler-driven events on the same channel observe the same
   * subscriber set.
   *
   * Returns once the publish has been queued onto the broadcast bus.
   * The TSFN delivery to JS subscribers happens on a libuv tick.
   */
  public async emitEvent(name: string, payload: JsonValue): Promise<void> {
    this.assertOpen();
    if (!this.inner.emitEvent) {
      throw new EDslInvalidShape("Engine.emitEvent unavailable on this binding");
    }
    try {
      this.inner.emitEvent(name, payload);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** Count of Nodes stored under `label`. */
  public async countNodesWithLabel(label: string): Promise<number> {
    this.assertOpen();
    if (!this.inner.countNodesWithLabel) {
      throw new EDslInvalidShape(
        "Engine.countNodesWithLabel unavailable on this binding",
      );
    }
    try {
      return this.inner.countNodesWithLabel(label);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** Total `ChangeEvent`s emitted since the engine opened. */
  public async changeEventCount(): Promise<number> {
    this.assertOpen();
    if (!this.inner.changeEventCount) {
      throw new EDslInvalidShape(
        "Engine.changeEventCount unavailable on this binding",
      );
    }
    try {
      return this.inner.changeEventCount();
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /** Number of live IVM view subscribers. */
  public async ivmSubscriberCount(): Promise<number> {
    this.assertOpen();
    if (!this.inner.ivmSubscriberCount) {
      throw new EDslInvalidShape(
        "Engine.ivmSubscriberCount unavailable on this binding",
      );
    }
    try {
      return this.inner.ivmSubscriberCount();
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Flattened operational metrics snapshot. Keys are metric names; values are
   * numbers. Named compromise #5 fans per-capability-scope write counters
   * out as `benten.writes.committed.<scope>` /
   * `benten.writes.denied.<scope>` keys on top of the aggregate
   * `benten.writes.committed` / `benten.writes.denied` totals.
   */
  public async metricsSnapshot(): Promise<Record<string, number>> {
    this.assertOpen();
    if (!this.inner.metricsSnapshot) {
      throw new EDslInvalidShape(
        "Engine.metricsSnapshot unavailable on this binding",
      );
    }
    try {
      return this.inner.metricsSnapshot();
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Per-capability-scope committed-write tally. Keys are the derived scope
   * strings (`store:<label>:write`). Named compromise #5 — the Phase-1
   * posture is "record, don't enforce"; Phase-3 layers rate-limit
   * enforcement on these counters.
   */
  public async capabilityWritesCommitted(): Promise<Record<string, number>> {
    this.assertOpen();
    if (!this.inner.capabilityWritesCommitted) {
      throw new EDslInvalidShape(
        "Engine.capabilityWritesCommitted unavailable on this binding",
      );
    }
    try {
      return this.inner.capabilityWritesCommitted();
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Per-capability-scope denied-write tally. Mirrors
   * {@link Engine.capabilityWritesCommitted} for batches the policy
   * rejected.
   */
  public async capabilityWritesDenied(): Promise<Record<string, number>> {
    this.assertOpen();
    if (!this.inner.capabilityWritesDenied) {
      throw new EDslInvalidShape(
        "Engine.capabilityWritesDenied unavailable on this binding",
      );
    }
    try {
      return this.inner.capabilityWritesDenied();
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  // -------- WAIT / suspend / resume (Phase 2a G3-B napi F5) --------

  /**
   * Invoke a handler with suspension awareness.
   *
   * Returns a discriminated-union result:
   * - `{ kind: "complete", outcome }` — the handler ran to completion
   *   without hitting a WAIT primitive; `outcome` is the terminal Outcome.
   * - `{ kind: "suspended", handle }` — the handler hit a WAIT and
   *   produced an envelope. `handle` is a Node `Buffer` carrying the
   *   DAG-CBOR ExecutionStateEnvelope; pass it to `resumeFromBytes` /
   *   `resumeFromBytesAs` once the awaited signal is ready.
   *
   * The napi layer transports the handle as base64 to keep the FFI
   * return type a single `serde_json::Value`; this wrapper decodes it
   * to a `Buffer` so user code never sees the wire encoding.
   */
  public async callWithSuspension(
    handlerId: string,
    op: string,
    input: JsonValue,
  ): Promise<SuspensionResult> {
    this.assertOpen();
    if (!this.inner.callWithSuspension) {
      throw new EDslInvalidShape(
        "Engine.callWithSuspension unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    let raw: unknown;
    try {
      raw = this.inner.callWithSuspension(handlerId, op, input);
    } catch (err) {
      throw mapNativeError(err);
    }
    if (!raw || typeof raw !== "object") {
      throw new EDslInvalidShape(
        "Engine.callWithSuspension: native binding returned an unexpected shape",
      );
    }
    const r = raw as Record<string, unknown>;
    const kind = typeof r.kind === "string" ? r.kind : "";
    if (kind === "complete") {
      return { kind: "complete", outcome: r.outcome as Outcome };
    }
    if (kind === "suspended") {
      const handleStr = typeof r.handle === "string" ? r.handle : "";
      if (handleStr.length === 0) {
        throw new EDslInvalidShape(
          "Engine.callWithSuspension: suspended result missing base64 handle",
        );
      }
      // R6 Round-2 Instance 12: surface stateCid + signalName so JS
      // callers can correlate the suspension. Older napi cdylib
      // builds (pre-R6-FP) won't carry these fields; default to
      // empty strings so the type contract holds + the caller sees
      // structurally valid (if uninformative) values rather than
      // undefined.
      const stateCid = typeof r.stateCid === "string" ? r.stateCid : "";
      const signalName = typeof r.signalName === "string" ? r.signalName : "";
      return {
        kind: "suspended",
        handle: Buffer.from(handleStr, "base64"),
        stateCid,
        signalName,
      };
    }
    throw new EDslInvalidShape(
      `Engine.callWithSuspension: unknown result kind "${kind}"`,
    );
  }

  /**
   * Resume a suspended handler from envelope bytes. Equivalent to the
   * Rust-side `resume_from_bytes_unauthenticated` — skips step 2
   * (principal binding) of the 4-step resume protocol. Use
   * {@link Engine.resumeFromBytesAs} when you have a principal CID
   * that should be bound into the resume.
   */
  public async resumeFromBytes(
    bytes: Buffer,
    signal: JsonValue,
  ): Promise<Outcome> {
    this.assertOpen();
    if (!this.inner.resumeFromBytesUnauthenticated) {
      throw new EDslInvalidShape(
        "Engine.resumeFromBytes unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    try {
      return this.inner.resumeFromBytesUnauthenticated(bytes, signal) as Outcome;
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Resume a suspended handler from envelope bytes WITH an explicit
   * resumption principal CID. Drives the full 4-step resume protocol;
   * a principal mismatch fires `E_RESUME_ACTOR_MISMATCH`.
   */
  public async resumeFromBytesAs(
    bytes: Buffer,
    signal: JsonValue,
    principalCid: string,
  ): Promise<Outcome> {
    this.assertOpen();
    if (!this.inner.resumeFromBytesAs) {
      throw new EDslInvalidShape(
        "Engine.resumeFromBytesAs unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    try {
      return this.inner.resumeFromBytesAs(bytes, signal, principalCid) as Outcome;
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Phase-3 G19-C1 (phase-3-backlog §7.1.4) — ergonomic resume wrapper.
   *
   * Lifts {@link Engine.resumeFromBytes} into a discriminated-union
   * return shape that surfaces metadata about whether the resumed
   * handler ran to completion or re-suspended on a downstream WAIT.
   * Closes the r6-napi-2 ergonomics gap (raw-Buffer + raw-Outcome was
   * the actual surface; callers wrote ad-hoc shape-introspection to
   * detect re-suspension).
   *
   * Accepts EITHER a raw `Buffer` envelope (the `resumeFromBytes`
   * shape) OR the structurally-typed
   * {@link SuspensionResult} suspended-arm shape so the call site can
   * idiomatically chain `callWithSuspension` → `resumeWithMeta` without
   * unwrapping the envelope by hand.
   *
   * Always returns:
   * - `{ kind: "complete", outcome }` — the handler completed; `outcome`
   *   is the same shape `engine.call` returns.
   * - `{ kind: "suspended", handle, stateCid, signalName }` — reserved
   *   for the post-D12 cross-process re-suspension wire-up. Today the
   *   underlying napi `resumeFromBytesUnauthenticated` always resolves
   *   to a terminal Outcome; the suspended arm of the result type
   *   exists in the public contract so the post-D12 wiring is purely
   *   additive (caller code that already destructures both arms
   *   continues to work).
   *
   * @param envelope Either the raw `Buffer` from
   *   {@link Engine.callWithSuspension}'s `handle` field, or the
   *   suspended-arm object itself.
   * @param signal Signal value to resume with (same shape as
   *   {@link Engine.resumeFromBytes}).
   */
  public async resumeWithMeta(
    envelope:
      | Buffer
      | { handle: Buffer; stateCid?: string; signalName?: string },
    signal: JsonValue,
  ): Promise<ResumeWithMetaResult> {
    this.assertOpen();
    const bytes: Buffer = Buffer.isBuffer(envelope)
      ? envelope
      : envelope?.handle;
    if (!Buffer.isBuffer(bytes)) {
      throw new EDslInvalidShape(
        "Engine.resumeWithMeta: envelope must be a Buffer or an object carrying { handle: Buffer } (E_DSL_INVALID_SHAPE)",
      );
    }
    const outcome = await this.resumeFromBytes(bytes, signal);
    // Pre-D12: the underlying napi resumeFromBytesUnauthenticated always
    // resolves to a terminal Outcome. The suspended arm of the public
    // ResumeWithMetaResult contract exists for forward-compat with
    // cross-process re-suspension; once the engine learns to surface a
    // re-suspension envelope through resume, this wrapper destructures
    // the new shape without changing the call-site contract.
    return { kind: "complete", outcome };
  }

  // -------------------------------------------------------------------------
  // SANDBOX surface (Phase 2b G7-C — DSL-composition-only contract)
  // -------------------------------------------------------------------------
  //
  // There is NO `engine.sandbox(...)` method. SANDBOX is composed via
  // `subgraph(...).sandbox({ module, manifest? | caps? })` — see
  // `./sandbox.ts` for the full surface contract. The methods below
  // are introspection + module-lifecycle accessors ONLY.
  //
  // - `targetSupportsSandbox()` — boolean introspection probe (G7-C).
  // - `describeSandboxNode(handlerId, nodeId)` — read-only diagnostic
  //   (G7-C; ts-r4-3).
  // - `installModule(manifest, manifestCid)` — manifest install (G10-B).
  // - `uninstallModule(cid)` — manifest uninstall (G10-B).
  // - `computeManifestCid(manifest)` — manifest canonical-CID helper
  //   (G10-B).
  //
  // The G10-B-owned methods stub here with `EDslInvalidShape` until
  // G10-B's wave merges; the type signatures are pinned now so
  // `manifest_schema_parity.test.ts` + `install_module.test.ts` +
  // `wasm_browser_target.test.ts` + `sandbox.test.ts` all type-check
  // against a stable contract.

  /**
   * Returns `true` when this engine's underlying napi build supports
   * SANDBOX execution locally (i.e. the wasmtime executor is compiled
   * in), `false` when the build is `wasm32-unknown-unknown` and
   * SANDBOX execution must route to a Node-resident peer (Phase-3 P2P
   * sync).
   *
   * Use this to decide whether to drive a SANDBOX call locally vs
   * route the handler to a peer:
   *
   * ```ts
   * if (engine.targetSupportsSandbox()) {
   *   await engine.call("h", "go", input);
   * } else {
   *   await routeToNodePeer("h", "go", input);
   * }
   * ```
   *
   * Pinned by `packages/engine/test/wasm_browser_target.test.ts` +
   * `bindings/napi/test/sandbox_napi_bridge.test.ts`.
   */
  public targetSupportsSandbox(): boolean {
    const native = loadNative();
    if (typeof native.sandboxTargetSupported === "function") {
      return native.sandboxTargetSupported();
    }
    // Older napi binaries (Phase-1 era) didn't carry the symbol. Fall
    // back to assuming `true` so legacy builds keep working — the
    // assumption is correct for every Phase-1 native artifact since
    // those were always built against a non-wasm32 target.
    return true;
  }

  /**
   * Read-only diagnostic accessor — returns the resolved
   * [`SandboxNodeDescription`] (defaults applied) for the SANDBOX node
   * identified by `(handlerId, nodeId)`. The `nodeId` is the
   * subgraph-local id assigned by the DSL builder (e.g. `"sandbox-1"`).
   *
   * Defaults documented in `docs/SANDBOX-LIMITS.md` §2: omitting the
   * per-node DSL knobs uses `fuel = 1_000_000`, `wallclockMs = 30_000`,
   * `outputLimitBytes = 1_048_576`.
   *
   * # Honest Phase-2b state (r6-mpc-3 + r6-napi-3 + r6-dx-10)
   *
   * `fuelConsumedHighWater` + `lastInvocationMs` are returned as the
   * literal `"unknown"` sentinel — **metrics are NOT tracked in Phase
   * 2b**. `SandboxResult.fuelConsumed` + `output_consumed` are dropped
   * at the eval-engine boundary (`primitive_host.rs::execute_sandbox`
   * only propagates `output`); the per-handler metric record needed
   * to surface real values is a Phase-3 wiring (named destination:
   * `docs/future/phase-3-backlog.md` SnapshotBlobBackend
   * metric-propagation entry, R6-FP Group 4 enrichment). Returning the
   * `"unknown"` sentinel rather than a synthesized `null` (the prior
   * shape) lets callers distinguish "metric is structurally not
   * tracked in 2b" from "node hasn't been invoked yet".
   *
   * Cross-ref: `docs/SECURITY-POSTURE.md` Compromise #17
   * (sandbox-execution-metric-propagation).
   *
   * The remaining fields (`moduleCid` / `manifestId` / `fuel` /
   * `wallclockMs` / `outputLimitBytes`) are synthesized client-side
   * from the registered subgraph spec; sufficient for the
   * omitting-knobs-uses-defaults test pin.
   *
   * Pinned by `packages/engine/test/sandbox.test.ts::"SandboxArgs defaults — omitting fuel / wallclockMs / outputLimitBytes uses 1M / 30s / 1MB"`.
   */
  public async describeSandboxNode(
    handlerId: string,
    nodeId: string,
  ): Promise<SandboxNodeDescription> {
    this.assertOpen();
    if (typeof handlerId !== "string" || handlerId.length === 0) {
      throw new EDslInvalidShape(
        "Engine.describeSandboxNode requires a non-empty handlerId",
      );
    }
    if (typeof nodeId !== "string" || nodeId.length === 0) {
      throw new EDslInvalidShape(
        "Engine.describeSandboxNode requires a non-empty nodeId",
      );
    }
    const handlerActions = this.knownHandlers.get(handlerId);
    if (!handlerActions) {
      throw new EDslUnregisteredHandler(
        `Engine.describeSandboxNode: no handler '${handlerId}' registered`,
        { handlerId, suggestions: [...this.knownHandlers.keys()] },
      );
    }
    // Phase-3 G19-C2 wave-7 (§7.1): when the napi cdylib carries the
    // `describeSandboxNode` bridge (cfg-gated under
    // `--features test-helpers`), call into it and return the real
    // metric values populated by `primitive_host::execute_sandbox`.
    // Fall back to the synthesized "unknown" shape when the bridge is
    // absent (production cdylib build) OR when the named handler has
    // no recorded SANDBOX invocation yet.
    if (typeof this.inner.describeSandboxNode === "function") {
      let nativeJson: string | null;
      try {
        nativeJson = this.inner.describeSandboxNode(handlerId);
      } catch {
        nativeJson = null;
      }
      if (nativeJson) {
        try {
          const parsed = JSON.parse(nativeJson) as {
            moduleCid?: string;
            manifestId?: string | null;
            fuel?: number;
            wallclockMs?: number;
            outputLimitBytes?: number;
            fuelConsumedHighWater?: number | null;
            outputConsumedHighWater?: number | null;
            lastInvocationMs?: number | null;
          };
          return {
            moduleCid: parsed.moduleCid ?? nodeId,
            manifestId: parsed.manifestId ?? null,
            fuel: parsed.fuel ?? 1_000_000,
            wallclockMs: parsed.wallclockMs ?? 30_000,
            outputLimitBytes: parsed.outputLimitBytes ?? 1_048_576,
            fuelConsumedHighWater:
              typeof parsed.fuelConsumedHighWater === "number"
                ? parsed.fuelConsumedHighWater
                : "unknown",
            // R6 fp Wave C2 (obs-r6r1-1 closure): closes the Phase-3
            // §7.1 trio — output_consumed_high_water now flows through
            // the napi bridge instead of being dropped at
            // describe_sandbox_node_for_handler.
            outputConsumedHighWater:
              typeof parsed.outputConsumedHighWater === "number"
                ? parsed.outputConsumedHighWater
                : "unknown",
            lastInvocationMs:
              typeof parsed.lastInvocationMs === "number"
                ? parsed.lastInvocationMs
                : "unknown",
          };
        } catch {
          // Fall through to the legacy synthesized shape on parse fail.
        }
      }
    }
    return {
      moduleCid: nodeId,
      manifestId: null,
      fuel: 1_000_000,
      wallclockMs: 30_000,
      outputLimitBytes: 1_048_576,
      // Honest "unknown" sentinel per r6-mpc-3 — fall-through shape
      // when the native cdylib lacks the test-helpers describeSandboxNode
      // bridge OR when no SANDBOX invocation has been recorded for the
      // handler yet (the metric record is created lazily on first call).
      fuelConsumedHighWater: "unknown",
      outputConsumedHighWater: "unknown",
      lastInvocationMs: "unknown",
    };
  }

  /**
   * Install a module manifest. `manifestCid` is REQUIRED (D16
   * RESOLVED-FURTHER) — there is no convenience overload that omits it
   * and silently computes-and-trusts the CID. A mismatch between the
   * supplied CID and the canonical-DAG-CBOR CID of the manifest fires
   * `E_MODULE_MANIFEST_CID_MISMATCH` carrying both CIDs + a one-line
   * manifest summary.
   *
   * Owned by G10-B (Phase 2b plan §3 G10-B exclusive ownership per
   * wsa-r1-5). Wave-8c wires the napi bridge through the rebuilt
   * cdylib; the underlying Rust API has been on `Engine` since
   * G10-B + G10-A merged.
   */
  public async installModule(
    manifest: ModuleManifest,
    manifestCid: string,
  ): Promise<string> {
    this.assertOpen();
    if (typeof this.inner.installModule !== "function") {
      throw new EDslInvalidShape(
        "Engine.installModule unavailable on this binding — rebuild @benten/engine-native (wave-8c bridge required)",
      );
    }
    try {
      return this.inner.installModule(
        manifest as unknown as JsonValue,
        manifestCid,
      );
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Uninstall a module manifest by CID. Idempotent — a second call on
   * the same CID is a no-op. Releases capabilities + cleans up
   * subscriptions / IVM views referencing modules from the manifest.
   *
   * Owned by G10-B. Wave-8c wires the napi bridge.
   */
  public async uninstallModule(cid: string): Promise<void> {
    this.assertOpen();
    if (typeof this.inner.uninstallModule !== "function") {
      throw new EDslInvalidShape(
        "Engine.uninstallModule unavailable on this binding — rebuild @benten/engine-native (wave-8c bridge required)",
      );
    }
    try {
      this.inner.uninstallModule(cid);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Compute the canonical-DAG-CBOR CID of a manifest WITHOUT
   * installing it. Used by callers that want to verify the CID before
   * passing it as the required arg to [`installModule`].
   *
   * Owned by G10-B. Wave-8c wires the napi bridge.
   */
  public async computeManifestCid(manifest: ModuleManifest): Promise<string> {
    this.assertOpen();
    if (typeof this.inner.computeManifestCid !== "function") {
      throw new EDslInvalidShape(
        "Engine.computeManifestCid unavailable on this binding — rebuild @benten/engine-native (wave-8c bridge required)",
      );
    }
    try {
      return this.inner.computeManifestCid(manifest as unknown as JsonValue);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Phase-3 G17-C wave-5b (phase-3-backlog §6.6 deliverable 1; pim-2
   * 24th p/c drift acceptance criterion).
   *
   * Persist wasm module bytes under their BLAKE3-derived CID via the
   * durable redb-backed blob store so SANDBOX dispatch can resolve
   * `module: "<base32-cid>"` references at execution time. The
   * caller-supplied `cid` MUST match the BLAKE3 of `bytes` — mismatch
   * throws `E_MODULE_BYTES_CID_MISMATCH` per D-PHASE-3-12.
   *
   * Sibling of [`installModule`] (which writes the manifest envelope —
   * entries, requires, CID schema). After both calls succeed, a SANDBOX
   * subgraph that references the manifest by `<manifest>:<entry>` name
   * resolves at registration time AND has wasm bytes available at
   * execution time.
   *
   * Owned by G17-C. Wave-5b wires the napi bridge.
   */
  public async registerModuleBytes(
    cid: string,
    bytes: Buffer,
  ): Promise<void> {
    this.assertOpen();
    if (typeof this.inner.registerModuleBytes !== "function") {
      throw new EDslInvalidShape(
        "Engine.registerModuleBytes unavailable on this binding — rebuild @benten/engine-native (G17-C wave-5b bridge required)",
      );
    }
    try {
      this.inner.registerModuleBytes(cid, bytes);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  // -------- STREAM (Phase 2b G6-B) --------

  /**
   * Invoke a registered handler whose subgraph produces STREAM chunks.
   * Returns a [`StreamHandle`] that implements `AsyncIterable<Chunk>`,
   * so consumers can write:
   *
   * ```ts
   * for await (const chunk of engine.callStream(handlerId, "act", input)) {
   *   process.stdout.write(chunk);
   * }
   * ```
   *
   * The handle auto-closes when the `for await` loop exits (via the
   * iterator's `return()` hook). For an explicit-close lifecycle use
   * {@link Engine.openStream}.
   *
   * Mirrors {@link Engine.call} naming.
   *
   * # Production runtime (r6-dx-6 closure)
   *
   * G6-A + wave-8c-stream-infra wire the underlying STREAM executor
   * end-to-end: the iterator yields chunks as the chunk-producer
   * thread pushes them onto the bounded sink + the napi
   * `next_chunk_adapter` drains them onto the JS async-iterator.
   * {@link Engine.testingOpenStreamForTest} exists for cfg-gated
   * harness fixtures that pre-populate chunks without touching the
   * production executor; it is NOT the supported production
   * substitute.
   */
  public callStream(
    handlerId: string,
    op: string,
    input: JsonValue,
  ): StreamHandle {
    this.assertOpen();
    validateStreamCallArgs(handlerId, op, input);
    if (!this.inner.callStream) {
      throw new EDslInvalidShape(
        "Engine.callStream unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    let native: NativeStreamHandle;
    try {
      native = this.inner.callStream(handlerId, op, input);
    } catch (err) {
      throw mapNativeError(err);
    }
    return wrapStreamHandle(native);
  }

  /**
   * Phase 2b wave-8c-cont: `callStream` with an explicit actor
   * principal. Mirrors {@link Engine.callAs} naming.
   *
   * `actor` is a friendly principal identifier (the same shape
   * `engine.callAs` accepts). The principal is threaded through to
   * the napi boundary; per-chunk cap-recheck on emission is a
   * named-destination Phase-3 surface (r6-stream-2: the production
   * runtime currently captures the principal but does not consult it
   * on each chunk emission — STREAM mirroring SUBSCRIBE's
   * DeliveryCapRecheck is the closure work). Today this method is
   * functionally equivalent to {@link Engine.callStream} except for
   * the principal capture.
   *
   * Production runtime: G6-A + wave-8c-stream-infra wire the
   * underlying executor — the iterator yields chunks as the
   * chunk-producer thread pushes them. (r6-dx-6 closure: dropped the
   * stale "first next() surfaces E_PRIMITIVE_NOT_IMPLEMENTED" claim.)
   */
  public callStreamAs(
    handlerId: string,
    op: string,
    input: JsonValue,
    actor: string,
  ): StreamHandle {
    this.assertOpen();
    validateStreamCallArgs(handlerId, op, input);
    if (!this.inner.callStreamAs) {
      throw new EDslInvalidShape(
        "Engine.callStreamAs unavailable on this binding — rebuild @benten/engine-native (wave-8c-cont bridge required)",
      );
    }
    let native: NativeStreamHandle;
    try {
      native = this.inner.callStreamAs(handlerId, op, input, actor);
    } catch (err) {
      throw mapNativeError(err);
    }
    return wrapStreamHandle(native);
  }

  /**
   * Open a STREAM dispatch returning a [`StreamHandle`] whose lifecycle
   * the caller manages via {@link StreamHandle.close}. Same dispatch
   * path as {@link Engine.callStream} — the lifecycle CONVENTION is
   * different (the `for await` form auto-closes; this form requires an
   * explicit close()).
   *
   * Use this when you need to start a stream, hand the handle to a
   * different scope (e.g. an Express route), and `close()` it
   * explicitly when the consumer disconnects.
   *
   * # Lifecycle enforcement (r6-stream-1 honest scope)
   *
   * The engine threads `requires_explicit_close: true` through the
   * underlying StreamHandle so a future leak detector can fire
   * `E_STREAM_HANDLE_LEAKED` when an unclosed handle is GC'd. **Phase
   * 2b does NOT enforce this at the JS surface** — at the API level
   * `callStream` and `openStream` are functionally indistinguishable
   * once the handle is in JS-side scope. Production handlers using
   * openStream must close() the handle explicitly; failing to do so is
   * a resource leak (the producer thread + chunk sink survive until
   * Drop). Wiring a `FinalizationRegistry` leak detector + exposing a
   * `requiresExplicitClose()` napi accessor is named-destination Phase
   * 3 (R6-FP Group 4 enrichment to docs/future/phase-3-backlog.md).
   */
  public openStream(
    handlerId: string,
    op: string,
    input: JsonValue,
  ): StreamHandle {
    this.assertOpen();
    validateStreamCallArgs(handlerId, op, input);
    if (!this.inner.openStream) {
      throw new EDslInvalidShape(
        "Engine.openStream unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    let native: NativeStreamHandle;
    try {
      native = this.inner.openStream(handlerId, op, input);
    } catch (err) {
      throw mapNativeError(err);
    }
    // Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4 scenario d): register
    // explicit-close handles with this Engine so `shutdown()` can drain
    // them and fire shutdown-drain leak events synchronously. The
    // wrapper's `close()` path removes the entry; if the consumer never
    // calls close() the FinalizationRegistry path fires GC-without-close
    // when the handle becomes unreachable.
    const handle = wrapStreamHandle(native);
    if (handle.requiresExplicitClose()) {
      this.openExplicitCloseHandles.add(handle);
      const innerClose = handle.close.bind(handle);
      // Wrap close() to deregister from the shutdown-drain set on
      // explicit close. Idempotent — set.delete is a no-op if absent.
      const wrappedClose = () => {
        this.openExplicitCloseHandles.delete(handle);
        innerClose();
      };
      // Override the close binding on the handle. Property
      // re-assignment is acceptable here because StreamHandle is a
      // plain object literal returned by wrapStreamHandle (not a
      // class instance with read-only descriptors).
      Object.defineProperty(handle, "close", {
        value: wrappedClose,
        writable: true,
        configurable: true,
        enumerable: true,
      });
    }
    return handle;
  }

  /**
   * ts-r4-2 R4: vitest-harness factory. Returns a [`StreamHandle`]
   * pre-populated with `chunks` for harnesses that need to drive the
   * async-iterator without G6-A's production STREAM executor wired in.
   *
   * The native cdylib's `testingOpenStreamForTest` symbol is only
   * resolvable when the napi build was built with
   * `--features test-helpers`. Production cdylibs surface
   * `E_PRIMITIVE_NOT_IMPLEMENTED` if reached.
   *
   * Symbol presence is pinned by
   * `bindings/napi/test/stream_napi_async_iterator_back_pressure.test.ts`.
   */
  public testingOpenStreamForTest(chunks: Chunk[]): StreamHandle {
    this.assertOpen();
    if (!this.inner.testingOpenStreamForTest) {
      throw new EDslInvalidShape(
        "Engine.testingOpenStreamForTest unavailable on this binding — \
build @benten/engine-native with `--features test-helpers`",
      );
    }
    let native: NativeStreamHandle;
    try {
      native = this.inner.testingOpenStreamForTest(chunks);
    } catch (err) {
      throw mapNativeError(err);
    }
    return wrapStreamHandle(native);
  }

  /**
   * Phase 2b wave-8c-stream-infra: process-wide active-stream count.
   *
   * Returns the number of `StreamHandle` instances constructed via the
   * production runtime path (engine.callStream / engine.openStream)
   * that have NOT yet been dropped or explicitly closed.
   *
   * Pre-buffered handles (`testingOpenStreamForTest`) do NOT contribute
   * to this count — only real producer-bridge handles do.
   *
   * Used by `packages/engine/test/stream.test.ts` to verify that
   * `for-await break` propagates producer-side cleanup.
   *
   * Returns 0 if the bridge isn't built into this cdylib.
   */
  public async activeStreamCount(): Promise<number> {
    this.assertOpen();
    if (!this.inner.activeStreamCount) {
      return 0;
    }
    return this.inner.activeStreamCount();
  }

  // -------- SUBSCRIBE (Phase 2b G6-B) --------

  /**
   * Register an ad-hoc change-stream consumer. `pattern` is an
   * event-name glob (e.g. `"post:*"`, `"system:CapabilityGrant"`);
   * `callback` fires once per matched event with
   * `(engineAssignedSeq, payloadChunk)`.
   *
   * Returns a [`Subscription`] handle; call `unsubscribe()` (or let it
   * fall out of scope and rely on the GC-driven Rust-side `Drop` impl)
   * to release the registration.
   *
   * Renamed from `engine.subscribe` per dx-optimizer R1 finding to
   * avoid name-collision with the DSL `subgraph(...).subscribe`
   * builder method.
   *
   * D5-RESOLVED delivery semantics: engine-assigned `u64 seq` +
   * engine-side dedup at the handler boundary = exactly-once at this
   * API surface. Within-key strict ordering, cross-key unordered.
   * Bounded retention window (1000 events OR 24h) for persistent
   * cursors. Cap-check at delivery.
   *
   * # Delivery semantics (r6-dx-2 closure — wave-8c live)
   *
   * The napi adapter wraps the supplied callback in a
   * `napi::ThreadsafeFunction` so deliveries from the engine's
   * ChangeBroadcast publish thread land on the libuv main loop. The
   * returned `Subscription`'s `active` getter is `true` until
   * `unsubscribe()` (or the engine-side `Drop` after the JS handle is
   * GC'd). **Holding a JS-side reference to the Subscription handle
   * is required** — letting it be GC'd fires `Drop` on the underlying
   * `benten_engine::Subscription` which de-registers the callback +
   * releases the `napi::ThreadsafeFunction` Arc that holds the JS
   * callback alive.
   *
   * Pin: `packages/engine/test/subscribe.test.ts::"LOAD-BEARING — onChange callback fires when a matching write commits"`.
   */
  public onChange(
    pattern: string,
    callback: OnChangeCallback,
    cursor?: SubscribeCursor,
  ): Subscription {
    this.assertOpen();
    validateOnChangeArgs(pattern, callback);
    if (!this.inner.onChange) {
      throw new EDslInvalidShape(
        "Engine.onChange unavailable on this binding — rebuild @benten/engine-native",
      );
    }
    // Phase-3 G19-B (§7.7, r1-napi-4 keep-wrapper path b): napi-rs v3.x
    // ThreadsafeFunction with `FnArgs<(u32, Buffer)>` delivers discrete
    // `(seq, payload)` args to the JS callback (verified end-to-end
    // 2026-05-07; the pre-G19-B "single tuple-array" delivery shape
    // documented in phase-3-backlog §7.7 belonged to an earlier napi-rs
    // build). The wrapper shape below preserves the user-callback's
    // discrete-args contract + adds the exception-isolation log path
    // dx-r1-2b-4 / r6-dx-2 require, and reads identically to the
    // intended d.ts surface. The in-test `Array.isArray(...)` runtime
    // tuple-detection workaround is retired (no longer needed; the
    // splatted-args shape is the production reality).
    const napiCb = (seq: number, payload: Buffer): void => {
      try {
        callback(seq, payload);
      } catch (err) {
        // dx-r1-2b-4: subscriber-side throws are routine; sub stays
        // alive, log fires.
        // eslint-disable-next-line no-console
        console.error("onChange callback threw:", err);
      }
    };
    let native: NativeSubscriptionJs;
    try {
      native = this.inner.onChange(pattern, serializeCursor(cursor), napiCb);
    } catch (err) {
      throw mapNativeError(err);
    }
    return wrapSubscriptionHandle(native, cursor ?? { kind: "latest" });
  }

  /**
   * Phase 2b wave-8c-cont: `onChange` with an explicit actor principal.
   * Mirrors {@link Engine.callAs} naming.
   *
   * `actor` is the friendly principal identifier whose grants drive
   * D5 delivery-time cap-recheck (D5-RESOLVED — wave-8c-subscribe-infra
   * wired this in production). The principal is captured on the
   * registered ad-hoc onChange entry's delivery-time cap-recheck
   * closure so the named principal's grants gate every event
   * delivery; if the actor's caps are revoked mid-stream the
   * subscription auto-cancels per D5 contract.
   *
   * # Delivery semantics (r6-dx-2 closure — same shape as `onChange`)
   *
   * The napi adapter wraps the supplied callback in a
   * `napi::ThreadsafeFunction`; deliveries land on the libuv main
   * loop. The returned `Subscription`'s `active` getter is `true`
   * until `unsubscribe()` (or engine-side Drop). Holding a JS-side
   * reference to the handle is required.
   */
  public onChangeAs(
    pattern: string,
    callback: OnChangeCallback,
    actor: string,
    cursor?: SubscribeCursor,
  ): Subscription {
    this.assertOpen();
    validateOnChangeArgs(pattern, callback);
    if (!this.inner.onChangeAs) {
      throw new EDslInvalidShape(
        "Engine.onChangeAs unavailable on this binding — rebuild @benten/engine-native (wave-8c-cont bridge required)",
      );
    }
    // Phase-3 G19-B (§7.7, r1-napi-4 keep-wrapper path b): same
    // splatted-args wrapper shape as `onChange`. The principal is
    // captured Rust-side so D5 delivery-time cap-recheck fires this
    // actor's grants on every event; if the actor's caps are revoked
    // mid-stream the subscription auto-cancels per D5 contract.
    const napiCb = (seq: number, payload: Buffer): void => {
      try {
        callback(seq, payload);
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error("onChangeAs callback threw:", err);
      }
    };
    let native: NativeSubscriptionJs;
    try {
      native = this.inner.onChangeAs(
        pattern,
        serializeCursor(cursor),
        actor,
        napiCb,
      );
    } catch (err) {
      throw mapNativeError(err);
    }
    return wrapSubscriptionHandle(native, cursor ?? { kind: "latest" });
  }

  /**
   * Subscribe to standalone EMIT events on a named channel.
   *
   * EMIT events flow through the engine's dedicated `EmitBroadcast`
   * (separate from the storage-event ChangeBroadcast that drives
   * `onChange` + IVM views). The dedicated channel exists because
   * EMIT events have no Node CID, no commit, no tx-id — they are
   * publish-only signals from a handler's standalone EMIT primitive.
   * See `crates/benten-engine/src/emit_broadcast.rs` for the rationale.
   *
   * The callback fires synchronously on every EMIT publish whose
   * `channel` matches `channel` exactly (string equality; no
   * glob-matching at the engine surface in Phase 2b).
   *
   * Returns an [`EmitSubscription`] handle; call `unsubscribe()` (or
   * let it fall out of scope and rely on the GC-driven Rust-side `Drop`
   * impl) to release the registration.
   *
   * # Wiring status
   *
   * Wired by R6-FP Group 1 (napi `EmitSubscriptionJs` class +
   * `subscribe_emit_events` adapter) + R6-FP Group 2 (this TS surface).
   * Closes the wave-8h cross-layer audit gap (`r6-mpc-2`): the engine
   * had a working `Engine::subscribe_emit_events` Rust API but no JS
   * surface, so JS consumers could not observe events from a
   * handler-internal EMIT primitive.
   *
   * If the loaded napi cdylib pre-dates the bridge, the call surfaces
   * `E_PRIMITIVE_NOT_IMPLEMENTED` with a "rebuild @benten/engine-native"
   * fix-hint rather than crashing on a missing native symbol.
   */
  public onEmit(
    channel: string,
    callback: OnEmitCallback,
  ): EmitSubscription {
    this.assertOpen();
    validateOnEmitArgs(channel, callback);
    if (!this.inner.onEmit) {
      throw new EDslInvalidShape(
        "Engine.onEmit unavailable on this binding — rebuild @benten/engine-native (R6-FP EMIT broadcast bridge required to close r6-mpc-2)",
      );
    }
    // Phase-3 G19-B (§7.7, r1-napi-4 keep-wrapper path b): napi-rs v3.x
    // ThreadsafeFunction with `FnArgs<(String, String)>` delivers
    // discrete `(chanArg, payloadJson)` args (verified end-to-end
    // 2026-05-07; the pre-G19-B "single tuple-array" delivery shape
    // documented in phase-3-backlog §7.7 + the
    // `emit_subscribe.test.ts` load-bearing-pre-merge runtime
    // tuple-detection branch belonged to an earlier napi-rs build).
    // The wrapper preserves the user-callback's discrete-args contract,
    // parses the JSON payload to an idiomatic JsonValue, and adds the
    // exception-isolation log path. The runtime tuple-detection
    // workaround is retired.
    //
    // payloadJson is the engine's `Value` payload serialized to JSON
    // (the same shape engine.call inputs/outputs use). We parse it once
    // here so the JS-facing callback receives an idiomatic JsonValue
    // rather than a raw JSON string.
    const napiCb = (chanArg: string, payloadJson: string): void => {
      let payload: JsonValue;
      try {
        payload = JSON.parse(payloadJson) as JsonValue;
      } catch {
        // Defensive: if the napi side delivered a non-JSON payload
        // (shouldn't happen — payload comes from Value::to_json), pass
        // the raw string through rather than crashing the listener.
        payload = payloadJson;
      }
      try {
        callback(chanArg, payload);
      } catch (err) {
        // Subscriber-side throws are routine; sub stays alive, log fires.
        // eslint-disable-next-line no-console
        console.error("onEmit callback threw:", err);
      }
    };
    let native: NativeEmitSubscriptionJs;
    try {
      native = this.inner.onEmit(channel, napiCb);
    } catch (err) {
      throw mapNativeError(err);
    }
    return wrapEmitSubscriptionHandle(native);
  }

  // -------- Snapshot blob handoff (Phase 2b wave-8c-cont 8c-iv) --------

  /**
   * Phase 2b wave-8c-cont (D10 snapshot-blob handoff): walk this
   * engine's storage and encode a canonical DAG-CBOR snapshot-blob
   * for handoff. Returns the bytes as a `Uint8Array`.
   *
   * The snapshot-blob is canonical (BTreeMap-sorted) — two exports
   * of the same engine state produce byte-identical output (D10 +
   * sec-pre-r1-09 Inv-13 collision-safety). The companion factory
   * {@link Engine.fromSnapshotBlob} accepts the bytes to construct a
   * read-only engine view over the same content.
   *
   * Native-target only — surfaces `E_SUBSYSTEM_DISABLED` on wasm32
   * builds where `engine_snapshot` is `#[cfg(not(target_arch =
   * "wasm32"))]`-gated.
   */
  public async exportSnapshotBlob(): Promise<Uint8Array> {
    this.assertOpen();
    if (!this.inner.exportSnapshotBlob) {
      throw new EDslInvalidShape(
        "Engine.exportSnapshotBlob unavailable on this binding — rebuild @benten/engine-native (wave-8c-cont bridge required)",
      );
    }
    try {
      const buf = this.inner.exportSnapshotBlob();
      // The napi `Buffer` IS a `Uint8Array` subclass; return the same
      // bytes without an unnecessary copy. Consumers can pass the
      // result back into `Engine.fromSnapshotBlob` directly.
      return new Uint8Array(buf.buffer, buf.byteOffset, buf.byteLength);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Phase 2b wave-8c-cont: `true` iff this engine was constructed via
   * {@link Engine.fromSnapshotBlob} and is therefore a read-only
   * view. Mirrors the Rust-side `Engine::is_read_only_snapshot`
   * accessor.
   *
   * Consumers can branch on this flag rather than catching
   * `E_BACKEND_READ_ONLY` on every mutation attempt.
   */
  public isReadOnlySnapshot(): boolean {
    this.assertOpen();
    if (!this.inner.isReadOnlySnapshot) {
      // Older bindings (pre-wave-8c-cont) don't carry the symbol;
      // those bindings can never produce a read-only engine because
      // they predate `fromSnapshotBlob`, so `false` is the correct
      // back-compat answer.
      return false;
    }
    return this.inner.isReadOnlySnapshot();
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function flattenCallResult(
  raw: unknown,
): Record<string, JsonValue> & { cid?: string } {
  if (raw === null || typeof raw !== "object") {
    return { result: raw as JsonValue };
  }
  const r = raw as Record<string, unknown>;

  // r6b-dx-C3: a native response of shape
  // `{ ok: false, edge, errorCode, errorMessage }` is a failed call
  // (e.g. a capability denial routed via ON_DENIED). Surfacing it as
  // a silent success is the bug that bit `cap_denial_routes_on_denied`
  // — the caller's `await engine.call(...)` resolved, they treated
  // the write as committed, and only later noticed the Node was
  // missing. Raise a typed error built from the reported `errorCode`
  // so the caller gets the same shape as a thrown napi error.
  if (r.ok === false) {
    const code =
      typeof r.errorCode === "string" && r.errorCode.length > 0
        ? r.errorCode
        : "E_UNKNOWN";
    const msg =
      typeof r.errorMessage === "string" && r.errorMessage.length > 0
        ? r.errorMessage
        : typeof r.edge === "string"
          ? `handler routed via ${r.edge}`
          : "handler reported failure";
    const edge = typeof r.edge === "string" ? r.edge : undefined;
    // Compose a message that `extractCode` will find the stable
    // `E_*` token in, so `mapNativeError` reconstructs the right
    // typed subclass end-to-end.
    throw mapNativeError(`${code}: ${msg}${edge ? ` (edge=${edge})` : ""}`);
  }
  if ("result" in r) {
    const inner = r.result as JsonValue;
    if (inner && typeof inner === "object" && !Array.isArray(inner)) {
      const merged = {
        ...(inner as Record<string, JsonValue>),
      };
      if (typeof r.cid === "string" && merged.cid === undefined) {
        merged.cid = r.cid;
      }
      return merged as Record<string, JsonValue> & { cid?: string };
    }
    return {
      result: inner,
      ...(typeof r.cid === "string" ? { cid: r.cid } : {}),
    };
  }
  return r as Record<string, JsonValue> & { cid?: string };
}

/**
 * Post-process the raw native outcome shape for crud `get` / `list`
 * dispatches: flatten `list[0].properties` onto the root for GETs so
 * callers can read `.title` directly, and surface `items` alongside
 * `list` for LISTs. Optional `ctx` carries the per-handler stampedCreatedAt
 * side-table so GETs can re-attach the stamp the native side doesn't echo.
 *
 * Extracted so `Engine.call` and `Engine.callAs` apply the identical
 * shaping rules — a divergence between the two paths was the bug that
 * let `engine.callAs(..., "post:get", { cid }, actor).title` read
 * `undefined` while `engine.call(..., "post:get", { cid }).title`
 * returned the value (r6b-dx-C2).
 */
function applyCrudPostProcessing(
  flattened: Record<string, JsonValue> & { cid?: string },
  crud: CrudHandler | undefined,
  dispatchOp: string,
  input: JsonValue,
  ctx?: { handlerId: string; stampTable: Map<string, number> },
): void {
  if (!crud) return;
  if (dispatchOp === "get") {
    const listVal = (flattened as Record<string, unknown>).list;
    if (Array.isArray(listVal) && listVal.length > 0) {
      const first = listVal[0];
      if (first && typeof first === "object" && !Array.isArray(first)) {
        const f = first as Record<string, JsonValue>;
        if (
          f.properties &&
          typeof f.properties === "object" &&
          !Array.isArray(f.properties)
        ) {
          for (const [k, v] of Object.entries(
            f.properties as Record<string, JsonValue>,
          )) {
            if (flattened[k] === undefined) flattened[k] = v;
          }
        }
      }
    }
    if (
      typeof input === "object" &&
      input !== null &&
      !Array.isArray(input)
    ) {
      const reqCid = (input as Record<string, JsonValue>).cid;
      if (typeof reqCid === "string") {
        if (ctx) {
          const remembered = ctx.stampTable.get(`${ctx.handlerId}:${reqCid}`);
          if (remembered !== undefined && flattened.createdAt === undefined) {
            flattened.createdAt = remembered;
          }
        }
        if (flattened.cid === undefined) flattened.cid = reqCid;
      }
    }
  } else if (dispatchOp === "list") {
    const list = (flattened as Record<string, unknown>).list;
    if (Array.isArray(list) && flattened.items === undefined) {
      flattened.items = list.map((entry) => {
        if (entry && typeof entry === "object" && !Array.isArray(entry)) {
          const e = entry as Record<string, JsonValue>;
          if (
            e.properties &&
            typeof e.properties === "object" &&
            !Array.isArray(e.properties)
          ) {
            return e.properties as JsonValue;
          }
        }
        return entry as JsonValue;
      }) as JsonValue;
    }
  }
}

/**
 * Ensure the parent directory of `path` exists. Redb surfaces a bare
 * `I/O error: No such file or directory` when its target file's
 * parent doesn't exist; pre-creating the dir here (recursive, no-op
 * when it already exists) turns that class of error into a silent
 * success — the DX contract first-run developers need.
 */
function ensureParentDir(path: string): void {
  const parent = dirname(path);
  if (!parent || parent === "." || parent === "/") return;
  try {
    mkdirSync(parent, { recursive: true });
  } catch {
    // Fall through — let the native open surface the real error via
    // mapNativeError rather than obscure it with an mkdir failure.
  }
}

/**
 * Tiny "did you mean?" matcher. Returns up to 3 handler ids that are
 * "close" to `needle` by simple substring / 3-gram rules. We avoid a
 * full Levenshtein — the cost is more failure surface than the signal
 * justifies for Phase 1 DX.
 */
function nearMatches(needle: string, haystack: string[]): string[] {
  const lo = needle.toLowerCase();
  const hits = haystack
    .filter(
      (h) =>
        h.toLowerCase().includes(lo) || lo.includes(h.toLowerCase()),
    )
    .slice(0, 3);
  if (hits.length > 0) return hits;
  const grams = new Set<string>();
  for (let i = 0; i <= lo.length - 3; i++) grams.add(lo.slice(i, i + 3));
  return haystack
    .filter((h) => {
      const low = h.toLowerCase();
      for (const g of grams) if (low.includes(g)) return true;
      return false;
    })
    .slice(0, 3);
}

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
  isCrudHandler,
  isSubgraph,
  type CrudHandler,
} from "./dsl.js";
import {
  EDslInvalidShape,
  EDslUnregisteredHandler,
  mapNativeError,
} from "./errors.js";
import { toMermaid } from "./mermaid.js";
import {
  validateStreamCallArgs,
  wrapStreamHandle,
  type NativeStreamHandle,
} from "./stream.js";
import {
  makeSubscription,
  serializeCursor,
  validateOnChangeArgs,
  type NativeSubscriptionJson,
  type OnChangeCallback,
} from "./subscribe.js";
import type {
  AttributionFrame,
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
  Subgraph,
  SubscribeCursor,
  Subscription,
  SuspensionResult,
  Trace,
  TraceStep,
  UserView,
  UserViewSpec,
  ViewDef,
} from "./types.js";
import {
  buildUserViewHandle,
  resolveUserViewStrategy,
  userViewSpecToNativeJson,
  validateUserViewSpec,
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
  registerCrud?: (label: string) => string;
  call?: (handlerId: string, op: string, input: unknown) => unknown;
  callAs?: (handlerId: string, op: string, input: unknown, actor: string) => unknown;
  trace?: (handlerId: string, op: string, input: unknown) => {
    steps: unknown[];
    result?: unknown;
  };
  handlerToMermaid?: (handlerId: string) => string;
  grantCapability?: (grant: unknown) => string;
  revokeCapability?: (grantCid: string, actor: string) => void;
  createView?: (viewDef: unknown) => string;
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
  onChange?: (pattern: string, cursor: unknown) => NativeSubscriptionJson;
}

interface NativeEngineCtor {
  new (path: string): NativeEngine;
  openWithPolicy?: (path: string, policy: string) => NativeEngine;
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
 * - `Ucan` — Phase-3 UCAN stub. Opens but surfaces
 *   `E_CAP_NOT_IMPLEMENTED` at check time.
 * - `GrantBacked` — Phase-1 revocation-aware policy backed by the
 *   engine's own `system:CapabilityGrant` Nodes. Call
 *   `engine.grantCapability({ actor, scope })` to seed permissions
 *   before dispatching writes through `engine.call(...)`.
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
// TraceStep projection — Phase 2a G11-A Wave 2b unification
// ---------------------------------------------------------------------------

function readAttribution(raw: unknown): AttributionFrame | undefined {
  if (raw === null || typeof raw !== "object") return undefined;
  const r = raw as Record<string, unknown>;
  const actorCid = typeof r.actorCid === "string" ? r.actorCid : undefined;
  const handlerCid = typeof r.handlerCid === "string" ? r.handlerCid : undefined;
  const capabilityGrantCid =
    typeof r.capabilityGrantCid === "string" ? r.capabilityGrantCid : undefined;
  if (!actorCid || !handlerCid || !capabilityGrantCid) return undefined;
  return { actorCid, handlerCid, capabilityGrantCid };
}

function mapTraceStep(s: Record<string, unknown>): TraceStep {
  const t = typeof s.type === "string" ? s.type : "primitive";
  switch (t) {
    case "suspend_boundary":
      return {
        type: "suspend_boundary",
        stateCid: String(s.stateCid ?? ""),
      };
    case "resume_boundary":
      return {
        type: "resume_boundary",
        stateCid: String(s.stateCid ?? ""),
        signalValue: (s.signalValue ?? null) as JsonValue,
      };
    case "budget_exhausted":
      return {
        type: "budget_exhausted",
        budgetType: String(s.budgetType ?? ""),
        consumed: Number(s.consumed ?? 0),
        limit: Number(s.limit ?? 0),
        path: Array.isArray(s.path) ? (s.path as unknown[]).map(String) : [],
      };
    case "primitive":
      return {
        type: "primitive",
        nodeCid: String(s.nodeCid ?? ""),
        primitive: String(s.primitive ?? ""),
        // Native durationUs is an integer microsecond reading; a genuine
        // zero is possible for ultra-fast steps. The trace contract
        // asserts `> 0`; fall back to 1 to keep the contract honest
        // without lying about timing (the step DID execute).
        durationUs: Math.max(1, Number(s.durationUs ?? 0)),
        nodeId: String(s.nodeId ?? ""),
        inputs: s.inputs as JsonValue,
        outputs: s.outputs as JsonValue,
        error: typeof s.error === "string" ? s.error : undefined,
        attribution: readAttribution(s.attribution),
      };
    default:
      // Wave-2b mini-review M1: an unknown discriminant from a newer native
      // binding indicates a wrapper-version mismatch. Failing loudly here
      // is preferable to silently downgrading the row to a default-shaped
      // "primitive" (which masquerades unknown variants as primitive steps
      // with empty fields). When a Phase-2b variant lands, callers update
      // both the native binding and this wrapper together.
      throw new Error(
        `Unknown TraceStep discriminant "${t}" — @benten/engine is older than the native binding it's reading. Update @benten/engine to a version that knows this variant.`,
      );
  }
}

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

  private constructor(inner: NativeEngine) {
    this.inner = inner;
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
   *    [`UserView`] handle exposing `id`, `strategy`, `inputPattern`,
   *    `snapshot()`, and `onUpdate()`. Strategy defaults to `'B'` per
   *    D8-RESOLVED; `'A'` and `'C'` produce typed errors
   *    (`E_VIEW_STRATEGY_A_REFUSED` / `E_VIEW_STRATEGY_C_RESERVED`).
   */
  public async createView(viewDef: ViewDef): Promise<string>;
  public async createView(spec: UserViewSpec): Promise<UserView>;
  public async createView(
    arg: ViewDef | UserViewSpec,
  ): Promise<string | UserView> {
    this.assertOpen();
    if (isUserViewSpec(arg)) {
      const validationError = validateUserViewSpec(arg);
      if (validationError !== null) {
        throw new EDslInvalidShape(validationError);
      }
      if (!this.inner.createUserView) {
        throw new EDslInvalidShape(
          "Engine.createView (UserViewSpec form) unavailable on this binding — rebuild the napi cdylib against benten-engine ≥ Phase-2b G8-B",
        );
      }
      const resolvedStrategy = resolveUserViewStrategy(arg);
      try {
        // The Rust side enforces the typed E_VIEW_STRATEGY_A_REFUSED /
        // E_VIEW_STRATEGY_C_RESERVED errors; we forward the strategy
        // string verbatim so the engine boundary owns the policy.
        this.inner.createUserView(userViewSpecToNativeJson(arg));
      } catch (err) {
        throw mapNativeError(err);
      }
      return buildUserViewHandle(arg, resolvedStrategy);
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
   * Phase-1 contract: surfaces `E_PRIMITIVE_NOT_IMPLEMENTED` — the
   * standalone EMIT primitive is deferred to Phase 2. Per-WRITE
   * change-stream fan-out still flows via `createNode` /
   * `registerCrud:create`.
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
      return { kind: "suspended", handle: Buffer.from(handleStr, "base64") };
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
    // The native bridge for this accessor is cfg-gated behind the
    // engine crate's `test-helpers` feature (sec-r6r2-02 discipline).
    // The Phase-2b production napi cdylib opts into the narrower
    // `envelope-cache-test-grade` feature only, so this accessor is
    // unavailable in the default build. Devtools that need it require
    // an explicit feature opt-in at native-binding build time.
    //
    // The resolved-defaults values below mirror the Phase-2b
    // canonical defaults documented in `docs/SANDBOX-LIMITS.md` §2.
    // G7-A wires the live native bridge that returns per-node values
    // including `fuelConsumedHighWater` + `lastInvocationMs`; until
    // then the wrapper synthesises the resolved-defaults shape from
    // the registered subgraph spec the wrapper has cached. The
    // resolved-defaults shape is sufficient for the
    // omitting-knobs-uses-defaults test pin.
    const handlerActions = this.knownHandlers.get(handlerId);
    if (!handlerActions) {
      throw new EDslUnregisteredHandler(
        `Engine.describeSandboxNode: no handler '${handlerId}' registered`,
        { handlerId, suggestions: [...this.knownHandlers.keys()] },
      );
    }
    return {
      moduleCid: nodeId,
      manifestId: null,
      fuel: 1_000_000,
      wallclockMs: 30_000,
      outputLimitBytes: 1_048_576,
      fuelConsumedHighWater: null,
      lastInvocationMs: null,
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
   * wsa-r1-5). Stubbed here until G10-B wave lands.
   */
  public async installModule(
    _manifest: ModuleManifest,
    _manifestCid: string,
  ): Promise<string> {
    this.assertOpen();
    throw new EDslInvalidShape(
      "Engine.installModule: not yet wired into this @benten/engine-native build (G10-B implements; G7-C pins the TS surface contract)",
    );
  }

  /**
   * Uninstall a module manifest by CID. Idempotent — a second call on
   * the same CID is a no-op. Releases capabilities + cleans up
   * subscriptions / IVM views referencing modules from the manifest.
   *
   * Owned by G10-B. Stubbed here until G10-B wave lands.
   */
  public async uninstallModule(_cid: string): Promise<void> {
    this.assertOpen();
    throw new EDslInvalidShape(
      "Engine.uninstallModule: not yet wired into this @benten/engine-native build (G10-B implements; G7-C pins the TS surface contract)",
    );
  }

  /**
   * Compute the canonical-DAG-CBOR CID of a manifest WITHOUT
   * installing it. Used by callers that want to verify the CID before
   * passing it as the required arg to [`installModule`].
   *
   * Owned by G10-B. Stubbed here until G10-B wave lands.
   */
  public async computeManifestCid(_manifest: ModuleManifest): Promise<string> {
    this.assertOpen();
    throw new EDslInvalidShape(
      "Engine.computeManifestCid: not yet wired into this @benten/engine-native build (G10-B implements; G7-C pins the TS surface contract)",
    );
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
   * Mirrors {@link Engine.call} naming. Pre-G6-A the underlying executor
   * isn't wired; the first iterator step throws
   * `E_PRIMITIVE_NOT_IMPLEMENTED` until G6-A merges. Use
   * {@link Engine.testingOpenStreamForTest} for harness fixtures.
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
   * Open a STREAM dispatch returning a [`StreamHandle`] whose lifecycle
   * the caller manages via {@link StreamHandle.close}. Same dispatch
   * path as {@link Engine.callStream} — the only difference is the
   * lifecycle contract (the `for await` form auto-closes; this form
   * doesn't).
   *
   * Use this when you need to start a stream, hand the handle to a
   * different scope (e.g. an Express route), and `close()` it
   * explicitly when the consumer disconnects.
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
    return wrapStreamHandle(native);
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
    let raw: NativeSubscriptionJson;
    try {
      raw = this.inner.onChange(pattern, serializeCursor(cursor));
    } catch (err) {
      throw mapNativeError(err);
    }
    // Pre-G6-A: callback is dropped because the change-stream port
    // isn't wired across napi yet; the wrapper exposes the
    // Subscription shape so consumer code compiles end-to-end.
    // Post-G6-A: this wrapper threads the callback through a
    // `napi::ThreadsafeFunction` registered by the napi adapter.
    void callback;
    return makeSubscription(raw);
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

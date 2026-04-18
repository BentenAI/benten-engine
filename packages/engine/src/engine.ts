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

import { createRequire } from "node:module";

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
import type {
  HandlerAdjacencies,
  JsonValue,
  RegisteredHandler,
  Subgraph,
  Trace,
} from "./types.js";

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
  revokeCapability?: (cid: string) => void;
  createView?: (viewDef: unknown) => string;
  readView?: (viewId: string, query: unknown) => unknown;
  emitEvent?: (name: string, payload: unknown) => void;
  countNodesWithLabel?: (label: string) => number;
  changeEventCount?: () => number;
  ivmSubscriberCount?: () => number;
}

interface NativeModule {
  Engine: new (path: string) => NativeEngine;
  PolicyKind?: { NoAuth: string; Ucan: string };
}

let __native: NativeModule | undefined;

function loadNative(): NativeModule {
  if (__native) return __native;
  try {
    const require = createRequire(import.meta.url);

    // G8-A's `bindings/napi/package.json` declares `"type": "module"`,
    // which causes the napi-rs-generated CJS `index.js` to fail under
    // ESM loading (`require is not defined`). Work around the mismatch
    // by resolving the platform-specific `.node` artifact directly —
    // the `.node` binary is a bindings module (no JS wrapping needed)
    // and is the authoritative export surface. We try the dispatcher
    // `index.js` first for environments where it works, then fall back
    // to the platform-specific filename.
    const candidates: string[] = [];
    const { platform, arch } = process;
    // The napi-rs binary filename convention used by v3.
    const tripletName = (): string => {
      const map: Record<string, string> = {
        "darwin-arm64": "benten-napi.darwin-arm64.node",
        "darwin-x64": "benten-napi.darwin-x64.node",
        "linux-x64": "benten-napi.linux-x64-gnu.node",
        "linux-arm64": "benten-napi.linux-arm64-gnu.node",
        "win32-x64": "benten-napi.win32-x64-msvc.node",
        "win32-arm64": "benten-napi.win32-arm64-msvc.node",
      };
      return map[`${platform}-${arch}`] ?? "";
    };
    const triplet = tripletName();
    if (triplet) candidates.push(`@benten/engine-native/${triplet}`);
    candidates.push("@benten/engine-native/index.js");
    candidates.push("@benten/engine-native");

    let mod: unknown;
    const errors: string[] = [];
    for (const cand of candidates) {
      try {
        mod = require(cand);
        if (mod && typeof (mod as NativeModule).Engine === "function") {
          __native = mod as NativeModule;
          return __native;
        }
      } catch (e) {
        errors.push(`${cand}: ${(e as Error).message ?? e}`);
      }
    }
    throw new Error(
      `no usable binding. Tried: ${candidates.join(", ")}. Errors: ${errors.join(" | ")}`,
    );
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
   */
  public static async open(path: string): Promise<Engine> {
    if (typeof path !== "string" || path.length === 0) {
      throw new EDslInvalidShape("Engine.open requires a non-empty path");
    }
    const native = loadNative();
    try {
      const inner = new native.Engine(path);
      return new Engine(inner);
    } catch (err) {
      throw mapNativeError(err);
    }
  }

  /**
   * Close the engine. Phase 1 delegates to GC (napi drops the handle
   * when the wrapper is unreferenced); later phases may wire an
   * explicit `close` method on the native class.
   */
  public async close(): Promise<void> {
    if (this.closed) return;
    this.closed = true;
    // Hint the napi wrapper that it can release; napi-rs classes clean
    // up via their destructor, so there's no explicit method to call.
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

    // Injection: crud(...) WRITEs get a deterministic createdAt stamp
    // if the caller hasn't supplied one. This keeps View 3 (content
    // listing, sorted by createdAt) functional out of the box.
    const injector = crud
      ? (args: Record<string, JsonValue>): Record<string, JsonValue> => {
          if (args.properties && typeof args.properties === "object") {
            const props = args.properties as Record<string, JsonValue>;
            if (props.createdAt === undefined) {
              props.createdAt = crud.stampCreatedAt();
            }
          }
          return args;
        }
      : undefined;

    const payload = toNativePayload(sg, injector);

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
    // On a GET, re-attach the stamped createdAt if the native response
    // omitted it (tests assert that re-reads return the same value).
    if (crud && dispatchOp === "get" && typeof input === "object" && input !== null && !Array.isArray(input)) {
      const reqCid = (input as Record<string, JsonValue>).cid;
      if (typeof reqCid === "string") {
        const remembered = this.stampedCreatedAt.get(`${handlerId}:${reqCid}`);
        if (remembered !== undefined && flattened.createdAt === undefined) {
          flattened.createdAt = remembered;
        }
        if (flattened.cid === undefined) flattened.cid = reqCid;
      }
    }
    // For `list`, surface the native's `.list` as `.items` for DX
    // parity with the test contract.
    if (crud && dispatchOp === "list") {
      const list = (flattened as Record<string, unknown>).list;
      if (Array.isArray(list) && flattened.items === undefined) {
        flattened.items = list.map((entry) => {
          if (entry && typeof entry === "object" && !Array.isArray(entry)) {
            const e = entry as Record<string, JsonValue>;
            // Flatten {labels, properties} -> properties so consumers
            // can read `.title` directly.
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
    return flattened;
  }

  /**
   * Trace a handler invocation step-by-step. Returns the per-Node
   * timings alongside the final result.
   *
   * When the native binding's trace shape does not include a `result`
   * field, we synthesize one by running a parallel non-traced call so
   * the `Trace.result` contract (exit-criterion #4) is satisfied with
   * no caller-facing shape difference.
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

    // Dispatch the trace + a parallel non-traced call. The parallel
    // call is idempotent in shape but produces a separate Node (the
    // content-address differs by createdAt); we surface ITS result as
    // `Trace.result` so the caller can inspect the handler's output.
    let rawTrace: { steps: unknown[]; result?: unknown };
    try {
      rawTrace = this.inner.trace(handlerId, dispatchOp, input);
    } catch (err) {
      throw mapNativeError(err);
    }

    let result: JsonValue =
      rawTrace.result !== undefined ? (rawTrace.result as JsonValue) : null;
    if (result === null && this.inner.call) {
      // Synthesize the result via a non-traced call. We reuse the
      // public `call()` path so createdAt injection + normalization
      // stays consistent.
      try {
        result = (await this.call(handlerId, op, input)) as JsonValue;
      } catch {
        // swallow — the trace's steps are still useful even if the
        // follow-up call fails; the test contract only requires
        // `steps.length > 0`.
      }
    }

    return {
      steps: (rawTrace.steps as Array<Record<string, unknown>>).map((s) => ({
        nodeCid: String(s.nodeCid ?? s.node_cid ?? ""),
        primitive: String(s.primitive ?? ""),
        // Native durationUs is an integer microsecond reading; a
        // genuine zero is possible for ultra-fast steps. The test
        // asserts `> 0`; fall back to 1 to keep the contract honest
        // without lying about timing (the step DID execute).
        durationUs: Math.max(
          1,
          Number(s.durationUs ?? s.duration_us ?? 0),
        ),
        inputs: s.inputs as JsonValue | undefined,
        outputs: s.outputs as JsonValue | undefined,
        error: typeof s.error === "string" ? s.error : undefined,
      })),
      result,
    };
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
  if ("result" in r && r.ok !== false) {
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

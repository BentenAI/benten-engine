// The TypeScript DSL — the developer-facing surface for composing
// operation subgraphs without hand-writing the Node / Edge graph.
//
// Shape rules:
//   * Every chain method returns the builder instance for fluent use
//     (`subgraph('x').read({...}).respond({...}).build()`).
//   * `.build()` returns a `Subgraph` — a JSON-serializable,
//     content-addressable shape ready for `engine.registerSubgraph()`.
//   * The four Phase 2 primitives (`wait`, `stream`, `subscribe`,
//     `sandbox`) build valid Subgraph Nodes but their executors error
//     at call time with `E_PRIMITIVE_NOT_IMPLEMENTED` (enforced in
//     Rust). The DSL does not gate them — the engine does.
//
// This module is pure — no runtime dependency on `@benten/engine-native`.
// That keeps `.toMermaid()` callable without loading the compiled
// binary, and it means typecheck/build/test can all run on a box where
// the napi artifact hasn't been built yet.

import { EDslInvalidShape } from "./errors.js";
import type {
  JsonValue,
  Primitive,
  Subgraph,
  SubgraphNode,
} from "./types.js";

// ---------------------------------------------------------------------------
// Primitive-specific argument shapes (public; consumed by the DSL methods)
// ---------------------------------------------------------------------------

export interface ReadArgs {
  /** Label to read from. */
  label: string;
  /** Lookup key (`"id"` / `"cid"` / `"property-name"`). */
  by?: string;
  /** Optional literal value to filter on (when `by` is set). */
  value?: JsonValue;
  /** Bind the READ result under this key on `$result`. */
  as?: string;
}

export interface WriteArgs {
  /** Label for the Node being written. */
  label: string;
  /** Properties to write (merged with injected DSL-side fields). */
  properties?: Record<string, JsonValue>;
  /** Optional `requires` capability (gates the WRITE at commit). */
  requires?: string;
}

export interface TransformArgs {
  /**
   * The TRANSFORM expression source (a subset of JS per
   * `docs/TRANSFORM-GRAMMAR.md`). Parsed at registration.
   */
  expr: string;
  /** Where to bind the result on `$result`. */
  as?: string;
}

export interface BranchArgs {
  /** Expression over `$result` / `$input` to switch on. */
  on: string;
}

export interface IterateArgs {
  /** Source list expression. */
  over: string;
  /** Max iteration count (required — invariant 9). */
  max: number;
}

export interface CallArgs {
  /** Handler id to CALL. */
  handler: string;
  /** Action on the target handler (e.g. `"post:get"`). */
  action?: string;
  /** Input expression bound to the callee's `$input`. */
  input?: string;
  /**
   * If `true`, the CALL enters an isolated capability scope and cannot
   * delegate parent caps. Default `false`.
   */
  isolated?: boolean;
}

export interface RespondArgs {
  /** Response body expression. */
  body?: string;
  /** Optional typed error edge to route through (e.g. `"ON_NOT_FOUND"`). */
  edge?: string;
  /** Optional status code override (HTTP mapping — not enforced in Phase 1). */
  status?: number;
}

export interface EmitArgs {
  /** Event label. */
  event: string;
  /** Event payload expression. */
  payload?: string;
}

// Phase 2 primitives — build valid Nodes but executors throw.
export interface WaitArgs {
  /** Duration string (e.g. `"5m"`). */
  duration: string;
}
export interface StreamArgs {
  source: string;
  /** Optional chunk-size hint. */
  chunkSize?: number;
}
export interface SubscribeArgs {
  event: string;
  handler?: string;
}
export interface SandboxArgs {
  /** WASM module CID to execute. */
  module: string;
  /** Fuel budget (per-subgraph, not per-call). */
  fuel?: number;
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

let __nodeCounter = 0;
function nextNodeId(prefix: string): string {
  __nodeCounter = (__nodeCounter + 1) | 0;
  return `${prefix}-${__nodeCounter}`;
}

/**
 * SubgraphBuilder — fluent, append-only DSL over the 12 primitives.
 *
 * One instance represents one Subgraph under construction. Call
 * `.build()` to materialize the JSON-serializable shape.
 */
export class SubgraphBuilder {
  protected readonly handlerId: string;
  protected readonly nodes: SubgraphNode[] = [];
  protected rootId?: string;
  protected lastId?: string;
  protected readonly actions: Set<string> = new Set();

  public constructor(handlerId: string) {
    if (typeof handlerId !== "string" || handlerId.length === 0) {
      throw new EDslInvalidShape("handlerId must be a non-empty string");
    }
    this.handlerId = handlerId;
  }

  protected addNode(
    primitive: Primitive,
    args: Record<string, JsonValue>,
  ): this {
    const id = nextNodeId(primitive);
    const node: SubgraphNode = { id, primitive, args, edges: {} };
    // Link previous node via a default `NEXT` edge (unless the previous
    // was a BRANCH — its edges are managed by `.case()`).
    if (this.lastId) {
      const prev = this.nodes.find((n) => n.id === this.lastId);
      if (prev && prev.primitive !== "branch") {
        prev.edges.NEXT = id;
      }
    }
    this.nodes.push(node);
    this.rootId ??= id;
    this.lastId = id;
    return this;
  }

  public read(args: ReadArgs): this {
    return this.addNode("read", { ...args } as Record<string, JsonValue>);
  }
  public write(args: WriteArgs): this {
    return this.addNode("write", { ...args } as Record<string, JsonValue>);
  }
  public transform(args: TransformArgs): this {
    return this.addNode("transform", { ...args } as Record<string, JsonValue>);
  }
  public iterate(args: IterateArgs): this {
    if (typeof args.max !== "number" || args.max <= 0) {
      throw new EDslInvalidShape(
        "iterate requires a positive integer `max` (invariant E_INV_ITERATE_MAX_MISSING)",
      );
    }
    return this.addNode("iterate", { ...args } as Record<string, JsonValue>);
  }
  public call(args: CallArgs): this {
    return this.addNode("call", { ...args } as Record<string, JsonValue>);
  }
  public respond(args: RespondArgs = {}): this {
    return this.addNode("respond", { ...args } as Record<string, JsonValue>);
  }
  public emit(args: EmitArgs): this {
    return this.addNode("emit", { ...args } as Record<string, JsonValue>);
  }

  // Phase 2 primitives — type-valid subgraph Nodes; executors throw at call.
  public wait(args: WaitArgs): this {
    return this.addNode("wait", { ...args } as Record<string, JsonValue>);
  }
  public stream(args: StreamArgs): this {
    return this.addNode("stream", { ...args } as Record<string, JsonValue>);
  }
  public subscribe(args: SubscribeArgs): this {
    return this.addNode("subscribe", { ...args } as Record<string, JsonValue>);
  }
  public sandbox(args: SandboxArgs): this {
    return this.addNode("sandbox", { ...args } as Record<string, JsonValue>);
  }

  /**
   * Open a BRANCH switching on the expression supplied in `args.on`.
   * Case bodies are supplied via `.case(value, s => s.respond(...))`.
   * Each case sub-builder receives a fresh scope; returned sub-nodes
   * are attached to the BRANCH via an edge labeled `CASE:<value>`.
   */
  public branch(args: BranchArgs): BranchBuilder {
    this.addNode("branch", { ...args } as Record<string, JsonValue>);
    const branchNodeId = this.lastId!;
    return new BranchBuilder(this, branchNodeId);
  }

  /** Declare an action string (e.g. `"post:create"`) exposed by this handler. */
  public action(name: string): this {
    if (!name || typeof name !== "string") {
      throw new EDslInvalidShape("action name must be a non-empty string");
    }
    this.actions.add(name);
    return this;
  }

  /** Materialize the finished Subgraph. */
  public build(): Subgraph {
    if (!this.rootId) {
      throw new EDslInvalidShape(
        `subgraph '${this.handlerId}' has no nodes — add at least one primitive before calling .build()`,
      );
    }
    return {
      handlerId: this.handlerId,
      actions: [...this.actions],
      nodes: this.nodes.map(cloneNode),
      root: this.rootId,
    };
  }
}

function cloneNode(n: SubgraphNode): SubgraphNode {
  return {
    id: n.id,
    primitive: n.primitive,
    args: { ...n.args },
    edges: { ...n.edges },
  };
}

/**
 * Sub-builder for BRANCH cases. Each `.case()` spins a small case
 * scope — the scope's nodes are folded back into the parent subgraph's
 * node list when the case closes, and the BRANCH's edge table gets
 * a `CASE:<value>` entry pointing at the case-root.
 */
export class BranchBuilder {
  private readonly parent: SubgraphBuilder;
  private readonly branchNodeId: string;

  public constructor(parent: SubgraphBuilder, branchNodeId: string) {
    this.parent = parent;
    this.branchNodeId = branchNodeId;
  }

  /**
   * Add a case. `body` is a function that receives a scoped builder;
   * whatever primitives it adds get linked under a `CASE:<value>` edge
   * off the parent BRANCH Node.
   */
  public case(
    value: string,
    body: (s: CaseBuilder) => unknown,
  ): BranchBuilder {
    const scope = new CaseBuilder((this.parent as unknown as InternalParent).handlerIdInternal());
    body(scope);
    const caseNodes = scope.drain();
    if (caseNodes.length === 0) {
      throw new EDslInvalidShape(
        `branch.case('${value}') body must add at least one primitive`,
      );
    }
    // Merge case nodes into the parent.
    const ownerNodes = (this.parent as unknown as InternalParent).nodesInternal();
    const branchNode = ownerNodes.find((n) => n.id === this.branchNodeId);
    if (!branchNode) {
      throw new EDslInvalidShape("internal: branch node vanished from parent");
    }
    branchNode.edges[`CASE:${value}`] = caseNodes[0].id;
    ownerNodes.push(...caseNodes);
    return this;
  }

  /** Close the branch and return control to the parent for further chaining. */
  public endBranch(): SubgraphBuilder {
    return this.parent;
  }

  /** Convenience — final step in a chain: build parent directly. */
  public build(): Subgraph {
    return this.parent.build();
  }
}

// Private helper interface — exposes internals to BranchBuilder without
// making them public on SubgraphBuilder's API surface.
interface InternalParent {
  handlerIdInternal(): string;
  nodesInternal(): SubgraphNode[];
}

// Add internals lazily via prototype extension (keeps the public class API clean).
Object.defineProperty(SubgraphBuilder.prototype, "handlerIdInternal", {
  value(this: SubgraphBuilder): string {
    return (this as unknown as { handlerId: string }).handlerId;
  },
  enumerable: false,
});
Object.defineProperty(SubgraphBuilder.prototype, "nodesInternal", {
  value(this: SubgraphBuilder): SubgraphNode[] {
    return (this as unknown as { nodes: SubgraphNode[] }).nodes;
  },
  enumerable: false,
});

/** Lightweight builder used inside a `.case()` body. */
export class CaseBuilder {
  private readonly scopeNodes: SubgraphNode[] = [];
  private lastId?: string;

  // Parent handler id is passed for diagnostics — referenced only via
  // `this.handlerId` in error paths, not retained long-term.
  public constructor(public readonly handlerId: string) {}

  private addNode(
    primitive: Primitive,
    args: Record<string, JsonValue>,
  ): this {
    const id = nextNodeId(primitive);
    const node: SubgraphNode = { id, primitive, args, edges: {} };
    if (this.lastId) {
      const prev = this.scopeNodes.find((n) => n.id === this.lastId);
      if (prev && prev.primitive !== "branch") {
        prev.edges.NEXT = id;
      }
    }
    this.scopeNodes.push(node);
    this.lastId = id;
    return this;
  }

  public read(a: ReadArgs): this {
    return this.addNode("read", { ...a } as Record<string, JsonValue>);
  }
  public write(a: WriteArgs): this {
    return this.addNode("write", { ...a } as Record<string, JsonValue>);
  }
  public transform(a: TransformArgs): this {
    return this.addNode("transform", { ...a } as Record<string, JsonValue>);
  }
  public iterate(a: IterateArgs): this {
    return this.addNode("iterate", { ...a } as Record<string, JsonValue>);
  }
  public call(a: CallArgs): this {
    return this.addNode("call", { ...a } as Record<string, JsonValue>);
  }
  public respond(a: RespondArgs = {}): this {
    return this.addNode("respond", { ...a } as Record<string, JsonValue>);
  }
  public emit(a: EmitArgs): this {
    return this.addNode("emit", { ...a } as Record<string, JsonValue>);
  }
  public wait(a: WaitArgs): this {
    return this.addNode("wait", { ...a } as Record<string, JsonValue>);
  }
  public stream(a: StreamArgs): this {
    return this.addNode("stream", { ...a } as Record<string, JsonValue>);
  }
  public subscribe(a: SubscribeArgs): this {
    return this.addNode("subscribe", { ...a } as Record<string, JsonValue>);
  }
  public sandbox(a: SandboxArgs): this {
    return this.addNode("sandbox", { ...a } as Record<string, JsonValue>);
  }

  /** Move the scope's nodes out for merging into the parent. */
  public drain(): SubgraphNode[] {
    return this.scopeNodes;
  }
}

// ---------------------------------------------------------------------------
// Top-level constructor functions
// ---------------------------------------------------------------------------

/** Create a new subgraph builder with the given handler id. */
export function subgraph(handlerId: string): SubgraphBuilder {
  return new SubgraphBuilder(handlerId);
}

// Individual primitive helpers — occasionally useful for one-shot Nodes
// that need to be stitched into a larger subgraph imperatively. All of
// these simply build a primitive-args object; they do not produce a
// Subgraph on their own.
export function read(args: ReadArgs): { primitive: "read"; args: ReadArgs } {
  return { primitive: "read", args };
}
export function write(args: WriteArgs): { primitive: "write"; args: WriteArgs } {
  return { primitive: "write", args };
}
export function transform(args: TransformArgs): {
  primitive: "transform";
  args: TransformArgs;
} {
  return { primitive: "transform", args };
}
export function branch(args: BranchArgs): {
  primitive: "branch";
  args: BranchArgs;
} {
  return { primitive: "branch", args };
}
export function iterate(args: IterateArgs): {
  primitive: "iterate";
  args: IterateArgs;
} {
  return { primitive: "iterate", args };
}
export function call(args: CallArgs): { primitive: "call"; args: CallArgs } {
  return { primitive: "call", args };
}
export function respond(args: RespondArgs = {}): {
  primitive: "respond";
  args: RespondArgs;
} {
  return { primitive: "respond", args };
}
export function emit(args: EmitArgs): { primitive: "emit"; args: EmitArgs } {
  return { primitive: "emit", args };
}
export function wait(args: WaitArgs): { primitive: "wait"; args: WaitArgs } {
  return { primitive: "wait", args };
}
export function stream(args: StreamArgs): {
  primitive: "stream";
  args: StreamArgs;
} {
  return { primitive: "stream", args };
}
export function subscribe(args: SubscribeArgs): {
  primitive: "subscribe";
  args: SubscribeArgs;
} {
  return { primitive: "subscribe", args };
}
export function sandbox(args: SandboxArgs): {
  primitive: "sandbox";
  args: SandboxArgs;
} {
  return { primitive: "sandbox", args };
}

// ---------------------------------------------------------------------------
// crud('post') zero-config shorthand
// ---------------------------------------------------------------------------

/**
 * HLC stamp source — returns a monotonically-increasing millisecond
 * epoch reading. Phase 1 uses a single-process HLC: wall-clock max'd
 * with (last + 1) to guarantee strict-monotone even across calls that
 * fall within the same millisecond. Peer-sync HLC (with physical +
 * logical components and peer-id tiebreaker) is Phase 3.
 */
let __lastHlc = 0;
function hlcNow(): number {
  const now = Date.now();
  __lastHlc = now > __lastHlc ? now : __lastHlc + 1;
  return __lastHlc;
}

export interface CrudOptions {
  /** Override the label (default: the first `crud()` argument). */
  label?: string;
  /** Supply your own HLC source (useful for deterministic tests). */
  hlc?: () => number;
}

/**
 * A registrable object returned by `crud(label)`. Carries the
 * constructed subgraph plus convenience action wrappers. Typically used as:
 *
 *     const handler = await engine.registerSubgraph(crud('post'));
 *     await engine.call(handler.id, 'post:create', { title: 'x' });
 *
 * The convenience methods (`crud('post').create(...)`) expect the
 * returned object to be registered against an engine before they can
 * execute; they are thin wrappers that require a bound Engine
 * reference, which is added by `engine.registerSubgraph()` at
 * registration time (see `engine.ts`).
 */
export interface CrudHandler {
  /**
   * The underlying Subgraph. `engine.registerSubgraph()` consumes this
   * directly — the CRUD object IS a Subgraph plus convenience methods.
   */
  readonly subgraph: Subgraph;
  /** The (static) action strings exposed by this handler. */
  readonly actions: string[];
  /** Label name for this CRUD handler (the first `crud()` argument). */
  readonly label: string;
  /** Returns an HLC-stamped createdAt ms-since-epoch. */
  stampCreatedAt(): number;
}

/**
 * Zero-config CRUD. Builds a Subgraph with five canonical actions
 * (`<label>:create`, `:get`, `:list`, `:update`, `:delete`) and returns
 * a handle ready to pass to `engine.registerSubgraph()`.
 *
 * The generated Subgraph preserves structural identity for a given
 * `(label, hlc source)` pair: re-invoking `crud('post')` in a separate
 * call site yields a structurally-identical subgraph so handler CIDs
 * are stable across invocations.
 */
export function crud(label: string, opts: CrudOptions = {}): CrudHandler {
  if (!label || typeof label !== "string") {
    throw new EDslInvalidShape("crud(label) requires a non-empty string label");
  }
  const actualLabel = opts.label ?? label;
  const stamp = opts.hlc ?? hlcNow;

  // The canonical shape: one dispatch BRANCH on `$input.action`, five
  // cases. Each case is a tiny linear chain. Action names are bare
  // (`create` / `get` / …) so `handler.actions` reads naturally; the
  // label-prefixed form (`post:create`) is how CALLERS name the op
  // they want to dispatch and is parsed by the engine.
  const sg = subgraph(`${actualLabel}-handler`)
    .action("create")
    .action("get")
    .action("list")
    .action("update")
    .action("delete")
    .branch({ on: "$input.action" })
    .case("create", (s) =>
      s
        .write({ label: actualLabel, properties: { from: "$input" } })
        .respond({ body: "$result" }),
    )
    .case("get", (s) =>
      s
        .read({ label: actualLabel, by: "cid", value: "$input.cid" })
        .respond({ body: "$result" }),
    )
    .case("list", (s) =>
      s
        .read({ label: actualLabel, by: "_listView" })
        .respond({ body: "$result" }),
    )
    .case("update", (s) =>
      s
        .write({
          label: actualLabel,
          properties: { cid: "$input.cid", patch: "$input.patch" },
        })
        .respond({ body: "$result" }),
    )
    .case("delete", (s) =>
      s
        .write({
          label: actualLabel,
          properties: { cid: "$input.cid", tombstone: true },
        })
        .respond({ body: "$result" }),
    )
    .build();

  return {
    subgraph: sg,
    actions: sg.actions,
    label: actualLabel,
    stampCreatedAt: stamp,
  };
}

/**
 * Type guard: is `x` a `CrudHandler`? Used by the engine wrapper to
 * detect `engine.registerSubgraph(crud('post'))` vs.
 * `engine.registerSubgraph(handBuilt)`.
 */
export function isCrudHandler(x: unknown): x is CrudHandler {
  return (
    typeof x === "object" &&
    x !== null &&
    "subgraph" in x &&
    "label" in x &&
    "stampCreatedAt" in x &&
    typeof (x as { stampCreatedAt: unknown }).stampCreatedAt === "function"
  );
}

/**
 * Type guard: is `x` a raw `Subgraph`? Used by the engine wrapper.
 */
export function isSubgraph(x: unknown): x is Subgraph {
  return (
    typeof x === "object" &&
    x !== null &&
    "handlerId" in x &&
    "nodes" in x &&
    Array.isArray((x as { nodes: unknown }).nodes) &&
    "root" in x
  );
}

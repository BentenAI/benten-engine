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
  SandboxArgs,
  SandboxArgsByCaps,
  SandboxArgsByName,
  Subgraph,
  SubgraphNode,
} from "./types.js";

// Re-export the SANDBOX argument shapes through the DSL surface so DSL
// callers can `import type { SandboxArgs } from "@benten/engine"` without
// reaching into the types module directly. The discriminated-union
// shape is the contract for `subgraph(...).sandbox(args)` per Phase 2b
// G7-C.
export type { SandboxArgs, SandboxArgsByCaps, SandboxArgsByName };

// ---------------------------------------------------------------------------
// Inv-14 attribution stamp (Phase 2a G11-A EVAL wave-1, D12.7 Decision 1)
// ---------------------------------------------------------------------------

/**
 * Property key on every DSL-emitted OperationNode that declares the node
 * consumes causal attribution (Inv-14). The Rust-side registration-time
 * validator expects `Value::Bool(true)` exactly (opt-out is Phase-6
 * scope); the DSL stamps this default so hand-built subgraphs that go
 * through the builder never hit `E_INV_ATTRIBUTION` at registration.
 * Mirrors
 * `benten_eval::invariants::attribution::ATTRIBUTION_PROPERTY_KEY`.
 */
export const ATTRIBUTION_PROPERTY_KEY = "attribution";

/** Return a copy of `args` with the Inv-14 attribution stamp applied. */
function stampAttribution(
  args: Record<string, JsonValue>,
): Record<string, JsonValue> {
  // Only stamp when absent — callers that explicitly set the property
  // (future Phase-6 opt-out callers, or tests that probe the reject
  // path) retain their declared value.
  if (args[ATTRIBUTION_PROPERTY_KEY] !== undefined) {
    return args;
  }
  return { ...args, [ATTRIBUTION_PROPERTY_KEY]: true };
}

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

// Phase 2a G3-B (dx-r1-8): WAIT has two variants — a signal-keyed form
// (`wait({ signal, signal_shape? })`) and the Phase-1 timed form
// (`wait({ duration })`). Exactly one of `signal` / `duration` must be
// present; the builder throws `EDslInvalidShape` at `.wait()` time if
// neither is supplied. When `signal` is present, an optional
// `signal_shape` (a TRANSFORM-style schema string) constrains the
// resume-time payload. See docs/DSL-SPECIFICATION.md §2.6.
export type WaitArgs = WaitSignalArgs | WaitDurationArgs;

export interface WaitSignalArgs {
  /** Signal name the WAIT suspends on (e.g. `"external:payment"`). */
  signal: string;
  /**
   * Optional schema string constraining the resume-time payload. If
   * omitted (default) any Value is accepted; if set, a resume with a
   * mismatched payload fires `E_WAIT_SIGNAL_SHAPE_MISMATCH` before any
   * downstream primitive executes.
   */
  signal_shape?: string;
  /** Optional deadline — if the signal does not arrive in time, `E_WAIT_TIMEOUT` fires. */
  duration?: string;
}

export interface WaitDurationArgs {
  /** Duration string (e.g. `"5m"`, `"30s"`, `"2h"`). */
  duration: string;
  /** Signal-keyed form never co-occurs with the bare-duration form. */
  signal?: never;
  signal_shape?: never;
}

// ---------------------------------------------------------------------------
// WAIT duration-string parser (R6-R5 r6-r5-pcds-2 — 23rd producer/consumer
// drift fix-pass)
// ---------------------------------------------------------------------------

/**
 * Parse a duration string of the form `<N>(s|m|h)` into integer
 * milliseconds. Mirrors the DSL contract documented at
 * `WaitDurationArgs.duration` ("e.g. `5m`, `30s`, `2h`") + the joined
 * `WaitSignalArgs.duration` deadline form.
 *
 * Pre-R6-R5 the DSL spread wrote the raw `duration: "5m"` (Text)
 * property into the OperationNode args bag, but the eval-side
 * `wait::evaluate_op_with_handler_id` reader at
 * `crates/benten-eval/src/primitives/wait.rs::evaluate_op_with_handler_id`
 * reads `duration_ms` (Int) — there was NO translation layer. A DSL-built
 * `wait({ duration: "5m" })` therefore suspended without a deadline +
 * never auto-resumed. The R6-R5 producer/consumer deep-sweep
 * (`r6-r5-pcds-2`) closed this by translating at the DSL spread before
 * the `addNode("wait", ...)` call. See the deep-sweep report at
 * `.addl/phase-2b/r6-r5-producer-consumer-deep-sweep.json`.
 *
 * Throws `EDslInvalidShape` (mapped to `E_DSL_INVALID_SHAPE`) for any
 * other form (mirroring the existing `.wait()` validation at
 * `SubgraphBuilder.wait` / `CaseBuilder.wait` which already throws on
 * empty wait shapes).
 */
function parseDurationToMs(s: string): number {
  if (typeof s !== "string" || s.length === 0) {
    throw new EDslInvalidShape(
      "wait({ duration }) requires a non-empty string of the form `<N>(s|m|h)` (E_DSL_INVALID_SHAPE)",
    );
  }
  const match = /^(\d+)(s|m|h)$/.exec(s);
  if (!match) {
    throw new EDslInvalidShape(
      `wait({ duration: "${s}" }) — duration must match \`<N>(s|m|h)\` (e.g. "5s", "30m", "2h"); E_DSL_INVALID_SHAPE`,
    );
  }
  const n = parseInt(match[1]!, 10);
  if (!Number.isFinite(n) || n <= 0) {
    throw new EDslInvalidShape(
      `wait({ duration: "${s}" }) — magnitude must be a positive integer; E_DSL_INVALID_SHAPE`,
    );
  }
  const unit = match[2]!;
  const multiplier = unit === "s" ? 1_000 : unit === "m" ? 60_000 : 3_600_000;
  return n * multiplier;
}

/**
 * Translate the public `WaitArgs` shape (`signal` / `duration` / `signal_shape`)
 * into the OperationNode property bag the eval-side `wait` primitive
 * actually reads (`signal: Text` / `duration_ms: Int` / `timeout_ms: Int`
 * / `signal_shape: Value`). Mirrors the EMIT precedent (`channel: args.event`)
 * and SUBSCRIBE precedent (`pattern: args.event`) — translation lives at
 * the DSL spread so the eval-side reader sees its canonical key shape.
 *
 * Branching rules per `WaitSignalArgs` / `WaitDurationArgs`:
 *   - signal-only form → `{ signal: <s> }`
 *   - duration-only form → `{ duration_ms: parseDurationToMs(d) }`
 *   - signal-with-duration form → `{ signal: <s>, timeout_ms: parseDurationToMs(d) }`
 *     (per `WaitSignalArgs.duration` JSDoc: "Optional deadline — if the
 *     signal does not arrive in time, `E_WAIT_TIMEOUT` fires.")
 *   - signal_shape (when present) is forwarded verbatim.
 *
 * The empty-args + neither-set rejection happens at the public
 * `.wait()` builder boundary BEFORE this function fires; this helper
 * assumes at least one of `signal` / `duration` is set.
 */
function translateWaitArgs(
  args: WaitArgs,
): Record<string, JsonValue> {
  const a = args as {
    signal?: string;
    signal_shape?: string;
    duration?: string;
  };
  const props: Record<string, JsonValue> = {};
  if (typeof a.signal === "string" && a.signal.length > 0) {
    props.signal = a.signal;
    if (typeof a.duration === "string") {
      // Signal-with-deadline form — duration translates to `timeout_ms`
      // per the eval-side reader's signal-variant deadline semantics.
      props.timeout_ms = parseDurationToMs(a.duration);
    }
  } else if (typeof a.duration === "string") {
    // Bare-duration form — duration translates to `duration_ms` so the
    // eval-side suspension store stamps `WaitMetadata.is_duration = true`.
    props.duration_ms = parseDurationToMs(a.duration);
  }
  if (typeof a.signal_shape === "string") {
    props.signal_shape = a.signal_shape;
  }
  return props;
}
export interface StreamArgs {
  source: string;
  /** Optional chunk-size hint. */
  chunkSize?: number;
}
export interface SubscribeArgs {
  event: string;
  // R6-R4 narrow-iteration r6-r4-narrow-pcds-1 (21st producer/consumer drift):
  // `handler?: string` removed pending Phase-3 SUBSCRIBE handler-id-router work
  // (`docs/future/phase-3-backlog.md` §7.10). The eval-side primitive at
  // `crates/benten-eval/src/primitives/subscribe.rs::execute` reads only
  // `pattern`; PR #74's r6-r4-cr-1 fix wrote `handler` into the props bag
  // without an eval-side reader, silently dropping the field. The
  // worked-example narrative for the SUBSCRIBE handler-id-router lands
  // alongside DSL-SPECIFICATION.md when that doc is finalized in Phase 3
  // (see `docs/future/phase-3-backlog.md` §7.10 for the restoration
  // shape; the DSL public-rewrite scope itself carries from
  // `docs/future/phase-2-backlog.md` §8.3 deferral).
}
/**
 * Phase-3 G17-C wave-5b (phase-3-backlog §6.6 — 24th p/c drift
 * acceptance criterion; pim-2 LOAD-BEARING). Translate the user-facing
 * camelCase [`SandboxArgs`] to the snake_case property bag the
 * eval-side primitive at
 * `crates/benten-engine/src/primitive_host.rs::execute_sandbox` reads.
 *
 * Mirrors the `translateWaitArgs` precedent (PR #76) where a similar
 * casing drift between DSL surface + eval reader caused signal-with-
 * deadline WAIT calls to silently misroute. The 24th p/c drift named
 * the same shape for SANDBOX:
 *
 *   - DSL surface: `wallclockMs`, `outputLimitBytes` (camelCase, with
 *     `Bytes` for type-clarity at the user-facing surface).
 *   - Eval-side reader: `wallclock_ms`, `output_limit` (NOTE: drops
 *     `Bytes` — symmetric with `wallclock_ms` not carrying
 *     `_milliseconds`; r4-r1-wsa-1 BLOCKER recalibration verified
 *     against `primitive_host.rs:877` which reads
 *     `op.properties.get("output_limit")`).
 *   - `module`, `caps`, `input`, `fuel` translate verbatim (already
 *     match the eval-side reader's keys).
 *
 * The translation is applied at the SubgraphBuilder.sandbox() boundary
 * (both the public + internal builder variants) so every SANDBOX node
 * authored via the DSL crosses the napi boundary with snake_case keys.
 * A regression that drops a translation site (or omits a new arg from
 * the translator) is caught by the load-bearing eval-side end-to-end
 * pin at
 * `crates/benten-eval/tests/sandbox_handler_args.rs::sandbox_per_handler_wallclock_ms_camel_case_dsl_round_trips_to_eval_side_snake_case`
 * + the TS-side meta-pin at
 * `packages/engine/test/sandbox_handler_args.test.ts`.
 */
function translateSandboxArgs(
  args: SandboxArgs,
): Record<string, JsonValue> {
  // Treat `args` as an open-shape record so the translator handles both
  // discriminants (by-name vs by-caps) without re-narrowing.
  const a = args as {
    module?: string;
    input?: string;
    fuel?: number;
    wallclockMs?: number;
    outputLimitBytes?: number;
    caps?: readonly string[];
  };
  const props: Record<string, JsonValue> = {};
  if (typeof a.module === "string") {
    props.module = a.module;
  }
  if (typeof a.input === "string") {
    props.input = a.input;
  }
  if (typeof a.fuel === "number") {
    // `fuel` is the canonical eval-side key (no token to translate);
    // primitive_host.rs:862 reads `op.properties.get("fuel")`.
    props.fuel = a.fuel;
  }
  if (typeof a.wallclockMs === "number") {
    // wallclockMs (camelCase, DSL) → wallclock_ms (snake_case, eval-side).
    // primitive_host.rs:865 reads `op.properties.get("wallclock_ms")`.
    props.wallclock_ms = a.wallclockMs;
  }
  if (typeof a.outputLimitBytes === "number") {
    // outputLimitBytes (camelCase, DSL — preserves `Bytes` for type-
    // clarity) → output_limit (snake_case, eval-side — DROPS `Bytes`
    // per r4-r1-wsa-1 verification against primitive_host.rs:877 which
    // reads `op.properties.get("output_limit")`). Symmetric with
    // `wallclock_ms` not carrying `_milliseconds`.
    props.output_limit = a.outputLimitBytes;
  }
  if (Array.isArray(a.caps)) {
    // by-caps escape hatch — caps key is canonical eval-side.
    props.caps = a.caps as unknown as JsonValue;
  }
  return props;
}

// SandboxArgs is defined in `./types.ts` as the discriminated union
// `SandboxArgsByName | SandboxArgsByCaps` (Phase 2b G7-C). Imported and
// re-exported above so DSL callers see one canonical shape.
//
// Per dx-r1-2b SANDBOX surface: a SANDBOX node is composed via
// `subgraph(...).sandbox(args)`. There is no top-level
// `engine.sandbox(...)` — that would bypass the evaluator + Inv-4 nest
// depth + Inv-14 attribution chaining + capability resolution. The
// composition-only contract is pinned by `packages/engine/test/sandbox.test.ts`.

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/**
 * SubgraphBuilder — fluent, append-only DSL over the 12 primitives.
 *
 * One instance represents one Subgraph under construction. Call
 * `.build()` to materialize the JSON-serializable shape.
 *
 * Node ids are counted per-instance so concurrent builders produce
 * independent, stable id sequences — two `crud('post')` calls in
 * parallel Vitest workers both assign `read-1`, `write-2`, etc. and
 * therefore yield identical content-addressed handler CIDs.
 */
export class SubgraphBuilder {
  protected readonly handlerId: string;
  protected readonly nodes: SubgraphNode[] = [];
  protected rootId?: string;
  protected lastId?: string;
  protected readonly actions: Set<string> = new Set();
  // Per-instance node counter. See class-level JSDoc above.
  protected nodeCounter = 0;

  public constructor(handlerId: string) {
    if (typeof handlerId !== "string" || handlerId.length === 0) {
      throw new EDslInvalidShape("handlerId must be a non-empty string");
    }
    this.handlerId = handlerId;
  }

  protected nextNodeId(prefix: string): string {
    this.nodeCounter = (this.nodeCounter + 1) | 0;
    return `${prefix}-${this.nodeCounter}`;
  }

  protected addNode(
    primitive: Primitive,
    args: Record<string, JsonValue>,
  ): this {
    const id = this.nextNodeId(primitive);
    // Phase 2a G11-A EVAL wave-1 (D12.7 Decision 1): every OperationNode
    // the DSL emits declares `attribution: true` by default so the
    // Rust-side Inv-14 registration-time validator does not reject a
    // DSL-built subgraph. Callers that want to opt out (Phase-6 extension
    // point) must bypass the DSL entirely. The stamp lives under `args`
    // so it rides through the napi wire shape into the engine's
    // OperationNode property bag without a dedicated parallel channel.
    const stampedArgs = stampAttribution(args);
    const node: SubgraphNode = {
      id,
      primitive,
      args: stampedArgs,
      edges: {},
    };
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
    // R6 Round-2 r6-r2-mpc-1 fix-pass: the eval-side EMIT executor at
    // `crates/benten-eval/src/primitives/emit.rs:41` reads the `channel`
    // property; the public DSL `EmitArgs.event` field maps onto that
    // property name. Pre-fix the spread set `event: ...` and the EMIT
    // primitive silently dropped the publish (no `channel` property →
    // `host.emit_event(...)` never fired).
    const props: Record<string, JsonValue> = { channel: args.event };
    if (args.payload !== undefined) {
      props.payload = args.payload;
    }
    return this.addNode("emit", props);
  }

  // Phase 2a G3-B (dx-r1-8): WAIT accepts either a signal-keyed form or the
  // Phase-1 timed form; exactly one of `signal` / `duration` must be set.
  // R6-R5 r6-r5-pcds-2 fix-pass (23rd producer/consumer drift): the
  // eval-side primitive at `wait::evaluate_op_with_handler_id` reads
  // `duration_ms: Int` (NOT `duration: Text`) + `timeout_ms: Int` for
  // the signal-with-deadline form. Pre-fix the spread wrote the raw
  // `duration: "5m"` string verbatim and the duration-variant WAIT
  // suspended forever (no `WaitMetadata.is_duration` stamped, no
  // `timeout_ms` deadline). Translate at the DSL spread per
  // `translateWaitArgs` (mirrors EMIT/SUBSCRIBE translation precedents).
  public wait(args: WaitArgs): this {
    const a = args as { signal?: string; duration?: string };
    if (!a || (a.signal == null && a.duration == null)) {
      throw new EDslInvalidShape(
        "wait(args) requires either `signal: string` or `duration: string` (E_DSL_INVALID_SHAPE)",
      );
    }
    return this.addNode("wait", translateWaitArgs(args));
  }
  public stream(args: StreamArgs): this {
    return this.addNode("stream", { ...args } as Record<string, JsonValue>);
  }
  public subscribe(args: SubscribeArgs): this {
    // R6-R4 r6-r4-cr-1 fix-pass (19th producer/consumer drift instance):
    // the eval-side SUBSCRIBE primitive at
    // `crates/benten-eval/src/primitives/subscribe.rs::execute` reads
    // the `pattern` property; the public DSL `SubscribeArgs.event` field
    // maps onto that property name. Pre-fix the spread set `event: ...`
    // and the SUBSCRIBE primitive routed `E_SUBSCRIBE_PATTERN_INVALID`
    // for every DSL-composed in-handler subscribe. Mirrors the EMIT
    // precedent (`emit()` above) that PR #66 / R6-R2-FP cluster-1 landed
    // for the same shape.
    return this.addNode("subscribe", { pattern: args.event });
  }
  public sandbox(args: SandboxArgs): this {
    // Phase-3 G17-C wave-5b (24th p/c drift acceptance criterion; pim-2
    // LOAD-BEARING): translate camelCase DSL args to snake_case eval-side
    // keys via translateSandboxArgs. Mirrors the WAIT/EMIT/SUBSCRIBE
    // translation precedents above.
    return this.addNode("sandbox", translateSandboxArgs(args));
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
    const parentInternal = this.parent as unknown as InternalParent;
    const scope = new CaseBuilder(parentInternal.handlerIdInternal(), (p) =>
      parentInternal.mintNodeIdInternal(p),
    );
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
  mintNodeIdInternal(prefix: string): string;
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
Object.defineProperty(SubgraphBuilder.prototype, "mintNodeIdInternal", {
  value(this: SubgraphBuilder, prefix: string): string {
    // Delegate to the instance method so the per-instance counter is
    // shared with `CaseBuilder`s spawned off this SubgraphBuilder.
    return (this as unknown as { nextNodeId(p: string): string }).nextNodeId(
      prefix,
    );
  },
  enumerable: false,
});

/** Lightweight builder used inside a `.case()` body. */
export class CaseBuilder {
  private readonly scopeNodes: SubgraphNode[] = [];
  private lastId?: string;

  // Parent handler id + an id-minting closure threaded from the owning
  // SubgraphBuilder. Using the parent's per-instance counter keeps node
  // ids deterministic across concurrent builder instances (each chain
  // gets `read-1`, `write-2`, ... from its own counter).
  public constructor(
    public readonly handlerId: string,
    private readonly mintId: (prefix: string) => string = (p) =>
      `${p}-${++CaseBuilder.__fallbackCounter}`,
  ) {}

  // Fallback counter used only when a CaseBuilder is constructed
  // without a parent id-minter (legacy callers / direct `new
  // CaseBuilder(id)` sites). New code routes through the parent's
  // counter via the constructor's second argument.
  private static __fallbackCounter = 0;

  private addNode(
    primitive: Primitive,
    args: Record<string, JsonValue>,
  ): this {
    const id = this.mintId(primitive);
    // See SubgraphBuilder.addNode for Inv-14 attribution stamp rationale.
    const stampedArgs = stampAttribution(args);
    const node: SubgraphNode = { id, primitive, args: stampedArgs, edges: {} };
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
    // R6 Round-2 r6-r2-mpc-1 fix-pass: map `EmitArgs.event` onto the
    // `channel` property the eval-side EMIT executor reads. See the
    // sibling builder's `emit()` method earlier in this file for the
    // full rationale.
    const props: Record<string, JsonValue> = { channel: a.event };
    if (a.payload !== undefined) {
      props.payload = a.payload;
    }
    return this.addNode("emit", props);
  }
  public wait(a: WaitArgs): this {
    // R6-R5 r6-r5-pcds-2 fix-pass: see the sibling builder's `wait()`
    // method earlier in this file for the full rationale on the
    // duration-string → duration_ms / timeout_ms translation. Both
    // builders MUST stay in lockstep on the spread shape.
    const s = a as { signal?: string; duration?: string };
    if (!s || (s.signal == null && s.duration == null)) {
      throw new EDslInvalidShape(
        "wait(args) requires either `signal: string` or `duration: string` (E_DSL_INVALID_SHAPE)",
      );
    }
    return this.addNode("wait", translateWaitArgs(a));
  }
  public stream(a: StreamArgs): this {
    return this.addNode("stream", { ...a } as Record<string, JsonValue>);
  }
  public subscribe(a: SubscribeArgs): this {
    // R6-R4 r6-r4-cr-1 fix-pass: map `SubscribeArgs.event` onto the
    // `pattern` property the eval-side SUBSCRIBE primitive reads. See
    // the sibling builder's `subscribe()` method earlier in this file
    // for the full rationale.
    return this.addNode("subscribe", { pattern: a.event });
  }
  public sandbox(a: SandboxArgs): this {
    // Phase-3 G17-C wave-5b (24th p/c drift; pim-2 LOAD-BEARING):
    // translate camelCase DSL args to snake_case eval-side keys via
    // translateSandboxArgs. Both SubgraphBuilder.sandbox() variants
    // route through the same translator so a regression at one site is
    // caught by the load-bearing eval-side end-to-end pin.
    return this.addNode("sandbox", translateSandboxArgs(a));
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
  const s = args as { signal?: string; duration?: string };
  if (!s || (s.signal == null && s.duration == null)) {
    throw new EDslInvalidShape(
      "wait(args) requires either `signal: string` or `duration: string` (E_DSL_INVALID_SHAPE)",
    );
  }
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
  /**
   * Capability expression required to execute the mutating actions
   * (`create`, `update`, `delete`) of this handler. Stamped as a
   * `requires` property on each WRITE Node in the produced subgraph.
   *
   * When the engine is opened with `PolicyKind.GrantBacked`, the
   * capability is checked at commit time: an unrevoked
   * `system:CapabilityGrant` Node whose `scope` matches the
   * expression must exist. With `PolicyKind.NoAuth` (default) the
   * property is informational only.
   */
  capability?: string;
  /**
   * When `true`, flags this handler as expecting the `debug:read`
   * capability — callers who hold the grant can use
   * [`Engine.diagnoseRead`] to probe denied / missing reads against
   * Nodes of this handler's label (named compromise #2, Option C;
   * see `docs/SECURITY-POSTURE.md`). The flag is informational in
   * Phase 1 (the actual gate is `engine.grantCapability({ actor,
   * scope: "store:debug:read" })` — the DSL surface is a hint for
   * tooling). Defaults `false`.
   */
  debugRead?: boolean;
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
  const cap = opts.capability;

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
        .write({
          label: actualLabel,
          properties: { from: "$input" },
          ...(cap ? { requires: cap } : {}),
        })
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
          ...(cap ? { requires: cap } : {}),
        })
        .respond({ body: "$result" }),
    )
    .case("delete", (s) =>
      s
        .write({
          label: actualLabel,
          properties: { cid: "$input.cid", tombstone: true },
          ...(cap ? { requires: cap } : {}),
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

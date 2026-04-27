// TypeScript types for the @benten/engine public surface.
//
// These mirror the shapes produced by the napi binding
// (`@benten/engine-native`) — a thin TS wrapper adds ergonomics (DSL
// builder, typed errors, toMermaid / trace helpers) but does NOT widen
// or narrow the runtime contract.

/**
 * The twelve operation primitives. Eight execute in Phase 1; four
 * (`wait`, `stream`, `subscribe`, `sandbox`) are type-defined only and
 * throw `E_PRIMITIVE_NOT_IMPLEMENTED` at call time.
 */
export type Primitive =
  | "read"
  | "write"
  | "transform"
  | "branch"
  | "iterate"
  | "wait"
  | "call"
  | "respond"
  | "emit"
  | "sandbox"
  | "subscribe"
  | "stream";

/**
 * A value at the TS <-> Rust boundary. Mirrors `benten_core::Value`:
 * null / bool / int / text / bytes / list / map. Floats are rejected
 * unless carried through `Value::Float` (not exposed in the raw JSON
 * shape of the DSL).
 */
export type Value =
  | null
  | boolean
  | number
  | string
  | Uint8Array
  | Value[]
  | { [key: string]: Value };

/** A JSON-serializable input/output shape for handler calls. */
export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue };

/** A subgraph Node — one operation primitive in the evaluated DAG. */
export interface SubgraphNode {
  /** Stable local id assigned by the builder; not the CID. */
  id: string;
  /** Which primitive this Node represents. */
  primitive: Primitive;
  /** Primitive-specific arguments (the payload the evaluator consumes). */
  args: Record<string, JsonValue>;
  /** Outgoing edges keyed by edge label (e.g. `NEXT`, `ON_NOT_FOUND`). */
  edges: Record<string, string>;
}

/** A subgraph ready to register with the engine. */
export interface Subgraph {
  /** Human-readable handler id (e.g. `"post-handler"`). */
  handlerId: string;
  /** Actions this subgraph exposes (e.g. `"post:create"`, `"post:list"`). */
  actions: string[];
  /** The Nodes composing the subgraph, in insertion order. */
  nodes: SubgraphNode[];
  /** Root Node id — evaluator entry point for action dispatch. */
  root: string;
  /** Optional input-schema hint (pure-doc; not enforced). */
  inputShape?: Record<string, JsonValue>;
}

/**
 * The value returned by `engine.registerSubgraph()`. Carries the
 * content-addressed handler id assigned by the Rust side plus DX helpers
 * (`toMermaid()`) that render locally from the subgraph structure
 * without a round-trip.
 */
export interface RegisteredHandler {
  /** Rust-assigned content-addressed handler id. */
  id: string;
  /** Action strings the handler responds to (e.g. `["post:create", ...]`). */
  actions: string[];
  /** Render the subgraph as a Mermaid flowchart string. Pure, deterministic. */
  toMermaid(): string;
  /** The underlying structural subgraph (useful for introspection). */
  subgraph: Subgraph;
}

/**
 * Inv-14 attribution frame — the (actor, handler, capability_grant) triple
 * authorizing each emitted trace step. Mirrors `benten_eval::AttributionFrame`.
 */
export interface AttributionFrame {
  /** CID of the actor (principal) that authored the step. */
  actorCid: string;
  /** CID of the handler subgraph that is executing. */
  handlerCid: string;
  /** CID of the capability grant authorising the step. */
  capabilityGrantCid: string;
}

/**
 * One per-primitive trace row — the dominant variant. Mirrors
 * `benten_engine::TraceStep::Step` after the Phase 2a G11-A Wave 2b
 * unification with `benten_eval::TraceStep`.
 */
export interface TraceStepPrimitive {
  type: "primitive";
  /** Content-addressed CID of the evaluated subgraph OperationNode. */
  nodeCid: string;
  /** Which primitive fired at this step. */
  primitive: Primitive | string;
  /** Duration of the step in microseconds (>0). */
  durationUs: number;
  /** Operation-node id within the registered handler. */
  nodeId: string;
  /** Input binding observed at step entry. Always emitted by the napi
   *  bridge (Value::Null serialises to JSON null, never absent). */
  inputs: JsonValue;
  /** Output produced by the step. Same always-present semantics as `inputs`. */
  outputs: JsonValue;
  /** Optional error-code string if the step routed to a typed error edge. */
  error?: string;
  /** Inv-14 attribution. Required slot; populated once G5-B-ii completes. */
  attribution?: AttributionFrame;
}

/** WAIT primitive drove the evaluator to suspension. */
export interface TraceStepSuspendBoundary {
  type: "suspend_boundary";
  /** CID of the persisted ExecutionStateEnvelope. */
  stateCid: string;
}

/** Resume re-entered a suspended execution. */
export interface TraceStepResumeBoundary {
  type: "resume_boundary";
  /** CID of the ExecutionStateEnvelope that was resumed. */
  stateCid: string;
  /** Signal payload handed to the resumed frame. */
  signalValue: JsonValue;
}

/** Inv-8 / Phase-2b SANDBOX-fuel budget exhausted. */
export interface TraceStepBudgetExhausted {
  type: "budget_exhausted";
  /** "inv_8_iteration" | "sandbox_fuel". */
  budgetType: string;
  /** How much budget was consumed before firing. */
  consumed: number;
  /** Configured limit. */
  limit: number;
  /** Path of operation-node ids that produced the exhaustion. */
  path: string[];
}

/**
 * One step of an evaluator trace. Phase 2a G11-A Wave 2b: discriminated
 * union mirroring the engine-side `TraceStep` enum. Switch on `.type` to
 * read variant-specific fields exhaustively.
 */
export type TraceStep =
  | TraceStepPrimitive
  | TraceStepSuspendBoundary
  | TraceStepResumeBoundary
  | TraceStepBudgetExhausted;

/** Full trace returned by `engine.trace()`. */
export interface Trace {
  steps: TraceStep[];
  result: JsonValue;
}

/**
 * Predecessor table derived from the subgraph DAG — used by the trace
 * test to validate that the trace respects topological order.
 */
export interface HandlerAdjacencies {
  predecessorsOf(nodeCid: string): Iterable<string>;
}

/**
 * Shape of an Edge as returned by `Engine.getEdge` / `edgesFrom` /
 * `edgesTo`. CIDs are base32-multibase strings (prefix `b`).
 */
export interface Edge {
  cid: string;
  source: string;
  target: string;
  label: string;
}

/**
 * Input shape for `Engine.grantCapability`. Phase-1 uses a flat
 * `{ actor, scope }` pair; Phase-3 adds optional `{ issuer, hlc, ... }`
 * fields for UCAN-grounded grants. Extra fields are tolerated on the
 * wire — the Rust parser consults only `actor` + `scope`.
 */
export interface CapabilityGrant {
  /** Principal (actor CID or string id in Phase 1) the grant applies to. */
  actor: string;
  /** Scope expression (e.g. `"store:post:write"`). */
  scope: string;
  /** Optional issuer CID (Phase-3 UCAN grounding — ignored in Phase 1). */
  issuer?: string;
  /** Optional HLC stamp (Phase-3 — ignored in Phase 1). */
  hlc?: number;
}

/**
 * Terminal outcome of a handler invocation. Mirrors the napi-side
 * `outcome_to_json` shape from `bindings/napi/src/subgraph.rs`.
 *
 * `ok: true` indicates the call routed via an `OK` edge (or its
 * synonyms). `ok: false` indicates the handler routed via an error
 * edge — `errorCode` / `errorMessage` carry the Rust-side typed error.
 *
 * The `cid` / `createdCid` aliases both refer to the CID of the
 * primary Node a CRUD `create` produced; `list` carries the materialized
 * list for `read_view` / `list` actions.
 */
export interface Outcome {
  ok: boolean;
  edge?: string;
  errorCode?: string;
  errorMessage?: string;
  cid?: string;
  createdCid?: string;
  list?: JsonValue[];
  completedIterations?: number;
  successfulWriteCount: number;
}

/**
 * Discriminated-union return shape from `Engine.callWithSuspension`.
 *
 * - `kind: "complete"` — the handler ran to completion without hitting
 *   a WAIT primitive; `outcome` is the terminal Outcome.
 * - `kind: "suspended"` — the handler hit a WAIT and persisted an
 *   `ExecutionStateEnvelope`; `handle` is the DAG-CBOR bytes you pass
 *   to `Engine.resumeFromBytes` / `Engine.resumeFromBytesAs` once the
 *   awaited signal is ready.
 *
 * Phase 2a G3-B napi F5 wiring: the napi layer transports the handle
 * as a base64 string under the hood; the TS wrapper decodes to `Buffer`
 * before exposing it to user code.
 */
export type SuspensionResult =
  | { kind: "complete"; outcome: Outcome }
  | { kind: "suspended"; handle: Buffer };

/**
 * Input shape for `Engine.createView` (legacy id-string form). Phase-1
 * recognizes the well-known id family `content_listing_<label>`; extra
 * fields are reserved for Phase-2 user-defined views.
 *
 * **Phase 2b G8-B note.** New code registering user-defined IVM views
 * should use the [`UserViewSpec`] shape with [`Engine.createView`]'s
 * builder overload — `engine.createView({ id, inputPattern, strategy?,
 * project? })`. The legacy `ViewDef` shape stays for the canonical
 * `content_listing_<label>` family the engine builds in.
 */
export interface ViewDef {
  /** View id string (e.g. `"content_listing_post"`). */
  viewId: string;
  /** Reserved for Phase-2 user-defined views. Ignored in Phase 1. */
  [key: string]: JsonValue;
}

// ---------------------------------------------------------------------------
// SANDBOX surface types (Phase 2b G7-C)
// ---------------------------------------------------------------------------

/**
 * Reserved-for-Phase-3 manifest signature shape. Phase 2b leaves this
 * structurally typed but always-undefined — D9 requires the canonical
 * DAG-CBOR encoding to OMIT the `signature` key entirely when undefined,
 * not to emit `null`. The `?` here is the load-bearing parity check.
 *
 * Pin source: `packages/engine/test/manifest_schema_parity.test.ts`.
 */
export interface ManifestSignature {
  /** Phase-3 Ed25519 signature bytes (base64). Reserved. */
  ed25519?: string;
}

/**
 * One module entry inside a [`ModuleManifest`].
 */
export interface ModuleManifestEntry {
  /** Module name — referenced from the DSL via `<manifestName>:<moduleName>`. */
  name: string;
  /** CIDv1 base32 string of the WebAssembly module bytes. */
  cid: string;
  /** Capabilities the module's host-fn imports require. */
  requires: string[];
}

/**
 * The shape `engine.installModule(manifest, manifestCid)` accepts.
 *
 * Phase 2b G10-B owns the install/uninstall surface; Phase 3 adds
 * Ed25519 signing on top of the same shape (the `signature?` field is
 * the forward-compat reservation per D9 + D16). The TS shape MUST stay
 * in lock-step with the Rust `ModuleManifest` struct — the parity check
 * lives in `packages/engine/test/manifest_schema_parity.test.ts`.
 */
export interface ModuleManifest {
  /** Manifest name (e.g. `"acme.posts"`). */
  name: string;
  /** Manifest version string (semver-shaped; not parsed in Phase 2b). */
  version: string;
  /** Modules this manifest declares. */
  modules: ModuleManifestEntry[];
  /**
   * Phase-3 reserved. Omit (i.e. `undefined`, NOT `null`) in Phase 2b —
   * the canonical-bytes serializer omits the key entirely when
   * undefined per D9 forward-compat.
   */
  signature?: ManifestSignature;
}

/**
 * SANDBOX argument shape — by-name variant.
 *
 * The `module` field is `<manifestName>:<moduleName>` (resolved against
 * the named-manifest registry G7-A owns). The `caps` escape hatch is
 * REJECTED at the type level on this variant — `SandboxArgsByName` and
 * `SandboxArgsByCaps` are mutually exclusive (per dx-r1-2b SANDBOX) so
 * a developer cannot half-and-half mix manifest lookup with explicit
 * caps in the same call.
 *
 * Per-call tuning knobs default to (omit them and the engine fills in):
 *   - `fuel`             = `1_000_000` (D24 + dx-r1-2b-5)
 *   - `wallclockMs`      = `30_000` (D24)
 *   - `outputLimitBytes` = `1_048_576` (D15 trap-loudly default)
 *
 * Pin source: `packages/engine/test/sandbox.test.ts`.
 */
export interface SandboxArgsByName {
  /** `<manifestName>:<moduleName>` — resolved at registration time. */
  module: string;
  /** Input expression (e.g. `"$input"`). */
  input?: string;
  /** Per-call fuel budget (default `1_000_000`). */
  fuel?: number;
  /** Per-call wallclock budget in milliseconds (default `30_000`). */
  wallclockMs?: number;
  /** Per-call output bound in bytes (default `1_048_576`). */
  outputLimitBytes?: number;
  /**
   * MUST NOT co-occur with `module`-by-name. The discriminated-union
   * type system rejects setting `caps` on this variant; flagged by the
   * `@ts-expect-error` pin in `sandbox.test.ts`.
   */
  caps?: never;
}

/**
 * SANDBOX argument shape — by-caps escape-hatch variant.
 *
 * The `module` field carries a raw module CID (NOT a `<manifest>:<module>`
 * lookup name). The `caps` field is REQUIRED and lists exactly the
 * `host:<domain>:<action>` capability strings the call asks the host to
 * satisfy. The escape hatch is intentional: it lets a power-user
 * compose SANDBOX calls without round-tripping through the
 * named-manifest registry, at the cost of dropping the registry's
 * named-bundle DX.
 *
 * Pin source: `packages/engine/test/sandbox.test.ts`.
 */
export interface SandboxArgsByCaps {
  /** Raw module CID (CIDv1 base32 string). */
  module: string;
  /** Required: explicit capability list (`host:<domain>:<action>` strings). */
  caps: string[];
  /** Optional: input expression. */
  input?: string;
  /** Per-call fuel budget (default `1_000_000`). */
  fuel?: number;
  /** Per-call wallclock budget in milliseconds (default `30_000`). */
  wallclockMs?: number;
  /** Per-call output bound in bytes (default `1_048_576`). */
  outputLimitBytes?: number;
}

/**
 * Discriminated union of the two SANDBOX argument shapes. The DSL
 * builder `subgraph(...).sandbox(args)` accepts either variant.
 *
 * Pin source: `packages/engine/test/sandbox.test.ts` —
 * "SandboxArgs by name vs by caps mutually exclusive (TS union)".
 */
export type SandboxArgs = SandboxArgsByName | SandboxArgsByCaps;

/**
 * Alias of [`SandboxArgs`] retained for the Phase 2a-era
 * `subgraph(...).sandbox(...)` callers — the discriminated-union shape
 * is the contract going forward but the alias keeps existing
 * Phase-2a-era TS code compiling.
 */
export type SandboxOptions = SandboxArgs;

/**
 * Terminal value SANDBOX returns to the evaluator's per-call frame.
 *
 * `output` carries the guest's emitted output (raw bytes for binary
 * payloads or a JSON value when the guest returns structured output).
 * `fuelConsumed` + `durationMs` are populated by the engine post-call
 * for diagnostic surfacing (`engine.describeSandboxNode(...)` returns
 * the running maxima across all invocations).
 *
 * Pin source: Phase-2b plan §3 G7-C row + `docs/SANDBOX-LIMITS.md` §2.
 */
export interface SandboxResult {
  /** Guest output payload (binary or JSON). */
  output: Uint8Array | JsonValue;
  /** Wasmtime fuel consumed by the call. */
  fuelConsumed: number;
  /** Wallclock duration of the call in milliseconds. */
  durationMs: number;
}

/**
 * Read-only diagnostic snapshot of a registered SANDBOX node returned
 * by `engine.describeSandboxNode(handlerId, nodeId)`.
 *
 * Mirrors the Rust `SandboxNodeDescription`
 * (`crates/benten-engine/src/engine_sandbox.rs`). Keep them in
 * lock-step.
 *
 * Defaults documented in `docs/SANDBOX-LIMITS.md` §2: omitting the
 * per-node DSL knobs uses `fuel = 1_000_000`, `wallclockMs = 30_000`,
 * `outputLimitBytes = 1_048_576` (D24 + dx-r1-2b-5).
 *
 * Pin source: ts-r4-3 R4 finding;
 * `packages/engine/test/sandbox.test.ts::"SandboxArgs defaults — omitting fuel / wallclockMs / outputLimitBytes uses 1M / 30s / 1MB"`.
 */
export interface SandboxNodeDescription {
  /** CID of the WebAssembly module the SANDBOX node references. */
  moduleCid: string;
  /**
   * Resolved manifest identifier (named-manifest registry lookup) when
   * the DSL form is by-name; `null` when the node uses the `caps`
   * escape hatch.
   */
  manifestId: string | null;
  /** Resolved per-call fuel budget. */
  fuel: number;
  /** Resolved per-call wallclock budget in milliseconds. */
  wallclockMs: number;
  /** Resolved per-call output bound in bytes. */
  outputLimitBytes: number;
  /**
   * Cumulative high-water mark of fuel consumed by this node across
   * every invocation since registration. `null` until the node is
   * invoked at least once.
   */
  fuelConsumedHighWater: number | null;
  /**
   * Wallclock duration of the most recent invocation in milliseconds.
   * `null` until the first call returns.
   */
  lastInvocationMs: number | null;
}

/**
 * IVM strategy enum (Phase 2b G8 D8-RESOLVED).
 *
 * - `'A'` — Phase-1 hand-written IVM views (Rust-only). User-registered
 *   views CANNOT claim this lane; passing `'A'` to `engine.createView`
 *   throws `E_VIEW_STRATEGY_A_REFUSED`.
 * - `'B'` — generalized Algorithm B (default for user views).
 * - `'C'` — Z-set / DBSP cancellation (reserved for Phase 3+; passing
 *   `'C'` throws `E_VIEW_STRATEGY_C_RESERVED`).
 */
export type Strategy = "A" | "B" | "C";

/**
 * Input-pattern selector for [`UserViewSpec`].
 *
 * Phase-2b ships two narrow selectors. The shape mirrors the Rust
 * `UserViewInputPattern` enum and round-trips across the napi boundary.
 *
 * ⚠️ PRE-G8-A SEMANTIC STUB: in the pre-G8-A engine, `anchorPrefix` is
 * silently coerced to a `label`-equality match against the prefix string
 * (because the underlying `ContentListingView` only knows label equality).
 * An app that declares `inputPattern: { anchorPrefix: "post" }` and then
 * reads the user view will see results filtered by `label === "post"`,
 * NOT by anchor prefix. This is a stub bridge until G8-A's per-strategy
 * view dispatch lands (then `anchorPrefix` will swap to the proper
 * anchor-prefix selector). DO NOT rely on `anchorPrefix` semantics in
 * tests or app code that targets the pre-G8-A engine.
 */
export type UserViewInputPattern =
  | { label: string }
  | { anchorPrefix: string };

/**
 * User-registered view spec (Phase 2b G8-B).
 *
 * `id` and `inputPattern` are required; `strategy` defaults to `'B'`
 * (D8-RESOLVED). `project` is reserved for the post-G8-A landing — once
 * the generalized Algorithm B port is in place the engine will invoke
 * the projection per change event to materialize rows; until then the
 * field is accepted but not invoked.
 */
export interface UserViewSpec {
  /** Stable view id (e.g. `"user_posts_by_author"`). */
  id: string;
  /** Selector that picks the change events the view observes. */
  inputPattern: UserViewInputPattern;
  /**
   * Strategy opt-in. Defaults to `'B'` per D8-RESOLVED. Pass `'A'` only
   * to verify the typed-error refusal path (the engine refuses A for
   * user views since A is reserved for the 5 Phase-1 hand-written
   * views). `'C'` is rejected as Phase-3-reserved.
   */
  strategy?: Strategy;
  /**
   * Optional projection invoked per change event to materialize a row.
   * Reserved for the G8-A generalized Algorithm B landing — the field
   * is accepted by the builder + round-tripped to the Rust side, but
   * not yet invoked by the runtime.
   */
  project?: (event: unknown) => unknown;
}

/**
 * Handle returned by `engine.createView(spec)` when `spec` is a
 * [`UserViewSpec`]. Exposes the resolved id + strategy and the per-view
 * `snapshot()` / `onUpdate()` surfaces (the snapshot iterator yields
 * the current materialized rows; `onUpdate` registers a per-diff
 * callback). Phase-2b G8-B ships the registration round-trip — runtime
 * materialization (snapshot rows + diff streaming) lights up once
 * G8-A's Algorithm B port lands.
 */
export interface UserView {
  /** Resolved view id. */
  readonly id: string;
  /** Resolved strategy (always `'B'` for accepted user views). */
  readonly strategy: Strategy;
  /** Resolved input pattern. */
  readonly inputPattern: UserViewInputPattern;
  /**
   * Async iterator over the currently-materialized rows. Phase-2b G8-B
   * returns an empty iterator until G8-A's Algorithm B port materializes
   * the row buffer — the surface exists so app code can be written
   * against the final shape today.
   */
  snapshot: () => AsyncIterable<unknown>;
  /**
   * Subscribe to per-diff change notifications. Returns a `Subscription`
   * with an `unsubscribe()` method. As with `snapshot()`, the runtime
   * dispatch lights up alongside G8-A; the subscription handle exists
   * pre-G8-A so app code is forward-compatible.
   */
  onUpdate: (cb: (diff: unknown) => void) => UserViewSubscription;
}

/** Handle returned by `UserView.onUpdate` for cleanup. */
export interface UserViewSubscription {
  unsubscribe: () => Promise<void>;
// ---------------------------------------------------------------------------
// STREAM (Phase 2b G6-B)
// ---------------------------------------------------------------------------

/**
 * One chunk of streamed output. Mirrors the napi-side
 * `benten_engine::Chunk` newtype around `Vec<u8>` — the wire form on
 * the JS side is a Node `Buffer` so consumers can decode straight to
 * UTF-8 / structured bytes without an intermediate copy.
 */
export type Chunk = Buffer;

/**
 * Cursor mode for STREAM consumers (G6-A D5 cursor surface symmetry).
 *
 * - `latest` — start from the next chunk produced after the call.
 * - `sequence` — start from the chunk at engine-assigned sequence
 *   number `seq` (replay within the bounded retention window).
 */
export type StreamCursor =
  | { kind: "latest" }
  | { kind: "sequence"; seq: number };

/**
 * Handle to an open STREAM dispatch returned by
 * {@link Engine.callStream} / {@link Engine.openStream} /
 * {@link Engine.testingOpenStreamForTest}.
 *
 * The handle implements `AsyncIterable<Chunk>` so consumers can
 * iterate naturally:
 *
 * ```ts
 * for await (const chunk of engine.callStream(handlerId, "act", input)) {
 *   process.stdout.write(chunk);
 * }
 * ```
 *
 * `openStream` returns a handle whose lifecycle the caller manages
 * explicitly via `close()`; `callStream` returns a handle that
 * auto-closes when the `for await` loop exits. Both share the same
 * underlying class.
 */
export interface StreamHandle extends AsyncIterable<Chunk> {
  /**
   * Pull the next chunk synchronously. Returns `null` at end-of-stream.
   * Throws if the underlying executor surfaces a typed error
   * (back-pressure drop, peer close, capability denial mid-stream).
   *
   * Most consumers should prefer the `for await ... of` form which
   * routes through `[Symbol.asyncIterator]()`.
   */
  next(): Chunk | null;

  /**
   * Explicitly close the handle. Idempotent. Once closed, all
   * subsequent `next()` calls return `null`.
   */
  close(): void;

  /**
   * `true` once the handle is drained (closed AND no buffered chunks
   * remain). Useful for harness assertions.
   */
  isDrained(): boolean;

  /**
   * Engine-assigned sequence count of chunks delivered so far. Bumped
   * per `next()` returning a chunk; `0` before the first chunk drains.
   */
  seqSoFar(): number;
}

// ---------------------------------------------------------------------------
// SUBSCRIBE (Phase 2b G6-B)
// ---------------------------------------------------------------------------

/**
 * Cursor mode for SUBSCRIBE consumers (G6-A D5-RESOLVED).
 *
 * - `latest` — start from the next event published after the
 *   `onChange` call returns.
 * - `sequence` — start from engine-assigned sequence number `seq`
 *   (replay within the bounded retention window: 1000 events OR 24h,
 *   whichever first).
 * - `persistent` — engine-managed cursor stored in the G12-E
 *   SuspensionStore so a re-subscribe across process restart resumes
 *   from `maxDeliveredSeq + 1`. The `subscriberId` is the persistent
 *   key.
 */
export type SubscribeCursor =
  | { kind: "latest" }
  | { kind: "sequence"; seq: number }
  | { kind: "persistent"; subscriberId: string };

/**
 * Subscription handle returned by {@link Engine.onChange}.
 *
 * Carries the live `active` flag, the matched `pattern`, the
 * registered `cursor`, and the dedup state machine's current
 * `maxDeliveredSeq` (D5-RESOLVED — exactly-once at the handler API).
 *
 * Drop the handle (or call {@link unsubscribe}) to release the
 * subscription. The Rust-side `Drop` impl flips the active flag and
 * de-registers the callback from the engine's change-stream port.
 */
export interface Subscription {
  /** `true` while the subscription is registered with the engine. */
  readonly active: boolean;
  /** Pattern the subscription was registered with (event-name glob). */
  readonly pattern: string;
  /** Cursor mode at registration time. */
  readonly cursor: SubscribeCursor;
  /**
   * Highest engine-assigned sequence number observed by this
   * subscription's delivery path. `0` before the first event lands.
   */
  readonly maxDeliveredSeq: number;
  /** Explicitly release the subscription. Idempotent. */
  unsubscribe(): void;
}

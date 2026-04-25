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
  /** Input binding observed at step entry. */
  inputs?: JsonValue;
  /** Output produced by the step. */
  outputs?: JsonValue;
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
 * Input shape for `Engine.createView`. Phase-1 recognizes the well-known
 * id family `content_listing_<label>`; extra fields are reserved for
 * Phase-2 user-defined views.
 */
export interface ViewDef {
  /** View id string (e.g. `"content_listing_post"`). */
  viewId: string;
  /** Reserved for Phase-2 user-defined views. Ignored in Phase 1. */
  [key: string]: JsonValue;
}

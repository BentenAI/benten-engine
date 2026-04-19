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

/** One step of an evaluator trace — per-node timing + I/O snapshot. */
export interface TraceStep {
  /** Content-addressed CID of the evaluated subgraph Node. */
  nodeCid: string;
  /** Which primitive fired at this step. */
  primitive: Primitive | string;
  /** Duration of the step in microseconds (>0). */
  durationUs: number;
  /** Optional input binding observed at step entry. */
  inputs?: JsonValue;
  /** Optional output produced by the step. */
  outputs?: JsonValue;
  /** Optional error-code string if the step routed to a typed error edge. */
  error?: string;
}

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

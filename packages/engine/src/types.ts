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
  /** Human-readable handler id (e.g. `"export-feed"` for hand-built
   * subgraphs; the engine assigns `"crud:<label>"` for crud()-registered
   * handlers — see `Engine.registerSubgraph` for the canonical id). */
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
 * authorizing each emitted trace step plus the Inv-4 SANDBOX nest-depth
 * counter. Mirrors `benten_eval::AttributionFrame`.
 *
 * # Phase-3 widening (R4-R1 pcds-r4-r1-1 — 25th p/c drift PRE-EMPTION)
 *
 * The Rust producer is widened at G14-D wave-5a (`device_did` per
 * sync_replica_attribution.rs) + G16-B wave-6b (`peer_did_set` per
 * loro_version_chain.rs + atrium_three_peer.rs). The corresponding TS
 * consumer fields are declared here as OPTIONAL (`?`) so pre-Phase-3
 * trace consumers continue to compile; post-G14-D / post-G16-B,
 * runtime-emitted attribution payloads carry these fields when the
 * underlying step participated in a Loro merge (peer_did_set) or
 * cross-device sync replica (device_did).
 *
 * Pin source (instance 25 candidate pre-emption): R4-R1 producer-consumer
 * deep sweep `pcds-r4-r1-1`. Same shape as Phase-2b Instance 18
 * (`sandboxDepth` widening) caught post-merge by R6-R3 r6-r3-pcds-1.
 * Pre-empting at R4-R1 corpus revision avoids a 25th instance landing
 * during R5 G14-D / G16-B implementation.
 */
export interface AttributionFrame {
  /** CID of the actor (principal) that authored the step. */
  actorCid: string;
  /** CID of the handler subgraph that is executing. */
  handlerCid: string;
  /** CID of the capability grant authorising the step. */
  capabilityGrantCid: string;
  /**
   * Inv-4 SANDBOX nest-depth counter (D20-RESOLVED). `0` when the step
   * is NOT inside a SANDBOX boundary; incremented at every SANDBOX
   * entry (INHERITED across CALL boundaries — CALL itself does NOT
   * increment). The Rust producer at `benten_eval::AttributionFrame`
   * canonicalizes the value into the content-addressed CID only when
   * non-zero so a SANDBOX-bearing attribution chain is provably
   * content-distinguishable from a non-SANDBOX chain (Inv-4 security
   * claim). R6-R3 r6-r3-pcds-1 surfaced that pre-fix the napi trace
   * projection dropped this field; this surface widening makes the
   * Inv-4 observability available to JS consumers (trace-rendering UIs,
   * Phase-6 AI workflow forking).
   */
  sandboxDepth: number;
  /**
   * Phase-3 G16-B widening (pcds-r4-r1-1 instance-25 PRE-EMPTION):
   * the set of peer DIDs that contributed to the Loro-merged version
   * the attribution chain references. Populated by G16-B wave-6b's
   * loro_version_chain producer when the step's underlying anchor
   * traversed a multi-peer merge; absent (`undefined`) for steps that
   * did not. Mirrors the Rust producer's
   * `AttributionFrame::peer_did_set` field at
   * `crates/benten-engine/src/loro_version_chain.rs`.
   */
  peerDidSet?: string[];
  /**
   * Phase-3 G14-D widening (pcds-r4-r1-1 instance-25 PRE-EMPTION):
   * the device DID that authored the step. Populated by G14-D wave-5a's
   * sync_replica_attribution producer for cross-device sync-replica
   * writes per CLAUDE.md baked-in #17 device-heterogeneity contract;
   * absent (`undefined`) when the step originated from the local-only
   * non-sync-replica path. Mirrors the Rust producer's
   * `AttributionFrame::device_did` field at
   * `crates/benten-engine/src/sync_replica_attribution.rs`.
   */
  deviceDid?: string;
  /**
   * Phase-3 G16-B widening (§13.9 Instance 25 closure 2026-05-10):
   * Loro CRDT merge-hop depth — increments by 1 at each `apply_atrium_merge`
   * boundary the underlying anchor traversed. `0` (omitted) for purely-
   * local writes; `>=1` for sync-merged writes. Bounded by
   * `benten_eval::SYNC_HOP_DEPTH_CAP` (default 8); overflow surfaces
   * as `E_SYNC_HOP_DEPTH_EXCEEDED` at the producer. Mirrors the Rust
   * producer's `AttributionFrame::sync_hop_depth: u32` field at
   * `crates/benten-eval/src/exec_state.rs::AttributionFrame`.
   *
   * The pre-fix TS interface declared a phantom `deviceCid?: string`
   * slot inherited from an earlier design that never landed on the
   * producer side (Rust `AttributionFrame` carries `sync_hop_depth: u32`
   * here, not a `device_cid` slot). The phantom is dropped + the real
   * producer field is mirrored in this fix-pass. The §13.9 brief +
   * sibling test docstrings reference the legacy `deviceCid` name; the
   * actual closure mirrors what the Rust producer carries.
   */
  syncHopDepth?: number;
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
 * Forward-compat catch-all variant — emitted by the wrapper-side
 * `mapTraceStep` projection when a row from the native binding carries
 * a `type` discriminant this version of `@benten/engine` does not yet
 * recognize (per Phase-2b D14-RESOLVED, "warning-passthrough").
 *
 * Routing semantics:
 *   - The original row is preserved verbatim under `raw` so callers
 *     willing to opt into pre-release variants can pattern-match on it.
 *   - `console.warn` is emitted ONCE per discriminant per process so
 *     a wrapper-version-skew shows up early in dev/CI but doesn't spam.
 *   - The trace.steps array PRESERVES unknown rows in topological
 *     position — they are NOT silently dropped.
 */
export interface TraceStepUnknown {
  type: "unknown";
  /** The unrecognized `type` discriminator string from the native row. */
  discriminant: string;
  /** The original row, preserved verbatim. */
  raw: Record<string, unknown>;
}

/**
 * One step of an evaluator trace. Phase 2a G11-A Wave 2b: discriminated
 * union mirroring the engine-side `TraceStep` enum. Switch on `.type` to
 * read variant-specific fields exhaustively. Phase-2b G12-F adds the
 * `TraceStepUnknown` forward-compat variant per D14-RESOLVED.
 */
export type TraceStep =
  | TraceStepPrimitive
  | TraceStepSuspendBoundary
  | TraceStepResumeBoundary
  | TraceStepBudgetExhausted
  | TraceStepUnknown;

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
 *
 * # Phase-3 G19-D §7.9 fix (TS-surface-parity sweep)
 *
 * Pre-G19-D this interface declared `cid: string` (a phantom field —
 * the napi producer at `bindings/napi/src/edge.rs::edge_to_json` never
 * emitted a `cid` key) AND omitted `properties` (a missing field — the
 * napi producer DOES emit `properties` when the underlying Edge carries
 * a non-empty property bag). G19-D drops the phantom + adds the missing
 * field per the producer/consumer audit + the `tests/edge_interface_*`
 * RED-PHASE pins. The Rust producer at `edge.rs::edge_to_json` is
 * UNCHANGED; the fix is purely TS-surface-parity. See the §7.10
 * `ts_surface_parity_meta_test` for the structural defense against
 * recurrence.
 *
 * # By-design omission
 *
 * The Rust `benten_core::Edge` struct carries an `anchor_id` field
 * marked `#[serde(skip)]`; the napi projection consequently never emits
 * it and the TS interface intentionally omits it (mirrors the
 * `Node.anchor_id` precedent in `crates/benten-core/src/lib.rs::Node`).
 */
export interface Edge {
  /** CID of the Edge's source Node (base32-multibase, prefix `b`). */
  source: string;
  /** CID of the Edge's target Node (base32-multibase, prefix `b`). */
  target: string;
  /** Edge label (e.g. `"NEXT"` / `"CURRENT"` / `"GRANTED_TO"`). */
  label: string;
  /**
   * Optional property bag carried on the Edge. Omitted (`undefined`)
   * when the underlying Edge has no properties; populated when the
   * napi producer emits a non-empty `properties` map. Mirrors the
   * Rust producer's `Edge::properties: Option<BTreeMap<String, Value>>`
   * field at `crates/benten-core/src/edge.rs::Edge`.
   */
  properties?: Record<string, JsonValue>;
}

/**
 * One capability claim inside a [`DeviceAttestation`]. Mirrors the Rust
 * producer's per-claim shape at
 * `bindings/napi/tests/device_attestation.rs` test rationale.
 *
 * Phase-3 device-mesh exploration (CLAUDE.md baked-in #17): a browser
 * tab declares which capabilities its device-DID may exercise against
 * the full peer's authoritative store; the full peer attenuates the
 * UCAN chain accordingly.
 */
export interface CapabilityClaim {
  /** Path-glob of resources the claim applies to (e.g. `"/zone/notifications/*"`). */
  path: string;
  /** Ability the claim grants (e.g. `"read"` / `"write"` / `"emit"`). */
  ability: string;
}

/**
 * Phase-3 device-attestation envelope declared via
 * `engine.atrium.declareDeviceAttestation(envelope)`. Mirrors the napi
 * binding's typed-struct contract (R3-C device_attestation.rs +
 * D-PHASE-3-25 + r1-napi-2).
 *
 * Pin source (instance 26 candidate pre-emption): R4-R1 producer-consumer
 * deep sweep `pcds-r4-r1-2`. Schema-level interface declared so TS
 * callers writing `engine.atrium.declareDeviceAttestation({ deviceDid,
 * capabilities, freshnessWindow })` get compile-time type-checking
 * rather than implicit `any`/`unknown`. Same shape as Phase-2b §7.9
 * Edge.cid phantom — the runtime contract was pinned at the test layer
 * but the TS schema-level declaration was missing.
 *
 * # Field semantics
 *
 * - `deviceDid` — the `did:key:...` identifier of the declaring device.
 *   The Rust producer canonicalizes via the keypair envelope; TS callers
 *   pass the resolved string.
 * - `capabilities` — the per-claim list narrowing what the device may
 *   exercise against the full peer's store. Replay-resistance via UCAN
 *   chain attenuation (G14-A2 + G14-B coverage).
 * - `freshnessWindow` — TTL in seconds before the attestation must be
 *   re-declared. Mirrors the Rust producer's freshness-window field
 *   used by the replay-resistance path at
 *   `crates/benten-engine/tests/ucan_replay_audience.rs`.
 *
 * Defends against Phase-2b Edge.cid phantom recurrence: the TS schema
 * MUST stay in lock-step with the napi-side typed `#[napi(object)]`
 * struct (G14-A2 + G16-D ship the typed struct, NOT a loose
 * `serde_json::Value` parameter). The §7.10 ts_surface_parity_meta_test
 * walks every `#[napi]` exported struct including this one; mode-5
 * (schema-parity-missing-field) drift would be caught at G19-D wave-7
 * meta-test landing.
 */
export interface DeviceAttestation {
  /** `did:key:...` identifier of the declaring device. */
  deviceDid: string;
  /** Per-claim capabilities this device may exercise. */
  capabilities: CapabilityClaim[];
  /** TTL in seconds before re-declaration is required. */
  freshnessWindow: number;
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
  /**
   * Issuer CID consumed by the durable UCAN backend's chain-walker
   * (G14-B + G21-T2). When present, the Rust-side `parse_grant_json`
   * threads this through to the durable `UCANBackend<B>` so the
   * grant's signing-issuer claim is verified against the chain root.
   *
   * Phase-3-pre-G21-T2 honest state: this field was silently dropped
   * by the napi parser (Phase-1 stub). Post-G21-T2 (audit-6-1
   * closure) it flows through to the durable backend.
   */
  issuer?: string;
  /**
   * HLC stamp consumed by the durable UCAN backend for replay-window
   * narrowing (G14-B). When present, the chain-walker uses this as
   * the `now` reference for nbf/exp checks instead of the engine's
   * wall-clock — useful for cross-peer correlation in Atrium sync.
   *
   * Phase-3-pre-G21-T2 honest state: this field was silently dropped
   * by the napi parser. Post-G21-T2 it flows through.
   */
  hlc?: number;
}

/**
 * Phase-3 G21-T2 — typed-CALL op-name closed registry.
 *
 * Mirrors the Rust closed-set `benten_eval::TypedCallOp` 10-variant
 * `#[non_exhaustive]` enum at
 * `crates/benten-eval/src/typed_call.rs::TypedCallOp`. Phase-3 G21-T1
 * ships exactly these 10 ops; the registry is closed (no
 * user-registered typed-CALL ops). Extending the registry is a
 * Rust-only engine concern.
 *
 * Per-op input/output schemas live below in the
 * [`TypedCallInput`] / [`TypedCallOutput`] discriminated unions.
 */
export type TypedCallOp =
  | "ed25519_sign"
  | "ed25519_verify"
  | "keypair_generate"
  | "keypair_from_seed"
  | "blake3_hash"
  | "multibase_encode"
  | "multibase_decode"
  | "did_resolve"
  | "ucan_validate_chain"
  | "vc_verify";

/**
 * Per-op input shapes for [`TypedCallOp`]. Mirrors the per-op
 * input/output rustdoc at
 * `crates/benten-eval/src/typed_call.rs::TypedCallOp`.
 *
 * Bytes fields cross the napi boundary as `Buffer` / `Uint8Array`
 * (napi-rs renders these as numeric-keyed objects on the JSON side
 * but the runtime detector at the napi layer reconstructs the bytes
 * unambiguously — see `bindings/napi/src/node.rs::detect_typed_array_bytes`).
 *
 * The discriminated union is keyed by op-name; callers pick the
 * matching shape per the op they invoke. Phase-3 G21-T2 callers
 * write `engine.typedCall("ed25519_sign", { privateKey, message })`
 * with the exact field names below.
 */
export interface TypedCallInputShapes {
  ed25519_sign: { private_key: Uint8Array | Buffer; message: Uint8Array | Buffer };
  ed25519_verify: {
    public_key: Uint8Array | Buffer;
    message: Uint8Array | Buffer;
    signature: Uint8Array | Buffer;
  };
  keypair_generate: Record<string, never> | { seed: null };
  keypair_from_seed: { seed: Uint8Array | Buffer };
  blake3_hash: { data: Uint8Array | Buffer };
  multibase_encode: { data: Uint8Array | Buffer; base: string };
  multibase_decode: { encoded: string };
  did_resolve: { did: string };
  ucan_validate_chain: {
    tokens: (Uint8Array | Buffer)[];
    audience: string;
    capability: string;
    now: number;
  };
  vc_verify: {
    credential: Uint8Array | Buffer;
    expected_issuer_did: string;
    now: number;
  };
}

/** Per-op output shapes for [`TypedCallOp`]. */
export interface TypedCallOutputShapes {
  ed25519_sign: { signature: Uint8Array };
  ed25519_verify: { valid: boolean };
  keypair_generate: { private_key: Uint8Array; public_key: Uint8Array };
  keypair_from_seed: { private_key: Uint8Array; public_key: Uint8Array };
  blake3_hash: { hash: Uint8Array };
  multibase_encode: { encoded: string };
  multibase_decode: { data: Uint8Array; base: string };
  did_resolve: { method: string; public_key: Uint8Array };
  ucan_validate_chain: { valid: boolean; reason: string };
  vc_verify: { valid: boolean; issuer: string; subject: string };
}

/** Look up the input shape for a given typed-CALL op. */
export type TypedCallInput<Op extends TypedCallOp> = TypedCallInputShapes[Op];
/** Look up the output shape for a given typed-CALL op. */
export type TypedCallOutput<Op extends TypedCallOp> = TypedCallOutputShapes[Op];

/**
 * Per-op required capability strings under the `cap:typed:*`
 * namespace. Mirrors `TypedCallOp::required_cap()`.
 *
 * Under `NoAuthBackend` all typed-CALL caps are permitted; under
 * the durable UCAN backend (G14-B + G21-T2) the chain-walker gates
 * each op by claim. See `crates/benten-caps/src/backends/ucan.rs`
 * for the consumer-side mapping (phase-3-backlog §2.5(c)).
 */
export const TYPED_CALL_REQUIRED_CAP: Record<TypedCallOp, string> = {
  ed25519_sign: "cap:typed:crypto-sign",
  ed25519_verify: "cap:typed:crypto-verify",
  keypair_generate: "cap:typed:crypto-keygen",
  keypair_from_seed: "cap:typed:crypto-keygen",
  blake3_hash: "cap:typed:hash",
  multibase_encode: "cap:typed:codec",
  multibase_decode: "cap:typed:codec",
  did_resolve: "cap:typed:did-resolve",
  ucan_validate_chain: "cap:typed:ucan-validate",
  vc_verify: "cap:typed:vc-verify",
};

/**
 * Reserved handler-id namespace prefix for typed-CALL dispatch.
 *
 * Mirrors `benten_eval::TYPED_CALL_PREFIX`. A CALL operation node
 * whose `target` starts with this prefix routes through the typed-CALL
 * registry instead of the user handler registry.
 */
export const TYPED_CALL_PREFIX = "engine:typed:";

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
 *   awaited signal is ready. `stateCid` is the engine-assigned base32
 *   CID of the persisted envelope (R6 Round-2 Instance 12 — added so
 *   JS callers can correlate the suspension across logs / external
 *   orchestration without parsing the opaque bytes); `signalName` is
 *   the WAIT primitive's signal name (e.g. `"external:payment"`),
 *   useful for routing the resume payload to the correct pending
 *   handler in multi-WAIT systems.
 *
 * Phase 2a G3-B napi F5 wiring: the napi layer transports the handle
 * as a base64 string under the hood; the TS wrapper decodes to `Buffer`
 * before exposing it to user code.
 */
export type SuspensionResult =
  | { kind: "complete"; outcome: Outcome }
  | {
      kind: "suspended";
      handle: Buffer;
      /** Engine-assigned base32 CID of the persisted envelope. */
      stateCid: string;
      /** Signal name the suspension is waiting for. */
      signalName: string;
    };

/**
 * Phase-3 G19-C1 (phase-3-backlog §7.1.4) — discriminated-union return
 * shape for [`Engine.resumeWithMeta`], the ergonomic wrapper over
 * [`Engine.resumeFromBytes`] that surfaces metadata about whether the
 * resumed handler ran to completion or suspended again on a downstream
 * WAIT.
 *
 * - `complete` — the handler completed; `outcome` carries the terminal
 *   `Outcome` shape (same shape `engine.call` returns).
 * - `suspended` — the resumed handler hit ANOTHER WAIT primitive and
 *   re-suspended; the resume cycle continues with a fresh
 *   `handle` / `stateCid` / `signalName` triple. Mirrors the
 *   [`SuspensionResult`] suspended-arm shape so callers can
 *   structurally re-enter the resume loop.
 */
export type ResumeWithMetaResult =
  | { kind: "complete"; outcome: Outcome }
  | {
      kind: "suspended";
      handle: Buffer;
      /** Engine-assigned base32 CID of the new persisted envelope. */
      stateCid: string;
      /** Signal name the new suspension is waiting for. */
      signalName: string;
    };

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
 * Manifest signature shape — PQ-hybrid-capable per §1.A.FROZEN item 10
 * (widened by G-CORE-2 #1300 per the 2026-05-19 PQ-default reframe).
 *
 * # Cross-language rule-mirror (§3.5g) with the Rust integration crate
 *
 * Mirrors `benten_crypto_suite::HybridSignature` + `SigCodepoint` from
 * the integration crate (`crates/benten-crypto-suite`). NO hardcoded
 * Ed25519-shaped (32 B-key / 64 B-sig) size assumption JS-side. The
 * ML-DSA-65 dimensions (~1952 B key / ~3309 B sig) are carried via
 * variable-length base64 strings — the JS surface does NOT pre-bound
 * the sig length to 88 base64 chars (Ed25519-sig × 4/3 + padding) the
 * way an Ed25519-only schema would.
 *
 * # Codepoint dispatch
 *
 * `codepoint` selects the dispatch arm:
 * - `0x0001` (`HYBRID_ED25519_MLDSA65`, v1-beta DEFAULT) — `ed25519` +
 *   `mlDsa65` + `commitment` MUST all be present (NF-4
 *   concatenated/committing/strip-resistant; both halves required to
 *   verify).
 * - `0x0002` (`CLASSICAL_ED25519`, non-default downgrade) — `ed25519`
 *   MUST be present; `mlDsa65` + `commitment` MUST be absent.
 *
 * **No silent fallback on unknown codepoints** — the napi/Rust dispatch
 * surfaces typed `E_CRYPTO_UNSUPPORTED_ALGORITHM` per the CLAUDE.md #5
 * typed-unsupported-arm clause.
 *
 * Pin source: `packages/engine/test/manifest_schema_parity.test.ts`.
 */
export interface ManifestSignature {
  /**
   * Signature codepoint — selects the dispatch arm. Mirrors
   * `benten_crypto_suite::SigCodepoint`:
   * - `0x0001` = hybrid Ed25519⊕ML-DSA-65 (v1-beta DEFAULT).
   * - `0x0002` = classical-only Ed25519 (non-default downgrade).
   *
   * Omitted in the legacy-shape signature (where only `ed25519` is
   * present); the canonical PQ-hybrid-aware encoder always emits the
   * codepoint.
   */
  codepoint?: number;
  /**
   * Ed25519 (classical) signature bytes — base64. **Variable-length
   * base64; no JS-side 88-char hardcode.** Always present for both
   * `codepoint=0x0001` and `codepoint=0x0002`.
   */
  ed25519?: string;
  /**
   * ML-DSA-65 (PQ) signature bytes — base64. Present iff
   * `codepoint=0x0001`. Variable-length (~3309 B raw → ~4412 base64
   * chars); the JS surface does NOT bound this length.
   */
  mlDsa65?: string;
  /**
   * NF-4 commitment binding both halves + message — base64 (32 B raw
   * = SHA3-256 output, ~44 base64 chars). Present iff `codepoint=0x0001`.
   * The committing construction is what makes the hybrid
   * strip-resistant: neither half can be stripped, truncated, or
   * cross-message-substituted without the verify failing closed.
   */
  commitment?: string;
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
 * Phase-3-reserved migration declaration. Mirrors the Rust
 * `crates/benten-engine/src/module_manifest.rs::MigrationStep` struct.
 *
 * On a `wasm32-unknown-unknown` browser-target install, declaring any
 * non-empty `migrations` array is rejected at install time with
 * `E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE` — there is no persistent
 * backing store for migrations to land in. Outside the browser target
 * the array is shipped through canonical-bytes encoding for CID parity
 * with the Rust producer.
 *
 * The TS shape MUST stay in lock-step with the Rust struct — the
 * parity check lives in `packages/engine/test/manifest_schema_parity.test.ts`
 * (R6-R4 r6-r4-pcds-1 added a migrations-bearing fixture pinning CID
 * parity against the symmetric Rust pin in
 * `crates/benten-engine/tests/manifest_schema_parity_pin.rs`).
 */
export interface MigrationStep {
  /** Stable migration id (e.g. `"add-author-index-2026-04"`). */
  id: string;
  /** Free-form description. Omit (`undefined`) when not provided. */
  description?: string;
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
   * Phase-3-reserved migration declarations (R6-R4 r6-r4-pcds-1
   * widening — 19th producer/consumer drift instance closure). When
   * non-empty on a `wasm32-unknown-unknown` target, install fires
   * `E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE`. Omit (`undefined` or
   * empty array) in Phase 2b production-like flows; the canonical-bytes
   * serializer drops the key entirely when the array is empty per D9
   * forward-compat.
   */
  migrations?: MigrationStep[];
  /**
   * Phase-3 G17-A2 — additive optional per-host-fn overrides
   * (CLAUDE.md baked-in #16 closure / Compromise #16). Currently the
   * only declared sub-field is `random.budget_bytes_per_call` which
   * lets a manifest tighten or widen the per-call entropy budget for
   * the `random` host-fn (codegen default = 4096 per r1-wsa-8).
   *
   * Omit (`undefined`) when no override is needed; the canonical-bytes
   * serializer omits the key entirely when undefined per D9
   * forward-compat — a Phase-2b manifest with no overrides keeps its
   * CID across this G17-A2 schema lift.
   */
  host_fns?: HostFnsOverride;
  /**
   * Phase-3 reserved. Omit (i.e. `undefined`, NOT `null`) in Phase 2b —
   * the canonical-bytes serializer omits the key entirely when
   * undefined per D9 forward-compat.
   */
  signature?: ManifestSignature;
}

/**
 * Phase-3 G17-A2 — per-host-fn overrides (CLAUDE.md baked-in #16
 * closure / Compromise #16). Additive optional carriers for fields
 * the codegen-default surface ships with a default value.
 *
 * Mirrors `crates/benten-engine/src/module_manifest.rs::HostFnsOverride`.
 */
export interface HostFnsOverride {
  /**
   * Override for the `random` host-fn — primarily the per-call
   * entropy budget (`budget_bytes_per_call`).
   */
  random?: RandomHostFnOverride;
}

/**
 * Phase-3 G17-A2 — per-manifest overrides for the `random` host-fn.
 * All fields additive optional; an undefined field == codegen default.
 *
 * Mirrors `crates/benten-engine/src/module_manifest.rs::RandomHostFnOverride`.
 */
export interface RandomHostFnOverride {
  /**
   * Per-call entropy budget in bytes. Codegen default is 4096
   * (per r1-wsa-8). Manifests MAY tighten or widen this for the
   * modules they declare; overrun fires
   * `E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED`.
   */
  budget_bytes_per_call?: number;
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
 * # Metrics tracking (Phase-3 G19-C2 wave-7 §7.1 — fully closed)
 *
 * `fuelConsumedHighWater` + `outputConsumedHighWater` + `lastInvocationMs`
 * are the per-node runtime-introspection metrics. They are tracked
 * end-to-end from the wasmtime executor through
 * `primitive_host.rs::execute_sandbox` → `EngineInner::sandbox_metrics`
 * → `Engine::describe_sandbox_node_for_handler` → the napi
 * `describeSandboxNode` JSON template → this TS surface.
 *
 * Returned shape:
 * - **`number`** — real measured value from a recorded invocation.
 *   `fuelConsumedHighWater` + `outputConsumedHighWater` are monotonic
 *   high-water marks across invocations within a single Engine
 *   instance; `lastInvocationMs` is the wall-clock duration of the
 *   most recent invocation only.
 * - **`null`** — no SANDBOX invocation has been recorded yet for this
 *   handler. The metric record is created lazily on first
 *   `engine.call(handlerId, ...)` against the SANDBOX-bearing handler.
 *   Distinguishable from `undefined` (which would indicate the field
 *   is absent from the descriptor shape entirely).
 *
 * Cross-process WAIT-resume note (per stream-r1-8): metrics are
 * RAM-only per Engine instance; the suspend/resume envelope does NOT
 * carry in-flight SANDBOX metrics across the boundary. A fresh
 * `Engine.open` starts with an empty metrics map by design. Durable
 * cross-restart promotion follows the GraphBackend umbrella trait
 * (`docs/future/phase-3-backlog.md` §1.1).
 *
 * Pin source: ts-r4-3 R4 finding +
 * `docs/future/phase-3-backlog.md` §7.1 closure;
 * `packages/engine/test/sandbox.test.ts::"SandboxArgs defaults — omitting fuel / wallclockMs / outputLimitBytes uses 1M / 30s / 1MB"` +
 * `packages/engine/test/sandbox.test.ts::"describeSandboxNode returns real numeric metrics after invocation (§7.1 closure)"`.
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
   * every invocation since registration. `null` when no invocation
   * has been recorded yet (lazy-created metric record).
   *
   * Monotonic non-decreasing within an Engine instance lifetime.
   */
  fuelConsumedHighWater: number | null;
  /**
   * Cumulative high-water mark of guest output bytes emitted by this
   * node across every invocation since registration. `null` when no
   * invocation has been recorded yet.
   *
   * Closes the Phase-3 §7.1 trio (fuel + output + wallclock) — was the
   * 25th producer/consumer drift instance, closed at R6 fp Wave C2
   * (`obs-r6r1-1` MAJOR).
   */
  outputConsumedHighWater: number | null;
  /**
   * Wallclock duration of the most recent invocation in milliseconds.
   * `null` when no invocation has been recorded yet.
   *
   * NOT a high-water mark — this is the most-recent invocation only.
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
 * One incremental delta yielded by [`UserView.onUpdate`].
 *
 * Phase-3 G19-C1-fp wave-7-fp shape: the underlying napi
 * `userViewDrainUpdates` accessor surfaces opaque ChangeEvent payloads
 * the engine's per-view side-table observed since the prior cursor.
 * The TS-side wrapper preserves the payload verbatim under
 * `payload` so app code can pattern-match against the raw
 * `ChangeEvent` JSON shape without an extra projection step.
 *
 * The `kind: "change"` discriminator is the intentional Phase-3
 * minimum-viable shape — the engine's Algorithm B generalization
 * (phase-3-backlog §5.1) will widen this to a discriminated union
 * (`"insert"` / `"update"` / `"delete"`) once the per-view side-table
 * tracks row-level mutation kind. Forward-compat: existing app code
 * `for await (const delta of view.onUpdate()) { if (delta.kind ===
 * "change") ... }` continues to compile against the widened union.
 */
export type ViewDelta<T = unknown> = {
  /** Discriminator. Phase-3 minimum-viable; widens to insert/update/delete post-Algorithm-B. */
  kind: "change";
  /** Raw ChangeEvent payload from the napi `userViewDrainUpdates` accessor. */
  payload: T;
};

/**
 * Handle returned by `engine.registerUserView(spec)`. Exposes the
 * resolved id + strategy and the per-view `snapshot()` / `onUpdate()`
 * surfaces:
 *
 * - `snapshot()` — async iterator over currently-materialized rows.
 * - `onUpdate()` — async-iterable iterator over incremental deltas.
 *   Consumed via `for await (const delta of view.onUpdate()) { ... }`;
 *   call `iterator.return()` (or `break` out of `for-await`) to stop
 *   polling cleanly. The native cdylib's per-call accessors
 *   (`userViewDrainUpdates` + `userViewChangeOffset`) drive the
 *   internal 25ms polling cadence; older napi cdylib builds (pre-G19-C1)
 *   yield zero deltas + close cleanly so app code is forward-compatible.
 *
 * Phase-3 G19-C1-fp wave-7-fp lifts `onUpdate` from the prior callback
 * shape (`onUpdate(cb) -> UserViewSubscription`) to the
 * AsyncIterableIterator shape; the callback overload was a clean break
 * since pre-Phase-3 surfaces aren't in customer hands.
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
   * Async-iterable iterator over per-diff change notifications.
   * Consumed via `for await (const delta of view.onUpdate()) { ... }`;
   * `iterator.return()` (or `break` from a `for-await` loop) stops the
   * polling loop cleanly without leaking the timer. When the runtime
   * shim is unavailable (pre-G19-C1 cdylib) the iterator yields zero
   * deltas + closes cleanly.
   */
  onUpdate: () => AsyncIterableIterator<ViewDelta>;
}

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

  /**
   * Phase-3 G19-C2 wave-7 (§7.1.2 + stream-r1-4): `true` for handles
   * produced by `engine.openStream(...)` (explicit-close lifecycle);
   * `false` for handles produced by `engine.callStream(...)`
   * (AsyncIterable auto-close on `for-await` scope-exit).
   *
   * The TS-side `FinalizationRegistry` leak detector consults this
   * accessor to decide whether to fire `E_STREAM_HANDLE_LEAKED` when
   * a handle is GC'd without `close()` being called. Native-side
   * stream ownership stays correct regardless (the producer thread
   * joins on Drop); the leak event is the JS-surface
   * observability hook.
   */
  requiresExplicitClose(): boolean;
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

/**
 * Handle returned by `engine.onEmit(channel, callback)`.
 *
 * Mirrors {@link Subscription} for the EMIT broadcast — the dedicated
 * channel that carries standalone EMIT events (handlers using EMIT
 * without a backing WRITE). See `crates/benten-engine/src/emit_broadcast.rs`
 * for the rationale on a separate channel from ChangeBroadcast.
 *
 * Lifecycle: hold the handle alive for the lifetime of the
 * subscription. Dropping the handle releases the engine-side registry
 * slot AND the `napi::ThreadsafeFunction` Arc backing the JS callback.
 *
 * Wired by R6-FP Group 2 (TS surface) + Group 1 (Rust napi bridge);
 * closes the wave-8h cross-layer audit gap (r6-mpc-2) where the engine
 * had a working `Engine::subscribe_emit_events` Rust API but no JS
 * surface.
 */
export interface EmitSubscription {
  /** `true` while the subscription is registered with the engine. */
  readonly active: boolean;
  /** Channel the subscription was registered with. */
  readonly channel: string;
  /** Explicitly release the subscription. Idempotent. */
  unsubscribe(): void;
}

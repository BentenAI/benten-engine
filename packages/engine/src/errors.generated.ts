// AUTO-GENERATED from docs/ERROR-CATALOG.md by scripts/codegen-errors.ts.
// DO NOT EDIT BY HAND. Run `npx tsx scripts/codegen-errors.ts` to regenerate.
//
// Each error class below corresponds to one `### E_XXX` entry in the
// catalog. The class carries a static `code`, a static `fixHint`, and
// exposes them as instance properties so `err.code` / `err.fixHint`
// work on any thrown instance. The drift-detect script asserts this
// file stays in sync with the catalog and the Rust `ErrorCode` enum
// at `crates/benten-errors/src/lib.rs`.

/* eslint-disable @typescript-eslint/no-unused-vars */

export class BentenError extends Error {
  /** Stable catalog code (e.g. "E_CAP_DENIED"). */
  readonly code: string;
  /** Human-readable fix hint from the catalog. */
  readonly fixHint: string;
  /** Optional structured context attached at throw site. */
  readonly context?: Record<string, unknown>;

  constructor(code: string, fixHint: string, message: string, context?: Record<string, unknown>) {
    super(message);
    this.name = "BentenError";
    this.code = code;
    this.fixHint = fixHint;
    this.context = context;
  }

  override toString(): string {
    return `${this.name} [${this.code}]: ${this.message}\n  fix: ${this.fixHint}`;
  }
}

/** Exhaustive list of catalog codes, for parity checks and narrowing. */
export const CATALOG_CODES = [
  "E_INV_CYCLE",
  "E_INV_DEPTH_EXCEEDED",
  "E_INV_FANOUT_EXCEEDED",
  "E_INV_TOO_MANY_NODES",
  "E_INV_TOO_MANY_EDGES",
  "E_INV_SYSTEM_ZONE",
  "E_INV_DETERMINISM",
  "E_INV_ITERATE_MAX_MISSING",
  "E_INV_ITERATE_BUDGET",
  "E_INV_ITERATE_NEST_DEPTH",
  "E_INV_CONTENT_HASH",
  "E_INV_REGISTRATION",
  "E_CAP_DENIED",
  "E_CAP_DENIED_READ",
  "E_CAP_REVOKED_MID_EVAL",
  "E_CAP_NOT_IMPLEMENTED",
  "E_CAP_REVOKED",
  "E_CAP_ATTENUATION",
  "E_WRITE_CONFLICT",
  "E_INV_SANDBOX_DEPTH",
  "E_INV_SANDBOX_OUTPUT",
  "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED",
  "E_IVM_VIEW_STALE",
  "E_TX_ABORTED",
  "E_NESTED_TRANSACTION_NOT_SUPPORTED",
  "E_PRIMITIVE_NOT_IMPLEMENTED",
  "E_SYSTEM_ZONE_WRITE",
  "E_TRANSFORM_SYNTAX",
  "E_INPUT_LIMIT",
  "E_SERIALIZE",
  "E_SYNC_HASH_MISMATCH",
  "E_SYNC_HLC_DRIFT",
  "E_SYNC_CAP_UNVERIFIED",
  "E_VALUE_FLOAT_NAN",
  "E_VALUE_FLOAT_NONFINITE",
  "E_CID_PARSE",
  "E_CID_UNSUPPORTED_CODEC",
  "E_CID_UNSUPPORTED_HASH",
  "E_VERSION_BRANCHED",
  "E_BACKEND_NOT_FOUND",
  "E_NOT_FOUND",
  "E_GRAPH_INTERNAL",
  "E_UNKNOWN",
  "E_DUPLICATE_HANDLER",
  "E_NO_CAPABILITY_POLICY_CONFIGURED",
  "E_PRODUCTION_REQUIRES_CAPS",
  "E_SUBSYSTEM_DISABLED",
  "E_UNKNOWN_VIEW",
  "E_NOT_IMPLEMENTED",
  "E_IVM_PATTERN_MISMATCH",
  "E_IVM_STRATEGY_NOT_IMPLEMENTED",
  "E_VERSION_UNKNOWN_PRIOR",
  "E_DSL_INVALID_SHAPE",
  "E_DSL_UNREGISTERED_HANDLER",
  "E_HOST_NOT_FOUND",
  "E_HOST_WRITE_CONFLICT",
  "E_HOST_BACKEND_UNAVAILABLE",
  "E_HOST_CAPABILITY_REVOKED",
  "E_HOST_CAPABILITY_EXPIRED",
  "E_EXEC_STATE_TAMPERED",
  "E_RESUME_ACTOR_MISMATCH",
  "E_RESUME_SUBGRAPH_DRIFT",
  "E_WAIT_TIMEOUT",
  "E_INV_IMMUTABILITY",
  "E_INV_ATTRIBUTION",
  "E_CAP_WALLCLOCK_EXPIRED",
  "E_CAP_CHAIN_TOO_DEEP",
  "E_CAP_SCOPE_LONE_STAR_REJECTED",
  "E_VIEW_STRATEGY_A_REFUSED",
  "E_VIEW_STRATEGY_C_RESERVED",
  "E_VIEW_LABEL_MISMATCH",
  "E_WAIT_SIGNAL_SHAPE_MISMATCH",
  "E_WAIT_SUSPENDED",
  "E_STREAM_BACKPRESSURE_DROPPED",
  "E_STREAM_CLOSED_BY_PEER",
  "E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED",
  "E_INV_STREAM_CONFIG",
  "E_STREAM_HANDLE_LEAKED",
  "E_SUBSCRIBE_DELIVERY_FAILED",
  "E_SUBSCRIBE_PATTERN_INVALID",
  "E_SUBSCRIBE_CURSOR_LOST",
  "E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED",
  "E_INV_11_SYSTEM_ZONE_READ",
  "E_SANDBOX_FUEL_EXHAUSTED",
  "E_SANDBOX_MEMORY_EXHAUSTED",
  "E_SANDBOX_WALLCLOCK_EXCEEDED",
  "E_SANDBOX_WALLCLOCK_INVALID",
  "E_SANDBOX_HOST_FN_DENIED",
  "E_SANDBOX_HOST_FN_NOT_FOUND",
  "E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED",
  "E_SANDBOX_MANIFEST_UNKNOWN",
  "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED",
  "E_SANDBOX_MODULE_INVALID",
  "E_SANDBOX_STACK_OVERFLOW",
  "E_SANDBOX_ESCAPE_ATTEMPT",
  "E_SANDBOX_MODULE_NOT_INSTALLED",
  "E_SANDBOX_NESTED_DISPATCH_DENIED",
  "E_MODULE_MANIFEST_CID_MISMATCH",
  "E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE",
  "E_ENGINE_CONFIG_INVALID",
  "E_BACKEND_READ_ONLY",
  "E_SANDBOX_UNAVAILABLE_ON_WASM",
  "E_RELOAD_SUBSCRIBER_UNSUBSCRIBED",
  "E_DEVSERVER_STOPPED",
  "E_STORAGE_QUOTA_EXCEEDED",
  "E_HLC_SKEW_EXCEEDED",
  "E_CAP_UCAN_EXPIRED",
  "E_CAP_UCAN_NOT_YET_VALID",
  "E_CAP_UCAN_BAD_SIGNATURE",
  "E_CAP_UCAN_ATTENUATION_VIOLATED",
  "E_CAP_BACKEND_STORAGE",
  "E_CAP_RATE_LIMIT_EXCEEDED",
  "E_CAP_PEER_BANDWIDTH_EXCEEDED",
  "E_CAP_SNAPSHOT_HASH_MISMATCH",
  "E_SUBSCRIBE_REVOKED_MID_STREAM",
  "E_SYNC_REVOKED_DURING_SESSION",
  "E_DEVICE_ATTESTATION_FORGED",
  "E_SYNC_HOP_DEPTH_EXCEEDED",
  "E_THIN_CLIENT_AUTH_REJECTED",
  "E_CAP_UCAN_AUDIENCE_MISMATCH",
  "E_ATRIUM_RELAY_UNREACHABLE",
  "E_ATRIUM_TRANSPORT_DEGRADED",
  "E_ATRIUM_INACTIVE",
  "E_SYNC_DIVERGENT_CID_REJECTED",
  "E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW",
  "E_WAIT_TTL_EXPIRED",
  "E_WAIT_TTL_INVALID",
  "E_WAIT_METADATA_MISSING",
  "E_TYPED_CALL_UNKNOWN_OP",
  "E_TYPED_CALL_INVALID_INPUT",
  "E_TYPED_CALL_CAP_DENIED",
  "E_TYPED_CALL_DISPATCH_ERROR",
  "E_UCAN_CLOCK_NOT_INJECTED",
  "E_RESERVED_HANDLER_NAMESPACE",
] as const;

export type CatalogCode = (typeof CATALOG_CODES)[number];

/**
 * E_INV_CYCLE
 *
 * Thrown at: Registration
 * Message template: "Subgraph contains a cycle involving Nodes: {cycle_path}"
 */
export class EInvCycle extends BentenError {
  static readonly code = "E_INV_CYCLE";
  static readonly fixHint = "Subgraphs must be DAGs. Replace the back-edge with an ITERATE primitive if repetition is intended.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_CYCLE", "Subgraphs must be DAGs. Replace the back-edge with an ITERATE primitive if repetition is intended.", message, context);
    this.name = "EInvCycle";
  }
}

/**
 * E_INV_DEPTH_EXCEEDED
 *
 * Thrown at: Registration
 * Message template: "Subgraph depth {actual} exceeds configured max {max}"
 */
export class EInvDepthExceeded extends BentenError {
  static readonly code = "E_INV_DEPTH_EXCEEDED";
  static readonly fixHint = "Reduce nesting of CALLs or increase max depth via capability grant.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_DEPTH_EXCEEDED", "Reduce nesting of CALLs or increase max depth via capability grant.", message, context);
    this.name = "EInvDepthExceeded";
  }
}

/**
 * E_INV_FANOUT_EXCEEDED
 *
 * Thrown at: Registration
 * Message template: "Node {node_id} has {actual} outgoing edges, exceeds max fan-out {max}"
 */
export class EInvFanoutExceeded extends BentenError {
  static readonly code = "E_INV_FANOUT_EXCEEDED";
  static readonly fixHint = "Reduce BRANCH cases or split the Node. BRANCH should be binary or multi-way; consider whether a match-table is cleaner.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_FANOUT_EXCEEDED", "Reduce BRANCH cases or split the Node. BRANCH should be binary or multi-way; consider whether a match-table is cleaner.", message, context);
    this.name = "EInvFanoutExceeded";
  }
}

/**
 * E_INV_TOO_MANY_NODES
 *
 * Thrown at: Registration
 * Message template: "Subgraph has {actual} Nodes, exceeds max {max}"
 */
export class EInvTooManyNodes extends BentenError {
  static readonly code = "E_INV_TOO_MANY_NODES";
  static readonly fixHint = "Break into smaller subgraphs connected via CALL.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_TOO_MANY_NODES", "Break into smaller subgraphs connected via CALL.", message, context);
    this.name = "EInvTooManyNodes";
  }
}

/**
 * E_INV_TOO_MANY_EDGES
 *
 * Thrown at: Registration
 * Message template: "Subgraph has {actual} Edges, exceeds max {max}"
 */
export class EInvTooManyEdges extends BentenError {
  static readonly code = "E_INV_TOO_MANY_EDGES";
  static readonly fixHint = "Same as E_INV_TOO_MANY_NODES.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_TOO_MANY_EDGES", "Same as E_INV_TOO_MANY_NODES.", message, context);
    this.name = "EInvTooManyEdges";
  }
}

/**
 * E_INV_SYSTEM_ZONE
 *
 * Thrown at: - Registration — literal-CID walker in `benten-eval::invariants::system_zone::validate_registration` (rejects a READ or WRITE operation node whose `"label"` property or node-id is a `system:*` literal). - Runtime — resolved-label probe in `benten-engine::primitive_host`: - `read_node` / `get_by_label` / `get_by_property` / `read_view` — TRANSFORM-computed CIDs whose resolved Node carries a `system:*` label collapse to `Ok(None)` / empty list at the user surface (symmetric with a backend miss). - `put_node` — fires `EvalError::Invariant(SystemZone)` before the `PendingHostOp` is buffered, so a handler WRITE of a `system:*`-labelled Node never reaches the storage-layer defence-in-depth guard (which would otherwise surface the Phase-1 `E_SYSTEM_ZONE_WRITE` code). - User-facing CRUD — `Engine::create_node` fires this code directly for any `system:*` label in the input Node's `labels` vector. `Engine::get_node` collapses system-zone reads to `Ok(None)` (the probe returns the typed code through the runtime telemetry path but not through the user-visible `Result`).
 * Message template: "Node IDs and labels cannot begin with the reserved 'system:' prefix — it's reserved for engine internals"
 */
export class EInvSystemZone extends BentenError {
  static readonly code = "E_INV_SYSTEM_ZONE";
  static readonly fixHint = "The `system:` prefix is reserved for engine internals; both labels AND node IDs that start with `system:` are rejected at registration as defence-in-depth (G5-B-i Decision 6 reserved-prefix DX improvement). Pick a non-reserved label/ID and re-register. Runtime probing of resolved (TRANSFORM-computed) CIDs collapses system-zone targets to `Ok(None)` on the user-visible surface; only the user-facing `create_node` path fires this error directly for an input label.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SYSTEM_ZONE", "The `system:` prefix is reserved for engine internals; both labels AND node IDs that start with `system:` are rejected at registration as defence-in-depth (G5-B-i Decision 6 reserved-prefix DX improvement). Pick a non-reserved label/ID and re-register. Runtime probing of resolved (TRANSFORM-computed) CIDs collapses system-zone targets to `Ok(None)` on the user-visible surface; only the user-facing `create_node` path fires this error directly for an input label.", message, context);
    this.name = "EInvSystemZone";
  }
}

/**
 * E_INV_DETERMINISM
 *
 * Thrown at: Registration
 * Message template: "Operation {op_type} is classified non-deterministic but appears in a context declared deterministic"
 */
export class EInvDeterminism extends BentenError {
  static readonly code = "E_INV_DETERMINISM";
  static readonly fixHint = "Move non-deterministic operations (SANDBOX, EMIT non-local) outside the deterministic context or relax the declaration.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_DETERMINISM", "Move non-deterministic operations (SANDBOX, EMIT non-local) outside the deterministic context or relax the declaration.", message, context);
    this.name = "EInvDeterminism";
  }
}

/**
 * E_INV_ITERATE_MAX_MISSING
 *
 * Thrown at: Registration
 * Message template: "ITERATE Node {node_id} missing required `max` property"
 */
export class EInvIterateMaxMissing extends BentenError {
  static readonly code = "E_INV_ITERATE_MAX_MISSING";
  static readonly fixHint = "ITERATE requires an explicit `max` to guarantee termination. Add `max: <integer>`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_ITERATE_MAX_MISSING", "ITERATE requires an explicit `max` to guarantee termination. Add `max: <integer>`.", message, context);
    this.name = "EInvIterateMaxMissing";
  }
}

/**
 * E_INV_ITERATE_BUDGET
 *
 * Thrown at: Registration (Phase 2a multiplicative-through-CALL / Code-as-graph Major #2) and Evaluation (Phase 1 runtime flat budget, preserved at `DEFAULT_ITERATION_BUDGET = 100_000` in `crates/benten-eval/src/evaluator.rs`).
 * Message template: "Cumulative iteration budget {actual} exceeds bound {bound} through nested ITERATE/CALL"
 */
export class EInvIterateBudget extends BentenError {
  static readonly code = "E_INV_ITERATE_BUDGET";
  static readonly fixHint = "Reduce the multiplicative iteration space. The cumulative budget is the worst-case product of ITERATE `max` values and non-isolated CALL callee bounds along any DAG path through the handler. Flatten the nested iteration, or declare `isolated: true` on a CALL whose callee runs under its own grant's bound (the callee frame resets the cumulative rather than inheriting the caller's remaining budget — Code-as-graph Major #2 / Option B).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_ITERATE_BUDGET", "Reduce the multiplicative iteration space. The cumulative budget is the worst-case product of ITERATE `max` values and non-isolated CALL callee bounds along any DAG path through the handler. Flatten the nested iteration, or declare `isolated: true` on a CALL whose callee runs under its own grant's bound (the callee frame resets the cumulative rather than inheriting the caller's remaining budget — Code-as-graph Major #2 / Option B).", message, context);
    this.name = "EInvIterateBudget";
  }
}

/**
 * E_INV_ITERATE_NEST_DEPTH
 *
 * Thrown at: Never (retired)
 * Message template: "ITERATE nesting depth {depth} exceeds Phase 1 limit {max}"
 */
export class EInvIterateNestDepth extends BentenError {
  static readonly code = "E_INV_ITERATE_NEST_DEPTH";
  static readonly fixHint = "Phase 1 bounded ITERATE nesting structurally at depth 3 as a stopgap for the cumulative-budget enforcement shipped in Phase 2a. Retired at Phase 2a open — `E_INV_ITERATE_BUDGET` supersedes it. The catalog entry + TS class spelling stay reserved (catalog IDs are stable across phases); the Rust `ErrorCode` variant has been removed because no production path constructs it. The reachability annotation above is the drift-detector's signal that this is a deliberate forward-/backward-compat retention rather than aspirational prose.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_ITERATE_NEST_DEPTH", "Phase 1 bounded ITERATE nesting structurally at depth 3 as a stopgap for the cumulative-budget enforcement shipped in Phase 2a. Retired at Phase 2a open — `E_INV_ITERATE_BUDGET` supersedes it. The catalog entry + TS class spelling stay reserved (catalog IDs are stable across phases); the Rust `ErrorCode` variant has been removed because no production path constructs it. The reachability annotation above is the drift-detector's signal that this is a deliberate forward-/backward-compat retention rather than aspirational prose.", message, context);
    this.name = "EInvIterateNestDepth";
  }
}

/**
 * E_INV_CONTENT_HASH
 *
 * Thrown at: (1) Subgraph load via `Subgraph::load_verified_with_cid` (graph-layer wrapper: `RedbBackend::load_subgraph_verified`); (2) Node load via `Node::load_verified` (graph-layer wrapper: `RedbBackend::get_node` — verify-on-read promoted in W9-T6 Phase-3 R5 wave-9); (3) cross-peer Node ingest via `Mst::apply_entries` per-entry rehash (sec-r4r2-1).
 * Message template: "Content hash mismatch for {node_id}: expected {expected}, computed {actual}"
 */
export class EInvContentHash extends BentenError {
  static readonly code = "E_INV_CONTENT_HASH";
  static readonly fixHint = "Stored bytes' computed content hash does not match the key under which they are addressed. Indicates on-disk corruption, hardware bit-flip, in-flight tamper, or an incompatible serialization migration. Re-hash from source; if persistent, restore from a backup or re-ingest.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_CONTENT_HASH", "Stored bytes' computed content hash does not match the key under which they are addressed. Indicates on-disk corruption, hardware bit-flip, in-flight tamper, or an incompatible serialization migration. Re-hash from source; if persistent, restore from a backup or re-ingest.", message, context);
    this.name = "EInvContentHash";
  }
}

/**
 * E_INV_REGISTRATION
 *
 * Thrown at: Registration
 * Message template: "Subgraph registration failed for {handler_id}: {reason}"
 */
export class EInvRegistration extends BentenError {
  static readonly code = "E_INV_REGISTRATION";
  static readonly fixHint = "Catch-all for registration failures where no more specific `E_INV_*` code applies. The `violated_invariants` list enumerates the specific invariants that rejected the subgraph.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_REGISTRATION", "Catch-all for registration failures where no more specific `E_INV_*` code applies. The `violated_invariants` list enumerates the specific invariants that rejected the subgraph.", message, context);
    this.name = "EInvRegistration";
  }
}

/**
 * E_CAP_DENIED
 *
 * Thrown at: Evaluation (at commit, not at individual WRITE, per the transaction-capability interaction rule)
 * Message template: "Capability {required} not granted to {entity} for WRITE on {target}"
 */
export class ECapDenied extends BentenError {
  static readonly code = "E_CAP_DENIED";
  static readonly fixHint = "Grant the capability, or call from a context that already has it. `requires` on the Node indicates the needed grant.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_DENIED", "Grant the capability, or call from a context that already has it. `requires` on the Node indicates the needed grant.", message, context);
    this.name = "ECapDenied";
  }
}

/**
 * E_CAP_DENIED_READ
 *
 * Thrown at: Evaluation (READ with capability policy configured)
 * Message template: "Capability {required} not granted to {entity} for READ on {target}"
 */
export class ECapDeniedRead extends BentenError {
  static readonly code = "E_CAP_DENIED_READ";
  static readonly fixHint = "Read-side capability denial. Phase 1 chooses honest-leaks-existence semantics: this error confirms the resource exists but the caller lacks read authority. Phase 3 sync may add a per-grant `existence_visibility: hidden` option that returns `E_NOT_FOUND` instead.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_DENIED_READ", "Read-side capability denial. Phase 1 chooses honest-leaks-existence semantics: this error confirms the resource exists but the caller lacks read authority. Phase 3 sync may add a per-grant `existence_visibility: hidden` option that returns `E_NOT_FOUND` instead.", message, context);
    this.name = "ECapDeniedRead";
  }
}

/**
 * E_CAP_REVOKED_MID_EVAL
 *
 * Thrown at: Evaluation
 * Message template: "Capability {grant_id} was revoked during ongoing evaluation at {revoked_at}"
 */
export class ECapRevokedMidEval extends BentenError {
  static readonly code = "E_CAP_REVOKED_MID_EVAL";
  static readonly fixHint = "Distinct from `E_CAP_REVOKED` (Phase 3 sync-side revocation). Fired when a cap is revoked between the start of evaluation and a capability re-check point (commit boundary, CALL entry, or every N ITERATE iterations, default 100). Phase 2 Invariant 13 tightens the window to per-operation.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_REVOKED_MID_EVAL", "Distinct from `E_CAP_REVOKED` (Phase 3 sync-side revocation). Fired when a cap is revoked between the start of evaluation and a capability re-check point (commit boundary, CALL entry, or every N ITERATE iterations, default 100). Phase 2 Invariant 13 tightens the window to per-operation.", message, context);
    this.name = "ECapRevokedMidEval";
  }
}

/**
 * E_CAP_NOT_IMPLEMENTED
 *
 * Thrown at: Evaluation (at commit when an unimplemented backend is configured)
 * Message template: "Capability backend '{backend}' does not implement check_write in phase {phase}"
 */
export class ECapNotImplemented extends BentenError {
  static readonly code = "E_CAP_NOT_IMPLEMENTED";
  static readonly fixHint = "Distinct from `E_CAP_DENIED` — this signals operator misconfiguration (configured a capability backend whose `check_write` arm isn't implemented for the requested phase), not an authorization failure. The Phase-3 `UCANBackend` ships durable + LIVE at G21-T2 audit-6-1 closure (the napi-side `PolicyKind::Ucan` wires to `EngineBuilder::capability_policy_ucan_durable()`); the historical Phase-1 stub form returned this code on the first WRITE prior to G14-B-promotion. Operators on bespoke backends still see this code if their custom `CapabilityPolicy` impl lacks the `check_write` arm; the canonical alternatives are `NoAuthBackend` for embedded/local-only use or layering on top of `GrantBackedPolicy`. Routes to the subgraph's `ON_ERROR` edge, not `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_NOT_IMPLEMENTED", "Distinct from `E_CAP_DENIED` — this signals operator misconfiguration (configured a capability backend whose `check_write` arm isn't implemented for the requested phase), not an authorization failure. The Phase-3 `UCANBackend` ships durable + LIVE at G21-T2 audit-6-1 closure (the napi-side `PolicyKind::Ucan` wires to `EngineBuilder::capability_policy_ucan_durable()`); the historical Phase-1 stub form returned this code on the first WRITE prior to G14-B-promotion. Operators on bespoke backends still see this code if their custom `CapabilityPolicy` impl lacks the `check_write` arm; the canonical alternatives are `NoAuthBackend` for embedded/local-only use or layering on top of `GrantBackedPolicy`. Routes to the subgraph's `ON_ERROR` edge, not `ON_DENIED`.", message, context);
    this.name = "ECapNotImplemented";
  }
}

/**
 * E_CAP_REVOKED
 *
 * Thrown at: Evaluation, sync-receive
 * Message template: "Capability {grant_id} was revoked at {revoked_at}"
 */
export class ECapRevoked extends BentenError {
  static readonly code = "E_CAP_REVOKED";
  static readonly fixHint = "Request a new grant. Revocation propagates via sync with priority.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_REVOKED", "Request a new grant. Revocation propagates via sync with priority.", message, context);
    this.name = "ECapRevoked";
  }
}

/**
 * E_CAP_ATTENUATION
 *
 * Thrown at: Registration (for static chains), evaluation (for dynamic CALL with `isolated: false`)
 * Message template: "Delegated capability scope '{child_scope}' is not a subset of parent scope '{parent_scope}'"
 */
export class ECapAttenuation extends BentenError {
  static readonly code = "E_CAP_ATTENUATION";
  static readonly fixHint = "UCAN attenuation must narrow, never widen. Review the delegation chain.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_ATTENUATION", "UCAN attenuation must narrow, never widen. Review the delegation chain.", message, context);
    this.name = "ECapAttenuation";
  }
}

/**
 * E_WRITE_CONFLICT
 *
 * Thrown at: Evaluation (CAS WRITE). **Runtime surface is edge-routed, not Rust-enum-valued:** WRITE's `cas` mode routes conflicts via the `ON_CONFLICT` edge; the engine stamps `error_code: "E_WRITE_CONFLICT"` on the routed step in `crates/benten-engine/src/primitive_host.rs::outcome_from_terminal_with_cid` (`"ON_CONFLICT"` arm of the edge match). Callers read the code off the edge-routing metadata, not via a `match` on an `Err(EvalError::WriteConflict)` — the enum variant exists for forward-compat with a Phase-2 native Rust path but has no construction site in Phase-1 production code. The drift-detector's `reachability: ignore` annotation reflects this asymmetry.
 * Message template: "Expected version {expected}, found {actual} on {target}"
 */
export class EWriteConflict extends BentenError {
  static readonly code = "E_WRITE_CONFLICT";
  static readonly fixHint = "Re-read, rebase changes, retry. Typical optimistic concurrency pattern.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WRITE_CONFLICT", "Re-read, rebase changes, retry. Typical optimistic concurrency pattern.", message, context);
    this.name = "EWriteConflict";
  }
}

/**
 * E_INV_SANDBOX_DEPTH
 *
 * Thrown at: **Registration** (static SubgraphSpec analysis at `invariants::sandbox_depth::validate_registration`) — fully active. **Runtime** — fully active at Phase 2b close (R6FP-G1 / PR #62, 3-lens convergent fix). `AttributionFrame.sandbox_depth` threads transitively through `ActiveCall` in `crates/benten-engine/src/primitive_host.rs::execute_sandbox` (`frame.sandbox_depth = frame.sandbox_depth.saturating_add(1)`); the dispatching frame is constructed with `sandbox_depth: nested_depth` in both match arms of the same function so SANDBOX-inside-CALL-inside-SANDBOX inherits the parent's depth. See `docs/INVARIANT-COVERAGE.md` §"Inv-4 + Inv-7 runtime arm status" for the wiring trace.
 * Message template: "SANDBOX nest depth {depth} exceeds configured max {max}"
 */
export class EInvSandboxDepth extends BentenError {
  static readonly code = "E_INV_SANDBOX_DEPTH";
  static readonly fixHint = "Reduce SANDBOX nesting (a SANDBOX whose subgraph CALLs another handler that itself SANDBOXes counts toward the same depth at registration time per D20). Either flatten the call chain or increase `max_sandbox_nest_depth` via capability grant.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SANDBOX_DEPTH", "Reduce SANDBOX nesting (a SANDBOX whose subgraph CALLs another handler that itself SANDBOXes counts toward the same depth at registration time per D20). Either flatten the call chain or increase `max_sandbox_nest_depth` via capability grant.", message, context);
    this.name = "EInvSandboxDepth";
  }
}

/**
 * E_INV_SANDBOX_OUTPUT
 *
 * Thrown at: **Evaluation — fully active post-wave-8b.** The `path` field distinguishes the D17 PRIMARY streaming `CountedSink::write` enforcement (fires before host-fn bytes are accepted, in `crates/benten-eval/src/sandbox/counted_sink.rs`) from the D17 BACKSTOP return-value enforcement at the primitive boundary (`CountedSink::backstop_check` after the wasm guest returns). Both arms wired through wave-8b's host-fn trampoline + primitive boundary.
 * Message template: "SANDBOX output {would_be} bytes exceeds max {limit} (consumed {consumed} + attempted {attempted})"
 */
export class EInvSandboxOutput extends BentenError {
  static readonly code = "E_INV_SANDBOX_OUTPUT";
  static readonly fixHint = "Reduce output emitted by the SANDBOX module's host-fn calls (or the primitive return value). D15 trap-loudly default — there is no opt-in silent-truncation flag. Use STREAM for progressive output if the workload genuinely needs unbounded byte volume.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SANDBOX_OUTPUT", "Reduce output emitted by the SANDBOX module's host-fn calls (or the primitive return value). D15 trap-loudly default — there is no opt-in silent-truncation flag. Use STREAM for progressive output if the workload genuinely needs unbounded byte volume.", message, context);
    this.name = "EInvSandboxOutput";
  }
}

/**
 * E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED
 *
 * Thrown at: Evaluation (saturation point at the SANDBOX entry — the counter-saturation check fires before the inner subgraph starts executing). Runtime firing site in `crates/benten-eval/src/primitives/sandbox.rs::execute` (depth-check guard).
 * Message template: "SANDBOX nested-dispatch depth saturated at {depth} (configured max {max})"
 */
export class ESandboxNestedDispatchDepthExceeded extends BentenError {
  static readonly code = "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED";
  static readonly fixHint = "SANDBOX nest-depth saturation overflow distinct from `E_INV_SANDBOX_DEPTH`. Two saturation paths fire this code: the `sandbox_depth: u8` counter saturates at `u8::MAX` (type-level ceiling — extremely deep CALL chains) and the configured `max_sandbox_nest_depth` boundary (capability-grant ceiling). Either case fires this typed error rather than wrapping silently. Reduce nesting per the same guidance as `E_INV_SANDBOX_DEPTH`; if hitting the u8 ceiling, the call topology is almost certainly accidentally recursive and needs structural redesign rather than a higher cap.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED", "SANDBOX nest-depth saturation overflow distinct from `E_INV_SANDBOX_DEPTH`. Two saturation paths fire this code: the `sandbox_depth: u8` counter saturates at `u8::MAX` (type-level ceiling — extremely deep CALL chains) and the configured `max_sandbox_nest_depth` boundary (capability-grant ceiling). Either case fires this typed error rather than wrapping silently. Reduce nesting per the same guidance as `E_INV_SANDBOX_DEPTH`; if hitting the u8 ceiling, the call topology is almost certainly accidentally recursive and needs structural redesign rather than a higher cap.", message, context);
    this.name = "ESandboxNestedDispatchDepthExceeded";
  }
}

/**
 * E_IVM_VIEW_STALE
 *
 * Thrown at: Evaluation (READ from IVM view)
 * Message template: "IVM view {view_id} marked stale; async recomputation in progress"
 */
export class EIvmViewStale extends BentenError {
  static readonly code = "E_IVM_VIEW_STALE";
  static readonly fixHint = "Usually not an error the developer should handle; wait and retry, or accept eventually-consistent semantics. Indicates the per-view CPU/memory budget was exceeded during incremental update.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_IVM_VIEW_STALE", "Usually not an error the developer should handle; wait and retry, or accept eventually-consistent semantics. Indicates the per-view CPU/memory budget was exceeded during incremental update.", message, context);
    this.name = "EIvmViewStale";
  }
}

/**
 * E_TX_ABORTED
 *
 * Thrown at: Evaluation
 * Message template: "Transaction aborted due to {reason}"
 */
export class ETxAborted extends BentenError {
  static readonly code = "E_TX_ABORTED";
  static readonly fixHint = "Inspect the cause. Transactional subgraphs roll back ALL WRITEs on any failure. Check the `failed_node` field for the specific operation that caused the abort.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TX_ABORTED", "Inspect the cause. Transactional subgraphs roll back ALL WRITEs on any failure. Check the `failed_node` field for the specific operation that caused the abort.", message, context);
    this.name = "ETxAborted";
  }
}

/**
 * E_NESTED_TRANSACTION_NOT_SUPPORTED
 *
 * Thrown at: Evaluation
 * Message template: "Nested transaction at {node_id} — Phase 1 does not support nested transaction scopes"
 */
export class ENestedTransactionNotSupported extends BentenError {
  static readonly code = "E_NESTED_TRANSACTION_NOT_SUPPORTED";
  static readonly fixHint = "Phase 1 limits transaction scopes to non-nested calls. Restructure so inner work completes within the outer transaction's single scope, or spawn it after the outer transaction commits. Phase 2 may lift this restriction.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_NESTED_TRANSACTION_NOT_SUPPORTED", "Phase 1 limits transaction scopes to non-nested calls. Restructure so inner work completes within the outer transaction's single scope, or spawn it after the outer transaction commits. Phase 2 may lift this restriction.", message, context);
    this.name = "ENestedTransactionNotSupported";
  }
}

/**
 * E_PRIMITIVE_NOT_IMPLEMENTED
 *
 * Thrown at: Evaluation
 * Message template: "Primitive {primitive_type} is defined but its executor is not implemented in phase {phase}"
 */
export class EPrimitiveNotImplemented extends BentenError {
  static readonly code = "E_PRIMITIVE_NOT_IMPLEMENTED";
  static readonly fixHint = "All 12 primitive *types* are defined in Phase 1 so structural validation can recognize them. The 4 primitives WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX have executors that ship in Phase 2. Avoid calling these primitives in Phase 1 subgraphs or rely on a subgraph whose branch containing them is unreachable on the executed paths.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_PRIMITIVE_NOT_IMPLEMENTED", "All 12 primitive *types* are defined in Phase 1 so structural validation can recognize them. The 4 primitives WAIT / STREAM / SUBSCRIBE-as-user-op / SANDBOX have executors that ship in Phase 2. Avoid calling these primitives in Phase 1 subgraphs or rely on a subgraph whose branch containing them is unreachable on the executed paths.", message, context);
    this.name = "EPrimitiveNotImplemented";
  }
}

/**
 * E_SYSTEM_ZONE_WRITE
 *
 * Thrown at: Evaluation (graph write-path)
 * Message template: "WRITE to system-zone labeled Node '{label}' rejected: operation is not from a privileged engine path"
 */
export class ESystemZoneWrite extends BentenError {
  static readonly code = "E_SYSTEM_ZONE_WRITE";
  static readonly fixHint = "Phase 1 stopgap for Invariant 11 (which fully enforces at registration in Phase 2). User-operation WRITEs cannot touch `system:`-prefixed labels. Use the engine's privileged APIs — `Engine::grant_capability`, `Engine::create_view`, `Engine::revoke_capability` — for system-zone Node mutations.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYSTEM_ZONE_WRITE", "Phase 1 stopgap for Invariant 11 (which fully enforces at registration in Phase 2). User-operation WRITEs cannot touch `system:`-prefixed labels. Use the engine's privileged APIs — `Engine::grant_capability`, `Engine::create_view`, `Engine::revoke_capability` — for system-zone Node mutations.", message, context);
    this.name = "ESystemZoneWrite";
  }
}

/**
 * E_TRANSFORM_SYNTAX
 *
 * Thrown at: Registration (TRANSFORM parser runs at registration time)
 * Message template: "TRANSFORM expression failed to parse: {reason} at position {offset}"
 */
export class ETransformSyntax extends BentenError {
  static readonly code = "E_TRANSFORM_SYNTAX";
  static readonly fixHint = "The TRANSFORM expression language is a positive-allowlist subset of JavaScript. Any token or AST shape not in the allowlist is rejected. Common causes: closures, `this`, imports, template literals with expressions, tagged templates, optional-chained method calls, computed property names referencing `__proto__`/`constructor`/`Symbol.*`, `new`/`with`/`eval`/`yield`/`async`/`await`, destructuring with getters.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TRANSFORM_SYNTAX", "The TRANSFORM expression language is a positive-allowlist subset of JavaScript. Any token or AST shape not in the allowlist is rejected. Common causes: closures, `this`, imports, template literals with expressions, tagged templates, optional-chained method calls, computed property names referencing `__proto__`/`constructor`/`Symbol.*`, `new`/`with`/`eval`/`yield`/`async`/`await`, destructuring with getters.", message, context);
    this.name = "ETransformSyntax";
  }
}

/**
 * E_INPUT_LIMIT
 *
 * Thrown at: Napi binding (before any Rust allocation)
 * Message template: "Napi boundary input exceeds {limit_kind} limit: {actual} > {max}"
 */
export class EInputLimit extends BentenError {
  static readonly code = "E_INPUT_LIMIT";
  static readonly fixHint = "The TS → Rust boundary rejects oversized or pathologically-nested inputs to prevent DoS. Default limits: Value::Map 10K keys, Value::List 10K items, Value::Bytes 16MB, Value::Text 1MB, nesting depth 128, subgraph pre-parse bytes 1MB. Limits are configurable via the engine builder. Either simplify the input or raise the relevant limit explicitly with a capability-grant-authorized override.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INPUT_LIMIT", "The TS → Rust boundary rejects oversized or pathologically-nested inputs to prevent DoS. Default limits: Value::Map 10K keys, Value::List 10K items, Value::Bytes 16MB, Value::Text 1MB, nesting depth 128, subgraph pre-parse bytes 1MB. Limits are configurable via the engine builder. Either simplify the input or raise the relevant limit explicitly with a capability-grant-authorized override.", message, context);
    this.name = "EInputLimit";
  }
}

/**
 * E_SERIALIZE
 *
 * Thrown at: `Node::cid` / `Edge::cid` (pre-hash canonicalization)
 * Message template: "DAG-CBOR serialization failed: {detail}"
 */
export class ESerialize extends BentenError {
  static readonly code = "E_SERIALIZE";
  static readonly fixHint = "The hash path's DAG-CBOR encoder refused the value. In Phase 1 this is effectively unreachable for well-typed input (all `Value` variants encode cleanly); the catalog entry exists so rare edge cases (e.g., encoder integer-overflow) surface a stable, non-empty code rather than an opaque \"unknown\" placeholder. Report as a bug.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SERIALIZE", "The hash path's DAG-CBOR encoder refused the value. In Phase 1 this is effectively unreachable for well-typed input (all `Value` variants encode cleanly); the catalog entry exists so rare edge cases (e.g., encoder integer-overflow) surface a stable, non-empty code rather than an opaque \"unknown\" placeholder. Report as a bug.", message, context);
    this.name = "ESerialize";
  }
}

/**
 * E_SYNC_HASH_MISMATCH
 *
 * Thrown at: Sync-receive (`crates/benten-sync/src/mst.rs::Mst::apply_entries` rehash check; the variant exists at the catalog level as a sync-crate-half closure of the MST-diff-CID-byte-mismatch attack surface. The `apply_atrium_merge` engine receive-boundary today consumes Loro-CRDT byte-merge bytes, NOT MstDiff entries — engine-side wireup is the missing half. Per Wave-C1 cryptography mini-review (c1-crypto-mr-1): scope the closure claim to "sync-crate-half pending engine wireup"; the engine-half lands when MstDiff routing through the Atrium receive-boundary lands as a future Phase-3 follow-up wave OR v1-window concern. Reachability-ignored in the meantime, mirroring `E_SYNC_CAP_UNVERIFIED` forward-compat reservation pattern.)
 * Message template: "Received content hash {received} does not match expected {expected}"
 */
export class ESyncHashMismatch extends BentenError {
  static readonly code = "E_SYNC_HASH_MISMATCH";
  static readonly fixHint = "Possible tampering or corruption. Sync is aborted; investigate the peer.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYNC_HASH_MISMATCH", "Possible tampering or corruption. Sync is aborted; investigate the peer.", message, context);
    this.name = "ESyncHashMismatch";
  }
}

/**
 * E_SYNC_HLC_DRIFT
 *
 * Thrown at: Sync-receive
 * Message template: "HLC timestamp {received} exceeds drift tolerance {max_drift} from local clock {local}"
 */
export class ESyncHlcDrift extends BentenError {
  static readonly code = "E_SYNC_HLC_DRIFT";
  static readonly fixHint = "Peer's clock is outside tolerance. Triggers clock reconciliation handshake; if that fails, sync pauses.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYNC_HLC_DRIFT", "Peer's clock is outside tolerance. Triggers clock reconciliation handshake; if that fails, sync pauses.", message, context);
    this.name = "ESyncHlcDrift";
  }
}

/**
 * E_SYNC_CAP_UNVERIFIED
 *
 * Thrown at: Sync-receive (reserved companion to `E_SYNC_REVOKED_DURING_SESSION` per Phase-3 R6-FP Wave-C1 — covers the missing-or-malformed cap-chain case where a peer never had a valid grant; the revoked-mid-session case fires `E_SYNC_REVOKED_DURING_SESSION` from `apply_atrium_merge`'s per-row recheck. The `SyncCapUnverified` construction site lands when the handshake-time cap-chain validator wires through; until then the variant is reachability-ignored as a forward-compat catalog reservation.)
 * Message template: "Received WRITE lacks valid capability chain from {peer}"
 */
export class ESyncCapUnverified extends BentenError {
  static readonly code = "E_SYNC_CAP_UNVERIFIED";
  static readonly fixHint = "Peer sent a change without proper authority. Sync-receive rejects; investigate peer trust level.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYNC_CAP_UNVERIFIED", "Peer sent a change without proper authority. Sync-receive rejects; investigate peer trust level.", message, context);
    this.name = "ESyncCapUnverified";
  }
}

/**
 * E_VALUE_FLOAT_NAN
 *
 * Thrown at: Value construction / deserialization
 * Message template: "Floating-point value is NaN; Value::Float rejects NaN for deterministic content-addressing"
 */
export class EValueFloatNan extends BentenError {
  static readonly code = "E_VALUE_FLOAT_NAN";
  static readonly fixHint = "The content-hash must be canonical; NaN compares unequal to itself and breaks hash determinism. Replace NaN with a sentinel (e.g. `Value::Null`) or with a specific finite value.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VALUE_FLOAT_NAN", "The content-hash must be canonical; NaN compares unequal to itself and breaks hash determinism. Replace NaN with a sentinel (e.g. `Value::Null`) or with a specific finite value.", message, context);
    this.name = "EValueFloatNan";
  }
}

/**
 * E_VALUE_FLOAT_NONFINITE
 *
 * Thrown at: Value construction / deserialization
 * Message template: "Floating-point value is non-finite (Infinity / -Infinity); Value::Float requires finite numbers"
 */
export class EValueFloatNonFinite extends BentenError {
  static readonly code = "E_VALUE_FLOAT_NONFINITE";
  static readonly fixHint = "DAG-CBOR's canonical form rejects ±Infinity. Clamp to a finite bound or use `Value::Null`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VALUE_FLOAT_NONFINITE", "DAG-CBOR's canonical form rejects ±Infinity. Clamp to a finite bound or use `Value::Null`.", message, context);
    this.name = "EValueFloatNonFinite";
  }
}

/**
 * E_CID_PARSE
 *
 * Thrown at: CID deserialization / napi boundary
 * Message template: "CID bytes could not be parsed into a CIDv1: {detail}"
 */
export class ECidParse extends BentenError {
  static readonly code = "E_CID_PARSE";
  static readonly fixHint = "Phase 1 accepts base32-lower-nopad multibase (`b`-prefixed) CIDv1 via both the napi boundary and the Rust `Cid::from_str` path. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CID_PARSE", "Phase 1 accepts base32-lower-nopad multibase (`b`-prefixed) CIDv1 via both the napi boundary and the Rust `Cid::from_str` path. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated.", message, context);
    this.name = "ECidParse";
  }
}

/**
 * E_CID_UNSUPPORTED_CODEC
 *
 * Thrown at: CID deserialization (`Cid::from_bytes` — distinct from `E_CID_PARSE`, which is reserved for length / version / digest-length structural failures)
 * Message template: "CID codec {codec} is not supported; Phase 1 recognizes DAG-CBOR (0x71)"
 */
export class ECidUnsupportedCodec extends BentenError {
  static readonly code = "E_CID_UNSUPPORTED_CODEC";
  static readonly fixHint = "Phase 1 only accepts DAG-CBOR multicodec (0x71). Re-encode under the expected codec or await later-phase codec support.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CID_UNSUPPORTED_CODEC", "Phase 1 only accepts DAG-CBOR multicodec (0x71). Re-encode under the expected codec or await later-phase codec support.", message, context);
    this.name = "ECidUnsupportedCodec";
  }
}

/**
 * E_CID_UNSUPPORTED_HASH
 *
 * Thrown at: CID deserialization (`Cid::from_bytes` — distinct from `E_CID_PARSE`, which is reserved for length / version / digest-length structural failures)
 * Message template: "CID hash function {code} is not supported; Phase 1 recognizes BLAKE3 (0x1e)"
 */
export class ECidUnsupportedHash extends BentenError {
  static readonly code = "E_CID_UNSUPPORTED_HASH";
  static readonly fixHint = "Phase 1 only accepts BLAKE3 multihash (0x1e). Re-hash with BLAKE3 or await later-phase multi-hash support.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CID_UNSUPPORTED_HASH", "Phase 1 only accepts BLAKE3 multihash (0x1e). Re-hash with BLAKE3 or await later-phase multi-hash support.", message, context);
    this.name = "ECidUnsupportedHash";
  }
}

/**
 * E_VERSION_BRANCHED
 *
 * Thrown at: Version-chain traversal
 * Message template: "Version chain has branched — multiple NEXT_VERSION edges from the same Version Node"
 */
export class EVersionBranched extends BentenError {
  static readonly code = "E_VERSION_BRANCHED";
  static readonly fixHint = "A Version Node should have at most one NEXT_VERSION successor on any linear chain. Branches are a Phase-3 sync consequence; in Phase 1 this indicates a programming error writing two NEXT_VERSION edges. Walk the chain, pick the intended successor, and remove the other NEXT_VERSION edge.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VERSION_BRANCHED", "A Version Node should have at most one NEXT_VERSION successor on any linear chain. Branches are a Phase-3 sync consequence; in Phase 1 this indicates a programming error writing two NEXT_VERSION edges. Walk the chain, pick the intended successor, and remove the other NEXT_VERSION edge.", message, context);
    this.name = "EVersionBranched";
  }
}

/**
 * E_BACKEND_NOT_FOUND
 *
 * Thrown at: Engine builder / backend resolution
 * Message template: "Named backend '{name}' is not registered on this engine"
 */
export class EBackendNotFound extends BentenError {
  static readonly code = "E_BACKEND_NOT_FOUND";
  static readonly fixHint = "Phase 1 wires a single in-memory + redb backend pair; alternate backends land with Phase-2. This error fires when a sub-component addresses a backend that is not configured.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_BACKEND_NOT_FOUND", "Phase 1 wires a single in-memory + redb backend pair; alternate backends land with Phase-2. This error fires when a sub-component addresses a backend that is not configured.", message, context);
    this.name = "EBackendNotFound";
  }
}

/**
 * E_NOT_FOUND
 *
 * Thrown at: Engine lookups
 * Message template: "Requested entity not found: {kind} {identifier}"
 */
export class ENotFound extends BentenError {
  static readonly code = "E_NOT_FOUND";
  static readonly fixHint = "Generic not-found — version-chain anchor miss, unknown view id, missing grant lookup, etc. Check that the caller has the correct CID / id. For unregistered-handler lookups specifically (post R6 fp Wave C2 / dx-r6-r1-1), the engine emits the more-specific `E_DSL_UNREGISTERED_HANDLER` instead so JS callers see the matching `EDslUnregisteredHandler` typed BentenError subclass.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_NOT_FOUND", "Generic not-found — version-chain anchor miss, unknown view id, missing grant lookup, etc. Check that the caller has the correct CID / id. For unregistered-handler lookups specifically (post R6 fp Wave C2 / dx-r6-r1-1), the engine emits the more-specific `E_DSL_UNREGISTERED_HANDLER` instead so JS callers see the matching `EDslUnregisteredHandler` typed BentenError subclass.", message, context);
    this.name = "ENotFound";
  }
}

/**
 * E_GRAPH_INTERNAL
 *
 * Thrown at: Graph backend (storage I/O)
 * Message template: "Graph storage internal error: {detail}"
 */
export class EGraphInternal extends BentenError {
  static readonly code = "E_GRAPH_INTERNAL";
  static readonly fixHint = "Stable code for `GraphError::RedbSource` / `GraphError::Redb` / `GraphError::Decode` — a storage-layer failure (redb I/O, transactional abort, DAG-CBOR decode of a stored Node). The underlying `std::error::Error::source()` chain is preserved on the Rust side for diagnostics; at the TS boundary only the stable code is surfaced. Inspect logs or retry; persistent errors indicate on-disk corruption and should prompt a restore from backup.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_GRAPH_INTERNAL", "Stable code for `GraphError::RedbSource` / `GraphError::Redb` / `GraphError::Decode` — a storage-layer failure (redb I/O, transactional abort, DAG-CBOR decode of a stored Node). The underlying `std::error::Error::source()` chain is preserved on the Rust side for diagnostics; at the TS boundary only the stable code is surfaced. Inspect logs or retry; persistent errors indicate on-disk corruption and should prompt a restore from backup.", message, context);
    this.name = "EGraphInternal";
  }
}

/**
 * E_UNKNOWN
 *
 * Thrown at: Forward-compat deserialization
 * Message template: "Unknown error code (forward-compat fallback)"
 */
export class EUnknown extends BentenError {
  static readonly code = "E_UNKNOWN";
  static readonly fixHint = "The drift-detect / catalog contract reserves `ErrorCode::Unknown(s)` as a forward-compat escape valve so a newer server emitting an unrecognized code does not crash an older client. If this code reaches a caller, update the engine / bindings to the latest release — the payload carries the raw code string the server actually emitted. Never thrown by Phase-1 Rust code deliberately; exists only to make the enum round-trip through `from_str` infallible.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_UNKNOWN", "The drift-detect / catalog contract reserves `ErrorCode::Unknown(s)` as a forward-compat escape valve so a newer server emitting an unrecognized code does not crash an older client. If this code reaches a caller, update the engine / bindings to the latest release — the payload carries the raw code string the server actually emitted. Never thrown by Phase-1 Rust code deliberately; exists only to make the enum round-trip through `from_str` infallible.", message, context);
    this.name = "EUnknown";
  }
}

/**
 * E_DUPLICATE_HANDLER
 *
 * Thrown at: Engine (`register_subgraph` / `register_crud`)
 * Message template: "Handler id '{handler_id}' already registered with different subgraph content"
 */
export class EDuplicateHandler extends BentenError {
  static readonly code = "E_DUPLICATE_HANDLER";
  static readonly fixHint = "Handler ids are unique within an engine. Either choose a distinct id, re-register with the same content (idempotent), or unregister the existing handler first. Two subgraphs with different CIDs cannot share an id.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_DUPLICATE_HANDLER", "Handler ids are unique within an engine. Either choose a distinct id, re-register with the same content (idempotent), or unregister the existing handler first. Two subgraphs with different CIDs cannot share an id.", message, context);
    this.name = "EDuplicateHandler";
  }
}

/**
 * E_NO_CAPABILITY_POLICY_CONFIGURED
 *
 * Thrown at: Engine builder
 * Message template: "No capability policy configured for .production() builder — call .capability_policy(...) or drop .production()"
 */
export class ENoCapabilityPolicyConfigured extends BentenError {
  static readonly code = "E_NO_CAPABILITY_POLICY_CONFIGURED";
  static readonly fixHint = "`Engine::builder().production()` refuses to build without an explicit `CapabilityPolicy` (R1 SC2 fail-early guardrail). Call `.capability_policy(policy)` before `.open(...)`, or drop `.production()` if the engine should accept the `NoAuthBackend` default for local/embedded use.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_NO_CAPABILITY_POLICY_CONFIGURED", "`Engine::builder().production()` refuses to build without an explicit `CapabilityPolicy` (R1 SC2 fail-early guardrail). Call `.capability_policy(policy)` before `.open(...)`, or drop `.production()` if the engine should accept the `NoAuthBackend` default for local/embedded use.", message, context);
    this.name = "ENoCapabilityPolicyConfigured";
  }
}

/**
 * E_PRODUCTION_REQUIRES_CAPS
 *
 * Thrown at: Engine builder
 * Message template: "Production mode requires capabilities — .production() and .without_caps() are mutually exclusive"
 */
export class EProductionRequiresCaps extends BentenError {
  static readonly code = "E_PRODUCTION_REQUIRES_CAPS";
  static readonly fixHint = "`.production()` enforces that a capability policy must be configured. `.without_caps()` explicitly tears one down. Picking both is a misconfiguration — drop one. Code-reviewer finding `g7-cr-1`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_PRODUCTION_REQUIRES_CAPS", "`.production()` enforces that a capability policy must be configured. `.without_caps()` explicitly tears one down. Picking both is a misconfiguration — drop one. Code-reviewer finding `g7-cr-1`.", message, context);
    this.name = "EProductionRequiresCaps";
  }
}

/**
 * E_SUBSYSTEM_DISABLED
 *
 * Thrown at: Engine operations (`read_view`, `grant_capability`, `create_view`, …)
 * Message template: "Subsystem disabled: {subsystem}"
 */
export class ESubsystemDisabled extends BentenError {
  static readonly code = "E_SUBSYSTEM_DISABLED";
  static readonly fixHint = "A thin engine configured with `.without_ivm()` or `.without_caps()` refuses operations that require the disabled subsystem — the \"honest no\" boundary. Either rebuild the engine without the opt-out, or restructure the caller to avoid the dependent surface.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SUBSYSTEM_DISABLED", "A thin engine configured with `.without_ivm()` or `.without_caps()` refuses operations that require the disabled subsystem — the \"honest no\" boundary. Either rebuild the engine without the opt-out, or restructure the caller to avoid the dependent surface.", message, context);
    this.name = "ESubsystemDisabled";
  }
}

/**
 * E_UNKNOWN_VIEW
 *
 * Thrown at: Engine (`read_view`)
 * Message template: "Unknown view: {view_id}"
 */
export class EUnknownView extends BentenError {
  static readonly code = "E_UNKNOWN_VIEW";
  static readonly fixHint = "The view id was not registered. From TypeScript use `engine.createView(viewDef)`; from Rust use `Engine::create_view` (or the built-in views wired at engine-build time). Check spelling, confirm the IVM subscriber has the view wired, and that `.without_ivm()` was not used on the builder.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_UNKNOWN_VIEW", "The view id was not registered. From TypeScript use `engine.createView(viewDef)`; from Rust use `Engine::create_view` (or the built-in views wired at engine-build time). Check spelling, confirm the IVM subscriber has the view wired, and that `.without_ivm()` was not used on the builder.", message, context);
    this.name = "EUnknownView";
  }
}

/**
 * E_NOT_IMPLEMENTED
 *
 * Thrown at: Engine (primitive-dispatch surfaces)
 * Message template: "Not implemented in Phase 1: {feature}"
 */
export class ENotImplemented extends BentenError {
  static readonly code = "E_NOT_IMPLEMENTED";
  static readonly fixHint = "The engine method is a typed-todo that is wired for Phase 2+ evaluator integration. Avoid the surface in Phase-1 code or pick an equivalent Phase-1-landed alternative. See the per-method rustdoc for the target phase.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_NOT_IMPLEMENTED", "The engine method is a typed-todo that is wired for Phase 2+ evaluator integration. Avoid the surface in Phase-1 code or pick an equivalent Phase-1-landed alternative. See the per-method rustdoc for the target phase.", message, context);
    this.name = "ENotImplemented";
  }
}

/**
 * E_IVM_PATTERN_MISMATCH
 *
 * Thrown at: IVM view read (`View::read` on any of the five Phase-1 views)
 * Message template: "IVM view query pattern does not match any maintained index: {detail}"
 */
export class EIvmPatternMismatch extends BentenError {
  static readonly code = "E_IVM_PATTERN_MISMATCH";
  static readonly fixHint = "The caller asked a view for an index partition it doesn't maintain. Each of the five Phase-1 views keys on a specific field and rejects queries that omit it: - `capability_grants` requires `entity_cid` - `event_dispatch` requires `event_name` - `content_listing` accepts `label` (optional — omitted returns full listing; a non-matching label is rejected) - `governance_inheritance` requires `entity_cid` - `version_current` requires `anchor_id` Consult the view's maintained-pattern list and restrict the `ViewQuery` to supported keys. Distinct from `E_INV_REGISTRATION` — the view is healthy; the query shape is wrong.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_IVM_PATTERN_MISMATCH", "The caller asked a view for an index partition it doesn't maintain. Each of the five Phase-1 views keys on a specific field and rejects queries that omit it: - `capability_grants` requires `entity_cid` - `event_dispatch` requires `event_name` - `content_listing` accepts `label` (optional — omitted returns full listing; a non-matching label is rejected) - `governance_inheritance` requires `entity_cid` - `version_current` requires `anchor_id` Consult the view's maintained-pattern list and restrict the `ViewQuery` to supported keys. Distinct from `E_INV_REGISTRATION` — the view is healthy; the query shape is wrong.", message, context);
    this.name = "EIvmPatternMismatch";
  }
}

/**
 * E_IVM_STRATEGY_NOT_IMPLEMENTED
 *
 * Thrown at: IVM view registration (`benten_ivm::testing::try_construct_view_with_strategy`)
 * Message template: "IVM strategy `{strategy:?}` is reserved but not implemented in this phase (deferred to {deferred_to_phase})"
 */
export class EIvmStrategyNotImplemented extends BentenError {
  static readonly code = "E_IVM_STRATEGY_NOT_IMPLEMENTED";
  static readonly fixHint = "Phase 2b ships `Strategy::A` (the 5 Phase-1 hand-written views) + `Strategy::B` (the generalized Algorithm B). `Strategy::C` (Z-set / DBSP cancellation) is reserved for Phase 3+ — the variant exists so the catalog of options is complete and stable, but constructing a `Strategy::C` view via `benten_ivm::testing::try_construct_view_with_strategy` returns this typed error rather than silently falling back. Pick `Strategy::B` for new user-registered views; pick `Strategy::A` for the 5 hand-written baselines (Rust-only, defaults applied automatically).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_IVM_STRATEGY_NOT_IMPLEMENTED", "Phase 2b ships `Strategy::A` (the 5 Phase-1 hand-written views) + `Strategy::B` (the generalized Algorithm B). `Strategy::C` (Z-set / DBSP cancellation) is reserved for Phase 3+ — the variant exists so the catalog of options is complete and stable, but constructing a `Strategy::C` view via `benten_ivm::testing::try_construct_view_with_strategy` returns this typed error rather than silently falling back. Pick `Strategy::B` for new user-registered views; pick `Strategy::A` for the 5 hand-written baselines (Rust-only, defaults applied automatically).", message, context);
    this.name = "EIvmStrategyNotImplemented";
  }
}

/**
 * E_VERSION_UNKNOWN_PRIOR
 *
 * Thrown at: Version-chain `append_version`
 * Message template: "Prior head was never observed by this anchor: {supplied}"
 */
export class EVersionUnknownPrior extends BentenError {
  static readonly code = "E_VERSION_UNKNOWN_PRIOR";
  static readonly fixHint = "Surfaces from the prior-head-threaded `benten_core::version::append_version` when the caller names a `prior_head` that is neither the anchor's root head nor any new_head from a previous successful append. Re-read the anchor's current head (`walk_versions`) and retry against the observed head. Distinct from `E_VERSION_BRANCHED` (which fires when two appends race the same legitimate prior).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VERSION_UNKNOWN_PRIOR", "Surfaces from the prior-head-threaded `benten_core::version::append_version` when the caller names a `prior_head` that is neither the anchor's root head nor any new_head from a previous successful append. Re-read the anchor's current head (`walk_versions`) and retry against the observed head. Distinct from `E_VERSION_BRANCHED` (which fires when two appends race the same legitimate prior).", message, context);
    this.name = "EVersionUnknownPrior";
  }
}

/**
 * E_DSL_INVALID_SHAPE
 *
 * Thrown at: TypeScript DSL wrapper (`packages/engine/src/errors.generated.ts::EDslInvalidShape`, used from `packages/engine/src/dsl.ts` builder methods) AND Rust DSL compiler (`crates/benten-dsl-compiler/src/lib.rs` — object/pair shape validation in the parser/emit pass) AND Rust engine (`crates/benten-engine/src/engine.rs::register_subgraph` — SANDBOX numeric-budget shape validation walk per `docs/SANDBOX-LIMITS.md` §2).
 * Message template: "DSL value does not match expected shape: {reason}"
 */
export class EDslInvalidShape extends BentenError {
  static readonly code = "E_DSL_INVALID_SHAPE";
  static readonly fixHint = "Check the DSL API documentation for the expected shape.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_DSL_INVALID_SHAPE", "Check the DSL API documentation for the expected shape.", message, context);
    this.name = "EDslInvalidShape";
  }
}

/**
 * E_DSL_UNREGISTERED_HANDLER
 *
 * Thrown at: TypeScript DSL wrapper (`call` method near-match suggestion path on `packages/engine/src/engine.ts::Engine`) AND Rust engine (`crates/benten-engine/src/engine.rs` — `dispatch_call_with_mode_and_trace`, `dispatch_call_inner`, `handler_to_mermaid`, `handler_predecessors`, `emit_with_handler`, `subscribe_with_handler`; `crates/benten-engine/src/engine_stream.rs::call_stream`).
 * Message template: "No handler registered for '{handler_id}'"
 */
export class EDslUnregisteredHandler extends BentenError {
  static readonly code = "E_DSL_UNREGISTERED_HANDLER";
  static readonly fixHint = "Check spelling; register via `engine.registerSubgraph(handler)` or `engine.registerSubgraph(crud('<label>'))`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_DSL_UNREGISTERED_HANDLER", "Check spelling; register via `engine.registerSubgraph(handler)` or `engine.registerSubgraph(crud('<label>'))`.", message, context);
    this.name = "EDslUnregisteredHandler";
  }
}

/**
 * E_HOST_NOT_FOUND
 *
 * Thrown at: `PrimitiveHost` implementation (G1-B)
 * Message template: "Host-boundary lookup miss: {kind} {identifier}"
 */
export class EHostNotFound extends BentenError {
  static readonly code = "E_HOST_NOT_FOUND";
  static readonly fixHint = "Reserved HostError discriminant. Surfaces from `PrimitiveHost` impls when the requested entity is not in the backend. Distinct from `E_NOT_FOUND` because it carries the host-layer boundary (preserves the `benten-eval` → `benten-graph` arch-1 dep break).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_NOT_FOUND", "Reserved HostError discriminant. Surfaces from `PrimitiveHost` impls when the requested entity is not in the backend. Distinct from `E_NOT_FOUND` because it carries the host-layer boundary (preserves the `benten-eval` → `benten-graph` arch-1 dep break).", message, context);
    this.name = "EHostNotFound";
  }
}

/**
 * E_HOST_WRITE_CONFLICT
 *
 * Thrown at: `PrimitiveHost` implementation (G1-B)
 * Message template: "Host-boundary optimistic-concurrency conflict on {target}"
 */
export class EHostWriteConflict extends BentenError {
  static readonly code = "E_HOST_WRITE_CONFLICT";
  static readonly fixHint = "Reserved HostError discriminant. Fires when a host-level compare-and-swap write detects a concurrent mutation. Surface is frozen at Phase 2a; firing site deferred to v1-assessment-window.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_WRITE_CONFLICT", "Reserved HostError discriminant. Fires when a host-level compare-and-swap write detects a concurrent mutation. Surface is frozen at Phase 2a; firing site deferred to v1-assessment-window.", message, context);
    this.name = "EHostWriteConflict";
  }
}

/**
 * E_HOST_BACKEND_UNAVAILABLE
 *
 * Thrown at: `PrimitiveHost` implementations + `benten_eval::resume_with_meta` (Phase-2a discriminant; Phase-3 firing sites listed above).
 * Message template: "Host-boundary backend unavailable: {detail}"
 */
export class EHostBackendUnavailable extends BentenError {
  static readonly code = "E_HOST_BACKEND_UNAVAILABLE";
  static readonly fixHint = "Fires when the underlying storage backend is offline (I/O error, disk full, network partition) OR as the eval-layer fail-loud surface for missing WAIT metadata (the engine layer promotes this to the typed `E_WAIT_METADATA_MISSING` at `engine_wait.rs::map_resume_eval_error`; the eval-side code stays `HostBackendUnavailable` as the broader generic-backend-unavailable surface). Retry with exponential backoff; if persistent, inspect the storage layer.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_BACKEND_UNAVAILABLE", "Fires when the underlying storage backend is offline (I/O error, disk full, network partition) OR as the eval-layer fail-loud surface for missing WAIT metadata (the engine layer promotes this to the typed `E_WAIT_METADATA_MISSING` at `engine_wait.rs::map_resume_eval_error`; the eval-side code stays `HostBackendUnavailable` as the broader generic-backend-unavailable surface). Retry with exponential backoff; if persistent, inspect the storage layer.", message, context);
    this.name = "EHostBackendUnavailable";
  }
}

/**
 * E_HOST_CAPABILITY_REVOKED
 *
 * Thrown at: `PrimitiveHost` implementation (G1-B)
 * Message template: "Host-boundary capability was revoked mid-operation"
 */
export class EHostCapabilityRevoked extends BentenError {
  static readonly code = "E_HOST_CAPABILITY_REVOKED";
  static readonly fixHint = "Reserved HostError discriminant. Fires when a host-level capability check observes a revocation between resolve and use. Retry after re-granting.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_CAPABILITY_REVOKED", "Reserved HostError discriminant. Fires when a host-level capability check observes a revocation between resolve and use. Retry after re-granting.", message, context);
    this.name = "EHostCapabilityRevoked";
  }
}

/**
 * E_HOST_CAPABILITY_EXPIRED
 *
 * Thrown at: `PrimitiveHost` implementation (G1-B)
 * Message template: "Host-boundary capability expired by TTL"
 */
export class EHostCapabilityExpired extends BentenError {
  static readonly code = "E_HOST_CAPABILITY_EXPIRED";
  static readonly fixHint = "Reserved HostError discriminant. Fires when a host-level capability check observes the grant's TTL has elapsed. Re-grant with a longer TTL or refresh the cap.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_CAPABILITY_EXPIRED", "Reserved HostError discriminant. Fires when a host-level capability check observes the grant's TTL has elapsed. Re-grant with a longer TTL or refresh the cap.", message, context);
    this.name = "EHostCapabilityExpired";
  }
}

/**
 * E_EXEC_STATE_TAMPERED
 *
 * Thrown at: `Engine::resume_from_bytes` (G3-A resume protocol step 1)
 * Message template: "ExecutionState payload_cid mismatch — envelope tampered"
 */
export class EExecStateTampered extends BentenError {
  static readonly code = "E_EXEC_STATE_TAMPERED";
  static readonly fixHint = "The resume envelope's `payload_cid` recomputation does not match the declared CID. Either the bytes were tampered in transit, or the Phase-2a serialization layer drifted. Verify the source of the bytes; never resume from untrusted storage without an integrity check.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_EXEC_STATE_TAMPERED", "The resume envelope's `payload_cid` recomputation does not match the declared CID. Either the bytes were tampered in transit, or the Phase-2a serialization layer drifted. Verify the source of the bytes; never resume from untrusted storage without an integrity check.", message, context);
    this.name = "EExecStateTampered";
  }
}

/**
 * E_RESUME_ACTOR_MISMATCH
 *
 * Thrown at: `Engine::resume_from_bytes_as` (G3-A resume protocol step 2)
 * Message template: "Resume principal does not match the suspended ExecutionState"
 */
export class EResumeActorMismatch extends BentenError {
  static readonly code = "E_RESUME_ACTOR_MISMATCH";
  static readonly fixHint = "The caller attempting `resume_from_bytes_as` does not match the actor recorded at suspend time. Only the same principal (or an equivalent delegated grant) can resume. Verify the caller identity; use `resume_from_bytes` only on the original actor's behalf.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_RESUME_ACTOR_MISMATCH", "The caller attempting `resume_from_bytes_as` does not match the actor recorded at suspend time. Only the same principal (or an equivalent delegated grant) can resume. Verify the caller identity; use `resume_from_bytes` only on the original actor's behalf.", message, context);
    this.name = "EResumeActorMismatch";
  }
}

/**
 * E_RESUME_SUBGRAPH_DRIFT
 *
 * Thrown at: `Engine::resume_from_bytes` (G3-A resume protocol step 3)
 * Message template: "Pinned subgraph CID drifted from the currently registered head"
 */
export class EResumeSubgraphDrift extends BentenError {
  static readonly code = "E_RESUME_SUBGRAPH_DRIFT";
  static readonly fixHint = "The subgraph the caller suspended against has since been re-registered under a new CID. Resumption deliberately refuses to cross that boundary. If the drift is expected, re-suspend under the new CID. Distinct from `E_INV_IMMUTABILITY` — the drift is detected at resume time, not write time.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_RESUME_SUBGRAPH_DRIFT", "The subgraph the caller suspended against has since been re-registered under a new CID. Resumption deliberately refuses to cross that boundary. If the drift is expected, re-suspend under the new CID. Distinct from `E_INV_IMMUTABILITY` — the drift is detected at resume time, not write time.", message, context);
    this.name = "EResumeSubgraphDrift";
  }
}

/**
 * E_WAIT_TIMEOUT
 *
 * Thrown at: WAIT executor (G3-B duration path)
 * Message template: "WAIT deadline elapsed before a resume signal arrived"
 */
export class EWaitTimeout extends BentenError {
  static readonly code = "E_WAIT_TIMEOUT";
  static readonly fixHint = "A WAIT declared `duration: <ms>` and the deadline elapsed without a matching signal. Either the orchestrator that was meant to resume the suspension never dispatched, or the deadline was too tight. Re-call with a longer duration, or wire a fallback ON_ERROR edge to downstream compensation logic.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WAIT_TIMEOUT", "A WAIT declared `duration: <ms>` and the deadline elapsed without a matching signal. Either the orchestrator that was meant to resume the suspension never dispatched, or the deadline was too tight. Re-call with a longer duration, or wire a fallback ON_ERROR edge to downstream compensation logic.", message, context);
    this.name = "EWaitTimeout";
  }
}

/**
 * E_INV_IMMUTABILITY
 *
 * Thrown at: graph write-path (G5-A, `benten-graph`); declaration-time affordance at `benten-eval::invariants::immutability` rejects WRITE primitives whose literal `target_cid` is already registered.
 * Message template: "Write would mutate a registered subgraph (Inv-13)"
 */
export class EInvImmutability extends BentenError {
  static readonly code = "E_INV_IMMUTABILITY";
  static readonly fixHint = "Phase-2a invariant 13 — once a Node/subgraph is persisted under a CID, its bytes are immutable from user-path writes. The firing matrix has five rows (plan §9.11):";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_IMMUTABILITY", "Phase-2a invariant 13 — once a Node/subgraph is persisted under a CID, its bytes are immutable from user-path writes. The firing matrix has five rows (plan §9.11):", message, context);
    this.name = "EInvImmutability";
  }
}

/**
 * E_INV_ATTRIBUTION
 *
 * Thrown at: registration + runtime trace emission (G5-B)
 * Message template: "Missing or malformed attribution frame (Inv-14)"
 */
export class EInvAttribution extends BentenError {
  static readonly code = "E_INV_ATTRIBUTION";
  static readonly fixHint = "Phase-2a invariant 14: every TraceStep MUST carry an `AttributionFrame` naming the actor, handler, and capability-grant CIDs. A primitive-type that refuses to declare its attribution source fails at registration. File a bug against the primitive's `attribution_for_step` impl.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_ATTRIBUTION", "Phase-2a invariant 14: every TraceStep MUST carry an `AttributionFrame` naming the actor, handler, and capability-grant CIDs. A primitive-type that refuses to declare its attribution source fails at registration. File a bug against the primitive's `attribution_for_step` impl.", message, context);
    this.name = "EInvAttribution";
  }
}

/**
 * E_CAP_WALLCLOCK_EXPIRED
 *
 * Thrown at: evaluator (G9-A, §9.13 refresh point #5). `CapError::WallclockExpired` is the upstream alias; the firing site is reserved at G9-A refresh-point-5 and is not yet wired in production code (drift-detector reachability is `ignore` until then).
 * Message template: "Capability wall-clock refresh bound breached"
 */
export class ECapWallclockExpired extends BentenError {
  static readonly code = "E_CAP_WALLCLOCK_EXPIRED";
  static readonly fixHint = "A long-running ITERATE crossed the 300s default wall-clock refresh boundary; the grant was revoked between the previous refresh and the boundary. Re-grant the capability and retry. Tighten handler shapes to stay under the refresh bound if latency matters.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_WALLCLOCK_EXPIRED", "A long-running ITERATE crossed the 300s default wall-clock refresh boundary; the grant was revoked between the previous refresh and the boundary. Re-grant the capability and retry. Tighten handler shapes to stay under the refresh bound if latency matters.", message, context);
    this.name = "ECapWallclockExpired";
  }
}

/**
 * E_CAP_CHAIN_TOO_DEEP
 *
 * Thrown at: capability policy attenuation walker (G9-A)
 * Message template: "Capability attenuation chain exceeds max_chain_depth"
 */
export class ECapChainTooDeep extends BentenError {
  static readonly code = "E_CAP_CHAIN_TOO_DEEP";
  static readonly fixHint = "A delegation chain was deeper than the configured `GrantReader::max_chain_depth` (default 64). Either shorten the chain or raise the configured cap through the engine builder. Ucca-6 guard against malicious delegator attacks.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_CHAIN_TOO_DEEP", "A delegation chain was deeper than the configured `GrantReader::max_chain_depth` (default 64). Either shorten the chain or raise the configured cap through the engine builder. Ucca-6 guard against malicious delegator attacks.", message, context);
    this.name = "ECapChainTooDeep";
  }
}

/**
 * E_CAP_SCOPE_LONE_STAR_REJECTED
 *
 * Thrown at: `GrantScope::parse` (G4-A)
 * Message template: "GrantScope::parse('*') rejected — lone star is a footgun"
 */
export class ECapScopeLoneStarRejected extends BentenError {
  static readonly code = "E_CAP_SCOPE_LONE_STAR_REJECTED";
  static readonly fixHint = "Lone `*` is refused because it collapses to a root-scope wildcard that cannot be meaningfully attenuated. Use a compound form (`*:<namespace>`) or name an explicit scope. Ucca-7 / G4-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_SCOPE_LONE_STAR_REJECTED", "Lone `*` is refused because it collapses to a root-scope wildcard that cannot be meaningfully attenuated. Use a compound form (`*:<namespace>`) or name an explicit scope. Ucca-7 / G4-A.", message, context);
    this.name = "ECapScopeLoneStarRejected";
  }
}

/**
 * E_VIEW_STRATEGY_A_REFUSED
 *
 * Thrown at: `Engine::create_view` registration (G8-B)
 * Message template: "user view '{view_id}' declared Strategy::A — Strategy A is reserved for the 5 hand-written Phase-1 IVM views (Rust-only); user views must use Strategy::B"
 */
export class EViewStrategyARefused extends BentenError {
  static readonly code = "E_VIEW_STRATEGY_A_REFUSED";
  static readonly fixHint = "D8-RESOLVED (Phase 2b). Strategy A is the hand-written-IVM lane reserved for the five Phase-1 baseline views (capability-grants, event-dispatch, content-listing, governance-inheritance, version-current). User-registered views go through generalized Algorithm B; either omit the `strategy` field (defaults to `B`) or pass `Strategy::B` explicitly.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VIEW_STRATEGY_A_REFUSED", "D8-RESOLVED (Phase 2b). Strategy A is the hand-written-IVM lane reserved for the five Phase-1 baseline views (capability-grants, event-dispatch, content-listing, governance-inheritance, version-current). User-registered views go through generalized Algorithm B; either omit the `strategy` field (defaults to `B`) or pass `Strategy::B` explicitly.", message, context);
    this.name = "EViewStrategyARefused";
  }
}

/**
 * E_VIEW_STRATEGY_C_RESERVED
 *
 * Thrown at: `Engine::create_view` registration (G8-B)
 * Message template: "user view '{view_id}' declared Strategy::C — Strategy C (Z-set / DBSP cancellation) is reserved for Phase 3+"
 */
export class EViewStrategyCReserved extends BentenError {
  static readonly code = "E_VIEW_STRATEGY_C_RESERVED";
  static readonly fixHint = "D8-RESOLVED (Phase 2b). Strategy C is the Z-set / DBSP cancellation algorithm slot reserved for Phase 3+; refused at registration time in Phase 2b. Use `Strategy::B` (or omit the field; user views default to B).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VIEW_STRATEGY_C_RESERVED", "D8-RESOLVED (Phase 2b). Strategy C is the Z-set / DBSP cancellation algorithm slot reserved for Phase 3+; refused at registration time in Phase 2b. Use `Strategy::B` (or omit the field; user views default to B).", message, context);
    this.name = "EViewStrategyCReserved";
  }
}

/**
 * E_VIEW_LABEL_MISMATCH
 *
 * Thrown at: `Engine::register_user_view` registration (R6-R3 fix-pass; mirrored at the TS-DSL pre-napi-boundary in `packages/engine/src/views.ts::validateUserViewSpec`).
 * Message template: "user view '{view_id}' is reserved for the canonical IVM view with the hardcoded label '{expected_label}'; cannot register with a different label '{got_label}'"
 */
export class EViewLabelMismatch extends BentenError {
  static readonly code = "E_VIEW_LABEL_MISMATCH";
  static readonly fixHint = "Phase-2b R6-R3 (r6-r3-ivm-1). Four canonical Phase-1 IVM view ids (`capability_grants`, `version_current`, `event_dispatch`, `governance_inheritance`) have hardcoded `input_pattern_label` semantics in the hand-written `AlgorithmBView::for_id` dispatch arms — re-using one of those ids with a different label silently registers a view that filters on the wrong label. Either pick a different `spec.id` (the user-defined fallback honors any label) OR change `spec.inputPattern.label` to match the hardcoded value listed in the message body.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VIEW_LABEL_MISMATCH", "Phase-2b R6-R3 (r6-r3-ivm-1). Four canonical Phase-1 IVM view ids (`capability_grants`, `version_current`, `event_dispatch`, `governance_inheritance`) have hardcoded `input_pattern_label` semantics in the hand-written `AlgorithmBView::for_id` dispatch arms — re-using one of those ids with a different label silently registers a view that filters on the wrong label. Either pick a different `spec.id` (the user-defined fallback honors any label) OR change `spec.inputPattern.label` to match the hardcoded value listed in the message body.", message, context);
    this.name = "EViewLabelMismatch";
  }
}

/**
 * E_WAIT_SIGNAL_SHAPE_MISMATCH
 *
 * Thrown at: WAIT executor resume path (G3-B DX signal-payload typing). The integration test at `crates/benten-engine/tests/integration/wait_signal_shape_optional_typing.rs` exercises the surface; the production firing site is reserved alongside the broader G3-B DX typing landing (drift-detector reachability is `ignore` until then).
 * Message template: "WAIT signal payload does not match declared signal_shape"
 */
export class EWaitSignalShapeMismatch extends BentenError {
  static readonly code = "E_WAIT_SIGNAL_SHAPE_MISMATCH";
  static readonly fixHint = "When a WAIT declares `signal_shape: Some(schema)`, a resume with a payload that fails schema validation is rejected BEFORE any downstream TRANSFORM runs. Either widen the schema, re-send with the correct shape, or drop the `signal_shape` to keep the untyped path.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WAIT_SIGNAL_SHAPE_MISMATCH", "When a WAIT declares `signal_shape: Some(schema)`, a resume with a payload that fails schema validation is rejected BEFORE any downstream TRANSFORM runs. Either widen the schema, re-send with the correct shape, or drop the `signal_shape` to keep the untyped path.", message, context);
    this.name = "EWaitSignalShapeMismatch";
  }
}

/**
 * E_WAIT_SUSPENDED
 *
 * Thrown at: `benten_eval::primitives::dispatch` (WAIT arm), surfaced as `EvalError::WaitSuspended`; round-trips through `eval_error_to_engine_error` to `EngineError::WaitSuspended { handle }` at the engine boundary.
 * Message template: "WAIT primitive suspended awaiting external signal/duration"
 */
export class EWaitSuspended extends BentenError {
  static readonly code = "E_WAIT_SUSPENDED";
  static readonly fixHint = "A regular `engine.call(handler, ...)` walk hit a WAIT primitive and the dispatcher routed through the eval-side `wait::evaluate`, producing a `SuspendedHandle`. This is a control-flow signal, NOT a runtime failure — the caller catches the typed error, inspects the carried `SuspendedHandle`, and either calls `Engine::call_with_suspension` (which surfaces the same boundary as `SuspensionOutcome::Suspended`) or persists the handle bytes via `Engine::suspend_to_bytes` for later resume. Phase-2b Wave-8i (option B closure of the WAIT regular-walk dispatcher gap surfaced by the docs-vs-code audit).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WAIT_SUSPENDED", "A regular `engine.call(handler, ...)` walk hit a WAIT primitive and the dispatcher routed through the eval-side `wait::evaluate`, producing a `SuspendedHandle`. This is a control-flow signal, NOT a runtime failure — the caller catches the typed error, inspects the carried `SuspendedHandle`, and either calls `Engine::call_with_suspension` (which surfaces the same boundary as `SuspensionOutcome::Suspended`) or persists the handle bytes via `Engine::suspend_to_bytes` for later resume. Phase-2b Wave-8i (option B closure of the WAIT regular-walk dispatcher gap surfaced by the docs-vs-code audit).", message, context);
    this.name = "EWaitSuspended";
  }
}

/**
 * E_STREAM_BACKPRESSURE_DROPPED
 *
 * Thrown at: `benten_eval::chunk_sink::BoundedSink::try_send` (lossy variant); evaluator emits a `TraceStep::BudgetExhausted { budget_type: "stream_backpressure" }` row BEFORE propagating the typed error per the D1 trace-preservation pattern.
 * Message template: "STREAM lossy mode dropped a chunk on a saturated buffer"
 */
export class EStreamBackpressureDropped extends BentenError {
  static readonly code = "E_STREAM_BACKPRESSURE_DROPPED";
  static readonly fixHint = "STREAM was created with lossy semantics (`try_send` on a full buffer drops rather than awaits). The drop fires loudly via the trace surface — never silent. Either switch to lossless `send`, increase the sink capacity, or pace the producer. D4-RESOLVED. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_STREAM_BACKPRESSURE_DROPPED", "STREAM was created with lossy semantics (`try_send` on a full buffer drops rather than awaits). The drop fires loudly via the trace surface — never silent. Either switch to lossless `send`, increase the sink capacity, or pace the producer. D4-RESOLVED. Phase-2b G6-A.", message, context);
    this.name = "EStreamBackpressureDropped";
  }
}

/**
 * E_STREAM_CLOSED_BY_PEER
 *
 * Thrown at: `benten_eval::chunk_sink::BoundedSink::send` / `try_send`.
 * Message template: "STREAM consumer disconnected; producer cannot deliver chunk"
 */
export class EStreamClosedByPeer extends BentenError {
  static readonly code = "E_STREAM_CLOSED_BY_PEER";
  static readonly fixHint = "The downstream `ChunkSource` was dropped (consumer detached, transport closed) before the producer's next send arrived. Resume the consumer, or terminate the producer. D4-RESOLVED. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_STREAM_CLOSED_BY_PEER", "The downstream `ChunkSource` was dropped (consumer detached, transport closed) before the producer's next send arrived. Resume the consumer, or terminate the producer. D4-RESOLVED. Phase-2b G6-A.", message, context);
    this.name = "EStreamClosedByPeer";
  }
}

/**
 * E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED
 *
 * Thrown at: `benten_eval::chunk_sink::BoundedSink::send` (wallclock-budgeted variant).
 * Message template: "STREAM producer wallclock budget elapsed while awaiting available capacity"
 */
export class EStreamProducerWallclockExceeded extends BentenError {
  static readonly code = "E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED";
  static readonly fixHint = "A lossless STREAM producer was created with a wallclock budget (`make_chunk_sink_with_wallclock`) and the budget elapsed while a slow consumer kept the buffer full. Either widen the budget, increase capacity, accelerate the consumer, or accept lossy mode. Kills permanently-stalled sends per streaming-systems implementation hint. D4-RESOLVED. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED", "A lossless STREAM producer was created with a wallclock budget (`make_chunk_sink_with_wallclock`) and the budget elapsed while a slow consumer kept the buffer full. Either widen the budget, increase capacity, accelerate the consumer, or accept lossy mode. Kills permanently-stalled sends per streaming-systems implementation hint. D4-RESOLVED. Phase-2b G6-A.", message, context);
    this.name = "EStreamProducerWallclockExceeded";
  }
}

/**
 * E_INV_STREAM_CONFIG
 *
 * Thrown at: `crates/benten-engine/src/engine_stream.rs::build_stream_handle` (resolves per-handler properties from the registered `SubgraphSpec` + validates against `ChunkProducerConfig::default`).
 * Message template: "STREAM per-handler config widens workspace grant ceiling"
 */
export class EInvStreamConfig extends BentenError {
  static readonly code = "E_INV_STREAM_CONFIG";
  static readonly fixHint = "Per-handler STREAM `chunkCountCap` / `wallclockBudgetMs` properties NARROW but cannot WIDEN the workspace defaults. Drop the per-handler override or align it below the workspace ceiling. Per stream-r1-9: extension-vs-replace policy is \"narrow only\" — widen attempts at registration / call time are rejected to defend against the over-permissive-escape failure mode. Phase-3 G19-C2.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_STREAM_CONFIG", "Per-handler STREAM `chunkCountCap` / `wallclockBudgetMs` properties NARROW but cannot WIDEN the workspace defaults. Drop the per-handler override or align it below the workspace ceiling. Per stream-r1-9: extension-vs-replace policy is \"narrow only\" — widen attempts at registration / call time are rejected to defend against the over-permissive-escape failure mode. Phase-3 G19-C2.", message, context);
    this.name = "EInvStreamConfig";
  }
}

/**
 * E_STREAM_HANDLE_LEAKED
 *
 * Thrown at: `packages/engine/src/stream.ts::ensureLeakRegistry` (FinalizationRegistry callback) + `packages/engine/src/stream.ts::fireStreamLeak` (broadcast helper used by both the FinalizationRegistry callback path and the `Engine.shutdown()` drain path on `packages/engine/src/engine.ts::Engine`); never thrown across the napi boundary.
 * Message template: "STREAM handle dropped without explicit close()"
 */
export class EStreamHandleLeaked extends BentenError {
  static readonly code = "E_STREAM_HANDLE_LEAKED";
  static readonly fixHint = "A `StreamHandle` returned by `engine.openStream(...)` was garbage-collected (or the engine was shut down) without an explicit `close()` / `cancel()` call. Native-side ownership is correct (Rust `Drop` joins the producer thread); this surface fires JS-side leak detection so operators can spot leaking call sites. Either consume the handle to natural completion (which auto-closes via the natural-final-chunk path), call `close()` explicitly, or use `engine.callStream(...)` which wraps for-await auto-close. Per stream-r1-4: 4 enumerated leak scenarios (handler-returns-no-close, handler-throws-no-close, natural-completion-no-fire-negative, engine-shutdown-while-open) plus a sub-mechanism GC-pressure-timeout polling fallback. Native-Node-only — V8 + WHATWG GC schedule per stream-r1-10. Phase-3 G19-C2.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_STREAM_HANDLE_LEAKED", "A `StreamHandle` returned by `engine.openStream(...)` was garbage-collected (or the engine was shut down) without an explicit `close()` / `cancel()` call. Native-side ownership is correct (Rust `Drop` joins the producer thread); this surface fires JS-side leak detection so operators can spot leaking call sites. Either consume the handle to natural completion (which auto-closes via the natural-final-chunk path), call `close()` explicitly, or use `engine.callStream(...)` which wraps for-await auto-close. Per stream-r1-4: 4 enumerated leak scenarios (handler-returns-no-close, handler-throws-no-close, natural-completion-no-fire-negative, engine-shutdown-while-open) plus a sub-mechanism GC-pressure-timeout polling fallback. Native-Node-only — V8 + WHATWG GC schedule per stream-r1-10. Phase-3 G19-C2.", message, context);
    this.name = "EStreamHandleLeaked";
  }
}

/**
 * E_SUBSCRIBE_DELIVERY_FAILED
 *
 * Thrown at: `benten_eval::primitives::subscribe::ActiveSubscription::inject` (delivery-time cap re-check).
 * Message template: "SUBSCRIBE delivery failed (capability re-check denied at delivery)"
 */
export class ESubscribeDeliveryFailed extends BentenError {
  static readonly code = "E_SUBSCRIBE_DELIVERY_FAILED";
  static readonly fixHint = "D5-RESOLVED requires capability re-intersection at every delivery boundary. A previously-granted READ cap was revoked mid-stream; the subscription auto-cancels. Re-grant the cap and re-register the subscription. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SUBSCRIBE_DELIVERY_FAILED", "D5-RESOLVED requires capability re-intersection at every delivery boundary. A previously-granted READ cap was revoked mid-stream; the subscription auto-cancels. Re-grant the cap and re-register the subscription. Phase-2b G6-A.", message, context);
    this.name = "ESubscribeDeliveryFailed";
  }
}

/**
 * E_SUBSCRIBE_PATTERN_INVALID
 *
 * Thrown at: `benten_eval::primitives::subscribe::ChangePattern::validate` (registration entry).
 * Message template: "SUBSCRIBE pattern is malformed (empty pattern, unclosed glob bracket, etc.)"
 */
export class ESubscribePatternInvalid extends BentenError {
  static readonly code = "E_SUBSCRIBE_PATTERN_INVALID";
  static readonly fixHint = "Pattern shape failed validation at registration. Fix the glob (balance `[` / `]`), provide a non-empty pattern, or switch from `LabelGlob` to `AnchorPrefix`. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SUBSCRIBE_PATTERN_INVALID", "Pattern shape failed validation at registration. Fix the glob (balance `[` / `]`), provide a non-empty pattern, or switch from `LabelGlob` to `AnchorPrefix`. Phase-2b G6-A.", message, context);
    this.name = "ESubscribePatternInvalid";
  }
}

/**
 * E_SUBSCRIBE_CURSOR_LOST
 *
 * Thrown at: `benten_eval::primitives::subscribe::ActiveSubscription::inject` (mid-stream retention check).
 * Message template: "SUBSCRIBE cursor lost (retention window exhausted mid-stream)"
 */
export class ESubscribeCursorLost extends BentenError {
  static readonly code = "E_SUBSCRIBE_CURSOR_LOST";
  static readonly fixHint = "D5 strengthening item 4 caps persistent-cursor retention at 1000 events OR 24h, whichever first. Beyond the bound, the subscription auto-cancels and the subscriber must restart from `Latest`. Adjust event-emission rate, drain promptly, or accept the bounded-replay contract. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SUBSCRIBE_CURSOR_LOST", "D5 strengthening item 4 caps persistent-cursor retention at 1000 events OR 24h, whichever first. Beyond the bound, the subscription auto-cancels and the subscriber must restart from `Latest`. Adjust event-emission rate, drain promptly, or accept the bounded-replay contract. Phase-2b G6-A.", message, context);
    this.name = "ESubscribeCursorLost";
  }
}

/**
 * E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED
 *
 * Thrown at: `benten_eval::primitives::subscribe::register_inner` (`Persistent` cursor re-registration).
 * Message template: "SUBSCRIBE persistent cursor restart attempted past the retention window"
 */
export class ESubscribeReplayWindowExceeded extends BentenError {
  static readonly code = "E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED";
  static readonly fixHint = "Equivalent surface to `E_SUBSCRIBE_CURSOR_LOST` raised at re-registration time rather than mid-stream. The persisted `max_delivered_seq` falls outside the retained event window; re-register with `start_from: Latest` to resume from the next published event. streaming-systems stream-d5-1. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED", "Equivalent surface to `E_SUBSCRIBE_CURSOR_LOST` raised at re-registration time rather than mid-stream. The persisted `max_delivered_seq` falls outside the retained event window; re-register with `start_from: Latest` to resume from the next published event. streaming-systems stream-d5-1. Phase-2b G6-A.", message, context);
    this.name = "ESubscribeReplayWindowExceeded";
  }
}

/**
 * E_INV_11_SYSTEM_ZONE_READ
 *
 * Thrown at: `benten_eval::primitives::subscribe::ChangePattern::validate` (registration entry).
 * Message template: "SUBSCRIBE pattern names a `system:*` zone (Inv-11)"
 */
export class EInv11SystemZoneRead extends BentenError {
  static readonly code = "E_INV_11_SYSTEM_ZONE_READ";
  static readonly fixHint = "User code attempted to subscribe to a `system:*` system-zone label. Distinct catalog code so SUBSCRIBE-side breaches are diagnostically separable from WRITE-side breaches (`E_INV_SYSTEM_ZONE` covers writes). Subscribe to a non-system pattern, or, for engine-internal observation, use a privileged path. Phase-2b G6-A.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_11_SYSTEM_ZONE_READ", "User code attempted to subscribe to a `system:*` system-zone label. Distinct catalog code so SUBSCRIBE-side breaches are diagnostically separable from WRITE-side breaches (`E_INV_SYSTEM_ZONE` covers writes). Subscribe to a non-system pattern, or, for engine-internal observation, use a privileged path. Phase-2b G6-A.", message, context);
    this.name = "EInv11SystemZoneRead";
  }
}

/**
 * E_SANDBOX_FUEL_EXHAUSTED
 *
 * Thrown at: SANDBOX executor — fully active post-wave-8b. The wasmtime `Store::set_fuel` cap + trap-callback maps fuel-exhaustion traps via `crates/benten-eval/src/sandbox/trap_to_typed.rs` to this typed variant. D3-RESOLVED per-call wasmtime `Store` lifecycle.
 * Message template: "SANDBOX fuel exhausted: limit={limit} consumed={consumed}"
 */
export class ESandboxFuelExhausted extends BentenError {
  static readonly code = "E_SANDBOX_FUEL_EXHAUSTED";
  static readonly fixHint = "wasmtime fuel-meter intercept. Either reduce the per-call computation, raise `SandboxConfig::fuel` (default 1_000_000), or split the workload across multiple SANDBOX calls. Concurrent with the typed-error propagation, the engine emits `TraceStep::BudgetExhausted { budget_type: \"sandbox_fuel\", consumed, limit, path }` so `engine.trace(...)` consumers observe the exhaustion in-band (mirrors G12-A's `inv_8_iteration` pattern).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_FUEL_EXHAUSTED", "wasmtime fuel-meter intercept. Either reduce the per-call computation, raise `SandboxConfig::fuel` (default 1_000_000), or split the workload across multiple SANDBOX calls. Concurrent with the typed-error propagation, the engine emits `TraceStep::BudgetExhausted { budget_type: \"sandbox_fuel\", consumed, limit, path }` so `engine.trace(...)` consumers observe the exhaustion in-band (mirrors G12-A's `inv_8_iteration` pattern).", message, context);
    this.name = "ESandboxFuelExhausted";
  }
}

/**
 * E_SANDBOX_MEMORY_EXHAUSTED
 *
 * Thrown at: SANDBOX executor — fully active post-wave-8b via `ResourceLimiter` impl + memory-trap → typed-error mapping.
 * Message template: "SANDBOX memory limit exhausted: {limit} bytes"
 */
export class ESandboxMemoryExhausted extends BentenError {
  static readonly code = "E_SANDBOX_MEMORY_EXHAUSTED";
  static readonly fixHint = "wasmtime `ResourceLimiter` intercept fires deterministically BEFORE host OOM (`crates/benten-eval/src/sandbox/resource_limiter.rs`). Either reduce module memory pressure, raise `SandboxConfig::memory_bytes` (default 64 MiB), or audit for runaway `memory.grow` (ESC-2 escape vector).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MEMORY_EXHAUSTED", "wasmtime `ResourceLimiter` intercept fires deterministically BEFORE host OOM (`crates/benten-eval/src/sandbox/resource_limiter.rs`). Either reduce module memory pressure, raise `SandboxConfig::memory_bytes` (default 64 MiB), or audit for runaway `memory.grow` (ESC-2 escape vector).", message, context);
    this.name = "ESandboxMemoryExhausted";
  }
}

/**
 * E_SANDBOX_WALLCLOCK_EXCEEDED
 *
 * Thrown at: SANDBOX executor — fully active post-wave-8b via `wasmtime::Store::set_epoch_deadline` + the wave-8b epoch-interruption ticker thread (`crates/benten-eval/src/sandbox/epoch_ticker.rs`) that ticks the shared engine's epoch on a configured cadence; D27 `async-support` ENABLED preserves the yield path for Phase-3 iroh forward-compat.
 * Message template: "SANDBOX wallclock deadline exceeded: {limit_ms} ms"
 */
export class ESandboxWallclockExceeded extends BentenError {
  static readonly code = "E_SANDBOX_WALLCLOCK_EXCEEDED";
  static readonly fixHint = "D24-RESOLVED defaults: 30s default / 5min ceiling. Per-handler `wallclock_ms` opt-in via `SubgraphSpec.primitives` (G12-D widening). Workspace-level overrides via `engine.toml` `[sandbox]` section (Ben's brief addition). Either shrink the workload, raise the per-handler value (within the engine.toml ceiling), or relax the engine.toml ceiling.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_WALLCLOCK_EXCEEDED", "D24-RESOLVED defaults: 30s default / 5min ceiling. Per-handler `wallclock_ms` opt-in via `SubgraphSpec.primitives` (G12-D widening). Workspace-level overrides via `engine.toml` `[sandbox]` section (Ben's brief addition). Either shrink the workload, raise the per-handler value (within the engine.toml ceiling), or relax the engine.toml ceiling.", message, context);
    this.name = "ESandboxWallclockExceeded";
  }
}

/**
 * E_SANDBOX_WALLCLOCK_INVALID
 *
 * Thrown at: SubgraphSpec validation / `SandboxConfig::with_wallclock_ms`.
 * Message template: "SANDBOX wallclock setting outside allowed range"
 */
export class ESandboxWallclockInvalid extends BentenError {
  static readonly code = "E_SANDBOX_WALLCLOCK_INVALID";
  static readonly fixHint = "Per-handler `wallclock_ms` must be > 0 and ≤ engine.toml `wallclock_max_ms` (defaults to D24-RESOLVED 5min ceiling). Reduce the per-handler value or relax `wallclock_max_ms` in `engine.toml`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_WALLCLOCK_INVALID", "Per-handler `wallclock_ms` must be > 0 and ≤ engine.toml `wallclock_max_ms` (defaults to D24-RESOLVED 5min ceiling). Reduce the per-handler value or relax `wallclock_max_ms` in `engine.toml`.", message, context);
    this.name = "ESandboxWallclockInvalid";
  }
}

/**
 * E_SANDBOX_HOST_FN_DENIED
 *
 * Thrown at: SANDBOX executor (init-time intersection; per-invocation re-check per D18 cadence).
 * Message template: "SANDBOX host-fn capability denied: {cap}"
 */
export class ESandboxHostFnDenied extends BentenError {
  static readonly code = "E_SANDBOX_HOST_FN_DENIED";
  static readonly fixHint = "Two firing paths: (1) D7 init-snapshot intersection — manifest claims a cap the dispatching grant lacks; fail before module link. (2) D18 per_call live recheck — cap revoked mid-call; subsequent host-fn invocation denied. Surfaces as a typed error THROUGH the host-fn ABI (NOT a wasmtime trap per sec-r1 D7) so the engine accounting stays clean. Either grant the missing cap, change the manifest, or relax the host-fn's `cap_recheck` declaration.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_HOST_FN_DENIED", "Two firing paths: (1) D7 init-snapshot intersection — manifest claims a cap the dispatching grant lacks; fail before module link. (2) D18 per_call live recheck — cap revoked mid-call; subsequent host-fn invocation denied. Surfaces as a typed error THROUGH the host-fn ABI (NOT a wasmtime trap per sec-r1 D7) so the engine accounting stays clean. Either grant the missing cap, change the manifest, or relax the host-fn's `cap_recheck` declaration.", message, context);
    this.name = "ESandboxHostFnDenied";
  }
}

/**
 * E_SANDBOX_HOST_FN_NOT_FOUND
 *
 * Thrown at: SANDBOX executor — wasmtime link-time resolver (other names than the 4 codegen-default).
 * Message template: "SANDBOX host-fn not found: {name}"
 */
export class ESandboxHostFnNotFound extends BentenError {
  static readonly code = "E_SANDBOX_HOST_FN_NOT_FOUND";
  static readonly fixHint = "Module attempted to call a host-fn name not in the active manifest. Phase-3 G17-A2 retired the Phase-2b `random`-host-fn deferral guard (CLAUDE.md baked-in #16 closure); `random` is now LIVE alongside `time` / `log` / `kv:read` (cap-string `host:random:read`). For names that fire this code post-G17-A2: check the manifest declaration matches the import + the codegen-default surface (4 host-fns at G17-A2). The wasmtime link-time resolver path fires when wasmtime fails to resolve an import against the linker.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_HOST_FN_NOT_FOUND", "Module attempted to call a host-fn name not in the active manifest. Phase-3 G17-A2 retired the Phase-2b `random`-host-fn deferral guard (CLAUDE.md baked-in #16 closure); `random` is now LIVE alongside `time` / `log` / `kv:read` (cap-string `host:random:read`). For names that fire this code post-G17-A2: check the manifest declaration matches the import + the codegen-default surface (4 host-fns at G17-A2). The wasmtime link-time resolver path fires when wasmtime fails to resolve an import against the linker.", message, context);
    this.name = "ESandboxHostFnNotFound";
  }
}

/**
 * E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED
 *
 * Thrown at: `register_default_host_fns` "random" trampoline at `crates/benten-eval/src/primitives/sandbox.rs::register_default_host_fns`. The `HostFnDenialMarker` carrier identifies the denial via the `random:per_call_budget_exceeded (requested=<n>, budget=<n>)` cap-string.
 * Message template: "SANDBOX random host-fn per-call entropy budget exceeded: requested={n} budget={n}"
 */
export class ESandboxHostFnRandomBudgetExceeded extends BentenError {
  static readonly code = "E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED";
  static readonly fixHint = "Phase-3 G17-A2 (CLAUDE.md baked-in #16 closure). A single `host.random(ptr, len)` call requested more entropy bytes than the per-call budget allows. The codegen default is **4096 bytes per call** (per r1-wsa-8). To draw more entropy, either (a) split the request across multiple sub-budget calls, or (b) override the default per-manifest via the additive optional `host_fns.random.budget_bytes_per_call` field on `ModuleManifest`. The aggregate-per-primitive cap is enforced separately at `CountedSink` (via `output_bytes`); the per-call budget is the additional ceiling on a single invocation. Routes through the `ON_DENIED` family (cap-denial precedent).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED", "Phase-3 G17-A2 (CLAUDE.md baked-in #16 closure). A single `host.random(ptr, len)` call requested more entropy bytes than the per-call budget allows. The codegen default is **4096 bytes per call** (per r1-wsa-8). To draw more entropy, either (a) split the request across multiple sub-budget calls, or (b) override the default per-manifest via the additive optional `host_fns.random.budget_bytes_per_call` field on `ModuleManifest`. The aggregate-per-primitive cap is enforced separately at `CountedSink` (via `output_bytes`); the per-call budget is the additional ceiling on a single invocation. Routes through the `ON_DENIED` family (cap-denial precedent).", message, context);
    this.name = "ESandboxHostFnRandomBudgetExceeded";
  }
}

/**
 * E_SANDBOX_MANIFEST_UNKNOWN
 *
 * Thrown at: - **Registration time (Phase-3 G17-C):** `Engine::register_subgraph::validate_sandbox_manifest_names` — walks SANDBOX nodes for unresolved manifest references via either the explicit `manifest` property or the colon-joined `<manifest>:<entry>` `module` property fallback. - **Dispatch time (legacy):** `ManifestRegistry::lookup` / `ManifestRef::resolve` — preserved for non-DSL spec construction paths that bypass the validation walk.
 * Message template: "SANDBOX manifest name '{manifest_name}' is not registered (codegen defaults: compute-basic, compute-with-kv; install via `engine.installModule(...)` or use a different name)"
 */
export class ESandboxManifestUnknown extends BentenError {
  static readonly code = "E_SANDBOX_MANIFEST_UNKNOWN";
  static readonly fixHint = "ESC-15 escape vector closure: NO permissive fall-through to a default manifest. Either install the manifest via `Engine::install_module` (paired with `Engine::register_module_bytes` for the underlying wasm payload) or use one of the codegen-default names (`compute-basic`, `compute-with-kv`). Phase-3 G17-C wave-5b adds the registration-time validation walk in `Engine::register_subgraph` so misspelled names + post-uninstall residual references trip THIS error at register time (operator-actionable: the wallclock-after-zero-progress masking is gone) instead of at dispatch time as a confusing wallclock trip.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MANIFEST_UNKNOWN", "ESC-15 escape vector closure: NO permissive fall-through to a default manifest. Either install the manifest via `Engine::install_module` (paired with `Engine::register_module_bytes` for the underlying wasm payload) or use one of the codegen-default names (`compute-basic`, `compute-with-kv`). Phase-3 G17-C wave-5b adds the registration-time validation walk in `Engine::register_subgraph` so misspelled names + post-uninstall residual references trip THIS error at register time (operator-actionable: the wallclock-after-zero-progress masking is gone) instead of at dispatch time as a confusing wallclock trip.", message, context);
    this.name = "ESandboxManifestUnknown";
  }
}

/**
 * E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED
 *
 * Thrown at: `ManifestRegistry::register_runtime`.
 * Message template: "Runtime manifest registration deferred to Phase 8"
 */
export class ESandboxManifestRegistrationDeferred extends BentenError {
  static readonly code = "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED";
  static readonly fixHint = "D2-RESOLVED hybrid: `ManifestRegistry::register_runtime(name, bundle)` exists in Phase 2b but returns this typed error (the API surface is reserved so Phase-8 marketplace work doesn't introduce a new public API — it just changes the body). Use a codegen-default manifest in 2b; revisit when Phase 8 ships.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED", "D2-RESOLVED hybrid: `ManifestRegistry::register_runtime(name, bundle)` exists in Phase 2b but returns this typed error (the API surface is reserved so Phase-8 marketplace work doesn't introduce a new public API — it just changes the body). Use a codegen-default manifest in 2b; revisit when Phase 8 ships.", message, context);
    this.name = "ESandboxManifestRegistrationDeferred";
  }
}

/**
 * E_SANDBOX_MODULE_INVALID
 *
 * Thrown at: SANDBOX executor (`Module::new` / link / instantiation).
 * Message template: "SANDBOX module invalid: {reason}"
 */
export class ESandboxModuleInvalid extends BentenError {
  static readonly code = "E_SANDBOX_MODULE_INVALID";
  static readonly fixHint = "Module bytes failed wasmtime structural validation (malformed module, type mismatch, OOB section, OOB linear-memory read, recursion-depth overflow, etc.). Audit the module compiler output. ESC-1 / ESC-3 / ESC-5 / ESC-11 / ESC-12 escape vectors all route here.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MODULE_INVALID", "Module bytes failed wasmtime structural validation (malformed module, type mismatch, OOB section, OOB linear-memory read, recursion-depth overflow, etc.). Audit the module compiler output. ESC-1 / ESC-3 / ESC-5 / ESC-11 / ESC-12 escape vectors all route here.", message, context);
    this.name = "ESandboxModuleInvalid";
  }
}

/**
 * E_SANDBOX_STACK_OVERFLOW
 *
 * Thrown at: SANDBOX executor — `wasmtime::Trap::StackOverflow` routes through `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` to the dedicated variant.
 * Message template: "SANDBOX stack overflow: guest exceeded max_wasm_stack ({max_wasm_stack} bytes)"
 */
export class ESandboxStackOverflow extends BentenError {
  static readonly code = "E_SANDBOX_STACK_OVERFLOW";
  static readonly fixHint = "SANDBOX guest module's call stack exceeded the configured `max_wasm_stack` ceiling (default 512 KiB; matches wasmtime's `Config::max_wasm_stack` default). Distinct from `E_SANDBOX_FUEL_EXHAUSTED` (CPU-bound runaway) and `E_SANDBOX_MODULE_INVALID` (structural validation failure) — stack-overflow-via-recursion is its own observable class so operator dashboards can distinguish a benign-but-buggy recursive guest from a generic invalid module. Either reduce module recursion depth, raise `SandboxConfig::max_wasm_stack`, or audit for adversarial recursion. Phase-3 G17-A1 wave-5b mints the dedicated typed variant per phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure (the prior R6FP-G1 r6-wsa-8 BELONGS-NAMED-NOW deferral is honored here).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_STACK_OVERFLOW", "SANDBOX guest module's call stack exceeded the configured `max_wasm_stack` ceiling (default 512 KiB; matches wasmtime's `Config::max_wasm_stack` default). Distinct from `E_SANDBOX_FUEL_EXHAUSTED` (CPU-bound runaway) and `E_SANDBOX_MODULE_INVALID` (structural validation failure) — stack-overflow-via-recursion is its own observable class so operator dashboards can distinguish a benign-but-buggy recursive guest from a generic invalid module. Either reduce module recursion depth, raise `SandboxConfig::max_wasm_stack`, or audit for adversarial recursion. Phase-3 G17-A1 wave-5b mints the dedicated typed variant per phase-3-backlog §6.4 + r1-wsa-7 BLOCKER closure (the prior R6FP-G1 r6-wsa-8 BELONGS-NAMED-NOW deferral is honored here).", message, context);
    this.name = "ESandboxStackOverflow";
  }
}

/**
 * E_SANDBOX_ESCAPE_ATTEMPT
 *
 * Thrown at: SANDBOX executor — `crates/benten-eval/src/sandbox/escape_defenses.rs::run_all_checks` (and per-vector `run_esc7_check` / `run_esc13_check` / `run_esc16_check`); routes through `crates/benten-eval/src/sandbox/trap_to_typed.rs::map_call_error` via the `EscapeAttemptMarker` cause-chain unwrap.
 * Message template: "SANDBOX escape attempt detected: {vector:?} — {reason}"
 */
export class ESandboxEscapeAttempt extends BentenError {
  static readonly code = "E_SANDBOX_ESCAPE_ATTEMPT";
  static readonly fixHint = "SANDBOX guest attempted one of the enumerated escape vectors. Phase-3 G17-A1 wave-5b ships defenses for **ESC-7** (fuel-refill via host-fn re-entry — guest calls a host-fn whose dispatch path attempts to re-enter the SANDBOX `Store` and `add_fuel` mid-execution; defense fires from the trampoline before the inner `add_fuel` takes effect), **ESC-13** (trap during fuel-meter callback / Store-poison — host-side fuel-meter callback panics or traps; defense maps via panic-catcher + per-call `Store` lifecycle ensures fresh Store on next call), and **ESC-16** (fingerprint-collapse via wallclock-correlated state read — guest reads a host-written wallclock-derived cell to fingerprint host nondeterminism; defense fires at the next host-fn boundary BEFORE the side-channel becomes guest-observable). The discriminating `EscVector` enum (declared in `crates/benten-eval/src/sandbox/escape_defenses.rs`) carries `Esc7FuelRefillViaReEntry` / `Esc13StorePoison` / `Esc16FingerprintCollapse` variants so audit pipelines can route per-vector. Closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13) + r1-wsa-4 (ESC-16) per phase-3-backlog §6.1 + D-E (R1 revision triage). Either harden the guest module (audit for the enumerated attack patterns) or — if the attack is in a research / test corpus — gate the corpus dispatch behind explicit testing-helper feature flags.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_ESCAPE_ATTEMPT", "SANDBOX guest attempted one of the enumerated escape vectors. Phase-3 G17-A1 wave-5b ships defenses for **ESC-7** (fuel-refill via host-fn re-entry — guest calls a host-fn whose dispatch path attempts to re-enter the SANDBOX `Store` and `add_fuel` mid-execution; defense fires from the trampoline before the inner `add_fuel` takes effect), **ESC-13** (trap during fuel-meter callback / Store-poison — host-side fuel-meter callback panics or traps; defense maps via panic-catcher + per-call `Store` lifecycle ensures fresh Store on next call), and **ESC-16** (fingerprint-collapse via wallclock-correlated state read — guest reads a host-written wallclock-derived cell to fingerprint host nondeterminism; defense fires at the next host-fn boundary BEFORE the side-channel becomes guest-observable). The discriminating `EscVector` enum (declared in `crates/benten-eval/src/sandbox/escape_defenses.rs`) carries `Esc7FuelRefillViaReEntry` / `Esc13StorePoison` / `Esc16FingerprintCollapse` variants so audit pipelines can route per-vector. Closes r1-wsa-1 BLOCKER (ESC-7 + ESC-13) + r1-wsa-4 (ESC-16) per phase-3-backlog §6.1 + D-E (R1 revision triage). Either harden the guest module (audit for the enumerated attack patterns) or — if the attack is in a research / test corpus — gate the corpus dispatch behind explicit testing-helper feature flags.", message, context);
    this.name = "ESandboxEscapeAttempt";
  }
}

/**
 * E_SANDBOX_MODULE_NOT_INSTALLED
 *
 * Thrown at: `impl PrimitiveHost for Engine::execute_sandbox` (`crates/benten-engine/src/primitive_host.rs`) when `Engine::module_bytes_for(cid)` returns `None`.
 * Message template: "SANDBOX module bytes not registered for CID {module_cid}"
 */
export class ESandboxModuleNotInstalled extends BentenError {
  static readonly code = "E_SANDBOX_MODULE_NOT_INSTALLED";
  static readonly fixHint = "A SANDBOX dispatch named a module CID for which no bytes have been registered through `Engine::register_module_bytes(cid, bytes)`. Distinct from `E_SANDBOX_MODULE_INVALID` (bytes are present but failed wasmtime structural validation): this fires BEFORE the executor sees any bytes, at the engine's lookup step. Either call `engine.register_module_bytes(module_cid, wasm_bytes)` before dispatch, or correct the SANDBOX node's `module` property to reference an already-registered CID. The Phase-2b in-memory module-bytes registry is process-local + transient (lost across `Engine` re-open); Phase 3 promotes the registry to a durable `BlobBackend` per Compromise #17. The `install_module(manifest, expected_cid)` path persists the manifest into a system-zone Node but does NOT persist the underlying wasm bytes — that asymmetry IS the Compromise #17 narrative.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MODULE_NOT_INSTALLED", "A SANDBOX dispatch named a module CID for which no bytes have been registered through `Engine::register_module_bytes(cid, bytes)`. Distinct from `E_SANDBOX_MODULE_INVALID` (bytes are present but failed wasmtime structural validation): this fires BEFORE the executor sees any bytes, at the engine's lookup step. Either call `engine.register_module_bytes(module_cid, wasm_bytes)` before dispatch, or correct the SANDBOX node's `module` property to reference an already-registered CID. The Phase-2b in-memory module-bytes registry is process-local + transient (lost across `Engine` re-open); Phase 3 promotes the registry to a durable `BlobBackend` per Compromise #17. The `install_module(manifest, expected_cid)` path persists the manifest into a system-zone Node but does NOT persist the underlying wasm bytes — that asymmetry IS the Compromise #17 narrative.", message, context);
    this.name = "ESandboxModuleNotInstalled";
  }
}

/**
 * E_SANDBOX_NESTED_DISPATCH_DENIED
 *
 * Thrown at: SANDBOX executor — fully active post-wave-8b. The host-fn callback path enforces the no-nested-`Engine::call` invariant via the trampoline's typed-error short-circuit before the host-side body runs.
 * Message template: "SANDBOX nested dispatch denied"
 */
export class ESandboxNestedDispatchDenied extends BentenError {
  static readonly code = "E_SANDBOX_NESTED_DISPATCH_DENIED";
  static readonly fixHint = "D19-RESOLVED: deny nested `Engine::call` from host-fn (the actual security claim). Closes the SANDBOX → CALL → SANDBOX cap-context-confusion attack class (sec-pre-r1-08). Renamed from the older `E_SANDBOX_REENTRANCY_DENIED` per wsa-7 + r1-security convergence — the name aligns with what's actually being denied. Refactor the host-fn to NOT re-enter the engine; if Phase-3 async host-fns are needed, acquire the reserved `host:async` cap.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_NESTED_DISPATCH_DENIED", "D19-RESOLVED: deny nested `Engine::call` from host-fn (the actual security claim). Closes the SANDBOX → CALL → SANDBOX cap-context-confusion attack class (sec-pre-r1-08). Renamed from the older `E_SANDBOX_REENTRANCY_DENIED` per wsa-7 + r1-security convergence — the name aligns with what's actually being denied. Refactor the host-fn to NOT re-enter the engine; if Phase-3 async host-fns are needed, acquire the reserved `host:async` cap.", message, context);
    this.name = "ESandboxNestedDispatchDenied";
  }
}

/**
 * E_MODULE_MANIFEST_CID_MISMATCH
 *
 * Thrown at: `Engine::install_module` (G10-B).
 * Message template: "Module manifest CID mismatch: expected={expected_cid} computed={computed_cid} summary={manifest_summary}"
 */
export class EModuleManifestCidMismatch extends BentenError {
  static readonly code = "E_MODULE_MANIFEST_CID_MISMATCH";
  static readonly fixHint = "D16-RESOLVED-FURTHER minimal CID-pin integrity gate. `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES the CID arg (not Optional — prevents the lazy `install_module(m, None)` footgun). The error includes both expected + computed CIDs + a 1-line manifest summary so an operator can diff without source-code dive. Either re-compute the expected CID against the actual manifest bytes or audit for tampering. Reserved here for the G10-B `install_module` surface; G7-C does NOT own this fire site (per wsa-r1-5 plan-internal conflict resolution).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_MODULE_MANIFEST_CID_MISMATCH", "D16-RESOLVED-FURTHER minimal CID-pin integrity gate. `Engine::install_module(manifest, expected_cid: Cid)` REQUIRES the CID arg (not Optional — prevents the lazy `install_module(m, None)` footgun). The error includes both expected + computed CIDs + a 1-line manifest summary so an operator can diff without source-code dive. Either re-compute the expected CID against the actual manifest bytes or audit for tampering. Reserved here for the G10-B `install_module` surface; G7-C does NOT own this fire site (per wsa-r1-5 plan-internal conflict resolution).", message, context);
    this.name = "EModuleManifestCidMismatch";
  }
}

/**
 * E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE
 *
 * Thrown at: `Engine::install_module` (G10-B) on `wasm32-unknown-unknown` only.
 * Message template: "module manifest declares N migration(s) but the target has no persistent backing store"
 */
export class EModuleMigrationsRequirePersistence extends BentenError {
  static readonly code = "E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE";
  static readonly fixHint = "`docs/SECURITY-POSTURE.md` Compromise #19 — browser (`wasm32-unknown-unknown`) engines ship in-memory-only manifest persistence in Phase 2b; the IndexedDB / OPFS persistence story lands in Phase 3. Manifests that declare `migrations` need a durable backing store; the rejection prevents the migration runner from silently dropping work. On native (redb-backed) targets the same manifest installs without error. Either (a) defer the migration to a Phase-3 build with persistent storage, or (b) split the manifest into a migrations-free in-memory variant for Phase-2b browser deployments.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE", "`docs/SECURITY-POSTURE.md` Compromise #19 — browser (`wasm32-unknown-unknown`) engines ship in-memory-only manifest persistence in Phase 2b; the IndexedDB / OPFS persistence story lands in Phase 3. Manifests that declare `migrations` need a durable backing store; the rejection prevents the migration runner from silently dropping work. On native (redb-backed) targets the same manifest installs without error. Either (a) defer the migration to a Phase-3 build with persistent storage, or (b) split the manifest into a migrations-free in-memory variant for Phase-2b browser deployments.", message, context);
    this.name = "EModuleMigrationsRequirePersistence";
  }
}

/**
 * E_ENGINE_CONFIG_INVALID
 *
 * Thrown at: `EngineConfig::load_or_default` (called at `Engine::open` time).
 * Message template: "engine.toml at {path} parse failure: {reason}"
 */
export class EEngineConfigInvalid extends BentenError {
  static readonly code = "E_ENGINE_CONFIG_INVALID";
  static readonly fixHint = "Workspace-level `engine.toml` (Ben's G7-A brief addition) failed to parse against the [`EngineConfig`] schema. Either fix the TOML (see `docs/SANDBOX-LIMITS.md` for the schema) or remove the file (built-in defaults apply when absent). The `[sandbox]` section accepts `wallclock_default_ms` (override D24 30s default) and `wallclock_max_ms` (override D24 5min ceiling).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_ENGINE_CONFIG_INVALID", "Workspace-level `engine.toml` (Ben's G7-A brief addition) failed to parse against the [`EngineConfig`] schema. Either fix the TOML (see `docs/SANDBOX-LIMITS.md` for the schema) or remove the file (built-in defaults apply when absent). The `[sandbox]` section accepts `wallclock_default_ms` (override D24 30s default) and `wallclock_max_ms` (override D24 5min ceiling).", message, context);
    this.name = "EEngineConfigInvalid";
  }
}

/**
 * E_BACKEND_READ_ONLY
 *
 * Thrown at: `SnapshotBlobBackend::{put,delete,put_batch}` (`crates/benten-graph/src/backends/snapshot_blob.rs`); `NetworkFetchStubBackend::{put,delete,put_batch}` (`crates/benten-graph/src/backends/network_fetch_stub.rs`); surfaces from `Engine::from_snapshot_blob`-constructed engines on any write call.
 * Message template: "backend is read-only: {operation} rejected ({backend_kind})"
 */
export class EBackendReadOnly extends BentenError {
  static readonly code = "E_BACKEND_READ_ONLY";
  static readonly fixHint = "D10-RESOLVED snapshot-blob `KVBackend` (constructed via `Engine::from_snapshot_blob(bytes)`) is a read-mostly view on a content-addressed handoff blob — Phase-3 sync can transmit the blob between peers, but the dst engine cannot write into it without breaking the canonical-bytes invariant the blob's CID is computed over. The same posture applies to the Phase-2a §9.8 `network_fetch_stub` `KVBackend`: writes landed in Phase 3 (G16 wave-6 iroh transport canary; full sync surface across G16-B/C/D wave-6b). To mutate state, open a redb-backed engine via `Engine::open(path)` instead, or import the snapshot blob into a fresh redb engine and reissue writes there.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_BACKEND_READ_ONLY", "D10-RESOLVED snapshot-blob `KVBackend` (constructed via `Engine::from_snapshot_blob(bytes)`) is a read-mostly view on a content-addressed handoff blob — Phase-3 sync can transmit the blob between peers, but the dst engine cannot write into it without breaking the canonical-bytes invariant the blob's CID is computed over. The same posture applies to the Phase-2a §9.8 `network_fetch_stub` `KVBackend`: writes landed in Phase 3 (G16 wave-6 iroh transport canary; full sync surface across G16-B/C/D wave-6b). To mutate state, open a redb-backed engine via `Engine::open(path)` instead, or import the snapshot blob into a fresh redb engine and reissue writes there.", message, context);
    this.name = "EBackendReadOnly";
  }
}

/**
 * E_SANDBOX_UNAVAILABLE_ON_WASM
 *
 * Thrown at: `crates/benten-engine/src/engine_sandbox.rs::execute_sandbox_wasm32_unavailable` (wasm32 cfg-gated stub) and the SANDBOX dispatcher path in `crates/benten-eval/src/primitives/mod.rs` when reached on a wasm32 target.
 * Message template: "SANDBOX is unavailable on the wasm32 build of the engine ({target})"
 */
export class ESandboxUnavailableOnWasm extends BentenError {
  static readonly code = "E_SANDBOX_UNAVAILABLE_ON_WASM";
  static readonly fixHint = "SANDBOX requires wasmtime, which does not compile to `wasm32-unknown-unknown` (browser target) and is not currently shipped on `wasm32-wasip1` engine builds either. The engine surfaces this typed error rather than `E_SUBSYSTEM_DISABLED` because the operator-actionable signal is target-specific: SANDBOX cannot run here, regardless of build flags. Phase-3 P2P sync re-routes SANDBOX invocations to a non-browser peer; until then, host SANDBOX-bearing handlers on a native `Engine::open(path)` engine and surface their results through SUBSCRIBE / STREAM to the wasm32-hosted client.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_UNAVAILABLE_ON_WASM", "SANDBOX requires wasmtime, which does not compile to `wasm32-unknown-unknown` (browser target) and is not currently shipped on `wasm32-wasip1` engine builds either. The engine surfaces this typed error rather than `E_SUBSYSTEM_DISABLED` because the operator-actionable signal is target-specific: SANDBOX cannot run here, regardless of build flags. Phase-3 P2P sync re-routes SANDBOX invocations to a non-browser peer; until then, host SANDBOX-bearing handlers on a native `Engine::open(path)` engine and surface their results through SUBSCRIBE / STREAM to the wasm32-hosted client.", message, context);
    this.name = "ESandboxUnavailableOnWasm";
  }
}

/**
 * E_RELOAD_SUBSCRIBER_UNSUBSCRIBED
 *
 * Thrown at: `bindings/napi/src/devserver.rs::ReloadSubscriberJs::{drain, has_events}` after `unsubscribe()` flips the inner `Mutex<Option<...>>` to `None`. R6 Round-2 r6-r2-napi-1 promoted this from a hand-typed `"E_RELOAD_SUBSCRIBER_UNSUBSCRIBED"` string to a typed catalog variant so JS callers get `EReloadSubscriberUnsubscribed` typed dispatch through `mapNativeError` rather than the synthetic `E_UNKNOWN` fallback.
 * Message template: "{operation} after unsubscribe"
 */
export class EReloadSubscriberUnsubscribed extends BentenError {
  static readonly code = "E_RELOAD_SUBSCRIBER_UNSUBSCRIBED";
  static readonly fixHint = "A `ReloadSubscriberJs` napi method (`drain` / `hasEvents`) was called after `unsubscribe()` released the underlying subscriber. The handle is single-shot; recreate the subscription via `devserver.subscribeReloadEvents()` if more events are expected.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_RELOAD_SUBSCRIBER_UNSUBSCRIBED", "A `ReloadSubscriberJs` napi method (`drain` / `hasEvents`) was called after `unsubscribe()` released the underlying subscriber. The handle is single-shot; recreate the subscription via `devserver.subscribeReloadEvents()` if more events are expected.", message, context);
    this.name = "EReloadSubscriberUnsubscribed";
  }
}

/**
 * E_DEVSERVER_STOPPED
 *
 * Thrown at: `bindings/napi/src/devserver.rs::devserver_stopped` (helper used by every devserver method that requires the dev-server to be running). R6 Round-2 r6-r2-napi-1 promoted this from a hand-typed `"E_DEVSERVER_STOPPED"` string to a typed catalog variant so JS callers get `EDevServerStopped` typed dispatch.
 * Message template: "dev-server has been stopped — call .start() before further operations"
 */
export class EDevServerStopped extends BentenError {
  static readonly code = "E_DEVSERVER_STOPPED";
  static readonly fixHint = "A devserver napi method was called after `DevServer.stop()` flipped the in-memory state to stopped. Restart the dev-server via `.start()` before invoking further operations, or construct a fresh `DevServer` instance.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_DEVSERVER_STOPPED", "A devserver napi method was called after `DevServer.stop()` flipped the in-memory state to stopped. Restart the dev-server via `.start()` before invoking further operations, or construct a fresh `DevServer` instance.", message, context);
    this.name = "EDevServerStopped";
  }
}

/**
 * E_STORAGE_QUOTA_EXCEEDED
 *
 * Thrown at: `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code` (Phase-3 G18-A wave-5a). Mapping is consumed by the IndexedDB-backed BlobBackend variant at `bindings/napi/src/browser_blob_store.rs` and the persistent module-manifest store at `bindings/napi/src/wasm_browser.rs`. Surface scope per CLAUDE.md baked-in #17: thin-client cache + manifest-store ONLY.
 * Message template: "IndexedDB write exceeded origin-storage quota"
 */
export class EStorageQuotaExceeded extends BentenError {
  static readonly code = "E_STORAGE_QUOTA_EXCEEDED";
  static readonly fixHint = "A browser thin-client cache write to IndexedDB exceeded the origin's storage allocation (the browser's per-origin quota). The browser surfaces `DOMException(name=\"QuotaExceededError\")` synchronously from the `IDBObjectStore.put` request's `onerror` handler; the napi binding maps this to the typed `E_STORAGE_QUOTA_EXCEEDED` variant via `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code`. Resolution is out-of-band: the user (or operator) frees origin-storage allocation by clearing site data, removing unused cached blobs, or migrating to a deployment with larger origin quota. Per CLAUDE.md baked-in #17 thin-client commitment, the browser tab's cache is non-authoritative — losing the cached bytes is recoverable: subsequent reads re-fetch from the connected full peer through the thin-client subscription protocol (D-PHASE-3-30).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_STORAGE_QUOTA_EXCEEDED", "A browser thin-client cache write to IndexedDB exceeded the origin's storage allocation (the browser's per-origin quota). The browser surfaces `DOMException(name=\"QuotaExceededError\")` synchronously from the `IDBObjectStore.put` request's `onerror` handler; the napi binding maps this to the typed `E_STORAGE_QUOTA_EXCEEDED` variant via `bindings/napi/src/browser_indexeddb.rs::map_dom_exception_to_error_code`. Resolution is out-of-band: the user (or operator) frees origin-storage allocation by clearing site data, removing unused cached blobs, or migrating to a deployment with larger origin quota. Per CLAUDE.md baked-in #17 thin-client commitment, the browser tab's cache is non-authoritative — losing the cached bytes is recoverable: subsequent reads re-fetch from the connected full peer through the thin-client subscription protocol (D-PHASE-3-30).", message, context);
    this.name = "EStorageQuotaExceeded";
  }
}

/**
 * E_HLC_SKEW_EXCEEDED
 *
 * Thrown at: `crates/benten-core/src/hlc.rs::Hlc::update` (Phase-3 G14-pre-D). Phase-3 sync wires the firing site into Loro per-property LWW + asymmetric-uptime MST-diff message ingest.
 * Message template: "HLC skew exceeded: remote physical_ms {remote_physical_ms} > local {local_physical_ms} + tolerance {tolerance_ms}ms"
 */
export class EHlcSkewExceeded extends BentenError {
  static readonly code = "E_HLC_SKEW_EXCEEDED";
  static readonly fixHint = "`Hlc::update(remote)` refused a remote stamp whose physical-clock component exceeds the local physical clock by more than the configured skew tolerance (default 5 minutes per `Hlc::DEFAULT_SKEW_TOLERANCE_MS`). The local HLC state is NOT mutated when this fires — Phase-3 sync rejects the offending message and continues. Inspect peer NTP / system-clock health; legitimate cross-region drift should fit comfortably inside 5 minutes. Operator-tunable knobs land alongside Phase-3 sync wiring.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HLC_SKEW_EXCEEDED", "`Hlc::update(remote)` refused a remote stamp whose physical-clock component exceeds the local physical clock by more than the configured skew tolerance (default 5 minutes per `Hlc::DEFAULT_SKEW_TOLERANCE_MS`). The local HLC state is NOT mutated when this fires — Phase-3 sync rejects the offending message and continues. Inspect peer NTP / system-clock health; legitimate cross-region drift should fit comfortably inside 5 minutes. Operator-tunable knobs land alongside Phase-3 sync wiring.", message, context);
    this.name = "EHlcSkewExceeded";
  }
}

/**
 * E_CAP_UCAN_EXPIRED
 *
 * Thrown at: `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B). Routes to `ON_DENIED`.
 * Message template: "UCAN expired (exp={exp}, now={now})"
 */
export class ECapUcanExpired extends BentenError {
  static readonly code = "E_CAP_UCAN_EXPIRED";
  static readonly fixHint = "Presented UCAN's `exp` window has elapsed at chain-walk time. Re-issue the UCAN with a fresh `exp`. Defends against the \"old proof sitting in disk forever, replayed by attacker who sniffed it pre-exp\" attack class per `crypto-blocker-2` BLOCKER + CLR-2.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_UCAN_EXPIRED", "Presented UCAN's `exp` window has elapsed at chain-walk time. Re-issue the UCAN with a fresh `exp`. Defends against the \"old proof sitting in disk forever, replayed by attacker who sniffed it pre-exp\" attack class per `crypto-blocker-2` BLOCKER + CLR-2.", message, context);
    this.name = "ECapUcanExpired";
  }
}

/**
 * E_CAP_UCAN_NOT_YET_VALID
 *
 * Thrown at: `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B).
 * Message template: "UCAN not yet valid (nbf={nbf}, now={now})"
 */
export class ECapUcanNotYetValid extends BentenError {
  static readonly code = "E_CAP_UCAN_NOT_YET_VALID";
  static readonly fixHint = "Presented UCAN's `nbf` window has not yet opened at chain-walk time. Wait until `now >= nbf` or re-issue with an earlier `nbf`. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_UCAN_NOT_YET_VALID", "Presented UCAN's `nbf` window has not yet opened at chain-walk time. Wait until `now >= nbf` or re-issue with an earlier `nbf`. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapUcanNotYetValid";
  }
}

/**
 * E_CAP_UCAN_BAD_SIGNATURE
 *
 * Thrown at: `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B). Constant-time comparison via `subtle::ConstantTimeEq` per `crypto-major-4`.
 * Message template: "UCAN signature failed verification (link_index={link_index})"
 */
export class ECapUcanBadSignature extends BentenError {
  static readonly code = "E_CAP_UCAN_BAD_SIGNATURE";
  static readonly fixHint = "Presented UCAN's signature failed to verify against the issuer's resolved public key. Likely tampered or signed by a different keypair than the one named in `iss`. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_UCAN_BAD_SIGNATURE", "Presented UCAN's signature failed to verify against the issuer's resolved public key. Likely tampered or signed by a different keypair than the one named in `iss`. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapUcanBadSignature";
  }
}

/**
 * E_CAP_UCAN_ATTENUATION_VIOLATED
 *
 * Thrown at: `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_at` (Phase-3 G14-B). Composes with `benten_id::ucan::validate_chain_at` per `crypto-blocker-2`.
 * Message template: "UCAN attenuation violated: child cap '{child_cap}' is not subsumed by parent caps"
 */
export class ECapUcanAttenuationViolated extends BentenError {
  static readonly code = "E_CAP_UCAN_ATTENUATION_VIOLATED";
  static readonly fixHint = "Child UCAN's capability widens its parent's authority — a structural delegation violation. Re-issue the child UCAN attenuated to a subset of the parent's `att`. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_UCAN_ATTENUATION_VIOLATED", "Child UCAN's capability widens its parent's authority — a structural delegation violation. Re-issue the child UCAN attenuated to a subset of the parent's `att`. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapUcanAttenuationViolated";
  }
}

/**
 * E_CAP_BACKEND_STORAGE
 *
 * Thrown at: `crates/benten-caps/src/backends/ucan.rs::UCANBackend::{record_grant, record_revocation, validate_chain_with_durable_revocations}` (Phase-3 G14-B).
 * Message template: "UCAN backend storage I/O failure: {reason}"
 */
export class ECapBackendStorage extends BentenError {
  static readonly code = "E_CAP_BACKEND_STORAGE";
  static readonly fixHint = "Durable UCAN backend failed to read or write its grant store. Surfaces a layered backend I/O failure to the policy hook caller. Inspect underlying `GraphBackend` health (redb file permissions, disk space). Distinct from `E_CAP_DENIED` — the backend cannot determine permitted-or-not when its store is unreadable. Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_BACKEND_STORAGE", "Durable UCAN backend failed to read or write its grant store. Surfaces a layered backend I/O failure to the policy hook caller. Inspect underlying `GraphBackend` health (redb file permissions, disk space). Distinct from `E_CAP_DENIED` — the backend cannot determine permitted-or-not when its store is unreadable. Routes to `ON_ERROR`.", message, context);
    this.name = "ECapBackendStorage";
  }
}

/**
 * E_CAP_RATE_LIMIT_EXCEEDED
 *
 * Thrown at: `crates/benten-caps/src/rate_limit.rs::RateLimitPolicy::check_writes_per_sec` (Phase-3 G14-B; D-F + D-PHASE-3-26).
 * Message template: "rate-limit exceeded for actor {actor} on zone {zone}"
 */
export class ECapRateLimitExceeded extends BentenError {
  static readonly code = "E_CAP_RATE_LIMIT_EXCEEDED";
  static readonly fixHint = "Per-actor writes/sec/zone bucket exceeded its budget. Configure a less restrictive `InMemoryRateLimitPolicyBuilder::actor_writes_per_second` for the actor, or back off and retry. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_RATE_LIMIT_EXCEEDED", "Per-actor writes/sec/zone bucket exceeded its budget. Configure a less restrictive `InMemoryRateLimitPolicyBuilder::actor_writes_per_second` for the actor, or back off and retry. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapRateLimitExceeded";
  }
}

/**
 * E_CAP_PEER_BANDWIDTH_EXCEEDED
 *
 * Thrown at: `crates/benten-caps/src/rate_limit.rs::RateLimitPolicy::check_peer_bandwidth` (Phase-3 G14-B; D-F + D-PHASE-3-26 + D-PHASE-3-30).
 * Message template: "peer bandwidth budget exceeded for peer {peer} ({bytes} bytes)"
 */
export class ECapPeerBandwidthExceeded extends BentenError {
  static readonly code = "E_CAP_PEER_BANDWIDTH_EXCEEDED";
  static readonly fixHint = "Per-peer bandwidth bytes/sec budget at the Atrium boundary exceeded its limit. Defends against a malicious or buggy peer flooding the sync channel. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_PEER_BANDWIDTH_EXCEEDED", "Per-peer bandwidth bytes/sec budget at the Atrium boundary exceeded its limit. Defends against a malicious or buggy peer flooding the sync channel. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapPeerBandwidthExceeded";
  }
}

/**
 * E_CAP_SNAPSHOT_HASH_MISMATCH
 *
 * Thrown at: `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` Step 3.5 (Phase-3 G14-D wave-5a; CLR-2 §11 + Compromise #10 engine-side asymmetry closure). The hash is computed by `crates/benten-engine/src/cap_snapshot_hash.rs::compute(actor_cid, &proof_chain_cids)` and persisted alongside the envelope via `Engine::put_cap_snapshot_for_envelope`.
 * Message template: "resume: cap_snapshot_hash mismatch for actor {actor} (proof-chain changed between suspend and resume; CLR-2 §11)"
 */
export class ECapSnapshotHashMismatch extends BentenError {
  static readonly code = "E_CAP_SNAPSHOT_HASH_MISMATCH";
  static readonly fixHint = "A WAIT-suspended execution attempted to resume against a UCAN proof-chain that materially changed between suspend and resume (e.g. one of the chain's tokens was revoked, or the chain was substituted). Per CLR-2 §11 the resume MUST reject — silently re-running a continuation against a downgraded chain would let an attacker race a revoke with a resume. Re-issue the suspended request from a current envelope; the prior envelope is no longer authoritative. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_SNAPSHOT_HASH_MISMATCH", "A WAIT-suspended execution attempted to resume against a UCAN proof-chain that materially changed between suspend and resume (e.g. one of the chain's tokens was revoked, or the chain was substituted). Per CLR-2 §11 the resume MUST reject — silently re-running a continuation against a downgraded chain would let an attacker race a revoke with a resume. Re-issue the suspended request from a current envelope; the prior envelope is no longer authoritative. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapSnapshotHashMismatch";
  }
}

/**
 * E_SUBSCRIBE_REVOKED_MID_STREAM
 *
 * Thrown at: `crates/benten-engine/src/cap_recheck.rs` per-event closure firing (Phase-3 G14-D wave-5a; F6 LOAD-BEARING + Compromise #2 D5). Wave-paired construction sites land alongside G14-B's durable UCAN backend `chain-for-audience` accessor.
 * Message template: "subscribe: cap revoked mid-stream for subscriber {subscriber} on channel {channel}"
 */
export class ESubscribeRevokedMidStream extends BentenError {
  static readonly code = "E_SUBSCRIBE_REVOKED_MID_STREAM";
  static readonly fixHint = "A SUBSCRIBE / sync-replica subscription was terminated mid-stream because the subscriber's read-coverage UCAN no longer holds — a partial revoke fired the per-event delivery-time cap-recheck on the next event. Distinct from `E_SUBSCRIBE_DELIVERY_FAILED` (transient delivery-channel failures) — this names the cap-recheck-driven termination per F6 LOAD-BEARING. Re-issue a fresh subscribe with current credentials. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SUBSCRIBE_REVOKED_MID_STREAM", "A SUBSCRIBE / sync-replica subscription was terminated mid-stream because the subscriber's read-coverage UCAN no longer holds — a partial revoke fired the per-event delivery-time cap-recheck on the next event. Distinct from `E_SUBSCRIBE_DELIVERY_FAILED` (transient delivery-channel failures) — this names the cap-recheck-driven termination per F6 LOAD-BEARING. Re-issue a fresh subscribe with current credentials. Routes to `ON_DENIED`.", message, context);
    this.name = "ESubscribeRevokedMidStream";
  }
}

/**
 * E_SYNC_REVOKED_DURING_SESSION
 *
 * Thrown at: `crates/benten-engine/src/engine.rs::apply_atrium_merge` per-row apply loop (Phase-3 G16-B-F; sec-r4r1-2 BLOCKER closure; CLR-2 mirror of SUBSCRIBE-side `E_SUBSCRIBE_REVOKED_MID_STREAM`).
 * Message template: "sync: peer {peer_did} grant revoked during session for zone {zone}"
 */
export class ESyncRevokedDuringSession extends BentenError {
  static readonly code = "E_SYNC_REVOKED_DURING_SESSION";
  static readonly fixHint = "A sync-replica inbound WRITE was rejected because the source peer's grant was revoked locally between the Atrium handshake and the next sync round. Per CLR-2 this mirrors the SUBSCRIBE delivery-time recheck — the receiving peer's per-write cap-recheck consults the local grant store via the `cap_recheck.rs` G13-pre-C scaffold + the `CapabilityPolicy::check_write` per-row hook. The peer may re-handshake with a current grant. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYNC_REVOKED_DURING_SESSION", "A sync-replica inbound WRITE was rejected because the source peer's grant was revoked locally between the Atrium handshake and the next sync round. Per CLR-2 this mirrors the SUBSCRIBE delivery-time recheck — the receiving peer's per-write cap-recheck consults the local grant store via the `cap_recheck.rs` G13-pre-C scaffold + the `CapabilityPolicy::check_write` per-row hook. The peer may re-handshake with a current grant. Routes to `ON_DENIED`.", message, context);
    this.name = "ESyncRevokedDuringSession";
  }
}

/**
 * E_DEVICE_ATTESTATION_FORGED
 *
 * Thrown at: `crates/benten-engine/src/engine_sync.rs::DeviceAttestationEnvelope::verify` (Phase-3 G16-D wave-6b fix-pass; cryptographic-attestation closure for criterion 16 per Ben ratification 2026-05-09). Composes the existing hardened `benten_id::DeviceAttestation` + `Acceptor::accept_at` + `FreshnessPolicy` primitives at the wire boundary rather than introducing parallel unsigned transport (per pim-N-cand-crypto-attestation-transport-reuse).
 * Message template: "device attestation envelope verification failed: {reason}"
 */
export class EDeviceAttestationForged extends BentenError {
  static readonly code = "E_DEVICE_ATTESTATION_FORGED";
  static readonly fixHint = "An inbound on-the-wire `DeviceAttestationEnvelope` (Phase-3 G16-D wave-6b) failed cryptographic verification at the sync-merge boundary. Three failure modes surface this single typed code: (a) **DID forgery** — the envelope's signature does not verify against the public key resolved from the declared `attestation.device_did`; (b) **parent-attestation chain rejection** — the embedded `benten_id::DeviceAttestation` was rejected by the receiver's `Acceptor::accept_at` (bad parent signature, expired freshness window via `FreshnessPolicy`, replayed nonce, revoked device-DID); (c) **frame-pair binding violation** — the envelope's signed `payload_hash` does not match the BLAKE3 hash of the Loro export payload received in the same exchange (MITM frame-substitution defense). All three reject with this single code so audit pipelines route on the wire-attestation boundary uniformly. Re-handshake from a non-revoked, non-replayed device-DID issued by the local trust-store's parent. Distinct from `E_THIN_CLIENT_AUTH_REJECTED` (browser-tab attestation boundary) and `E_SYNC_REVOKED_DURING_SESSION` (mid-session local-grant revocation). Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_DEVICE_ATTESTATION_FORGED", "An inbound on-the-wire `DeviceAttestationEnvelope` (Phase-3 G16-D wave-6b) failed cryptographic verification at the sync-merge boundary. Three failure modes surface this single typed code: (a) **DID forgery** — the envelope's signature does not verify against the public key resolved from the declared `attestation.device_did`; (b) **parent-attestation chain rejection** — the embedded `benten_id::DeviceAttestation` was rejected by the receiver's `Acceptor::accept_at` (bad parent signature, expired freshness window via `FreshnessPolicy`, replayed nonce, revoked device-DID); (c) **frame-pair binding violation** — the envelope's signed `payload_hash` does not match the BLAKE3 hash of the Loro export payload received in the same exchange (MITM frame-substitution defense). All three reject with this single code so audit pipelines route on the wire-attestation boundary uniformly. Re-handshake from a non-revoked, non-replayed device-DID issued by the local trust-store's parent. Distinct from `E_THIN_CLIENT_AUTH_REJECTED` (browser-tab attestation boundary) and `E_SYNC_REVOKED_DURING_SESSION` (mid-session local-grant revocation). Routes to `ON_DENIED`.", message, context);
    this.name = "EDeviceAttestationForged";
  }
}

/**
 * E_SYNC_HOP_DEPTH_EXCEEDED
 *
 * Thrown at: `crates/benten-engine/src/engine_sync.rs::AtriumHandle::walk_chain` constructs `Err(AtriumError::SyncHopDepthExceeded)` at the chain-bound checks (lines ~1437 + ~1442); the routing arm at `engine_sync.rs::604` maps `AtriumError::SyncHopDepthExceeded` to `ErrorCode::SyncHopDepthExceeded`. Originally reserved at G14-D wave-5a; production firing site landed in Phase-3 sync.
 * Message template: "sync: chain hop depth {depth} exceeds bound {bound}"
 */
export class ESyncHopDepthExceeded extends BentenError {
  static readonly code = "E_SYNC_HOP_DEPTH_EXCEEDED";
  static readonly fixHint = "An inbound sync-replica AttributionFrame chain exceeded the documented hop-depth bound (mirrors Inv-4 `sandbox_depth`). Defends against DOS/chain-bloat where an adversarial peer constructs an unbounded false chain. The peer should either issue against a shorter chain or re-handshake with a fresh authority root. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYNC_HOP_DEPTH_EXCEEDED", "An inbound sync-replica AttributionFrame chain exceeded the documented hop-depth bound (mirrors Inv-4 `sandbox_depth`). Defends against DOS/chain-bloat where an adversarial peer constructs an unbounded false chain. The peer should either issue against a shorter chain or re-handshake with a fresh authority root. Routes to `ON_DENIED`.", message, context);
    this.name = "ESyncHopDepthExceeded";
  }
}

/**
 * E_THIN_CLIENT_AUTH_REJECTED
 *
 * Thrown at: `crates/benten-engine/src/thin_client_subscribe.rs::ThinClientConnection::connect` (Phase-3 G14-D wave-5a; D-PHASE-3-30 + CLAUDE.md baked-in #17 — thin compute surface as device with minimum capability envelope).
 * Message template: "thin-client connect: device attestation rejected ({reason})"
 */
export class EThinClientAuthRejected extends BentenError {
  static readonly code = "E_THIN_CLIENT_AUTH_REJECTED";
  static readonly fixHint = "A thin-client (browser tab / edge-worker) connection attempt was rejected at the full-peer auth boundary because the connecting tab presented no device-attestation OR presented one bound to a revoked device-DID. Distinct from generic `E_CAP_DENIED` so audit pipelines can route on the thin-client auth boundary independently. Re-attest from a non-revoked device-DID. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_THIN_CLIENT_AUTH_REJECTED", "A thin-client (browser tab / edge-worker) connection attempt was rejected at the full-peer auth boundary because the connecting tab presented no device-attestation OR presented one bound to a revoked device-DID. Distinct from generic `E_CAP_DENIED` so audit pipelines can route on the thin-client auth boundary independently. Re-attest from a non-revoked device-DID. Routes to `ON_DENIED`.", message, context);
    this.name = "EThinClientAuthRejected";
  }
}

/**
 * E_CAP_UCAN_AUDIENCE_MISMATCH
 *
 * Thrown at: `crates/benten-caps/src/backends/ucan.rs::UCANBackend::validate_chain_for_audience_at` (Phase-3 G14-B mini-review fix-pass; CLR-2 audience-binding pinned at the durable chain-walk seam). Constant-time DID-bytes comparison via `subtle::ConstantTimeEq` at the `benten_id::ucan::validate_chain_for_audience` upstream.
 * Message template: "UCAN audience mismatch: token aud '{actual}' != expected '{expected}'"
 */
export class ECapUcanAudienceMismatch extends BentenError {
  static readonly code = "E_CAP_UCAN_AUDIENCE_MISMATCH";
  static readonly fixHint = "The presented UCAN's audience DID does not match the validation context's expected audience. Defends against cross-atrium replay (a UCAN issued to atrium A persisted in atrium B's durable store and replayed against atrium B). Re-issue the UCAN with the correct `aud` for the local atrium. Distinct from `E_CAP_DENIED` so audit pipelines can route on cross-atrium replay independently. Routes to `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_UCAN_AUDIENCE_MISMATCH", "The presented UCAN's audience DID does not match the validation context's expected audience. Defends against cross-atrium replay (a UCAN issued to atrium A persisted in atrium B's durable store and replayed against atrium B). Re-issue the UCAN with the correct `aud` for the local atrium. Distinct from `E_CAP_DENIED` so audit pipelines can route on cross-atrium replay independently. Routes to `ON_DENIED`.", message, context);
    this.name = "ECapUcanAudienceMismatch";
  }
}

/**
 * E_ATRIUM_RELAY_UNREACHABLE
 *
 * Thrown at: `crates/benten-sync/src/transport.rs::Endpoint::bind_with_relay_url` + `crates/benten-sync/src/transport.rs::Endpoint::connect` (Phase-3 G16-A wave-6; net-blocker-2 BLOCKER). Mapped from the `AtriumTransportError::RelayUnreachable` typed variant via `crates/benten-sync/src/errors.rs::AtriumTransportError::code`.
 * Message template: "atrium relay unreachable at {url}: {reason}"
 */
export class EAtriumRelayUnreachable extends BentenError {
  static readonly code = "E_ATRIUM_RELAY_UNREACHABLE";
  static readonly fixHint = "The configured iroh relay endpoint is unreachable (DNS-resolution failure, TLS handshake refused, transport-level timeout). Verify the relay URL is reachable from this peer's network (curl / nslookup / openssl s_client). For Phase-3 deployments the iroh public relay default applies; operators with stricter metadata threat models can opt into self-hosted relay infrastructure (Compromise #22 in `docs/SECURITY-POSTURE.md` — Phase-7 Garden-relays land as the operator-controlled alternative). Per `net-blocker-2` BLOCKER, this is a typed error variant — never a panic, never an untyped String. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (which signals an established connection has degraded mid-flight). Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_ATRIUM_RELAY_UNREACHABLE", "The configured iroh relay endpoint is unreachable (DNS-resolution failure, TLS handshake refused, transport-level timeout). Verify the relay URL is reachable from this peer's network (curl / nslookup / openssl s_client). For Phase-3 deployments the iroh public relay default applies; operators with stricter metadata threat models can opt into self-hosted relay infrastructure (Compromise #22 in `docs/SECURITY-POSTURE.md` — Phase-7 Garden-relays land as the operator-controlled alternative). Per `net-blocker-2` BLOCKER, this is a typed error variant — never a panic, never an untyped String. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (which signals an established connection has degraded mid-flight). Routes to `ON_ERROR`.", message, context);
    this.name = "EAtriumRelayUnreachable";
  }
}

/**
 * E_ATRIUM_TRANSPORT_DEGRADED
 *
 * Thrown at: `crates/benten-sync/src/transport.rs::Endpoint::*` (Phase-3 G16-A wave-6 connection-establishment + send/recv paths; net-blocker-2 BLOCKER). Also fires from `crates/benten-sync/src/handshake_wire.rs::HandshakeFrame::from_canonical_bytes` when the wire-format frame is missing required fields per net-blocker-4 BLOCKER. Mapped from the `AtriumTransportError::TransportDegraded` / `AtriumTransportError::HandshakeWireFormat` typed variants via `crates/benten-sync/src/errors.rs::AtriumTransportError::code`.
 * Message template: "atrium transport degraded: {reason}"
 */
export class EAtriumTransportDegraded extends BentenError {
  static readonly code = "E_ATRIUM_TRANSPORT_DEGRADED";
  static readonly fixHint = "The established Atrium transport has degraded — packet-loss above threshold, relay-fallback active mid-stream, direct connection lost, or handshake wire-format violation surfaced at the transport layer. The engine-side `engine.atrium_status()` surface (Phase-3 G16-B/D) propagates this state observably so operators can react. Investigate network conditions (packet-loss, NAT path) and the connecting peer's reachability. Per `net-blocker-2` BLOCKER, the degraded transport state is EXPLICIT — not a missing value, not a panic. Distinct from `E_ATRIUM_RELAY_UNREACHABLE` (which signals the relay endpoint itself is unreachable at connect time). Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_ATRIUM_TRANSPORT_DEGRADED", "The established Atrium transport has degraded — packet-loss above threshold, relay-fallback active mid-stream, direct connection lost, or handshake wire-format violation surfaced at the transport layer. The engine-side `engine.atrium_status()` surface (Phase-3 G16-B/D) propagates this state observably so operators can react. Investigate network conditions (packet-loss, NAT path) and the connecting peer's reachability. Per `net-blocker-2` BLOCKER, the degraded transport state is EXPLICIT — not a missing value, not a panic. Distinct from `E_ATRIUM_RELAY_UNREACHABLE` (which signals the relay endpoint itself is unreachable at connect time). Routes to `ON_ERROR`.", message, context);
    this.name = "EAtriumTransportDegraded";
  }
}

/**
 * E_ATRIUM_INACTIVE
 *
 * Thrown at: `crates/benten-engine/src/engine_sync.rs::AtriumHandle::merge_remote_change` (inbound sync) + outbound fan-out paths (publish-view-result + share-doc-update + close-share) when `is_active` flag is `false`. Mapped from `AtriumError::InvalidState` typed variant via `engine_sync.rs::AtriumError::code`.
 * Message template: "atrium handle is in graceful-leave state: {operation} requires rejoin()"
 */
export class EAtriumInactive extends BentenError {
  static readonly code = "E_ATRIUM_INACTIVE";
  static readonly fixHint = "An `AtriumHandle` was used after `leave()` flipped its `is_active` flag to false but before `rejoin()` flipped it back. The handle is in a graceful-leave quiesced state — distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer degrade) because the iroh endpoint remains bound + the lifecycle change is intentional (operator-initiated, not a fault). Distinct from `E_ATRIUM_RELAY_UNREACHABLE` (relay unavailability) because the relay link was never lost. Call `AtriumHandle::rejoin()` to re-activate; calling `rejoin()` is idempotent (no-op if already active). Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_ATRIUM_INACTIVE", "An `AtriumHandle` was used after `leave()` flipped its `is_active` flag to false but before `rejoin()` flipped it back. The handle is in a graceful-leave quiesced state — distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer degrade) because the iroh endpoint remains bound + the lifecycle change is intentional (operator-initiated, not a fault). Distinct from `E_ATRIUM_RELAY_UNREACHABLE` (relay unavailability) because the relay link was never lost. Call `AtriumHandle::rejoin()` to re-activate; calling `rejoin()` is idempotent (no-op if already active). Routes to `ON_ERROR`.", message, context);
    this.name = "EAtriumInactive";
  }
}

/**
 * E_SYNC_DIVERGENT_CID_REJECTED
 *
 * Thrown at: `crates/benten-engine/src/engine_sync.rs::AtriumError::DivergentCidRejected` (Phase-3 G16-B wave-6b; ds-4 Inv-13 row-4 SPLIT). PRE-merge classifier at `engine_sync.rs::merge_remote_change` walks `SYSTEM_ZONE_PREFIXES` and rejects divergent CIDs targeting system-zone paths before applying any Loro state. Mapped via `engine_sync.rs::AtriumError::code` to the stable code.
 * Message template: "sync replica frame rejected: system-zone target {zone} carries divergent CID {observed_cid} (Anchor-immutable per Inv-13 row-4b)"
 */
export class ESyncDivergentCidRejected extends BentenError {
  static readonly code = "E_SYNC_DIVERGENT_CID_REJECTED";
  static readonly fixHint = "An inbound sync-replica frame targets a system-zone / Anchor-immutable path (per `crates/benten-engine::system_zones::SYSTEM_ZONE_PREFIXES`) with a divergent CID. Per ds-4 Inv-13 row-4b, system-zone targets are immutable-via-sync — divergent CIDs are rejected PRE-merge by the classifier walk in `crates/benten-engine/src/engine_sync.rs::merge_remote_change` BEFORE the Loro merge applies (not post-merge cleanup). The remote peer SHOULD treat the rejection as authoritative for the system-zone path; user-data zones (Inv-13 row-4a) continue to merge via the Loro CRDT + D-C HYBRID Anchor+Version+CURRENT pattern. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer degrade) and `E_ATRIUM_RELAY_UNREACHABLE` (relay unavailability) — this is a semantic-layer reject, not a transport failure. Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SYNC_DIVERGENT_CID_REJECTED", "An inbound sync-replica frame targets a system-zone / Anchor-immutable path (per `crates/benten-engine::system_zones::SYSTEM_ZONE_PREFIXES`) with a divergent CID. Per ds-4 Inv-13 row-4b, system-zone targets are immutable-via-sync — divergent CIDs are rejected PRE-merge by the classifier walk in `crates/benten-engine/src/engine_sync.rs::merge_remote_change` BEFORE the Loro merge applies (not post-merge cleanup). The remote peer SHOULD treat the rejection as authoritative for the system-zone path; user-data zones (Inv-13 row-4a) continue to merge via the Loro CRDT + D-C HYBRID Anchor+Version+CURRENT pattern. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer degrade) and `E_ATRIUM_RELAY_UNREACHABLE` (relay unavailability) — this is a semantic-layer reject, not a transport failure. Routes to `ON_ERROR`.", message, context);
    this.name = "ESyncDivergentCidRejected";
  }
}

/**
 * E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW
 *
 * Thrown at: `crates/benten-sync/src/handshake.rs::HandshakeError::ReplayWithinBoundedWindow` (Phase-3 G16-D wave-6b; ds-r4-3). Surfaces from `Handshake::respond` and `Handshake::finalise` when the carried HLC drift exceeds the replay window. Composes with G14-pre-D HLC bounded-window math.
 * Message template: "handshake replay within bounded window: original_hlc={original_hlc} replay_hlc={replay_hlc} window_ms={window_ms}"
 */
export class EHandshakeReplayWithinBoundedWindow extends BentenError {
  static readonly code = "E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW";
  static readonly fixHint = "A handshake frame was replayed within the bounded HLC acceptance window (default `DEFAULT_REPLAY_WINDOW_MS = 5000`). The handshake state machine rejects bounded-window replays via symmetric drift math (`now.abs_diff(hlc_physical_ms) > replay_window_ms`) so future-stamped frames are also rejected — defends against clock-skew injection. The diagnostic fields (`original_hlc`, `replay_hlc`, `window_ms`) let operators distinguish bounded-window replay from transport-layer degradation. Per `ds-r4-3`, the replay defense is EXPLICIT and TYPED — not a generic transport error. The canonical replay-detection mechanism (per-peer nonce cache) is deferred to a follow-on wave per the source comment at `crates/benten-sync/src/handshake.rs::Handshake::respond`; G16-D ships only the bounded-window math. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer signal) — this is a semantic-layer reject. Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW", "A handshake frame was replayed within the bounded HLC acceptance window (default `DEFAULT_REPLAY_WINDOW_MS = 5000`). The handshake state machine rejects bounded-window replays via symmetric drift math (`now.abs_diff(hlc_physical_ms) > replay_window_ms`) so future-stamped frames are also rejected — defends against clock-skew injection. The diagnostic fields (`original_hlc`, `replay_hlc`, `window_ms`) let operators distinguish bounded-window replay from transport-layer degradation. Per `ds-r4-3`, the replay defense is EXPLICIT and TYPED — not a generic transport error. The canonical replay-detection mechanism (per-peer nonce cache) is deferred to a follow-on wave per the source comment at `crates/benten-sync/src/handshake.rs::Handshake::respond`; G16-D ships only the bounded-window math. Distinct from `E_ATRIUM_TRANSPORT_DEGRADED` (transport-layer signal) — this is a semantic-layer reject. Routes to `ON_ERROR`.", message, context);
    this.name = "EHandshakeReplayWithinBoundedWindow";
  }
}

/**
 * E_WAIT_TTL_EXPIRED
 *
 * Thrown at: `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` (Phase-3 G20-A2 wave-8a; D12). The pre-deadline check at the resume hot-path consults `crate::wait_ttl_gc::is_expired` against the persisted `WaitMetadata`; on expiry, calls `crate::wait_ttl_gc::reap_one` + increments stats + returns this typed error. Companion GC machinery at `crates/benten-engine/src/wait_ttl_gc.rs` runs three sweep paths (event-driven on suspend / resume + interval-backstop + drop-final).
 * Message template: "resume: WAIT TTL deadline elapsed for envelope {envelope_cid} (suspended {suspend_wallclock_ms} ms wall-clock; ttl_hours={ttl_hours}; now {now_ms} ms)"
 */
export class EWaitTtlExpired extends BentenError {
  static readonly code = "E_WAIT_TTL_EXPIRED";
  static readonly fixHint = "A `resume_with_meta` (or `resume_from_bytes_*`) call landed against a SuspensionStore entry whose wall-clock TTL deadline has elapsed. The TTL is anchored at suspend time as `suspend_wallclock_ms + ttl_hours * 3_600_000` and persisted alongside the envelope; a fresh engine opening the same redb path computes the same deadline (cross-process correctness). When elapsed, the resume hot-path reaps the entry from the SuspensionStore + bumps the `WaitTtlGcStats.reaped_count` counter + returns this typed error. Distinct from `E_WAIT_TIMEOUT` (in-process / per-call deadline that fires from the eval-side resume_with_meta consumer) — `E_WAIT_TTL_EXPIRED` is the wall-clock deadline that survives suspend / restart. Per the D12 wave-8a hybrid-GC contract, expiry is detected on every resume regardless of whether the GC sweep ran first (deadline-on-resume safety is independent of the sweep schedule). Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WAIT_TTL_EXPIRED", "A `resume_with_meta` (or `resume_from_bytes_*`) call landed against a SuspensionStore entry whose wall-clock TTL deadline has elapsed. The TTL is anchored at suspend time as `suspend_wallclock_ms + ttl_hours * 3_600_000` and persisted alongside the envelope; a fresh engine opening the same redb path computes the same deadline (cross-process correctness). When elapsed, the resume hot-path reaps the entry from the SuspensionStore + bumps the `WaitTtlGcStats.reaped_count` counter + returns this typed error. Distinct from `E_WAIT_TIMEOUT` (in-process / per-call deadline that fires from the eval-side resume_with_meta consumer) — `E_WAIT_TTL_EXPIRED` is the wall-clock deadline that survives suspend / restart. Per the D12 wave-8a hybrid-GC contract, expiry is detected on every resume regardless of whether the GC sweep ran first (deadline-on-resume safety is independent of the sweep schedule). Routes to `ON_ERROR`.", message, context);
    this.name = "EWaitTtlExpired";
  }
}

/**
 * E_WAIT_TTL_INVALID
 *
 * Thrown at: `crates/benten-engine/src/engine.rs::register_subgraph` (Phase-3 G20-A2 wave-8a; D12). The validation walk inspects every WAIT node's `ttl_hours` property; non-integer payloads + out-of-range integers + zero values all fire this code with the offending node id + raw value carried in the message.
 * Message template: "register_subgraph: WAIT node {node_id} has out-of-range ttl_hours={raw}; expected integer in [1, 720]"
 */
export class EWaitTtlInvalid extends BentenError {
  static readonly code = "E_WAIT_TTL_INVALID";
  static readonly fixHint = "A WAIT primitive's `ttl_hours` property failed registration-time validation. `ttl_hours == 0` would expire immediately on suspend (a footgun); `ttl_hours > 720` exceeds the documented 30-day ceiling. The check fires at `register_subgraph` time so a miswritten spec does not survive into running state. Either (a) drop the `ttl_hours` property entirely (defaults to no-TTL, matching the Phase-2b behaviour), (b) set it to an integer in `[1, 720]`, or (c) split the wait into staged shorter waits at the spec layer if a wait longer than 30 days is genuinely required. Distinct from `E_WAIT_TTL_EXPIRED` (runtime-deadline elapse) and `E_WAIT_TIMEOUT` (in-process per-call deadline) — this is a configuration-time error, not a runtime-deadline failure. Routes to `None` (the registration-time disposition matching `E_INV_REGISTRATION` / `E_DUPLICATE_HANDLER` / `E_INV_SANDBOX_DEPTH`); the registration error surfaces at the `register_subgraph` call site, not along an in-graph primitive edge.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WAIT_TTL_INVALID", "A WAIT primitive's `ttl_hours` property failed registration-time validation. `ttl_hours == 0` would expire immediately on suspend (a footgun); `ttl_hours > 720` exceeds the documented 30-day ceiling. The check fires at `register_subgraph` time so a miswritten spec does not survive into running state. Either (a) drop the `ttl_hours` property entirely (defaults to no-TTL, matching the Phase-2b behaviour), (b) set it to an integer in `[1, 720]`, or (c) split the wait into staged shorter waits at the spec layer if a wait longer than 30 days is genuinely required. Distinct from `E_WAIT_TTL_EXPIRED` (runtime-deadline elapse) and `E_WAIT_TIMEOUT` (in-process per-call deadline) — this is a configuration-time error, not a runtime-deadline failure. Routes to `None` (the registration-time disposition matching `E_INV_REGISTRATION` / `E_DUPLICATE_HANDLER` / `E_INV_SANDBOX_DEPTH`); the registration error surfaces at the `register_subgraph` call site, not along an in-graph primitive edge.", message, context);
    this.name = "EWaitTtlInvalid";
  }
}

/**
 * E_WAIT_METADATA_MISSING
 *
 * Thrown at: Primary site at `crates/benten-engine/src/engine_wait.rs::resume_from_bytes_inner` (Phase-3 G20-A2 wave-8a; D12) Step 1.5 envelope-vs-metadata mismatch check. Secondary mapping at `crates/benten-engine/src/engine_wait.rs::map_resume_eval_error` promotes the eval-side `HostBackendUnavailable` fail-loud (from `crates/benten-eval/src/primitives/wait.rs::resume_with_meta`'s `meta: None` arm) to this typed code at the resume boundary so the engine API surface preserves the metadata-missing semantic uniformly across direct-eval and engine-mediated callers. The eval-side ErrorCode stays `HostBackendUnavailable` (broader semantic — generic backend-unavailable surface); the engine-layer typed code is the user-facing one.
 * Message template: "resume: suspension store has no WAIT metadata for envelope CID {envelope_cid} (cross-process resume without a shared SuspensionStore, fabricated handle, or evicted entry)"
 */
export class EWaitMetadataMissing extends BentenError {
  static readonly code = "E_WAIT_METADATA_MISSING";
  static readonly fixHint = "A resume call landed against an envelope whose WAIT metadata is absent from the SuspensionStore. Per Compromise #9 / G12-E closure, missing metadata is a fail-loud surface — Phase-2a's permissive `Complete(value)` fallback was a documented gap that silently dropped the deadline + signal-shape checks. The discriminator is the envelope-side record's presence: the eval-side wait primitive persists BOTH `put_wait(cid, meta)` AND `put_envelope(envelope)` for every real WAIT suspend, so a mismatch (envelope present, metadata absent) is the engine-detectable signature of metadata-missing for a real WAIT envelope. Three legitimate scenarios trigger this: (a) a cross-process resume against a different physical SuspensionStore that holds the envelope record but lost metadata; (b) the metadata-side entry was evicted by the WAIT TTL GC (event-driven sweep / interval-backstop / drop-final) without the envelope side being reaped (impossible by `reap_one`'s contract — a partial-GC-corruption signal); (c) a caller fabricated an envelope-side record without a metadata-side counterpart. Distinct from `E_WAIT_TTL_EXPIRED` (entry exists but deadline has passed — a timing failure that the resume actively detects and reaps) — `E_WAIT_METADATA_MISSING` fires when no metadata entry exists for an envelope record that should have one. The eval-layer surfaces a parallel fail-loud at `benten_eval::resume_with_meta` via `EvalError::Host(HostBackendUnavailable)` (when the public eval API is called directly with `meta: None`); the engine's `map_resume_eval_error` ALSO promotes that path to `E_WAIT_METADATA_MISSING` so direct-eval-callers route consistently. Routes to `ON_ERROR`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_WAIT_METADATA_MISSING", "A resume call landed against an envelope whose WAIT metadata is absent from the SuspensionStore. Per Compromise #9 / G12-E closure, missing metadata is a fail-loud surface — Phase-2a's permissive `Complete(value)` fallback was a documented gap that silently dropped the deadline + signal-shape checks. The discriminator is the envelope-side record's presence: the eval-side wait primitive persists BOTH `put_wait(cid, meta)` AND `put_envelope(envelope)` for every real WAIT suspend, so a mismatch (envelope present, metadata absent) is the engine-detectable signature of metadata-missing for a real WAIT envelope. Three legitimate scenarios trigger this: (a) a cross-process resume against a different physical SuspensionStore that holds the envelope record but lost metadata; (b) the metadata-side entry was evicted by the WAIT TTL GC (event-driven sweep / interval-backstop / drop-final) without the envelope side being reaped (impossible by `reap_one`'s contract — a partial-GC-corruption signal); (c) a caller fabricated an envelope-side record without a metadata-side counterpart. Distinct from `E_WAIT_TTL_EXPIRED` (entry exists but deadline has passed — a timing failure that the resume actively detects and reaps) — `E_WAIT_METADATA_MISSING` fires when no metadata entry exists for an envelope record that should have one. The eval-layer surfaces a parallel fail-loud at `benten_eval::resume_with_meta` via `EvalError::Host(HostBackendUnavailable)` (when the public eval API is called directly with `meta: None`); the engine's `map_resume_eval_error` ALSO promotes that path to `E_WAIT_METADATA_MISSING` so direct-eval-callers route consistently. Routes to `ON_ERROR`.", message, context);
    this.name = "EWaitMetadataMissing";
  }
}

/**
 * E_TYPED_CALL_UNKNOWN_OP
 *
 * Thrown at: `crates/benten-eval/src/typed_call.rs::TypedCallOp::parse` (Phase-3 G21-T1; CLAUDE.md baked-in commitment #16 SANDBOX-vs-CALL framing). The dispatch fork at `crates/benten-eval/src/primitives/call.rs::execute` recognises the `engine:typed:` prefix and routes to the typed-CALL registry; an unknown op surfaces this code rather than falling through to the user handler registry.
 * Message template: "typed-CALL dispatch: unknown op '{op_name}' (engine:typed:* registry has no matching entry)"
 */
export class ETypedCallUnknownOp extends BentenError {
  static readonly code = "E_TYPED_CALL_UNKNOWN_OP";
  static readonly fixHint = "A CALL primitive dispatched a `target` in the reserved `engine:typed:*` namespace, but the trailing op name does not match any registered typed-CALL op. Phase-3 G21-T1 ships 10 ops: `ed25519_sign`, `ed25519_verify`, `keypair_generate`, `keypair_from_seed`, `blake3_hash`, `multibase_encode`, `multibase_decode`, `did_resolve`, `ucan_validate_chain`, `vc_verify`. Verify the op name spelling; the registry is closed (no user-registered typed-CALL ops in Phase 3 — extension is a Rust-only engine concern per CLAUDE.md baked-in commitment #16). Distinct from `E_NOT_FOUND` (handler-id miss in the user handler registry): this code fires AFTER the `engine:typed:` prefix is recognised. See [`docs/TYPED-CALL.md`](TYPED-CALL.md) for the engineer-facing reference.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TYPED_CALL_UNKNOWN_OP", "A CALL primitive dispatched a `target` in the reserved `engine:typed:*` namespace, but the trailing op name does not match any registered typed-CALL op. Phase-3 G21-T1 ships 10 ops: `ed25519_sign`, `ed25519_verify`, `keypair_generate`, `keypair_from_seed`, `blake3_hash`, `multibase_encode`, `multibase_decode`, `did_resolve`, `ucan_validate_chain`, `vc_verify`. Verify the op name spelling; the registry is closed (no user-registered typed-CALL ops in Phase 3 — extension is a Rust-only engine concern per CLAUDE.md baked-in commitment #16). Distinct from `E_NOT_FOUND` (handler-id miss in the user handler registry): this code fires AFTER the `engine:typed:` prefix is recognised. See [`docs/TYPED-CALL.md`](TYPED-CALL.md) for the engineer-facing reference.", message, context);
    this.name = "ETypedCallUnknownOp";
  }
}

/**
 * E_TYPED_CALL_INVALID_INPUT
 *
 * Thrown at: `crates/benten-eval/src/typed_call.rs` per-op input validation (Phase-3 G21-T1). Each op's `validate_input` arm rejects malformed input with this code + a per-op `reason` string before the engine-side handler in `crates/benten-engine/src/primitive_host.rs::dispatch_typed_call` is invoked.
 * Message template: "typed-CALL '{op_name}' input shape rejected: {reason}"
 */
export class ETypedCallInvalidInput extends BentenError {
  static readonly code = "E_TYPED_CALL_INVALID_INPUT";
  static readonly fixHint = "A typed-CALL dispatch supplied an input shape that does not match the named op's expected schema. Failure modes include: missing required field (e.g. `ed25519_sign` requires both `private_key` and `message`); wrong CBOR type (string passed where bytes expected); byte-length mismatch for fixed-width fields (Ed25519 secret keys MUST be 32 bytes; signatures MUST be 64 bytes; public keys MUST be 32 bytes). The op's input/output schema is documented inline at `crates/benten-eval/src/typed_call.rs::TypedCallOp` per-op rustdoc + tabulated at [`docs/TYPED-CALL.md`](TYPED-CALL.md). Distinct from `E_TRANSFORM_SYNTAX` (TRANSFORM expression parse failure) — this is a typed-CALL op-input validation failure that fires at dispatch time before any underlying crypto/codec call.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TYPED_CALL_INVALID_INPUT", "A typed-CALL dispatch supplied an input shape that does not match the named op's expected schema. Failure modes include: missing required field (e.g. `ed25519_sign` requires both `private_key` and `message`); wrong CBOR type (string passed where bytes expected); byte-length mismatch for fixed-width fields (Ed25519 secret keys MUST be 32 bytes; signatures MUST be 64 bytes; public keys MUST be 32 bytes). The op's input/output schema is documented inline at `crates/benten-eval/src/typed_call.rs::TypedCallOp` per-op rustdoc + tabulated at [`docs/TYPED-CALL.md`](TYPED-CALL.md). Distinct from `E_TRANSFORM_SYNTAX` (TRANSFORM expression parse failure) — this is a typed-CALL op-input validation failure that fires at dispatch time before any underlying crypto/codec call.", message, context);
    this.name = "ETypedCallInvalidInput";
  }
}

/**
 * E_TYPED_CALL_CAP_DENIED
 *
 * Thrown at: `crates/benten-engine/src/primitive_host.rs::dispatch_typed_call` (Phase-3 G21-T1). The cap-check fires BEFORE the underlying `benten-id` / `benten-core` op is invoked so a denied call has zero observable side effect.
 * Message template: "typed-CALL '{op_name}' denied: required capability '{required}' not held"
 */
export class ETypedCallCapDenied extends BentenError {
  static readonly code = "E_TYPED_CALL_CAP_DENIED";
  static readonly fixHint = "A typed-CALL dispatch was rejected because the dispatching grant's capability set does not include the per-op required capability. Each typed-CALL op declares a cap requirement (e.g. `cap:typed:crypto-sign` for `ed25519_sign`, `cap:typed:crypto-verify` for `ed25519_verify`, `cap:typed:did-resolve` for `did_resolve`, `cap:typed:ucan-validate` for `ucan_validate_chain`); the host's `check_capability` hook gates the op before dispatch. Under `NoAuthBackend` all typed-CALL caps are permitted; UCAN backend gates per chain claim (Phase-3-backlog §2.5(c) tracks the UCANBackend → `cap:typed:*` policy mapping carry). See [`docs/TYPED-CALL.md`](TYPED-CALL.md) §\"Capability model\".";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TYPED_CALL_CAP_DENIED", "A typed-CALL dispatch was rejected because the dispatching grant's capability set does not include the per-op required capability. Each typed-CALL op declares a cap requirement (e.g. `cap:typed:crypto-sign` for `ed25519_sign`, `cap:typed:crypto-verify` for `ed25519_verify`, `cap:typed:did-resolve` for `did_resolve`, `cap:typed:ucan-validate` for `ucan_validate_chain`); the host's `check_capability` hook gates the op before dispatch. Under `NoAuthBackend` all typed-CALL caps are permitted; UCAN backend gates per chain claim (Phase-3-backlog §2.5(c) tracks the UCANBackend → `cap:typed:*` policy mapping carry). See [`docs/TYPED-CALL.md`](TYPED-CALL.md) §\"Capability model\".", message, context);
    this.name = "ETypedCallCapDenied";
  }
}

/**
 * E_TYPED_CALL_DISPATCH_ERROR
 *
 * Thrown at: `crates/benten-engine/src/primitive_host.rs::dispatch_typed_call` (Phase-3 G21-T1). Per-op error mapping promotes the underlying typed error from `benten-id` / `benten-core` to this code with the op name + a brief `reason` string for diagnostic routing.
 * Message template: "typed-CALL '{op_name}' dispatch failed: {reason}"
 */
export class ETypedCallDispatchError extends BentenError {
  static readonly code = "E_TYPED_CALL_DISPATCH_ERROR";
  static readonly fixHint = "A typed-CALL op's underlying implementation in `benten-id` / `benten-core` returned a typed error that bubbles out of the typed-CALL dispatch boundary. Examples: `keypair_from_seed` against a malformed envelope (returns `KeypairError`); `did_resolve` against an unsupported method (returns `DidError`); `ucan_validate_chain` against a malformed JWT (returns `UcanError::Decode`). Note: a clean negative result (Ed25519 verify returns `false`, UCAN chain expired) is NOT this code — those return a structured `{ valid: false, ... }` Map with the op-internal failure reason. This code fires only when the underlying API call cannot produce a well-formed result. See [`docs/TYPED-CALL.md`](TYPED-CALL.md) §\"did_resolve DID-method validation\" for the §2.5(b) carry on `did_resolve` non-`did:key:` methods.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TYPED_CALL_DISPATCH_ERROR", "A typed-CALL op's underlying implementation in `benten-id` / `benten-core` returned a typed error that bubbles out of the typed-CALL dispatch boundary. Examples: `keypair_from_seed` against a malformed envelope (returns `KeypairError`); `did_resolve` against an unsupported method (returns `DidError`); `ucan_validate_chain` against a malformed JWT (returns `UcanError::Decode`). Note: a clean negative result (Ed25519 verify returns `false`, UCAN chain expired) is NOT this code — those return a structured `{ valid: false, ... }` Map with the op-internal failure reason. This code fires only when the underlying API call cannot produce a well-formed result. See [`docs/TYPED-CALL.md`](TYPED-CALL.md) §\"did_resolve DID-method validation\" for the §2.5(b) carry on `did_resolve` non-`did:key:` methods.", message, context);
    this.name = "ETypedCallDispatchError";
  }
}

/**
 * E_UCAN_CLOCK_NOT_INJECTED
 *
 * Thrown at: `crates/benten-caps/src/ucan_grounded.rs::UcanGroundedPolicy::typed_cap_permitted_by_proof` (Phase-3 G16-B-B-rest sub-item D). The fail-closed branch is the load-bearing assertion at the policy boundary; the `chain_has_time_bounds` helper at the same site distinguishes "chain depends on wallclock" from "chain is unbounded."
 * Message template: "UCAN chain-walker invoked with no clock injected (now_secs=0 sentinel) against a chain with time-bounded delegations; inject a real clock via with_now_for_test (or wait for WriteContext::now threading per phase-3-backlog §2.3 (i))"
 */
export class EUcanClockNotInjected extends BentenError {
  static readonly code = "E_UCAN_CLOCK_NOT_INJECTED";
  static readonly fixHint = "The `UcanGroundedPolicy` chain-walker observed the `DEFAULT_NOW_SECS = 0` sentinel against a UCAN chain that carries time-bounded delegations (`nbf > 0` OR `exp > 0`). Pre-fail-closed-fix the chain-walker silently fail-OPENed: it walked tokens against `now=0`, so a forged chain with `nbf=0` + `exp > 0` accepted whenever the rest of the chain-walk passed (no operator-visible surface signaling the missing-clock misconfiguration). The inversion at G16-B-B-rest sub-item D fail-CLOSES with this typed code so the caller MUST inject a real wallclock. Production callers will inject via the `WriteContext::now`-threading work named in `docs/future/phase-3-backlog.md §2.3 (i)`; tests inject via `UcanGroundedPolicy::with_now_for_test`. A chain WITHOUT time bounds (`nbf=0` AND `exp` unset) is safe to walk at the sentinel and does NOT trigger this code.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_UCAN_CLOCK_NOT_INJECTED", "The `UcanGroundedPolicy` chain-walker observed the `DEFAULT_NOW_SECS = 0` sentinel against a UCAN chain that carries time-bounded delegations (`nbf > 0` OR `exp > 0`). Pre-fail-closed-fix the chain-walker silently fail-OPENed: it walked tokens against `now=0`, so a forged chain with `nbf=0` + `exp > 0` accepted whenever the rest of the chain-walk passed (no operator-visible surface signaling the missing-clock misconfiguration). The inversion at G16-B-B-rest sub-item D fail-CLOSES with this typed code so the caller MUST inject a real wallclock. Production callers will inject via the `WriteContext::now`-threading work named in `docs/future/phase-3-backlog.md §2.3 (i)`; tests inject via `UcanGroundedPolicy::with_now_for_test`. A chain WITHOUT time bounds (`nbf=0` AND `exp` unset) is safe to walk at the sentinel and does NOT trigger this code.", message, context);
    this.name = "EUcanClockNotInjected";
  }
}

/**
 * E_RESERVED_HANDLER_NAMESPACE
 *
 * Thrown at: `crates/benten-engine/src/engine.rs::register_subgraph` + `register_subgraph_replace` (Phase-3 G21-T3 §2.5(d) fold-in; corr-minor-3 carry from G21-T1 fp-mini-review). Fires BEFORE invariant validation / subgraph CID derivation so a misnamed registration has zero observable side effect on engine state.
 * Message template: "register_subgraph: handler_id `{handler_id}` is in the reserved `engine:typed:` namespace; this prefix is the typed-CALL registry (see CLAUDE.md baked-in #16 + phase-3-backlog §2.5(d)). E_RESERVED_HANDLER_NAMESPACE"
 */
export class EReservedHandlerNamespace extends BentenError {
  static readonly code = "E_RESERVED_HANDLER_NAMESPACE";
  static readonly fixHint = "A user attempted to register a handler whose `handler_id` starts with the reserved `engine:typed:` namespace. The eval-side dispatch fork (`crates/benten-eval/src/primitives/call.rs::execute`) pre-empts user-handler routing for this prefix — the typed-CALL registry is closed (10 ops at Phase-3 G21-T1), and extension is a Rust-only engine concern per CLAUDE.md baked-in commitment #16 (SANDBOX is for compute that doesn't fit other primitives — typed crypto / hash / DID / UCAN / VC ops fit CALL). Without this guard the user registration would be silent dead code; the registration-time reject surfaces the user-error sooner than the eval-time `E_TYPED_CALL_UNKNOWN_OP` would. Choose a non-`engine:typed:` handler_id (e.g. drop the prefix, or use a project-specific namespace). The catalog entry is paper-trail: this code does NOT route along a primitive edge (registration-time refusal, same disposition as `E_VIEW_STRATEGY_A_REFUSED` / `E_DUPLICATE_HANDLER`).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_RESERVED_HANDLER_NAMESPACE", "A user attempted to register a handler whose `handler_id` starts with the reserved `engine:typed:` namespace. The eval-side dispatch fork (`crates/benten-eval/src/primitives/call.rs::execute`) pre-empts user-handler routing for this prefix — the typed-CALL registry is closed (10 ops at Phase-3 G21-T1), and extension is a Rust-only engine concern per CLAUDE.md baked-in commitment #16 (SANDBOX is for compute that doesn't fit other primitives — typed crypto / hash / DID / UCAN / VC ops fit CALL). Without this guard the user registration would be silent dead code; the registration-time reject surfaces the user-error sooner than the eval-time `E_TYPED_CALL_UNKNOWN_OP` would. Choose a non-`engine:typed:` handler_id (e.g. drop the prefix, or use a project-specific namespace). The catalog entry is paper-trail: this code does NOT route along a primitive edge (registration-time refusal, same disposition as `E_VIEW_STRATEGY_A_REFUSED` / `E_DUPLICATE_HANDLER`).", message, context);
    this.name = "EReservedHandlerNamespace";
  }
}

/**
 * Phase-3 G19-B (§7.6): codegen-emitted CODE_TO_CTOR_GENERATED map. Keys are stable
 * catalog codes (`E_*`); values are the typed BentenError subclass constructor for each
 * code. Updated automatically every time `scripts/codegen-errors.ts` runs against
 * `docs/ERROR-CATALOG.md`. The runtime helper `mapNativeError` consults this map so
 * every catalog entry resolves to a typed subclass — there is NO `E_UNKNOWN` fallback
 * for known catalog codes.
 */
export const CODE_TO_CTOR_GENERATED: Readonly<Record<string, new (message: string, context?: Record<string, unknown>) => BentenError>> = Object.freeze({
  "E_INV_CYCLE": EInvCycle,
  "E_INV_DEPTH_EXCEEDED": EInvDepthExceeded,
  "E_INV_FANOUT_EXCEEDED": EInvFanoutExceeded,
  "E_INV_TOO_MANY_NODES": EInvTooManyNodes,
  "E_INV_TOO_MANY_EDGES": EInvTooManyEdges,
  "E_INV_SYSTEM_ZONE": EInvSystemZone,
  "E_INV_DETERMINISM": EInvDeterminism,
  "E_INV_ITERATE_MAX_MISSING": EInvIterateMaxMissing,
  "E_INV_ITERATE_BUDGET": EInvIterateBudget,
  "E_INV_ITERATE_NEST_DEPTH": EInvIterateNestDepth,
  "E_INV_CONTENT_HASH": EInvContentHash,
  "E_INV_REGISTRATION": EInvRegistration,
  "E_CAP_DENIED": ECapDenied,
  "E_CAP_DENIED_READ": ECapDeniedRead,
  "E_CAP_REVOKED_MID_EVAL": ECapRevokedMidEval,
  "E_CAP_NOT_IMPLEMENTED": ECapNotImplemented,
  "E_CAP_REVOKED": ECapRevoked,
  "E_CAP_ATTENUATION": ECapAttenuation,
  "E_WRITE_CONFLICT": EWriteConflict,
  "E_INV_SANDBOX_DEPTH": EInvSandboxDepth,
  "E_INV_SANDBOX_OUTPUT": EInvSandboxOutput,
  "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED": ESandboxNestedDispatchDepthExceeded,
  "E_IVM_VIEW_STALE": EIvmViewStale,
  "E_TX_ABORTED": ETxAborted,
  "E_NESTED_TRANSACTION_NOT_SUPPORTED": ENestedTransactionNotSupported,
  "E_PRIMITIVE_NOT_IMPLEMENTED": EPrimitiveNotImplemented,
  "E_SYSTEM_ZONE_WRITE": ESystemZoneWrite,
  "E_TRANSFORM_SYNTAX": ETransformSyntax,
  "E_INPUT_LIMIT": EInputLimit,
  "E_SERIALIZE": ESerialize,
  "E_SYNC_HASH_MISMATCH": ESyncHashMismatch,
  "E_SYNC_HLC_DRIFT": ESyncHlcDrift,
  "E_SYNC_CAP_UNVERIFIED": ESyncCapUnverified,
  "E_VALUE_FLOAT_NAN": EValueFloatNan,
  "E_VALUE_FLOAT_NONFINITE": EValueFloatNonFinite,
  "E_CID_PARSE": ECidParse,
  "E_CID_UNSUPPORTED_CODEC": ECidUnsupportedCodec,
  "E_CID_UNSUPPORTED_HASH": ECidUnsupportedHash,
  "E_VERSION_BRANCHED": EVersionBranched,
  "E_BACKEND_NOT_FOUND": EBackendNotFound,
  "E_NOT_FOUND": ENotFound,
  "E_GRAPH_INTERNAL": EGraphInternal,
  "E_UNKNOWN": EUnknown,
  "E_DUPLICATE_HANDLER": EDuplicateHandler,
  "E_NO_CAPABILITY_POLICY_CONFIGURED": ENoCapabilityPolicyConfigured,
  "E_PRODUCTION_REQUIRES_CAPS": EProductionRequiresCaps,
  "E_SUBSYSTEM_DISABLED": ESubsystemDisabled,
  "E_UNKNOWN_VIEW": EUnknownView,
  "E_NOT_IMPLEMENTED": ENotImplemented,
  "E_IVM_PATTERN_MISMATCH": EIvmPatternMismatch,
  "E_IVM_STRATEGY_NOT_IMPLEMENTED": EIvmStrategyNotImplemented,
  "E_VERSION_UNKNOWN_PRIOR": EVersionUnknownPrior,
  "E_DSL_INVALID_SHAPE": EDslInvalidShape,
  "E_DSL_UNREGISTERED_HANDLER": EDslUnregisteredHandler,
  "E_HOST_NOT_FOUND": EHostNotFound,
  "E_HOST_WRITE_CONFLICT": EHostWriteConflict,
  "E_HOST_BACKEND_UNAVAILABLE": EHostBackendUnavailable,
  "E_HOST_CAPABILITY_REVOKED": EHostCapabilityRevoked,
  "E_HOST_CAPABILITY_EXPIRED": EHostCapabilityExpired,
  "E_EXEC_STATE_TAMPERED": EExecStateTampered,
  "E_RESUME_ACTOR_MISMATCH": EResumeActorMismatch,
  "E_RESUME_SUBGRAPH_DRIFT": EResumeSubgraphDrift,
  "E_WAIT_TIMEOUT": EWaitTimeout,
  "E_INV_IMMUTABILITY": EInvImmutability,
  "E_INV_ATTRIBUTION": EInvAttribution,
  "E_CAP_WALLCLOCK_EXPIRED": ECapWallclockExpired,
  "E_CAP_CHAIN_TOO_DEEP": ECapChainTooDeep,
  "E_CAP_SCOPE_LONE_STAR_REJECTED": ECapScopeLoneStarRejected,
  "E_VIEW_STRATEGY_A_REFUSED": EViewStrategyARefused,
  "E_VIEW_STRATEGY_C_RESERVED": EViewStrategyCReserved,
  "E_VIEW_LABEL_MISMATCH": EViewLabelMismatch,
  "E_WAIT_SIGNAL_SHAPE_MISMATCH": EWaitSignalShapeMismatch,
  "E_WAIT_SUSPENDED": EWaitSuspended,
  "E_STREAM_BACKPRESSURE_DROPPED": EStreamBackpressureDropped,
  "E_STREAM_CLOSED_BY_PEER": EStreamClosedByPeer,
  "E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED": EStreamProducerWallclockExceeded,
  "E_INV_STREAM_CONFIG": EInvStreamConfig,
  "E_STREAM_HANDLE_LEAKED": EStreamHandleLeaked,
  "E_SUBSCRIBE_DELIVERY_FAILED": ESubscribeDeliveryFailed,
  "E_SUBSCRIBE_PATTERN_INVALID": ESubscribePatternInvalid,
  "E_SUBSCRIBE_CURSOR_LOST": ESubscribeCursorLost,
  "E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED": ESubscribeReplayWindowExceeded,
  "E_INV_11_SYSTEM_ZONE_READ": EInv11SystemZoneRead,
  "E_SANDBOX_FUEL_EXHAUSTED": ESandboxFuelExhausted,
  "E_SANDBOX_MEMORY_EXHAUSTED": ESandboxMemoryExhausted,
  "E_SANDBOX_WALLCLOCK_EXCEEDED": ESandboxWallclockExceeded,
  "E_SANDBOX_WALLCLOCK_INVALID": ESandboxWallclockInvalid,
  "E_SANDBOX_HOST_FN_DENIED": ESandboxHostFnDenied,
  "E_SANDBOX_HOST_FN_NOT_FOUND": ESandboxHostFnNotFound,
  "E_SANDBOX_HOST_FN_RANDOM_BUDGET_EXCEEDED": ESandboxHostFnRandomBudgetExceeded,
  "E_SANDBOX_MANIFEST_UNKNOWN": ESandboxManifestUnknown,
  "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED": ESandboxManifestRegistrationDeferred,
  "E_SANDBOX_MODULE_INVALID": ESandboxModuleInvalid,
  "E_SANDBOX_STACK_OVERFLOW": ESandboxStackOverflow,
  "E_SANDBOX_ESCAPE_ATTEMPT": ESandboxEscapeAttempt,
  "E_SANDBOX_MODULE_NOT_INSTALLED": ESandboxModuleNotInstalled,
  "E_SANDBOX_NESTED_DISPATCH_DENIED": ESandboxNestedDispatchDenied,
  "E_MODULE_MANIFEST_CID_MISMATCH": EModuleManifestCidMismatch,
  "E_MODULE_MIGRATIONS_REQUIRE_PERSISTENCE": EModuleMigrationsRequirePersistence,
  "E_ENGINE_CONFIG_INVALID": EEngineConfigInvalid,
  "E_BACKEND_READ_ONLY": EBackendReadOnly,
  "E_SANDBOX_UNAVAILABLE_ON_WASM": ESandboxUnavailableOnWasm,
  "E_RELOAD_SUBSCRIBER_UNSUBSCRIBED": EReloadSubscriberUnsubscribed,
  "E_DEVSERVER_STOPPED": EDevServerStopped,
  "E_STORAGE_QUOTA_EXCEEDED": EStorageQuotaExceeded,
  "E_HLC_SKEW_EXCEEDED": EHlcSkewExceeded,
  "E_CAP_UCAN_EXPIRED": ECapUcanExpired,
  "E_CAP_UCAN_NOT_YET_VALID": ECapUcanNotYetValid,
  "E_CAP_UCAN_BAD_SIGNATURE": ECapUcanBadSignature,
  "E_CAP_UCAN_ATTENUATION_VIOLATED": ECapUcanAttenuationViolated,
  "E_CAP_BACKEND_STORAGE": ECapBackendStorage,
  "E_CAP_RATE_LIMIT_EXCEEDED": ECapRateLimitExceeded,
  "E_CAP_PEER_BANDWIDTH_EXCEEDED": ECapPeerBandwidthExceeded,
  "E_CAP_SNAPSHOT_HASH_MISMATCH": ECapSnapshotHashMismatch,
  "E_SUBSCRIBE_REVOKED_MID_STREAM": ESubscribeRevokedMidStream,
  "E_SYNC_REVOKED_DURING_SESSION": ESyncRevokedDuringSession,
  "E_DEVICE_ATTESTATION_FORGED": EDeviceAttestationForged,
  "E_SYNC_HOP_DEPTH_EXCEEDED": ESyncHopDepthExceeded,
  "E_THIN_CLIENT_AUTH_REJECTED": EThinClientAuthRejected,
  "E_CAP_UCAN_AUDIENCE_MISMATCH": ECapUcanAudienceMismatch,
  "E_ATRIUM_RELAY_UNREACHABLE": EAtriumRelayUnreachable,
  "E_ATRIUM_TRANSPORT_DEGRADED": EAtriumTransportDegraded,
  "E_ATRIUM_INACTIVE": EAtriumInactive,
  "E_SYNC_DIVERGENT_CID_REJECTED": ESyncDivergentCidRejected,
  "E_HANDSHAKE_REPLAY_WITHIN_BOUNDED_WINDOW": EHandshakeReplayWithinBoundedWindow,
  "E_WAIT_TTL_EXPIRED": EWaitTtlExpired,
  "E_WAIT_TTL_INVALID": EWaitTtlInvalid,
  "E_WAIT_METADATA_MISSING": EWaitMetadataMissing,
  "E_TYPED_CALL_UNKNOWN_OP": ETypedCallUnknownOp,
  "E_TYPED_CALL_INVALID_INPUT": ETypedCallInvalidInput,
  "E_TYPED_CALL_CAP_DENIED": ETypedCallCapDenied,
  "E_TYPED_CALL_DISPATCH_ERROR": ETypedCallDispatchError,
  "E_UCAN_CLOCK_NOT_INJECTED": EUcanClockNotInjected,
  "E_RESERVED_HANDLER_NAMESPACE": EReservedHandlerNamespace,
}) as Readonly<Record<string, new (message: string, context?: Record<string, unknown>) => BentenError>>;

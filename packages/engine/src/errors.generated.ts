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
  "E_WAIT_SIGNAL_SHAPE_MISMATCH",
  "E_STREAM_BACKPRESSURE_DROPPED",
  "E_STREAM_CLOSED_BY_PEER",
  "E_STREAM_PRODUCER_WALLCLOCK_EXCEEDED",
  "E_SUBSCRIBE_DELIVERY_FAILED",
  "E_SUBSCRIBE_PATTERN_INVALID",
  "E_SUBSCRIBE_CURSOR_LOST",
  "E_SUBSCRIBE_REPLAY_WINDOW_EXCEEDED",
  "E_INV_11_SYSTEM_ZONE_READ",
  "E_INV_SANDBOX_DEPTH",
  "E_INV_SANDBOX_OUTPUT",
  "E_SANDBOX_FUEL_EXHAUSTED",
  "E_SANDBOX_MEMORY_EXHAUSTED",
  "E_SANDBOX_WALLCLOCK_EXCEEDED",
  "E_SANDBOX_WALLCLOCK_INVALID",
  "E_SANDBOX_HOST_FN_DENIED",
  "E_SANDBOX_HOST_FN_NOT_FOUND",
  "E_SANDBOX_MANIFEST_UNKNOWN",
  "E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED",
  "E_SANDBOX_MODULE_INVALID",
  "E_SANDBOX_NESTED_DISPATCH_DENIED",
  "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED",
  "E_MODULE_MANIFEST_CID_MISMATCH",
  "E_ENGINE_CONFIG_INVALID",
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
 * Thrown at: Registration / read
 * Message template: "Content hash mismatch for {node_id}: expected {expected}, computed {actual}"
 */
export class EInvContentHash extends BentenError {
  static readonly code = "E_INV_CONTENT_HASH";
  static readonly fixHint = "A stored Node's computed content hash does not match its key. Indicates on-disk corruption or an incompatible serialization migration. Re-hash the Node from source; if persistent, restore from a backup or re-ingest.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_CONTENT_HASH", "A stored Node's computed content hash does not match its key. Indicates on-disk corruption or an incompatible serialization migration. Re-hash the Node from source; if persistent, restore from a backup or re-ingest.", message, context);
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
  static readonly fixHint = "Distinct from `E_CAP_DENIED` — this signals operator misconfiguration (configured a capability backend that isn't implemented yet), not an authorization failure. `UCANBackend` ships as a stub in Phase 1 and fully in Phase 3. Configure `NoAuthBackend` for embedded/local-only use, or provide a custom `CapabilityPolicy` impl. Routes to the subgraph's `ON_ERROR` edge, not `ON_DENIED`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CAP_NOT_IMPLEMENTED", "Distinct from `E_CAP_DENIED` — this signals operator misconfiguration (configured a capability backend that isn't implemented yet), not an authorization failure. `UCANBackend` ships as a stub in Phase 1 and fully in Phase 3. Configure `NoAuthBackend` for embedded/local-only use, or provide a custom `CapabilityPolicy` impl. Routes to the subgraph's `ON_ERROR` edge, not `ON_DENIED`.", message, context);
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
 * Thrown at: Evaluation (CAS WRITE). **Runtime surface is edge-routed, not Rust-enum-valued:** WRITE's `cas` mode routes conflicts via the `ON_CONFLICT` edge; the engine stamps `error_code: "E_WRITE_CONFLICT"` on the routed step (`crates/benten-engine/src/primitive_host.rs:~362`). Callers read the code off the edge-routing metadata, not via a `match` on an `Err(EvalError::WriteConflict)` — the enum variant exists for forward-compat with a Phase-2 native Rust path but has no construction site in Phase-1 production code. The drift-detector's `reachability: ignore` annotation reflects this asymmetry.
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
 * Thrown at: Registration (static SubgraphSpec analysis) and Evaluation (TRANSFORM-computed SANDBOX targets that exceed the ceiling at runtime)
 * Message template: "SANDBOX nest depth {depth} exceeds configured max {max}"
 */
export class EInvSandboxDepth extends BentenError {
  static readonly code = "E_INV_SANDBOX_DEPTH";
  static readonly fixHint = "Reduce SANDBOX nesting (a SANDBOX whose subgraph CALLs another handler that itself SANDBOXes counts toward the same depth — D20 inheritance across CALL boundaries). Either flatten the call chain or increase `max_sandbox_nest_depth` via capability grant.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SANDBOX_DEPTH", "Reduce SANDBOX nesting (a SANDBOX whose subgraph CALLs another handler that itself SANDBOXes counts toward the same depth — D20 inheritance across CALL boundaries). Either flatten the call chain or increase `max_sandbox_nest_depth` via capability grant.", message, context);
    this.name = "EInvSandboxDepth";
  }
}

/**
 * E_INV_SANDBOX_OUTPUT
 *
 * Thrown at: Evaluation. The `path` field distinguishes the D17 PRIMARY streaming `CountedSink` enforcement (fires before host-fn bytes are accepted) from the D17 BACKSTOP return-value enforcement (defense-in-depth at the primitive boundary).
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
 * Thrown at: Evaluation (saturation point at the SANDBOX entry — the counter-saturation check fires before the inner subgraph starts executing).
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
 * Thrown at: Sync-receive
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
 * Thrown at: Sync-receive
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
export class EValueFloatNonfinite extends BentenError {
  static readonly code = "E_VALUE_FLOAT_NONFINITE";
  static readonly fixHint = "DAG-CBOR's canonical form rejects ±Infinity. Clamp to a finite bound or use `Value::Null`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_VALUE_FLOAT_NONFINITE", "DAG-CBOR's canonical form rejects ±Infinity. Clamp to a finite bound or use `Value::Null`.", message, context);
    this.name = "EValueFloatNonfinite";
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
  static readonly fixHint = "Generic not-found — version-chain anchor miss, unregistered handler lookup, unknown view id, etc. Check that the caller has the correct CID / id; for handlers, confirm `registerSubgraph` / `registerCrud` ran successfully.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_NOT_FOUND", "Generic not-found — version-chain anchor miss, unregistered handler lookup, unknown view id, etc. Check that the caller has the correct CID / id; for handlers, confirm `registerSubgraph` / `registerCrud` ran successfully.", message, context);
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
 * Thrown at: DSL wrapper (TypeScript layer, before engine call)
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
 * Thrown at: DSL wrapper
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
  static readonly fixHint = "Reserved HostError discriminant. Fires when a host-level compare-and-swap write detects a concurrent mutation. Surface is frozen at Phase 2a; first firing site in Phase 3 sync.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_WRITE_CONFLICT", "Reserved HostError discriminant. Fires when a host-level compare-and-swap write detects a concurrent mutation. Surface is frozen at Phase 2a; first firing site in Phase 3 sync.", message, context);
    this.name = "EHostWriteConflict";
  }
}

/**
 * E_HOST_BACKEND_UNAVAILABLE
 *
 * Thrown at: `PrimitiveHost` implementation (G1-B)
 * Message template: "Host-boundary backend unavailable: {detail}"
 */
export class EHostBackendUnavailable extends BentenError {
  static readonly code = "E_HOST_BACKEND_UNAVAILABLE";
  static readonly fixHint = "Reserved HostError discriminant. Fires when the underlying storage backend is offline (I/O error, disk full, network partition). Retry with exponential backoff; if persistent, inspect the storage layer.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_HOST_BACKEND_UNAVAILABLE", "Reserved HostError discriminant. Fires when the underlying storage backend is offline (I/O error, disk full, network partition). Retry with exponential backoff; if persistent, inspect the storage layer.", message, context);
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
 * E_INV_SANDBOX_DEPTH
 *
 * Thrown at: Registration (Inv-4 structural check on subgraph nesting).
 * Message template: "SANDBOX nest-depth {actual} exceeds max {max} (Inv-4)"
 */
export class EInvSandboxDepth extends BentenError {
  static readonly code = "E_INV_SANDBOX_DEPTH";
  static readonly fixHint = "Inv-4 — `AttributionFrame.sandbox_depth: u8` saturating-counter; default max nest 4 (D20-RESOLVED). Flatten the SANDBOX → SANDBOX chain or move logic into a single SANDBOX call. Per D20, the depth INHERITS across CALL boundaries (not reset) so a SANDBOX → CALL → SANDBOX chain still counts cumulatively.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SANDBOX_DEPTH", "Inv-4 — `AttributionFrame.sandbox_depth: u8` saturating-counter; default max nest 4 (D20-RESOLVED). Flatten the SANDBOX → SANDBOX chain or move logic into a single SANDBOX call. Per D20, the depth INHERITS across CALL boundaries (not reset) so a SANDBOX → CALL → SANDBOX chain still counts cumulatively.", message, context);
    this.name = "EInvSandboxDepth";
  }
}

/**
 * E_INV_SANDBOX_OUTPUT
 *
 * Thrown at: SANDBOX executor (host-fn trampoline PRIMARY path; primitive-boundary BACKSTOP path).
 * Message template: "SANDBOX output budget exceeded: consumed={consumed} limit={limit} emitter={emitter_kind} path={path}"
 */
export class EInvSandboxOutput extends BentenError {
  static readonly code = "E_INV_SANDBOX_OUTPUT";
  static readonly fixHint = "Inv-7 — D17-RESOLVED defense-in-depth output enforcement. `path == primary_streaming` indicates the streaming `CountedSink` caught the overflow before accepting bytes; `path == return_backstop` indicates the primitive-boundary backstop caught a host-fn that bypassed the sink. Reduce per-call output via aggregation, raise the per-call cap if the operator trusts the workload, or split into multiple SANDBOX calls.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SANDBOX_OUTPUT", "Inv-7 — D17-RESOLVED defense-in-depth output enforcement. `path == primary_streaming` indicates the streaming `CountedSink` caught the overflow before accepting bytes; `path == return_backstop` indicates the primitive-boundary backstop caught a host-fn that bypassed the sink. Reduce per-call output via aggregation, raise the per-call cap if the operator trusts the workload, or split into multiple SANDBOX calls.", message, context);
    this.name = "EInvSandboxOutput";
  }
}

/**
 * E_SANDBOX_FUEL_EXHAUSTED
 *
 * Thrown at: SANDBOX executor (D3-RESOLVED per-call wasmtime `Store` lifecycle).
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
 * Thrown at: SANDBOX executor.
 * Message template: "SANDBOX memory limit exhausted: {limit} bytes"
 */
export class ESandboxMemoryExhausted extends BentenError {
  static readonly code = "E_SANDBOX_MEMORY_EXHAUSTED";
  static readonly fixHint = "wasmtime `StoreLimits` intercept fires deterministically BEFORE host OOM. Either reduce module memory pressure, raise `SandboxConfig::memory_bytes` (default 64 MiB), or audit for runaway `memory.grow` (ESC-2 escape vector).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MEMORY_EXHAUSTED", "wasmtime `StoreLimits` intercept fires deterministically BEFORE host OOM. Either reduce module memory pressure, raise `SandboxConfig::memory_bytes` (default 64 MiB), or audit for runaway `memory.grow` (ESC-2 escape vector).", message, context);
    this.name = "ESandboxMemoryExhausted";
  }
}

/**
 * E_SANDBOX_WALLCLOCK_EXCEEDED
 *
 * Thrown at: SANDBOX executor (wasmtime `epoch_interruption` driven by a thread-side ticker; D27 `async-support` ENABLED preserves the yield path for Phase-3 iroh forward-compat).
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
 * Thrown at: SANDBOX executor (link-time preferred; call-time fallback).
 * Message template: "SANDBOX host-fn not found: {name}"
 */
export class ESandboxHostFnNotFound extends BentenError {
  static readonly code = "E_SANDBOX_HOST_FN_NOT_FOUND";
  static readonly fixHint = "Module attempted to call a host-fn name not in the active manifest. In Phase 2b: this fires for `random` (deferred to Phase 2c per D1 + sec-pre-r1-06 §2.3 — workspace CSPRNG framework decision pending). The error message hint MUST mention \"deferred to Phase 2c\" for `random` so developers don't think it's a typo. For other names: check the manifest declaration matches the import.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_HOST_FN_NOT_FOUND", "Module attempted to call a host-fn name not in the active manifest. In Phase 2b: this fires for `random` (deferred to Phase 2c per D1 + sec-pre-r1-06 §2.3 — workspace CSPRNG framework decision pending). The error message hint MUST mention \"deferred to Phase 2c\" for `random` so developers don't think it's a typo. For other names: check the manifest declaration matches the import.", message, context);
    this.name = "ESandboxHostFnNotFound";
  }
}

/**
 * E_SANDBOX_MANIFEST_UNKNOWN
 *
 * Thrown at: `ManifestRegistry::lookup` / `ManifestRef::resolve`.
 * Message template: "SANDBOX manifest unknown: {name}"
 */
export class ESandboxManifestUnknown extends BentenError {
  static readonly code = "E_SANDBOX_MANIFEST_UNKNOWN";
  static readonly fixHint = "ESC-15 escape vector closure: NO permissive fall-through to a default manifest. Either register the manifest (Phase 8 marketplace work — see [E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED]) or use one of the codegen-default names (`compute-basic`, `compute-with-kv`).";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_MANIFEST_UNKNOWN", "ESC-15 escape vector closure: NO permissive fall-through to a default manifest. Either register the manifest (Phase 8 marketplace work — see [E_SANDBOX_MANIFEST_REGISTRATION_DEFERRED]) or use one of the codegen-default names (`compute-basic`, `compute-with-kv`).", message, context);
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
 * E_SANDBOX_NESTED_DISPATCH_DENIED
 *
 * Thrown at: SANDBOX executor (host-fn callback path).
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
 * E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED
 *
 * Thrown at: SANDBOX executor (depth saturation at the inheritance point — distinct from [E_SANDBOX_NESTED_DISPATCH_DENIED] which fires at the dispatch attempt).
 * Message template: "SANDBOX nested dispatch depth exceeded: max={max}"
 */
export class ESandboxNestedDispatchDepthExceeded extends BentenError {
  static readonly code = "E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED";
  static readonly fixHint = "D20-RESOLVED: `AttributionFrame.sandbox_depth: u8` saturating-counter, INHERITED across CALL boundaries (not reset). Default max nest 4. The counter sits on the AttributionFrame so SANDBOX → CALL → SANDBOX → CALL → SANDBOX chains accumulate cumulatively. Flatten the chain or audit for accidental recursion.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_NESTED_DISPATCH_DEPTH_EXCEEDED", "D20-RESOLVED: `AttributionFrame.sandbox_depth: u8` saturating-counter, INHERITED across CALL boundaries (not reset). Default max nest 4. The counter sits on the AttributionFrame so SANDBOX → CALL → SANDBOX → CALL → SANDBOX chains accumulate cumulatively. Flatten the chain or audit for accidental recursion.", message, context);
    this.name = "ESandboxNestedDispatchDepthExceeded";
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

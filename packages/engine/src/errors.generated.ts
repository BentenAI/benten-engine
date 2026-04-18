// AUTO-GENERATED from docs/ERROR-CATALOG.md by scripts/codegen-errors.ts.
// DO NOT EDIT BY HAND. Run `npx tsx scripts/codegen-errors.ts` to regenerate.
//
// Each error class below corresponds to one `### E_XXX` entry in the
// catalog. The class carries a static `code`, a static `fixHint`, and
// exposes them as instance properties so `err.code` / `err.fixHint`
// work on any thrown instance. The drift-detect script asserts this
// file stays in sync with the catalog and the Rust `ErrorCode` enum
// at `crates/benten-core/src/error_code.rs`.

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
  "E_INV_SANDBOX_NESTED",
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
  "E_SANDBOX_FUEL_EXHAUSTED",
  "E_SANDBOX_TIMEOUT",
  "E_SANDBOX_OUTPUT_LIMIT",
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
  "E_DSL_INVALID_SHAPE",
  "E_DSL_UNREGISTERED_HANDLER",
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
 * E_INV_SANDBOX_NESTED
 *
 * Thrown at: Registration
 * Message template: "SANDBOX Node {node_id} calls another SANDBOX, nesting depth {depth} exceeds max {max}"
 */
export class EInvSandboxNested extends BentenError {
  static readonly code = "E_INV_SANDBOX_NESTED";
  static readonly fixHint = "SANDBOX should not call SANDBOX. Flatten or use CALL with a SANDBOX-terminated subgraph.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SANDBOX_NESTED", "SANDBOX should not call SANDBOX. Flatten or use CALL with a SANDBOX-terminated subgraph.", message, context);
    this.name = "EInvSandboxNested";
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
 * Thrown at: Registration
 * Message template: "Node {node_id} references system-zone label '{label}', unreachable from user operations"
 */
export class EInvSystemZone extends BentenError {
  static readonly code = "E_INV_SYSTEM_ZONE";
  static readonly fixHint = "System-zone labels are reserved for engine internals. Use a non-reserved label.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_SYSTEM_ZONE", "System-zone labels are reserved for engine internals. Use a non-reserved label.", message, context);
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
 * Thrown at: Evaluation (Phase 1 flat budget) / Registration (Phase 2 multiplicative-through-CALL)
 * Message template: "Cumulative iteration budget {actual} exceeds max {max} through nested ITERATE/CALL"
 */
export class EInvIterateBudget extends BentenError {
  static readonly code = "E_INV_ITERATE_BUDGET";
  static readonly fixHint = "Reduce the multiplicative iteration space. Total iterations across nested ITERATE/CALL is bounded by the capability grant.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_ITERATE_BUDGET", "Reduce the multiplicative iteration space. Total iterations across nested ITERATE/CALL is bounded by the capability grant.", message, context);
    this.name = "EInvIterateBudget";
  }
}

/**
 * E_INV_ITERATE_NEST_DEPTH
 *
 * Thrown at: Registration
 * Message template: "ITERATE nesting depth {depth} exceeds Phase 1 limit {max}"
 */
export class EInvIterateNestDepth extends BentenError {
  static readonly code = "E_INV_ITERATE_NEST_DEPTH";
  static readonly fixHint = "Phase 1 bounds ITERATE nesting structurally at depth 3 as a stopgap for the cumulative-budget enforcement coming in Phase 2. Flatten the nested iteration, or split into multiple CALL-connected subgraphs.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_INV_ITERATE_NEST_DEPTH", "Phase 1 bounds ITERATE nesting structurally at depth 3 as a stopgap for the cumulative-budget enforcement coming in Phase 2. Flatten the nested iteration, or split into multiple CALL-connected subgraphs.", message, context);
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
 * Thrown at: Evaluation (CAS WRITE)
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
 * E_SANDBOX_FUEL_EXHAUSTED
 *
 * Thrown at: Evaluation
 * Message template: "SANDBOX exhausted fuel budget {budget} before completion"
 */
export class ESandboxFuelExhausted extends BentenError {
  static readonly code = "E_SANDBOX_FUEL_EXHAUSTED";
  static readonly fixHint = "Increase fuel budget (via capability), or reduce computational complexity. Fuel is per-subgraph, not per-call.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_FUEL_EXHAUSTED", "Increase fuel budget (via capability), or reduce computational complexity. Fuel is per-subgraph, not per-call.", message, context);
    this.name = "ESandboxFuelExhausted";
  }
}

/**
 * E_SANDBOX_TIMEOUT
 *
 * Thrown at: Evaluation
 * Message template: "SANDBOX exceeded wall-clock timeout {timeout}ms"
 */
export class ESandboxTimeout extends BentenError {
  static readonly code = "E_SANDBOX_TIMEOUT";
  static readonly fixHint = "Increase timeout or split into smaller SANDBOX calls.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_TIMEOUT", "Increase timeout or split into smaller SANDBOX calls.", message, context);
    this.name = "ESandboxTimeout";
  }
}

/**
 * E_SANDBOX_OUTPUT_LIMIT
 *
 * Thrown at: Evaluation
 * Message template: "SANDBOX output {actual} bytes exceeds max {max}"
 */
export class ESandboxOutputLimit extends BentenError {
  static readonly code = "E_SANDBOX_OUTPUT_LIMIT";
  static readonly fixHint = "Return smaller output. Use STREAM for progressive output.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_SANDBOX_OUTPUT_LIMIT", "Return smaller output. Use STREAM for progressive output.", message, context);
    this.name = "ESandboxOutputLimit";
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
  static readonly fixHint = "The TRANSFORM expression language is a positive-allowlist subset of JavaScript. Any token or AST shape not in the published grammar (`docs/TRANSFORM-GRAMMAR.md`) is rejected. Common causes: closures, `this`, imports, template literals with expressions, tagged templates, optional-chained method calls, computed property names referencing `__proto__`/`constructor`/`Symbol.*`, `new`/`with`/`eval`/`yield`/`async`/`await`, destructuring with getters. See the grammar doc's \"Rejected constructs\" appendix.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_TRANSFORM_SYNTAX", "The TRANSFORM expression language is a positive-allowlist subset of JavaScript. Any token or AST shape not in the published grammar (`docs/TRANSFORM-GRAMMAR.md`) is rejected. Common causes: closures, `this`, imports, template literals with expressions, tagged templates, optional-chained method calls, computed property names referencing `__proto__`/`constructor`/`Symbol.*`, `new`/`with`/`eval`/`yield`/`async`/`await`, destructuring with getters. See the grammar doc's \"Rejected constructs\" appendix.", message, context);
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
  static readonly fixHint = "Phase 1 accepts only base32-lower-nopad multibase (`b`-prefixed) CIDv1. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_CID_PARSE", "Phase 1 accepts only base32-lower-nopad multibase (`b`-prefixed) CIDv1. Check that the caller is not passing a base58btc / base64 / hex form, and that the bytes are not truncated.", message, context);
    this.name = "ECidParse";
  }
}

/**
 * E_CID_UNSUPPORTED_CODEC
 *
 * Thrown at: CID deserialization
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
 * Thrown at: CID deserialization
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
  static readonly fixHint = "Check spelling; register via `ctx.registerSubgraphs()` or `crud()`.";
  constructor(message: string, context?: Record<string, unknown>) {
    super("E_DSL_UNREGISTERED_HANDLER", "Check spelling; register via `ctx.registerSubgraphs()` or `crud()`.", message, context);
    this.name = "EDslUnregisteredHandler";
  }
}

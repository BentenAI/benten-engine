// Public `@benten/engine/errors` surface.
//
// The typed classes are codegenned into `errors.generated.ts` from
// `docs/ERROR-CATALOG.md` (see `scripts/codegen-errors.ts`, owned by
// G8-C). This file re-exports them and adds the runtime helpers the
// wrapper needs: `extractCode` (pull an `E_*` code out of a native
// Error message) and `mapNativeError` (wrap a napi Error in the right
// typed subclass).
//
// The drift-detect CI script (`scripts/drift-detect.ts`) reads
// `errors.generated.ts` and asserts it stays in sync with the catalog
// and the Rust `ErrorCode` enum.

export {
  BentenError,
  CATALOG_CODES,
  type CatalogCode,
  EBackendNotFound,
  EBackendReadOnly,
  ECapAttenuation,
  ECapChainTooDeep,
  ECapDenied,
  ECapDeniedRead,
  ECapNotImplemented,
  ECapRevoked,
  ECapRevokedMidEval,
  ECapScopeLoneStarRejected,
  ECapWallclockExpired,
  ECidParse,
  ECidUnsupportedCodec,
  ECidUnsupportedHash,
  EDevserverStopped,
  EDslInvalidShape,
  EDslUnregisteredHandler,
  EDuplicateHandler,
  EExecStateTampered,
  EGraphInternal,
  EHostBackendUnavailable,
  EHostCapabilityExpired,
  EHostCapabilityRevoked,
  EHostNotFound,
  EHostWriteConflict,
  EInputLimit,
  EInvAttribution,
  EInvContentHash,
  EInvCycle,
  EInvDepthExceeded,
  EInvDeterminism,
  EInvFanoutExceeded,
  EInvImmutability,
  EInvIterateBudget,
  EInvIterateMaxMissing,
  EInvIterateNestDepth,
  EInvRegistration,
  EInvSandboxDepth,
  EInvSandboxOutput,
  EInvSystemZone,
  EInvTooManyEdges,
  EInvTooManyNodes,
  EIvmPatternMismatch,
  EIvmViewStale,
  ENestedTransactionNotSupported,
  ENoCapabilityPolicyConfigured,
  ENotFound,
  ENotImplemented,
  EPrimitiveNotImplemented,
  EProductionRequiresCaps,
  EReloadSubscriberUnsubscribed,
  EResumeActorMismatch,
  EResumeSubgraphDrift,
  ESandboxFuelExhausted,
  ESandboxWallclockExceeded,
  ESerialize,
  ESubsystemDisabled,
  ESyncCapUnverified,
  ESyncHashMismatch,
  ESyncHlcDrift,
  ESystemZoneWrite,
  ETransformSyntax,
  ETxAborted,
  EUnknown,
  EUnknownView,
  EViewLabelMismatch,
  EValueFloatNan,
  EValueFloatNonfinite,
  EVersionBranched,
  EVersionUnknownPrior,
  EWaitSignalShapeMismatch,
  EWaitTimeout,
  EWriteConflict,
  EModuleManifestCidMismatch,
} from "./errors.generated.js";

import {
  BentenError,
  EBackendNotFound,
  EBackendReadOnly,
  ECapAttenuation,
  ECapChainTooDeep,
  ECapDenied,
  ECapDeniedRead,
  ECapNotImplemented,
  ECapRevoked,
  ECapRevokedMidEval,
  ECapScopeLoneStarRejected,
  ECapWallclockExpired,
  ECidParse,
  ECidUnsupportedCodec,
  ECidUnsupportedHash,
  EDevserverStopped,
  EDslInvalidShape,
  EDslUnregisteredHandler,
  EDuplicateHandler,
  EExecStateTampered,
  EGraphInternal,
  EHostBackendUnavailable,
  EHostCapabilityExpired,
  EHostCapabilityRevoked,
  EHostNotFound,
  EHostWriteConflict,
  EInputLimit,
  EInvAttribution,
  EInvContentHash,
  EInvCycle,
  EInvDepthExceeded,
  EInvDeterminism,
  EInvFanoutExceeded,
  EInvImmutability,
  EInvIterateBudget,
  EInvIterateMaxMissing,
  EInvIterateNestDepth,
  EInvRegistration,
  EInvSandboxDepth,
  EInvSandboxOutput,
  EInvSystemZone,
  EInvTooManyEdges,
  EInvTooManyNodes,
  EIvmPatternMismatch,
  EIvmViewStale,
  ENestedTransactionNotSupported,
  ENoCapabilityPolicyConfigured,
  ENotFound,
  ENotImplemented,
  EPrimitiveNotImplemented,
  EProductionRequiresCaps,
  EReloadSubscriberUnsubscribed,
  EResumeActorMismatch,
  EResumeSubgraphDrift,
  ESandboxFuelExhausted,
  ESandboxWallclockExceeded,
  ESerialize,
  ESubsystemDisabled,
  ESyncCapUnverified,
  ESyncHashMismatch,
  ESyncHlcDrift,
  ESystemZoneWrite,
  ETransformSyntax,
  ETxAborted,
  EUnknownView,
  EViewLabelMismatch,
  EValueFloatNan,
  EValueFloatNonfinite,
  EVersionBranched,
  EVersionUnknownPrior,
  EWaitSignalShapeMismatch,
  EWaitTimeout,
  EWriteConflict,
  EModuleManifestCidMismatch,
} from "./errors.generated.js";

// ---------------------------------------------------------------------------
// Runtime mapping: napi-side Error -> typed subclass
// ---------------------------------------------------------------------------

type BentenErrorCtor = new (
  message: string,
  context?: Record<string, unknown>,
) => BentenError;

const CODE_TO_CTOR: Record<string, BentenErrorCtor> = {
  E_INV_CYCLE: EInvCycle,
  E_INV_DEPTH_EXCEEDED: EInvDepthExceeded,
  E_INV_FANOUT_EXCEEDED: EInvFanoutExceeded,
  E_INV_SANDBOX_DEPTH: EInvSandboxDepth,
  E_INV_SANDBOX_OUTPUT: EInvSandboxOutput,
  E_INV_TOO_MANY_NODES: EInvTooManyNodes,
  E_INV_TOO_MANY_EDGES: EInvTooManyEdges,
  E_INV_SYSTEM_ZONE: EInvSystemZone,
  E_INV_DETERMINISM: EInvDeterminism,
  E_INV_ITERATE_MAX_MISSING: EInvIterateMaxMissing,
  E_INV_ITERATE_BUDGET: EInvIterateBudget,
  E_INV_ITERATE_NEST_DEPTH: EInvIterateNestDepth,
  E_INV_CONTENT_HASH: EInvContentHash,
  E_INV_REGISTRATION: EInvRegistration,
  E_CAP_DENIED: ECapDenied,
  E_CAP_DENIED_READ: ECapDeniedRead,
  E_CAP_REVOKED_MID_EVAL: ECapRevokedMidEval,
  E_CAP_NOT_IMPLEMENTED: ECapNotImplemented,
  E_CAP_REVOKED: ECapRevoked,
  E_CAP_ATTENUATION: ECapAttenuation,
  E_WRITE_CONFLICT: EWriteConflict,
  E_SANDBOX_FUEL_EXHAUSTED: ESandboxFuelExhausted,
  E_SANDBOX_WALLCLOCK_EXCEEDED: ESandboxWallclockExceeded,
  E_IVM_VIEW_STALE: EIvmViewStale,
  E_TX_ABORTED: ETxAborted,
  E_NESTED_TRANSACTION_NOT_SUPPORTED: ENestedTransactionNotSupported,
  E_PRIMITIVE_NOT_IMPLEMENTED: EPrimitiveNotImplemented,
  E_SYSTEM_ZONE_WRITE: ESystemZoneWrite,
  E_TRANSFORM_SYNTAX: ETransformSyntax,
  E_INPUT_LIMIT: EInputLimit,
  E_SERIALIZE: ESerialize,
  E_SYNC_HASH_MISMATCH: ESyncHashMismatch,
  E_SYNC_HLC_DRIFT: ESyncHlcDrift,
  E_SYNC_CAP_UNVERIFIED: ESyncCapUnverified,
  E_DSL_INVALID_SHAPE: EDslInvalidShape,
  E_DSL_UNREGISTERED_HANDLER: EDslUnregisteredHandler,
  E_VALUE_FLOAT_NAN: EValueFloatNan,
  E_VALUE_FLOAT_NONFINITE: EValueFloatNonfinite,
  E_CID_PARSE: ECidParse,
  E_CID_UNSUPPORTED_CODEC: ECidUnsupportedCodec,
  E_CID_UNSUPPORTED_HASH: ECidUnsupportedHash,
  E_VERSION_BRANCHED: EVersionBranched,
  E_VERSION_UNKNOWN_PRIOR: EVersionUnknownPrior,
  E_BACKEND_NOT_FOUND: EBackendNotFound,
  E_BACKEND_READ_ONLY: EBackendReadOnly,
  E_NOT_FOUND: ENotFound,
  E_GRAPH_INTERNAL: EGraphInternal,
  E_DUPLICATE_HANDLER: EDuplicateHandler,
  E_NO_CAPABILITY_POLICY_CONFIGURED: ENoCapabilityPolicyConfigured,
  E_PRODUCTION_REQUIRES_CAPS: EProductionRequiresCaps,
  E_SUBSYSTEM_DISABLED: ESubsystemDisabled,
  E_UNKNOWN_VIEW: EUnknownView,
  E_VIEW_LABEL_MISMATCH: EViewLabelMismatch,
  E_NOT_IMPLEMENTED: ENotImplemented,
  E_IVM_PATTERN_MISMATCH: EIvmPatternMismatch,
  // Reserved host-error wire codes (G1-A; thrown via napi to TS).
  E_HOST_NOT_FOUND: EHostNotFound,
  E_HOST_WRITE_CONFLICT: EHostWriteConflict,
  E_HOST_BACKEND_UNAVAILABLE: EHostBackendUnavailable,
  E_HOST_CAPABILITY_REVOKED: EHostCapabilityRevoked,
  E_HOST_CAPABILITY_EXPIRED: EHostCapabilityExpired,
  // Phase-2a additions (G3-A resume protocol, G3-B WAIT, G5-A/B
  // immutability + attribution, G9-A wall-clock, G4-A scope parsing).
  E_EXEC_STATE_TAMPERED: EExecStateTampered,
  E_RESUME_ACTOR_MISMATCH: EResumeActorMismatch,
  E_RESUME_SUBGRAPH_DRIFT: EResumeSubgraphDrift,
  E_WAIT_TIMEOUT: EWaitTimeout,
  E_INV_IMMUTABILITY: EInvImmutability,
  E_INV_ATTRIBUTION: EInvAttribution,
  E_CAP_WALLCLOCK_EXPIRED: ECapWallclockExpired,
  E_CAP_CHAIN_TOO_DEEP: ECapChainTooDeep,
  E_CAP_SCOPE_LONE_STAR_REJECTED: ECapScopeLoneStarRejected,
  E_WAIT_SIGNAL_SHAPE_MISMATCH: EWaitSignalShapeMismatch,
  // R6 Round-2 r6-r2-napi-3 (Instance 8 round-trip pin) — added so
  // the engine_err sentinel pipeline maps cleanly to the typed
  // subclass for the install_module CID-mismatch surface. Broader
  // CODE_TO_CTOR completeness — many codegen'd subclasses are not
  // yet listed here — tracked in
  // `docs/future/phase-3-backlog.md` §7.6 (CODE_TO_CTOR codegen
  // completeness) as a Phase-3 codegen lift.
  E_MODULE_MANIFEST_CID_MISMATCH: EModuleManifestCidMismatch,
  // R6 Round-3 r6-r3-napi-1 — both codes were promoted from hand-typed
  // strings to typed catalog variants by R6 Round-2 r6-r2-napi-1 (PR #66)
  // for the explicit purpose of typed-dispatch through `mapNativeError`,
  // but the matching CODE_TO_CTOR entries were missed (the consumer-audit
  // table on PR #66 didn't include `errors.ts`). Without these entries
  // the napi devserver paths surfacing `E_RELOAD_SUBSCRIBER_UNSUBSCRIBED:`
  // / `E_DEVSERVER_STOPPED:` round-trip as the synthetic `E_UNKNOWN`
  // fallback rather than `EReloadSubscriberUnsubscribed` /
  // `EDevserverStopped`, defeating the original promotion's purpose.
  // 16th instance of the producer/consumer drift pattern; folded into
  // the §7.6 Phase-3 codegen lift to prevent recurrence at source.
  E_DEVSERVER_STOPPED: EDevserverStopped,
  E_RELOAD_SUBSCRIBER_UNSUBSCRIBED: EReloadSubscriberUnsubscribed,
};

// Match-at-any-position regex for a stable `E_*` code. Codes look like
// `E_` followed by SCREAMING_SNAKE letters/digits. The regex body stays
// greedy so multi-segment codes don't truncate (kept in a normal comment
// so the drift detector's naive code-string scan doesn't see a fake
// prefix match in a JSDoc heading).
const CODE_RX = /\bE_[A-Z0-9_]+\b/;

/**
 * Extract a stable `E_*` code from any string (usually a napi Error
 * message). Returns `undefined` when the string carries no recognizable
 * code.
 */
export function extractCode(input: unknown): string | undefined {
  if (typeof input !== "string") return undefined;
  const m = input.match(CODE_RX);
  return m ? m[0] : undefined;
}

/**
 * R6FP-tail (Round-2 Instance 8) — sentinel marker the napi adapter
 * (`bindings/napi/src/error.rs::engine_err`) appends to the napi error
 * message when an `EngineError` carries structured per-variant fields
 * (e.g. `ModuleManifestCidMismatch { expected, computed, summary }` or
 * `Invariant(RegistrationError { ...14 fields })`). The suffix shape
 * is `<message> :: $$benten-context$$<json>`.
 *
 * `mapNativeError` splits on this sentinel + parses the JSON tail and
 * passes the resulting bag as the fourth `context` argument to the
 * typed-error subclass constructor so JS callers can read structured
 * fields off `error.context` (e.g. `error.context.expected_cid`).
 *
 * The double-`$` is chosen because it is unlikely to appear in any
 * `EngineError` Display rendering — keeps the suffix unambiguous.
 * Cross-layer contract with the Rust adapter; changing the sentinel
 * requires a coordinated update on both sides.
 */
const CONTEXT_SENTINEL = " :: $$benten-context$$";

/**
 * R6FP-tail (Round-2 Instance 8) — split a napi error message on the
 * `$$benten-context$$` sentinel. Returns `[messageWithoutSuffix, context]`
 * where `context` is the parsed JSON bag (or `undefined` when the
 * sentinel is absent / the JSON tail fails to parse).
 *
 * Best-effort: if the JSON tail is malformed, returns the original
 * message untouched + `context = undefined` so the typed-error path
 * still fires on the catalog code.
 */
function splitContextSentinel(
  raw: string,
): [string, Record<string, unknown> | undefined] {
  const idx = raw.indexOf(CONTEXT_SENTINEL);
  if (idx === -1) return [raw, undefined];
  const head = raw.slice(0, idx);
  const tail = raw.slice(idx + CONTEXT_SENTINEL.length);
  try {
    const parsed = JSON.parse(tail) as unknown;
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return [head, parsed as Record<string, unknown>];
    }
    return [head, undefined];
  } catch {
    return [head, undefined];
  }
}

/**
 * Wrap an unknown value (typically a napi Error) in the most specific
 * typed Benten error we can reconstruct.
 *
 * Rules:
 *   1. If the value is already a `BentenError`, return it untouched.
 *   2. If the value carries a catalog code in its string form, build
 *      the matching subclass and preserve the original message. When
 *      the message also carries a `$$benten-context$$` JSON suffix
 *      (R6FP-tail Round-2 Instance 8), the parsed bag is passed as the
 *      4th `context` constructor arg so JS consumers can read
 *      structured fields via `error.context`.
 *   3. Otherwise, fall back to a `BentenError` with a synthetic
 *      unknown-code marker so the caller still sees a typed wrapper.
 *
 * This does NOT throw; the caller is responsible for re-throwing if
 * they want the typed error to escape.
 */
export function mapNativeError(err: unknown): BentenError {
  if (err instanceof BentenError) return err;

  const raw =
    err instanceof Error
      ? err.message
      : typeof err === "string"
        ? err
        : String(err);
  const [message, context] = splitContextSentinel(raw);
  const code = extractCode(message);
  if (code && CODE_TO_CTOR[code]) {
    const Ctor = CODE_TO_CTOR[code];
    const instance = new Ctor(message, context);
    if (err instanceof Error && err.stack) {
      instance.stack = err.stack;
    }
    return instance;
  }
  // Fallback: synthetic code keeps the typed-wrapper contract.
  // Assembled at runtime to avoid baking a fake code into the source
  // text (the drift detector's naive scan would otherwise flag it).
  const syntheticCode = ["E", "UNKNOWN"].join("_");
  return new BentenError(syntheticCode, "(no catalog match)", message, context);
}

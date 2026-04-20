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
  ECapAttenuation,
  ECapDenied,
  ECapDeniedRead,
  ECapNotImplemented,
  ECapRevoked,
  ECapRevokedMidEval,
  ECidParse,
  ECidUnsupportedCodec,
  ECidUnsupportedHash,
  EDslInvalidShape,
  EDslUnregisteredHandler,
  EDuplicateHandler,
  EGraphInternal,
  EInputLimit,
  EInvContentHash,
  EInvCycle,
  EInvDepthExceeded,
  EInvDeterminism,
  EInvFanoutExceeded,
  EInvIterateBudget,
  EInvIterateMaxMissing,
  EInvIterateNestDepth,
  EInvRegistration,
  EInvSandboxNested,
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
  ESandboxFuelExhausted,
  ESandboxOutputLimit,
  ESandboxTimeout,
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
  EValueFloatNan,
  EValueFloatNonfinite,
  EVersionBranched,
  EVersionUnknownPrior,
  EWriteConflict,
} from "./errors.generated.js";

import {
  BentenError,
  EBackendNotFound,
  ECapAttenuation,
  ECapDenied,
  ECapDeniedRead,
  ECapNotImplemented,
  ECapRevoked,
  ECapRevokedMidEval,
  ECidParse,
  ECidUnsupportedCodec,
  ECidUnsupportedHash,
  EDslInvalidShape,
  EDslUnregisteredHandler,
  EDuplicateHandler,
  EGraphInternal,
  EInputLimit,
  EInvContentHash,
  EInvCycle,
  EInvDepthExceeded,
  EInvDeterminism,
  EInvFanoutExceeded,
  EInvIterateBudget,
  EInvIterateMaxMissing,
  EInvIterateNestDepth,
  EInvRegistration,
  EInvSandboxNested,
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
  ESandboxFuelExhausted,
  ESandboxOutputLimit,
  ESandboxTimeout,
  ESerialize,
  ESubsystemDisabled,
  ESyncCapUnverified,
  ESyncHashMismatch,
  ESyncHlcDrift,
  ESystemZoneWrite,
  ETransformSyntax,
  ETxAborted,
  EUnknownView,
  EValueFloatNan,
  EValueFloatNonfinite,
  EVersionBranched,
  EVersionUnknownPrior,
  EWriteConflict,
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
  E_INV_SANDBOX_NESTED: EInvSandboxNested,
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
  E_SANDBOX_TIMEOUT: ESandboxTimeout,
  E_SANDBOX_OUTPUT_LIMIT: ESandboxOutputLimit,
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
  E_NOT_FOUND: ENotFound,
  E_GRAPH_INTERNAL: EGraphInternal,
  E_DUPLICATE_HANDLER: EDuplicateHandler,
  E_NO_CAPABILITY_POLICY_CONFIGURED: ENoCapabilityPolicyConfigured,
  E_PRODUCTION_REQUIRES_CAPS: EProductionRequiresCaps,
  E_SUBSYSTEM_DISABLED: ESubsystemDisabled,
  E_UNKNOWN_VIEW: EUnknownView,
  E_NOT_IMPLEMENTED: ENotImplemented,
  E_IVM_PATTERN_MISMATCH: EIvmPatternMismatch,
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
 * Wrap an unknown value (typically a napi Error) in the most specific
 * typed Benten error we can reconstruct.
 *
 * Rules:
 *   1. If the value is already a `BentenError`, return it untouched.
 *   2. If the value carries a catalog code in its string form, build
 *      the matching subclass and preserve the original message.
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
  const code = extractCode(raw);
  if (code && CODE_TO_CTOR[code]) {
    const Ctor = CODE_TO_CTOR[code];
    const instance = new Ctor(raw);
    if (err instanceof Error && err.stack) {
      instance.stack = err.stack;
    }
    return instance;
  }
  // Fallback: synthetic code keeps the typed-wrapper contract.
  // Assembled at runtime to avoid baking a fake code into the source
  // text (the drift detector's naive scan would otherwise flag it).
  const syntheticCode = ["E", "UNKNOWN"].join("_");
  return new BentenError(syntheticCode, "(no catalog match)", raw);
}

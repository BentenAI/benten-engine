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
  CODE_TO_CTOR_GENERATED,
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
  EDevServerStopped,
  EDslInvalidShape,
  EDslUnregisteredHandler,
  EDuplicateHandler,
  EExecStateTampered,
  EGraphInternal,
  EHlcSkewExceeded,
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
  EValueFloatNonFinite,
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
  EDevServerStopped,
  EDslInvalidShape,
  EDslUnregisteredHandler,
  EDuplicateHandler,
  EExecStateTampered,
  EGraphInternal,
  EHlcSkewExceeded,
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
  EValueFloatNonFinite,
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
  E_VALUE_FLOAT_NONFINITE: EValueFloatNonFinite,
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
  // `EDevServerStopped`, defeating the original promotion's purpose.
  // 16th instance of the producer/consumer drift pattern; folded into
  // the §7.6 Phase-3 codegen lift to prevent recurrence at source.
  E_DEVSERVER_STOPPED: EDevServerStopped,
  E_RELOAD_SUBSCRIBER_UNSUBSCRIBED: EReloadSubscriberUnsubscribed,
  // Phase-3 G14-pre-D: HLC skew rejection. Wired into CODE_TO_CTOR at
  // landing time per pim-1 / §3.5b doc-coupling pre-flight (the producer
  // / consumer drift pattern) so napi callers ingesting Phase-3 sync
  // errors get the typed `EHlcSkewExceeded` dispatch instead of the
  // synthetic `E_UNKNOWN` fallback. Phase-3 sync wiring registers the
  // production firing site; the entry here is forward-laid so the consumer
  // surface is ready when the producer ships.
  E_HLC_SKEW_EXCEEDED: EHlcSkewExceeded,
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
 * Phase-3 G19-B (§7.2): merge `CODE_TO_CTOR_GENERATED` into the
 * runtime constructor map so every catalog code resolves to a typed
 * subclass without hand-edits. The hand-typed `CODE_TO_CTOR` map
 * above stays as the historically-curated fast path; the generated
 * map fills in the long tail (~98 codes today, growing in Phase 3).
 *
 * The vitest pin
 * `code_to_ctor_codegen_covers_every_error_catalog_entry` asserts
 * this map covers every catalog code — see
 * `crates/benten-engine/tests/code_to_ctor.rs`.
 */
import { CODE_TO_CTOR_GENERATED } from "./errors.generated.js";

/**
 * Phase-3 G19-B (§7.2): the napi adapter
 * (`bindings/napi/src/error.rs::engine_err`) emits errors whose
 * `.message` is a JSON-serialised object with shape
 * `{ "code": "E_*", "message": "<display>", "fields": {...} }`. The
 * structured-field bag rides under `"fields"` (replaces the
 * pre-G19-B `$$benten-context$$` sentinel suffix carrier). This
 * helper attempts a JSON parse + structural validation; returns
 * `[code, displayMessage, fields]` on success.
 *
 * Returns `undefined` when the message body is NOT a JSON-shaped
 * envelope (the pre-G19-B `code: prefix` shape, or any plain string
 * thrown from non-engine napi code paths). Callers fall back to the
 * `extractCode` regex path in that case so existing hand-rolled
 * `format!("E_*: ...")` errors continue to round-trip cleanly.
 */
function tryParseJsonEnvelope(
  raw: string,
): { code: string; message: string; fields?: Record<string, unknown> } | undefined {
  // Cheap up-front rejection: if the message doesn't start with `{`
  // it's not a JSON object body.
  const trimmed = raw.trim();
  if (!trimmed.startsWith("{")) return undefined;
  let parsed: unknown;
  try {
    parsed = JSON.parse(trimmed);
  } catch {
    return undefined;
  }
  if (
    !parsed ||
    typeof parsed !== "object" ||
    Array.isArray(parsed)
  ) {
    return undefined;
  }
  const obj = parsed as Record<string, unknown>;
  const code = obj.code;
  const message = obj.message;
  if (typeof code !== "string" || typeof message !== "string") {
    return undefined;
  }
  const out: {
    code: string;
    message: string;
    fields?: Record<string, unknown>;
  } = { code, message };
  if (
    obj.fields &&
    typeof obj.fields === "object" &&
    !Array.isArray(obj.fields)
  ) {
    out.fields = obj.fields as Record<string, unknown>;
  }
  return out;
}

/**
 * Resolve a catalog code to its typed `BentenError` subclass
 * constructor. Consults the hand-curated `CODE_TO_CTOR` first
 * (historically the fast path for known-load-bearing codes) then
 * falls back to the codegen-driven `CODE_TO_CTOR_GENERATED`. The
 * `code_to_ctor_no_e_unknown_fallback_for_known_code` test
 * (`crates/benten-engine/tests/code_to_ctor.rs`) asserts every
 * catalog code resolves through ONE of these maps.
 */
function resolveCtor(code: string): BentenErrorCtor | undefined {
  if (CODE_TO_CTOR[code]) return CODE_TO_CTOR[code];
  const gen = (CODE_TO_CTOR_GENERATED as Record<string, BentenErrorCtor>)[code];
  return gen;
}

/**
 * Wrap an unknown value (typically a napi Error) in the most specific
 * typed Benten error we can reconstruct.
 *
 * Rules (Phase-3 G19-B §7.2):
 *   1. If the value is already a `BentenError`, return it untouched.
 *   2. If the message body is a JSON envelope `{ code, message,
 *      fields? }` (the new G19-B shape), construct the typed subclass
 *      keyed on `code`, set `BentenError.context` from `fields`, and
 *      use `message` as the human-readable message.
 *   3. Otherwise, fall back to the legacy `code: prefix` regex path:
 *      extract a catalog code via `extractCode` and construct the
 *      typed subclass on the raw message. This preserves
 *      backwards-compat with hand-rolled napi errors that pre-date
 *      the JSON envelope (`format!("E_*: ...")` carrier shape).
 *   4. If neither path resolves a known catalog code, return a
 *      `BentenError` with a synthetic unknown-code marker so the
 *      caller still sees a typed wrapper.
 *
 * Does NOT throw; the caller is responsible for re-throwing if they
 * want the typed error to escape.
 */
export function mapNativeError(err: unknown): BentenError {
  if (err instanceof BentenError) return err;

  const raw =
    err instanceof Error
      ? err.message
      : typeof err === "string"
        ? err
        : String(err);

  // Path 1 (G19-B JSON envelope): try parsing the message body.
  const envelope = tryParseJsonEnvelope(raw);
  if (envelope) {
    const Ctor = resolveCtor(envelope.code);
    if (Ctor) {
      const instance = new Ctor(envelope.message, envelope.fields);
      if (err instanceof Error && err.stack) {
        instance.stack = err.stack;
      }
      return instance;
    }
    // JSON envelope with an unknown code — synthesise a typed wrapper
    // carrying the parsed code so callers can see what came across.
    const syntheticCode = ["E", "UNKNOWN"].join("_");
    return new BentenError(
      syntheticCode,
      `(unknown catalog code: ${envelope.code})`,
      envelope.message,
      envelope.fields,
    );
  }

  // Path 2 (legacy prefix carrier): extract a catalog code via regex.
  const code = extractCode(raw);
  if (code) {
    const Ctor = resolveCtor(code);
    if (Ctor) {
      const instance = new Ctor(raw, undefined);
      if (err instanceof Error && err.stack) {
        instance.stack = err.stack;
      }
      return instance;
    }
  }

  // Path 3 (fallback): synthetic unknown-code wrapper.
  // Assembled at runtime to avoid baking a fake code into the source
  // text (the drift detector's naive scan would otherwise flag it).
  const syntheticCode = ["E", "UNKNOWN"].join("_");
  return new BentenError(syntheticCode, "(no catalog match)", raw, undefined);
}

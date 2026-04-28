// `@benten/engine/internal/trace` — wrapper-side TraceStep projection.
//
// This subpath is a deliberately narrow internal surface (Phase-2b R4
// ts-r4-1 + G12-F per D14-RESOLVED) that exposes ONLY the pieces a
// unit test needs to drive `mapTraceStep` directly without spinning up
// a full Rust engine round-trip:
//
//   - `mapTraceStep`          — the actual projection used by `engine.ts`
//   - `mapTraceStepForTest`   — alias re-export, kept stable for tests
//   - `resetUnknownDiscriminantWarningsForTest`
//                             — clears the module-scope dedupe set so
//                               tests that exercise the warning path
//                               don't bleed into one another
//
// The "internal" subpath name is the contract — production callers
// MUST consume `engine.trace()` and friends from `@benten/engine`. We
// route the engine wrapper through this module so there is exactly one
// implementation; the test shim is the same code under test.
//
// D14-RESOLVED behaviour (replaces the prior loud-fail at engine.ts):
//   - Unknown `type` discriminator → returns a typed
//     `{ type: "unknown", discriminant, raw }` row (TraceStepUnknown).
//   - `console.warn` fires ONCE per discriminant per process; the
//     dedupe Set lives in module scope.
//   - Known variants route exactly as before.

import type {
  AttributionFrame,
  JsonValue,
  TraceStep,
  TraceStepUnknown,
} from "../types.js";

// Module-scope dedupe set. One entry per `discriminant` string seen.
// Reset by `resetUnknownDiscriminantWarningsForTest` for unit-test
// isolation; production code never resets it (one-shot-per-process
// is the contract).
const seenUnknownDiscriminants = new Set<string>();

function readAttribution(raw: unknown): AttributionFrame | undefined {
  if (raw === null || typeof raw !== "object") return undefined;
  const r = raw as Record<string, unknown>;
  const actorCid = typeof r.actorCid === "string" ? r.actorCid : undefined;
  const handlerCid = typeof r.handlerCid === "string" ? r.handlerCid : undefined;
  const capabilityGrantCid =
    typeof r.capabilityGrantCid === "string" ? r.capabilityGrantCid : undefined;
  if (!actorCid || !handlerCid || !capabilityGrantCid) return undefined;
  return { actorCid, handlerCid, capabilityGrantCid };
}

/**
 * Project a single native trace row into the typed `TraceStep` union.
 * Forward-compat per D14: unknown discriminants surface as a typed
 * `TraceStepUnknown` row + a one-shot `console.warn`, never throw.
 */
export function mapTraceStep(s: Record<string, unknown>): TraceStep {
  const t = typeof s.type === "string" ? s.type : "primitive";
  switch (t) {
    case "suspend_boundary":
      return {
        type: "suspend_boundary",
        stateCid: String(s.stateCid ?? ""),
      };
    case "resume_boundary":
      return {
        type: "resume_boundary",
        stateCid: String(s.stateCid ?? ""),
        signalValue: (s.signalValue ?? null) as JsonValue,
      };
    case "budget_exhausted":
      return {
        type: "budget_exhausted",
        budgetType: String(s.budgetType ?? ""),
        consumed: Number(s.consumed ?? 0),
        limit: Number(s.limit ?? 0),
        path: Array.isArray(s.path) ? (s.path as unknown[]).map(String) : [],
      };
    case "primitive":
      return {
        type: "primitive",
        nodeCid: String(s.nodeCid ?? ""),
        primitive: String(s.primitive ?? ""),
        // Native durationUs is an integer microsecond reading; a genuine
        // zero is possible for ultra-fast steps. The trace contract
        // asserts `> 0`; fall back to 1 to keep the contract honest
        // without lying about timing (the step DID execute).
        durationUs: Math.max(1, Number(s.durationUs ?? 0)),
        nodeId: String(s.nodeId ?? ""),
        inputs: s.inputs as JsonValue,
        outputs: s.outputs as JsonValue,
        error: typeof s.error === "string" ? s.error : undefined,
        attribution: readAttribution(s.attribution),
      };
    default: {
      // Phase-2b D14-RESOLVED: warning-passthrough. An unknown
      // discriminant from a newer native binding indicates a wrapper-
      // version skew. We surface it as a typed `TraceStepUnknown` row
      // (so the trace renders end-to-end + callers can pattern-match)
      // and emit a one-shot console.warn the first time each distinct
      // discriminant is seen so the skew is visible in dev/CI without
      // log-spam. The historical loud-fail path at engine.ts:249-258
      // is intentionally REMOVED.
      if (!seenUnknownDiscriminants.has(t)) {
        seenUnknownDiscriminants.add(t);
        // Keep the message actionable: name the discriminant + the
        // upgrade hint so the developer's first encounter points at
        // the fix (per dx-r1-2b D14 rationale).
        // eslint-disable-next-line no-console
        console.warn(
          `[@benten/engine] Unknown TraceStep discriminant "${t}" — ` +
            `the native binding emitted a variant this version of ` +
            `@benten/engine does not recognize. The row is preserved ` +
            `under TraceStepUnknown.raw. Consider upgrading ` +
            `@benten/engine to a release that knows this variant.`,
        );
      }
      // Drop the `type` key from the preserved `raw` payload so callers
      // pattern-matching on `.raw` see only the variant-specific fields.
      const { type: _ignored, ...rest } = s;
      void _ignored;
      const unknown: TraceStepUnknown = {
        type: "unknown",
        discriminant: t,
        raw: rest,
      };
      return unknown;
    }
  }
}

/**
 * Test-shim alias. Kept named distinctly from `mapTraceStep` so tests
 * read clearly + a future refactor that renames the production export
 * doesn't silently break the test signal.
 */
export const mapTraceStepForTest = mapTraceStep;

/**
 * Clear the module-scope dedupe set. Test-only — production code MUST
 * NOT call this (the one-shot-per-process contract is load-bearing for
 * the "no log-spam" guarantee).
 */
export function resetUnknownDiscriminantWarningsForTest(): void {
  seenUnknownDiscriminants.clear();
}

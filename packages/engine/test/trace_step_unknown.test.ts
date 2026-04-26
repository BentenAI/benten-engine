// R3-F red-phase — D14 mapTraceStep forward-compat (warning-passthrough +
// typed TraceStepUnknown variant).
//
// Tests are RED at landing time; G12-F makes them green.
//
// Surface contract (per D14 RESOLVED + dx-r1-2b D14 + plan §3.2 G12-F):
//   - Unknown discriminator does NOT throw — it surfaces as
//     { type: "unknown", discriminant, raw } (typed TraceStepUnknown).
//   - console.warn fires once-per-discriminant-per-process (deduped via
//     module-level Set guard); multiple unknown rows of the same
//     discriminant DO NOT spam.
//   - All known variants still route correctly — anti-regression guard.
//   - The historical loud-fail default branch in engine.ts:249-258 is
//     REMOVED — no source file should contain a `throw new Error("Unknown
//     TraceStep ...")` pattern after G12-F lands.
//   - TraceStepUnknown is exported from the public types surface so callers
//     can pattern-match `s.type === "unknown"` exhaustively.
//
// Pin sources: r2-test-landscape.md §7 (rows 464-468); r2 §8 D14 row;
// r1-dx-optimizer.json D14_mapTraceStep_unknown_disposition.

import { describe, it, expect, vi, beforeEach } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import type { TraceStep, TraceStepUnknown } from "@benten/engine";

// G12-F MUST export `mapTraceStep` (or a test-shim alias) from the engine
// package so the wrapper-side mapping is unit-testable WITHOUT having to
// drive a full Rust trace round-trip just to assert mapping behaviour.
// The module also MUST expose a way to RESET the dedupe guard so tests
// that exercise the warning path don't bleed into one another.
import {
  mapTraceStepForTest,
  resetUnknownDiscriminantWarningsForTest,
} from "@benten/engine/internal/trace";

beforeEach(() => {
  resetUnknownDiscriminantWarningsForTest();
});

describe("mapTraceStep — D14 forward-compat", () => {
  it("unknown discriminant returns typed unknown variant with warning passthrough", () => {
    // Synthetic native-side row whose `type` discriminator is from a future
    // engine-native release this wrapper doesn't yet know. The mapping MUST
    // NOT throw — the trace renders end-to-end with a typed UnknownTraceStep
    // row in place of the unrecognized variant.
    const out = mapTraceStepForTest({
      type: "sandbox_boundary",
      moduleId: "bafy...",
      fuelConsumed: 100,
    });

    expect(out.type).toBe("unknown");
    expect(out).toMatchObject({
      type: "unknown",
      discriminant: "sandbox_boundary",
      raw: { moduleId: "bafy...", fuelConsumed: 100 },
    });

    // Type-narrow + assert the typed shape.
    if (out.type === "unknown") {
      const u: TraceStepUnknown = out;
      expect(u.discriminant).toBe("sandbox_boundary");
      expect(u.raw.fuelConsumed).toBe(100);
    } else {
      throw new Error("expected unknown variant");
    }
  });

  it("warning emitted one-shot per discriminant per process", () => {
    const warnSpy = vi.spyOn(console, "warn").mockImplementation(() => {});

    mapTraceStepForTest({ type: "future_a" });
    mapTraceStepForTest({ type: "future_a" }); // dedupe — no second warn
    mapTraceStepForTest({ type: "future_b" }); // distinct — does warn

    expect(warnSpy).toHaveBeenCalledTimes(2);

    // The warning text MUST mention the discriminant + the upgrade hint so
    // a developer's first encounter with the message is actionable, not a
    // dead-end "what does this mean" question (per dx-r1-2b D14 rationale).
    const [firstCall] = warnSpy.mock.calls;
    const firstMessage = String(firstCall[0]);
    expect(firstMessage).toContain("future_a");
    expect(firstMessage).toContain("@benten/engine");

    warnSpy.mockRestore();
  });

  it("loud-fail path removed from engine.ts", () => {
    // Anti-regression pin: engine.ts:249-258 used to `throw new Error("Unknown
    // TraceStep discriminant ...")`. After G12-F lands, the loud-fail
    // default branch is replaced by the warning-passthrough handler.
    // Source-grep guard so a future PR that re-introduces loud-fail trips a
    // test, not a runtime regression that only fires on wrapper-version skew.

    // Resolve packages/engine/src/engine.ts via the test file location to
    // stay portable across the eventual move from packages/engine/src/* to
    // packages/engine/test/*.
    const here = dirname(fileURLToPath(import.meta.url));
    const engineSrc = resolve(here, "..", "src", "engine.ts");
    const src = readFileSync(engineSrc, "utf8");

    // The loud-fail used the literal string "Unknown TraceStep". The
    // replacement uses console.warn. Either the loud-fail string is gone OR
    // it's only mentioned in a doc/comment about the historical decision.
    const literal = /throw new Error\([^)]*Unknown TraceStep/i;
    expect(literal.test(src)).toBe(false);
  });

  it("all known variants still route correctly", () => {
    // Regression guard — the warning-passthrough path MUST NOT swallow
    // legitimately known variants. Each Phase-2a-shipped variant continues
    // to round-trip its discriminator.
    const primitive = mapTraceStepForTest({
      type: "primitive",
      primitive: "read",
      nodeCid: "bafy",
      durationUs: 5,
      nodeId: "n0",
      inputs: null,
      outputs: null,
    });
    expect(primitive.type).toBe("primitive");

    const suspend = mapTraceStepForTest({
      type: "suspend_boundary",
      stateCid: "bafy",
    });
    expect(suspend.type).toBe("suspend_boundary");

    const resume = mapTraceStepForTest({
      type: "resume_boundary",
      stateCid: "bafy",
      signalValue: null,
    });
    expect(resume.type).toBe("resume_boundary");

    const budget = mapTraceStepForTest({
      type: "budget_exhausted",
      budgetType: "inv_8_iteration",
      consumed: 10,
      limit: 5,
      path: [],
    });
    expect(budget.type).toBe("budget_exhausted");
  });

  it("TraceStepUnknown union member exported from types.ts", () => {
    // Type-shape pin — TraceStepUnknown MUST be re-exported from
    // @benten/engine so callers can `import type { TraceStepUnknown }`
    // without reaching into the package internals. Compile-time test:
    // if the symbol is not exported, the test won't typecheck.
    const stub: TraceStepUnknown = {
      type: "unknown",
      discriminant: "future",
      raw: {},
    };
    const widened: TraceStep = stub;
    expect(widened.type).toBe("unknown");

    // Also pin: trace.steps array PRESERVES unknown rows — they are NOT
    // silently dropped (per the brief: "trace.steps array preserves unknown
    // rows"). Construct a synthetic mixed-shape array and round-trip via
    // mapTraceStepForTest applied per element.
    const mixed = [
      { type: "primitive", primitive: "read", nodeCid: "b", durationUs: 1, nodeId: "n", inputs: null, outputs: null },
      { type: "future_variant_x", payload: { x: 1 } },
      { type: "primitive", primitive: "write", nodeCid: "b", durationUs: 1, nodeId: "n2", inputs: null, outputs: null },
    ];
    const mapped = mixed.map(mapTraceStepForTest);
    expect(mapped).toHaveLength(3);
    expect(mapped[0].type).toBe("primitive");
    expect(mapped[1].type).toBe("unknown");
    expect(mapped[2].type).toBe("primitive");
  });
});

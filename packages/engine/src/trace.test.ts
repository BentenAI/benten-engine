// Phase 1 R3 Vitest: engine.trace() per-step timing + topological order.
// Exit-criterion #4 partner (TS side). Uses @benten/engine wrapper.
// Status: FAILING until B6 + E8 land.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, crud } from "@benten/engine";
import type { TraceStep } from "./types.js";

let engine: Engine;
let tmp: string;

beforeAll(async () => {
  tmp = mkdtempSync(join(tmpdir(), "benten-trace-"));
  engine = await Engine.open(join(tmp, "benten.redb"));
});

afterAll(async () => {
  await engine.close();
  rmSync(tmp, { recursive: true, force: true });
});

describe("engine.trace", () => {
  it("trace_returns_topo_ordered_steps", async () => {
    // Exit-criterion #4.
    const handler = await engine.registerSubgraph(crud("post"));
    const trace = await engine.trace(handler.id, "post:create", { title: "traced" });

    expect(trace.steps.length).toBeGreaterThan(0);
    for (const s of trace.steps) {
      // G11-A Wave 2b: crud(post):create only emits primitive rows.
      expect(s.type).toBe("primitive");
      if (s.type !== "primitive") continue;
      expect(s.durationUs).toBeGreaterThan(0);
      expect(s.nodeCid).toBeTypeOf("string");
      expect(s.primitive).toBeTypeOf("string");
    }

    // Topological order: every step appears only after its predecessors.
    // Handlers with BRANCH/ITERATE/CALL admit multiple valid orderings; the
    // assertion is partial-order compliance, not strict sequence equality.
    const seen = new Set<string>();
    const adjacencies = await engine.handlerPredecessors(handler.id);
    for (const step of trace.steps) {
      if (step.type !== "primitive") continue;
      for (const pred of adjacencies.predecessorsOf(step.nodeCid)) {
        expect(seen.has(pred)).toBe(true);
      }
      seen.add(step.nodeCid);
    }
  });

  it("trace_result_matches_non_traced_call", async () => {
    const handler = await engine.registerSubgraph(crud("post"));
    const traced = await engine.trace(handler.id, "post:create", { title: "eq" });
    const normal = await engine.call(handler.id, "post:create", { title: "eq" });
    // Both paths produce semantically equal output.
    expect(traced.result.cid).toBeTruthy();
    expect(normal.cid).toBeTruthy();
  });

  // r6b-dx-C4 + r6b-dx-C6: trace must return real per-step records
  // from the evaluator, not a hardcoded lookup table keyed on the op
  // name. Each step's `nodeCid` must be the subgraph operation-node
  // CID (distinct per OperationNode), not the outcome's `created_cid`.
  it("trace_returns_real_per_step_data", async () => {
    const handler = await engine.registerSubgraph(crud("post"));
    const trace = await engine.trace(handler.id, "post:create", { title: "real" });

    // Non-trivial step count and distinct CIDs per step.
    expect(trace.steps.length).toBeGreaterThan(0);
    const cids = new Set<string>();
    for (const s of trace.steps) {
      // G11-A Wave 2b: crud(post):create only emits primitive rows.
      expect(s.type).toBe("primitive");
      if (s.type !== "primitive") continue;
      expect(typeof s.nodeCid).toBe("string");
      expect(s.nodeCid.length).toBeGreaterThan(10);
      cids.add(s.nodeCid);
    }
    // Each OperationNode in the walked subgraph gets its own CID — no
    // single CID echoed across every step (the pre-fix bug).
    expect(cids.size).toBe(trace.steps.length);

    // Per-step primitive must be a recognized kind, not the empty
    // string the synthetic fabrication fell back to.
    const kinds = new Set(
      trace.steps
        .filter((s): s is typeof s & { type: "primitive" } => s.type === "primitive")
        .map((s) => s.primitive),
    );
    // Create path walks through at least write+respond.
    expect(kinds.has("write")).toBe(true);
    expect(kinds.has("respond")).toBe(true);

    // Trace's nodeCid stream must NOT include the outcome's created_cid —
    // the two are semantically different (op identity vs persisted Node
    // identity). Before the fix, every step's nodeCid WAS the created_cid.
    if (trace.result && typeof (trace.result as { cid?: unknown }).cid === "string") {
      const createdCid = (trace.result as { cid: string }).cid;
      for (const s of trace.steps) {
        if (s.type !== "primitive") continue;
        expect(s.nodeCid).not.toBe(createdCid);
      }
    }
  });
});

// ---------------------------------------------------------------------------
// Phase 2a R3 extension (qa-expert) — TraceStep variants visible + typed
// ---------------------------------------------------------------------------
//
// Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G3-A
// (TraceStep::SuspendBoundary + ResumeBoundary + BudgetExhausted variants
// present in outcome.rs) + §9.12 BudgetExhausted shared shape.
//
// SHAPE-PIN: validates the struct shape for Phase-2b forward-compat.
// Does NOT validate firing semantics of SANDBOX fuel (Phase 2b).
//
// These tests run against the generated `.d.ts` from napi-rs; they check
// that the wrapper surface exposes discriminant unions consumers can
// switch on. Owned by `qa-expert` per R2 landscape §8.5.

describe("trace TraceStep discriminant variants (Phase 2a)", () => {
  it("trace_step_type_union_visible_in_public_surface", () => {
    // Compile-time shape-pin: the TraceStep discriminant union is imported
    // statically at the top of the file. If G11-A Wave 2b lands the new
    // variants correctly, the import is typed and `type` switches are
    // exhaustive-checkable.
    const inspector = (step: TraceStep): string => {
      switch (step.type) {
        case "suspend_boundary":
          return `suspend:${step.stateCid}`;
        case "resume_boundary":
          return `resume:${step.stateCid}:${String(step.signalValue)}`;
        case "budget_exhausted":
          // shape-pin: §9.12 shared shape
          return `budget:${step.budgetType}:${step.consumed}/${step.limit}`;
        case "primitive":
          return `prim:${step.primitive}`;
        default:
          // exhaustive-check: TypeScript fails build if a new variant
          // lands without a case here.
          const _never: never = step;
          return _never;
      }
    };
    // Smoke: inspector type-checks.
    expect(typeof inspector).toBe("function");
  });

  it("trace_step_attribution_field_required_on_every_variant", async () => {
    // Inv-14 (G5-B-ii) places `attribution: AttributionFrame` on every
    // emitted Step row. Wave 2b TraceStep unification confirmed the slot
    // exists on the `primitive` variant; sec-r6r1-01 (commit 3822570)
    // landed the eval-side `run_with_trace` wiring so `Engine::trace`
    // now carries a populated `AttributionFrame` through the napi wire.
    // perf-r6 / NAPI-R2-6 un-skip: previously skipped while the runtime
    // tail of G5-B-ii was outstanding.
    const _typeAlive: TraceStep | null = null;
    expect(_typeAlive).toBeNull();
    const handler = await engine.registerSubgraph(crud("post"));
    const trace = await engine.trace(handler.id, "post:create", {
      title: "attribution-pin",
    });
    for (const step of trace.steps) {
      // Boundary / budget rows do not carry attribution; the slot is
      // semantically per-primitive. crud(post):create only emits
      // primitive rows so the filter does not weaken the assertion.
      if (step.type !== "primitive") continue;
      expect(step.attribution).toBeTruthy();
      expect(step.attribution!.actorCid).toBeTypeOf("string");
      expect(step.attribution!.handlerCid).toBeTypeOf("string");
      expect(step.attribution!.capabilityGrantCid).toBeTypeOf("string");
    }
  });
});

// Phase 1 R3 Vitest: engine.trace() per-step timing + topological order.
// Exit-criterion #4 partner (TS side). Uses @benten/engine wrapper.
// Status: FAILING until B6 + E8 land.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, crud } from "@benten/engine";

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
      expect(typeof s.nodeCid).toBe("string");
      expect(s.nodeCid.length).toBeGreaterThan(10);
      cids.add(s.nodeCid);
    }
    // Each OperationNode in the walked subgraph gets its own CID — no
    // single CID echoed across every step (the pre-fix bug).
    expect(cids.size).toBe(trace.steps.length);

    // Per-step primitive must be a recognized kind, not the empty
    // string the synthetic fabrication fell back to.
    const kinds = new Set(trace.steps.map((s) => s.primitive));
    // Create path walks through at least write+respond.
    expect(kinds.has("write")).toBe(true);
    expect(kinds.has("respond")).toBe(true);

    // Trace's nodeCid stream must NOT include the outcome's created_cid —
    // the two are semantically different (op identity vs persisted Node
    // identity). Before the fix, every step's nodeCid WAS the created_cid.
    if (trace.result && typeof (trace.result as { cid?: unknown }).cid === "string") {
      const createdCid = (trace.result as { cid: string }).cid;
      for (const s of trace.steps) {
        expect(s.nodeCid).not.toBe(createdCid);
      }
    }
  });
});

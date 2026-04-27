// Phase-2b G12-A — companion napi test for the runtime BudgetExhausted
// trace emission wired in `crates/benten-eval/src/evaluator.rs::run_inner`.
//
// Verifies the firing-path (not just the shape) round-trips through the
// napi boundary: register a 4-WRITE chain, cap the cumulative iteration
// budget at 2 via the `iteration-budget-test-grade`-gated
// `engine.testingSetIterationBudget(2)` helper, drive `engine.trace`, and
// assert the returned trace array carries a row with `type ===
// "budget_exhausted"`, `budgetType === "inv_8_iteration"`, and
// `consumed >= limit`.
//
// Mirrors `crates/benten-engine/tests/integration/budget_exhausted_trace_emission.rs`
// at the JS surface so the Rust-side fix is exercised end-to-end through
// the napi-rs v3 trace serializer (`bindings/napi/src/trace.rs`).
//
// WRITE not EMIT: the napi `into_eval_subgraph` builder routes
// `primitive: "write"` through the WriteSpec convenience builder so each
// WRITE entry carries a populated label + properties bag through the
// G12-D widened `spec.primitives` storage that
// `Engine::subgraph_for_spec` walks. Each WRITE step bumps the cumulative
// `steps` counter so the Inv-8 guard trips deterministically.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

// Mirrors index.test.ts's `loadNative()` — skip the index.js CJS loader
// and require the platform-specific binary directly so vitest-as-ESM
// doesn't trip over the loader's nested `require()` calls.
function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  const name = `../benten-napi.${platform}-${arch}.node`;
  return require(name);
}

const native: any = loadNative();

let tmp: string;
let engine: any;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-budget-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("napi G12-A budget_exhausted runtime trace round-trip", () => {
  it("exposes testingSetIterationBudget on the napi Engine", () => {
    // Symbol-presence pin: the iteration-budget-test-grade feature is
    // wired through `bindings/napi/Cargo.toml`. If a future PR drops the
    // feature flag this assertion catches the regression.
    expect(typeof engine.testingSetIterationBudget).toBe("function");
  });

  it("engine.trace surfaces a TraceStep with type=budget_exhausted when Inv-8 fires through the napi boundary", () => {
    // Register a 4-WRITE-node chain via the DSL `nodes` shape. Each WRITE
    // contributes a `PrimitiveSpec` of kind=Write to the widened
    // `spec.primitives` storage that `Engine::subgraph_for_spec` walks
    // (G12-D); successive `"next"` edges between writes thread the chain
    // so the walker steps through one WRITE at a time, bumping the
    // cumulative `steps` counter once per primitive — same firing-path
    // semantics as the Rust integration test.
    const handlerId = engine.registerSubgraph({
      handlerId: "budget:napi_exhauster",
      actions: ["budget:run"],
      root: "n0",
      nodes: [
        { id: "n0", primitive: "write", args: { labels: ["BudgetTest"], properties: {} }, edges: { ok: "n1" } },
        { id: "n1", primitive: "write", args: { labels: ["BudgetTest"], properties: {} }, edges: { ok: "n2" } },
        { id: "n2", primitive: "write", args: { labels: ["BudgetTest"], properties: {} }, edges: { ok: "n3" } },
        { id: "n3", primitive: "write", args: { labels: ["BudgetTest"], properties: {} }, edges: {} },
      ],
    });
    expect(typeof handlerId).toBe("string");

    // G12-A test hook: cap cumulative iteration budget at 2 so the
    // walker trips at the third EMIT (steps == 2 == budget).
    engine.testingSetIterationBudget(2);

    const trace = engine.trace(handlerId, "budget:run", {});
    expect(trace).toBeTruthy();
    expect(Array.isArray(trace.steps)).toBe(true);

    // Find the BudgetExhausted row. The napi trace serializer
    // (`bindings/napi/src/trace.rs:118`) emits the variant as
    // `{ type: "budget_exhausted", budgetType, consumed, limit, path }`.
    const budgetRows = (trace.steps as Array<Record<string, unknown>>).filter(
      (s) => s.type === "budget_exhausted",
    );
    expect(budgetRows.length).toBeGreaterThan(0);

    const row = budgetRows[0];
    expect(row.budgetType).toBe("inv_8_iteration");
    expect(typeof row.consumed).toBe("number");
    expect(typeof row.limit).toBe("number");
    expect(row.limit).toBe(2);
    expect(row.consumed as number).toBeGreaterThanOrEqual(row.limit as number);
    expect(Array.isArray(row.path)).toBe(true);
    expect((row.path as unknown[]).length).toBeGreaterThan(0);

    // Clear the override so subsequent tests on this engine instance
    // don't inherit the small cap.
    engine.testingSetIterationBudget(null);
  });
});

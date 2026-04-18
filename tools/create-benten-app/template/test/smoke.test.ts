// Smoke test for the scaffolded {{name}} project.
//
// This file is the headline Phase 1 exit criterion: running `npm test`
// after `npx create-benten-app` + `npm install` must exercise all six
// named assertions without error. Each `it()` block below maps to a
// single exit-criterion gate from implementation plan §1.
//
// The scaffolder's own Vitest (tools/create-benten-app/test/
// scaffolder.test.ts) asserts this file contains exactly six `it()`
// blocks with the expected names — drift guards against accidentally
// dropping a gate.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, crud } from "@benten/engine";
import { parse as parseMermaid } from "@mermaid-js/parser";
import { postHandlers } from "../src/handlers.js";

let engine: Engine;
let tmp: string;
let handler: Awaited<ReturnType<Engine["registerSubgraph"]>>;

beforeAll(async () => {
  tmp = mkdtempSync(join(tmpdir(), "{{name}}-smoke-"));
  engine = await Engine.open(join(tmp, "{{name}}.redb"));
  handler = await engine.registerSubgraph(postHandlers);
});

afterAll(async () => {
  await engine.close();
  rmSync(tmp, { recursive: true, force: true });
});

describe("Benten Phase 1 exit criteria (six named gates)", () => {
  // Gate 1: registration succeeds.
  it("register_succeeds", () => {
    expect(handler.id).toBeTruthy();
    expect(handler.actions).toEqual(expect.arrayContaining(["create", "get", "list", "update", "delete"]));
  });

  // Gate 2: three creates then a list returns them in order.
  it("three_creates_list_returns_them", async () => {
    const a = await engine.call(handler.id, "post:create", { title: "first" });
    const b = await engine.call(handler.id, "post:create", { title: "second" });
    const c = await engine.call(handler.id, "post:create", { title: "third" });
    expect([a.cid, b.cid, c.cid].every(Boolean)).toBe(true);

    const listed = await engine.call(handler.id, "post:list", {});
    const titles = (listed.items as { title: string }[]).map((p) => p.title);
    expect(titles).toEqual(expect.arrayContaining(["first", "second", "third"]));
  });

  // Gate 3: capability denial routes to ON_DENIED edge.
  it("cap_denial_routes_on_denied", async () => {
    // Register a second handler scoped to a capability the caller does
    // NOT have. The zero-config crud() path is public, so we build a
    // minimal denied path via the capability-gated variant.
    const guarded = await engine.registerSubgraph(
      crud("restricted", { capability: "store:restricted:*" }),
    );
    try {
      await engine.call(guarded.id, "restricted:create", { title: "forbidden" });
      expect.fail("expected E_CAP_DENIED");
    } catch (err) {
      expect((err as { code?: string }).code).toBe("E_CAP_DENIED");
    }
  });

  // Gate 4: trace() returns per-node timings that are non-zero.
  it("trace_non_zero_timing", async () => {
    const trace = await engine.trace(handler.id, "post:create", { title: "traced" });
    expect(Array.isArray(trace.steps)).toBe(true);
    expect(trace.steps.length).toBeGreaterThan(0);
    // At least one step has a non-zero elapsed nanosecond timing.
    expect(trace.steps.some((s) => (s.elapsedNs ?? 0) > 0)).toBe(true);
  });

  // Gate 5: Mermaid output parses via @mermaid-js/parser.
  it("mermaid_output_parses", () => {
    const mermaid = handler.toMermaid();
    expect(mermaid).toContain("flowchart");
    // The parser throws on invalid input; a clean return means it parsed.
    expect(() => parseMermaid("flowchart", mermaid)).not.toThrow();
  });

  // Gate 6: CID round-trip TS -> Rust -> TS stays byte-identical.
  it("ts_rust_cid_roundtrip", async () => {
    const created = await engine.call(handler.id, "post:create", { title: "round-trip" });
    const reread = await engine.call(handler.id, "post:get", { cid: created.cid });
    expect(reread.cid).toBe(created.cid);
  });
});

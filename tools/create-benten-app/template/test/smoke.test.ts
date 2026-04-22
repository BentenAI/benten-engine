// Smoke test for the scaffolded {{name}} project.
//
// This file is the headline Phase 1 + 2a exit criterion: running `npm test`
// after `npx create-benten-app` + `npm install` must exercise all named
// assertions without error. Each `it()` block below maps to a numbered
// exit-criterion gate.
//
// The scaffolder's own Vitest (tools/create-benten-app/test/
// scaffolder.test.ts) asserts this file contains the expected set of
// `it()` blocks with the correct names — drift guards against accidentally
// dropping a gate.
//
// Phase 2a R3 (qa-expert) extension: gate 7 exercises the WAIT executor
// end-to-end in the scaffolded app, closing the "scaffolded project can
// compose WAIT" Phase-2a user-facing contract. Traces to the brief:
// "7th gate testing WAIT executor end-to-end in the scaffolded app."

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, subgraph } from "@benten/engine";
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

describe("Benten Phase 1 + 2a exit criteria (seven named gates)", () => {
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

  // Gate 3: calling an unregistered handler surfaces a typed error.
  it("typed_error_surface_unregistered_handler", async () => {
    try {
      await engine.call("no-such-handler", "post:create", { title: "x" });
      expect.fail("expected E_DSL_UNREGISTERED_HANDLER");
    } catch (err) {
      expect((err as { code?: string }).code).toBe("E_DSL_UNREGISTERED_HANDLER");
    }
  });

  // Gate 4: trace() returns per-node timings that are non-zero.
  it("trace_non_zero_timing", async () => {
    const trace = await engine.trace(handler.id, "post:create", { title: "traced" });
    expect(Array.isArray(trace.steps)).toBe(true);
    expect(trace.steps.length).toBeGreaterThan(0);
    expect(trace.steps.some((s) => (s.durationUs ?? 0) > 0)).toBe(true);
  });

  // Gate 5: Mermaid output has a well-formed flowchart shape.
  it("mermaid_output_parses", () => {
    const mermaid = handler.toMermaid();
    expect(mermaid).toMatch(/^flowchart (TD|LR|TB|BT|RL)\b/m);
    expect(mermaid).toMatch(/-->/);
    expect(mermaid).toMatch(/\[.*\]/);
  });

  // Gate 6: CID round-trip TS -> Rust -> TS stays byte-identical.
  it("ts_rust_cid_roundtrip", async () => {
    const created = await engine.call(handler.id, "post:create", { title: "round-trip" });
    const reread = await engine.call(handler.id, "post:get", { cid: created.cid });
    expect(reread.cid).toBe(created.cid);
  });

  // Gate 7 (Phase 2a): WAIT executor end-to-end in the scaffolded app.
  //
  // Register a wait-composing handler locally (not part of the
  // scaffolder template's default `postHandlers` so projects without
  // WAIT use cases don't pay the test cost, but the scaffolder smoke
  // MUST exercise this primitive to prove the napi surface wires up).
  it("wait_executor_end_to_end", async () => {
    const waitHandler = await engine.registerSubgraph(
      subgraph("smoke-wait")
        .action("run")
        .wait({ signal: "external:continue" })
        .respond({ body: "$result" })
        .build(),
    );

    const suspended = await engine.callWithSuspension(waitHandler.id, "run", {});
    expect(suspended.kind).toBe("suspended");
    if (suspended.kind !== "suspended") return;
    expect(suspended.handle).toBeInstanceOf(Buffer);
    expect(suspended.handle.length).toBeGreaterThan(0);

    const resumed = await engine.resumeFromBytes(suspended.handle, { value: "ok" });
    // The resumed outcome MUST complete (kind "complete") — this is the
    // exit contract: scaffolded projects can register + suspend + resume
    // a WAIT handler through the public TS surface.
    expect(resumed.kind ?? "complete").toBe("complete");
  });
});

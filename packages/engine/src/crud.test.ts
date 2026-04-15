// Phase 1 R3 Vitest: crud(post) zero-config DSL round-trip.
// Exit-criterion #2 partner. Uses @benten/engine wrapper, NOT direct napi.
// Status: FAILING until B6 DSL wrapper lands.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, crud } from "@benten/engine";

let engine: Engine;
let tmp: string;

beforeAll(async () => {
  tmp = mkdtempSync(join(tmpdir(), "benten-crud-"));
  engine = await Engine.open(join(tmp, "benten.redb"));
});

afterAll(async () => {
  await engine.close();
  rmSync(tmp, { recursive: true, force: true });
});

describe("crud(post) zero-config", () => {
  it("register_succeeds", async () => {
    // Exit-criterion #1.
    const handler = await engine.registerSubgraph(crud("post"));
    expect(handler.id).toBeTruthy();
    expect(handler.actions).toEqual(expect.arrayContaining(["create", "get", "list", "update", "delete"]));
  });

  it("crud_post_zero_config_full_cycle", async () => {
    // Exit-criterion #2.
    const handler = await engine.registerSubgraph(crud("post"));
    const a = await engine.call(handler.id, "post:create", { title: "first" });
    const b = await engine.call(handler.id, "post:create", { title: "second" });
    const c = await engine.call(handler.id, "post:create", { title: "third" });
    expect([a.cid, b.cid, c.cid].every(Boolean)).toBe(true);

    const listed = await engine.call(handler.id, "post:list", {});
    expect(listed.items).toHaveLength(3);
    expect(listed.items.map((p: { title: string }) => p.title)).toEqual(["first", "second", "third"]);
  });

  it("crud_post_zero_config_injects_createdAt_deterministically", async () => {
    const handler = await engine.registerSubgraph(crud("post"));
    const first = await engine.call(handler.id, "post:create", { title: "a" });
    await new Promise((r) => setTimeout(r, 1));
    const second = await engine.call(handler.id, "post:create", { title: "b" });
    expect(first.createdAt).toBeTypeOf("number");
    expect(second.createdAt).toBeTypeOf("number");
    expect(second.createdAt).toBeGreaterThan(first.createdAt);

    // Deterministic: re-read same post yields same createdAt (stamped once).
    const reread = await engine.call(handler.id, "post:get", { cid: first.cid });
    expect(reread.createdAt).toBe(first.createdAt);
  });
});

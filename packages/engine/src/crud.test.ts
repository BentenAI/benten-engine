// Phase 1 R3 Vitest: crud(post) zero-config DSL round-trip.
// Exit-criterion #2 partner. Uses @benten/engine wrapper, NOT direct napi.
// Status: FAILING until B6 DSL wrapper lands.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { Engine, PolicyKind, crud } from "@benten/engine";

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

  // r6b-dx-C2: update was previously missing from the canonical CRUD
  // subgraph — the engine responded `E_NOT_FOUND: unknown crud op:
  // update`. Pin the round-trip behavior so the regression doesn't
  // re-appear: create, update with a patch, re-read, expect the patch.
  it("crud_post_update_applies_patch_and_persists", async () => {
    const handler = await engine.registerSubgraph(crud("post"));
    const created = await engine.call(handler.id, "post:create", {
      title: "orig",
      body: "first draft",
    });
    expect(created.cid).toBeTruthy();

    const updated = await engine.call(handler.id, "post:update", {
      cid: created.cid,
      patch: { body: "second draft" },
    });
    expect(typeof updated.cid).toBe("string");
    // The updated Node lives at a NEW content-addressed CID (the body
    // property changed) — the CRUD update arm deletes the old Node and
    // writes the merged properties under a fresh CID.
    expect(updated.cid).not.toBe(created.cid);

    const reread = await engine.call(handler.id, "post:get", { cid: updated.cid });
    expect((reread as { body?: unknown }).body).toBe("second draft");
    // Title came from the old Node's property bag and survives the merge.
    expect((reread as { title?: unknown }).title).toBe("orig");
  });

  // r6b-dx-C3 + r6b-qa-3: a denied write used to return `{ ok: false,
  // errorCode: "E_CAP_DENIED", ... }` which `flattenCallResult` silently
  // flattened into a success-shaped object — the caller saw no error,
  // and the Node was never persisted. This pins the contract that
  // denials throw a typed `E_CAP_DENIED` so a user running QUICKSTART
  // learns immediately that their write was rejected.
  it("denied_write_throws_ecapdenied_not_silent_success", async () => {
    const denyTmp = mkdtempSync(join(tmpdir(), "benten-deny-"));
    const denied = await Engine.openWithPolicy(
      join(denyTmp, "benten.redb"),
      PolicyKind.GrantBacked,
    );
    try {
      const handler = await denied.registerSubgraph(
        crud("post", { capability: "store:post:write" }),
      );
      let threw = false;
      let caughtCode: string | undefined;
      try {
        await denied.call(handler.id, "post:create", { title: "denied" });
      } catch (err) {
        threw = true;
        caughtCode = (err as { code?: string }).code;
      }
      expect(threw).toBe(true);
      expect(caughtCode).toBe("E_CAP_DENIED");
      // The Node must NOT have been persisted despite the "success-shaped"
      // silent flatten that used to hide the denial.
      const count = await denied.countNodesWithLabel("post");
      expect(count).toBe(0);
    } finally {
      await denied.close();
      rmSync(denyTmp, { recursive: true, force: true });
    }
  });

  // r6b-dx-C1 + r6b-qa-2: wildcard capability. QUICKSTART shows
  // `crud('post', { capability: 'store:post:*' })` — before the fix the
  // wildcard never matched the derived concrete scope (`store:post:write`)
  // and every create came back denied. This pins the wildcard
  // attenuation semantics so a user following QUICKSTART verbatim gets
  // a working create path.
  it("wildcard_capability_permits_derived_concrete_scopes", async () => {
    const wildTmp = mkdtempSync(join(tmpdir(), "benten-wild-"));
    const wild = await Engine.openWithPolicy(
      join(wildTmp, "benten.redb"),
      PolicyKind.GrantBacked,
    );
    try {
      const handler = await wild.registerSubgraph(
        crud("post", { capability: "store:post:*" }),
      );
      await wild.grantCapability({ actor: "alice", scope: "store:post:*" });
      const created = await wild.callAs(
        handler.id,
        "post:create",
        { title: "wild" },
        "alice",
      );
      expect(typeof created.cid).toBe("string");
      const count = await wild.countNodesWithLabel("post");
      expect(count).toBe(1);
    } finally {
      await wild.close();
      rmSync(wildTmp, { recursive: true, force: true });
    }
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

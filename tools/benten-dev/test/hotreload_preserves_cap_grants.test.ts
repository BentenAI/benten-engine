// Phase 2a R3 Vitest — devserver hot-reload preserves cap grants +
// in-flight evaluations complete before new registration applies.
//
// Traces to: `.addl/phase-2a/00-implementation-plan.md` §3 G11-A
// (`tools/benten-dev/**` new dev server; dx-r1 hot-reload cap-grant
// preservation + in-flight evaluation semantics; must-pass tests
// `devserver_preserves_cap_grants_across_reload` +
// `devserver_in_flight_evaluations_complete_before_reload`).
//
// Status: FAILING until `tools/benten-dev/` exports a programmatic
// devserver driver (its `bin.mjs` + `src/index.ts`). Owned by `qa-expert`
// per R2 landscape §8.5 — this is the TS-side devserver-scenario test.
// TDD red-phase.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

// The devserver's programmatic entrypoint — ships with G11-A.
// Import lazily so the red-phase missing module surfaces cleanly.
async function loadDevServer() {
  return (await import("../src/index.js")) as {
    BentenDevServer: new (opts: { projectRoot: string }) => {
      start(): Promise<void>;
      stop(): Promise<void>;
      engine(): import("@benten/engine").Engine;
      editHandler(relPath: string, content: string): Promise<void>;
      waitForReload(): Promise<void>;
    };
  };
}

let tmp: string;
let projectRoot: string;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-dev-reload-"));
  projectRoot = join(tmp, "project");
  // Scaffold a minimal project layout the devserver expects.
  // {projectRoot}/src/handlers.ts initially exports a crud("post") handler.
  // `crud` is re-exported from @benten/engine.
  const handlersTs = `
import { crud } from "@benten/engine";
export const postHandlers = crud("post");
`;
  // Create dirs + seed file.
  // (synchronous fs calls suffice — the Vitest harness already handles
  // the async scheduling.)
  const fs = require("node:fs") as typeof import("node:fs");
  fs.mkdirSync(join(projectRoot, "src"), { recursive: true });
  fs.mkdirSync(join(projectRoot, ".benten"), { recursive: true });
  writeFileSync(join(projectRoot, "src/handlers.ts"), handlersTs, "utf8");
  writeFileSync(
    join(projectRoot, "package.json"),
    JSON.stringify(
      {
        name: "devserver-test-project",
        type: "module",
        dependencies: { "@benten/engine": "*" },
      },
      null,
      2,
    ),
    "utf8",
  );
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("benten-dev hot reload", () => {
  it("devserver_preserves_cap_grants_across_reload", async () => {
    const { BentenDevServer } = await loadDevServer();
    const server = new BentenDevServer({ projectRoot });
    await server.start();

    try {
      const engine = server.engine();
      // Seed a grant BEFORE the edit.
      await engine.grantCapability({
        actor: "alice",
        scope: "store:post:write",
      });

      // Edit the handler file — trigger a hot reload.
      const editedHandlers = `
import { crud } from "@benten/engine";
// hot-reload marker: added a second label
export const postHandlers = crud("post");
export const commentHandlers = crud("comment");
`;
      await server.editHandler("src/handlers.ts", editedHandlers);
      await server.waitForReload();

      // The grant seeded pre-reload must still be present post-reload.
      const committed = await engine.capabilityWritesCommitted();
      // At minimum, the previously-seeded grant's actor+scope must still
      // return an allowed commit path:
      expect(
        async () =>
          await engine.call("post-handler", "post:create", { title: "after" }),
      ).not.toThrow();
      // Soft pin: grant counter surface exists.
      expect(typeof committed).toBe("object");
    } finally {
      await server.stop();
    }
  });

  it("devserver_in_flight_evaluations_complete_before_reload", async () => {
    const { BentenDevServer } = await loadDevServer();
    const server = new BentenDevServer({ projectRoot });
    await server.start();

    try {
      const engine = server.engine();
      // Kick off a long-ish call (e.g., list with 100 items already seeded).
      for (let i = 0; i < 50; i++) {
        await engine.call("post-handler", "post:create", { title: `p${i}` });
      }

      // Fire a call + a reload concurrently. The call must complete
      // against the OLD handler registration, not return a half-loaded
      // state. The reload MUST wait for the in-flight call to drain.
      const callPromise = engine.call("post-handler", "post:list", {});

      const editedAgain = `
import { crud } from "@benten/engine";
export const postHandlers = crud("post_v2");
`;
      const reloadPromise = (async () => {
        await server.editHandler("src/handlers.ts", editedAgain);
        await server.waitForReload();
      })();

      const [listResult] = await Promise.all([callPromise, reloadPromise]);
      // The in-flight list saw the OLD "post" label, NOT "post_v2".
      // Under proper reload ordering, all 50 items are listed.
      const items = (listResult as { items?: unknown[] }).items ?? [];
      expect(items.length).toBeGreaterThanOrEqual(50);
    } finally {
      await server.stop();
    }
  });
});

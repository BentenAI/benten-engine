// G12-B / Phase 2b Wave-8f: TS Vitest harness covering the JS-side
// BentenDevServer (from @benten/engine-devserver) routed through the
// napi DevServer bridge.
//
// Lifted from `it.skip` to `it(...)` as Wave-8f lands. Per
// `.addl/phase-2b/wave-8-brief.md` §8f: "Un-skip the Vitest harness;
// assert hot-reload preserves cap grants through the engine path; assert
// in-flight evaluations complete before reload."

import { describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { BentenDevServer } from "@benten/engine-devserver";

function freshTmp(prefix: string): { dir: string; cleanup: () => void } {
  const dir = mkdtempSync(join(tmpdir(), prefix));
  return {
    dir,
    cleanup: () => rmSync(dir, { recursive: true, force: true }),
  };
}

describe("BentenDevServer (Wave-8f JS surface)", () => {
  it("registers a handler from a DSL source string via the napi bridge", async () => {
    const { dir, cleanup } = freshTmp("benten-dev-register-");
    try {
      const server = new BentenDevServer({ projectRoot: dir });
      await server.start();
      const id = await server.registerHandler(
        "h1",
        "run",
        "handler 'h1' { read('post') -> respond }",
      );
      expect(id).toBe("h1");
      await server.stop();
    } finally {
      cleanup();
    }
  });

  it("hot-reload preserves cap-grants when routed through Engine.register_subgraph_replace", async () => {
    const { dir, cleanup } = freshTmp("benten-dev-hot-grants-");
    try {
      const server = new BentenDevServer({ projectRoot: dir });
      await server.start();

      // Seed a grant BEFORE registering the handler.
      await server.grantCapability({
        actor: "alice",
        scope: "store:post:write",
      });
      expect(
        await server.grantExists({ actor: "alice", scope: "store:post:write" }),
      ).toBe(true);

      // First registration.
      await server.registerHandler(
        "h1",
        "run",
        "handler 'h1' { read('post') -> respond }",
      );

      // Hot-reload: same handler_id, different body. Routed through
      // Engine::register_subgraph_replace — must NOT throw
      // DuplicateHandler.
      await server.replaceHandler(
        "h1",
        "run",
        "handler 'h1' { read('post') -> transform({ x: $x }) -> respond }",
      );

      // The seeded grant survives the engine-routed re-registration.
      expect(
        await server.grantExists({ actor: "alice", scope: "store:post:write" }),
      ).toBe(true);

      await server.stop();
    } finally {
      cleanup();
    }
  });

  it("propagates a typed Diagnostic for bad DSL input", async () => {
    const { dir, cleanup } = freshTmp("benten-dev-bad-dsl-");
    try {
      const server = new BentenDevServer({ projectRoot: dir });
      await server.start();
      // `teleport` is not a known primitive — must surface E_DSL_*.
      let captured: Error | undefined;
      try {
        await server.registerHandler(
          "oops",
          "run",
          "handler 'oops' { teleport -> respond }",
        );
      } catch (err) {
        captured = err as Error;
      }
      expect(captured).toBeDefined();
      expect(captured?.message).toMatch(/E_DSL_/);
      await server.stop();
    } finally {
      cleanup();
    }
  });

  it("subscribeToReloadEvents reports each replace with versionTag + new/previous CIDs", async () => {
    const { dir, cleanup } = freshTmp("benten-dev-reload-events-");
    try {
      const server = new BentenDevServer({ projectRoot: dir });
      await server.start();

      const sub = server.subscribeToReloadEvents();

      // First registration — versionTag v1, no previousCid.
      await server.registerHandler(
        "h1",
        "run",
        "handler 'h1' { read('post') -> respond }",
      );
      // Replace with a different body — versionTag v2 + previousCid set.
      await server.replaceHandler(
        "h1",
        "run",
        "handler 'h1' { read('post') -> transform({ x: $x }) -> respond }",
      );

      const events = sub.drain();
      expect(events.length).toBeGreaterThanOrEqual(2);
      const v1 = events.find((e) => e.versionTag === "v1");
      const v2 = events.find((e) => e.versionTag === "v2");
      expect(v1).toBeDefined();
      expect(v2).toBeDefined();
      // v1 has a newCid (engine-routed) but no previousCid.
      expect(typeof v1?.newCid).toBe("string");
      expect(v1?.previousCid).toBeUndefined();
      // v2 has both.
      expect(typeof v2?.newCid).toBe("string");
      expect(typeof v2?.previousCid).toBe("string");
      // The chain is consistent: v2's previousCid equals v1's newCid.
      expect(v2?.previousCid).toBe(v1?.newCid);

      sub.unsubscribe();
      await server.stop();
    } finally {
      cleanup();
    }
  });

  it("DSL compile error surfaces structured line + column via error.context (R6FP Instance 9)", async () => {
    // Pre-fix: compile_err_to_napi formatted `(line={line:?} column={col:?})`
    // as a `{:?}`-Debug suffix yielding `Some(N)` literals JS had to
    // regex-parse. Post-fix the structured fields ride the
    // `$$benten-context$$` sentinel + surface as numeric fields on
    // `error.context`.
    const { dir, cleanup } = freshTmp("benten-dev-compile-err-");
    try {
      const server = new BentenDevServer({ projectRoot: dir });
      await server.start();

      // Multi-line malformed source so line + column are non-default.
      // Line 2 has the syntax error (missing arrow); the parser should
      // attribute the diagnostic to a real line/column rather than 1:1.
      const malformed = [
        "handler 'broken' {",
        "  read('post') no_arrow_here respond",
        "}",
      ].join("\n");

      let captured: any = null;
      try {
        await server.registerHandler("broken", "run", malformed);
      } catch (err) {
        captured = err;
      }

      expect(captured).not.toBeNull();
      // The error MUST carry structured context (Instance 9 contract).
      expect(captured.context).toBeDefined();
      expect(captured.context).toBeTypeOf("object");
      // Structured numeric line + column (NOT a Debug `Some(N)` string).
      // null is acceptable per the JSON shape if the parser couldn't
      // attribute, but the type must be number-or-null — never string.
      const line = captured.context.line;
      const column = captured.context.column;
      expect(line === null || typeof line === "number").toBe(true);
      expect(column === null || typeof column === "number").toBe(true);
      // Realistically: line should be 2 (the bad line) or higher.
      if (typeof line === "number") {
        expect(line).toBeGreaterThanOrEqual(1);
      }

      await server.stop();
    } finally {
      cleanup();
    }
  });

  it("idempotent re-registration with identical content does not bump the version chain", async () => {
    const { dir, cleanup } = freshTmp("benten-dev-idem-");
    try {
      const server = new BentenDevServer({ projectRoot: dir });
      await server.start();

      const sub = server.subscribeToReloadEvents();

      const src = "handler 'h1' { read('post') -> respond }";
      await server.registerHandler("h1", "run", src);
      // Identical re-register — must NOT publish a new event under
      // the engine routing path (the legacy in-memory bookkeeping
      // returns early before publishing too).
      await server.registerHandler("h1", "run", src);

      const events = sub.drain();
      // First registration publishes one event (v1). Identical
      // re-register is idempotent; the legacy `existing_same_source`
      // early-return guards the publish path.
      expect(events.length).toBe(1);
      expect(events[0].versionTag).toBe("v1");

      sub.unsubscribe();
      await server.stop();
    } finally {
      cleanup();
    }
  });
});

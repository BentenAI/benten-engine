// Phase 2b G11-2b — `packages/engine/examples/**` MUST type-check
// against the public DSL surface.
//
// The brief calls this `quickstart_examples_compile_and_run`. The
// "compile" half is enforced here by importing the example modules
// (Vitest's transform pipeline elides them but the import resolves
// the module's exports + types). The "run" half is exercised
// manually per `examples/README.md` — running the examples requires
// the napi binding be built, which is not a Vitest precondition.
//
// What this test pins:
//   1. The example handler files export the expected `Subgraph` /
//      `buildSandboxHandler(...)` symbols at the names the README
//      documents.
//   2. The exported handlers carry the structural shape the
//      `Engine.registerSubgraph` API contract expects (a
//      `SubgraphNode[]` array with the `primitive` discriminant +
//      stamp_attribution-applied args).
//   3. The example's primitive composition uses ONLY the documented
//      DSL surface — a regression that drops `subgraph(...).stream()`
//      / `.subscribe()` / `.sandbox()` would surface here.

import { describe, expect, it } from "vitest";
import {
  streamHandler,
  streamHandlerAction,
  streamHandlerId,
} from "../examples/stream-handler.js";
import {
  subscribeHandler,
  subscribeHandlerId,
} from "../examples/subscribe-handler.js";
import {
  buildSandboxHandler,
  sandboxHandlerAction,
  sandboxHandlerId,
} from "../examples/sandbox-handler.js";

describe("quickstart_examples_compile_and_run", () => {
  it("STREAM example handler shape pin", () => {
    expect(streamHandlerId).toBe("export-feed");
    expect(streamHandlerAction).toBe("default");
    expect(streamHandler.handlerId).toBe("export-feed");
    const primitives = streamHandler.nodes.map((n) => n.primitive);
    expect(primitives).toContain("read");
    expect(primitives).toContain("iterate");
    expect(primitives).toContain("stream");
    expect(primitives).toContain("respond");
  });

  it("SUBSCRIBE example handler shape pin", () => {
    expect(subscribeHandlerId).toBe("post-summary-view");
    expect(subscribeHandler.handlerId).toBe("post-summary-view");
    const primitives = subscribeHandler.nodes.map((n) => n.primitive);
    expect(primitives).toContain("subscribe");
    expect(primitives).toContain("transform");
    expect(primitives).toContain("write");
    expect(primitives).toContain("emit");
  });

  it("SANDBOX example handler shape pin", () => {
    expect(sandboxHandlerId).toBe("summarize");
    expect(sandboxHandlerAction).toBe("default");
    const handler = buildSandboxHandler("example.summarizer:summarize-v1");
    expect(handler.handlerId).toBe("summarize");
    const primitives = handler.nodes.map((n) => n.primitive);
    expect(primitives).toContain("read");
    expect(primitives).toContain("sandbox");
    expect(primitives).toContain("write");
    expect(primitives).toContain("respond");

    // SANDBOX node MUST carry the per-call tuning knobs the example
    // illustrates — the example doc claim "fuel / wallclockMs /
    // outputLimitBytes (per-call)" is load-bearing. Note: the DSL
    // builder translates camelCase user input to snake_case eval-form
    // at the `.sandbox()` boundary (see dsl.ts::translateSandboxArgs),
    // so post-build args use snake_case + `output_limit` (not
    // `outputLimitBytes`) per r4-r1-wsa-1.
    const sandboxNode = handler.nodes.find((n) => n.primitive === "sandbox");
    expect((sandboxNode?.args as Record<string, unknown>).fuel).toBe(1_000_000);
    expect((sandboxNode?.args as Record<string, unknown>).wallclock_ms).toBe(
      30_000,
    );
    expect((sandboxNode?.args as Record<string, unknown>).output_limit).toBe(
      1_048_576,
    );
  });

  it("examples cover all three Phase-2b primitives at least once", () => {
    const allPrimitives = [
      ...streamHandler.nodes,
      ...subscribeHandler.nodes,
      ...buildSandboxHandler("x:y").nodes,
    ].map((n) => n.primitive);
    for (const required of ["stream", "subscribe", "sandbox"] as const) {
      expect(allPrimitives).toContain(required);
    }
  });
});

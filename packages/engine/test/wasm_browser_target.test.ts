// R3-F red-phase — browser-target SANDBOX-disabled UX.
//
// CRITICAL distinction (per brief + sec-pre-r1-05 + dx-r1-2b SANDBOX):
//   - SANDBOX EXECUTOR is absent on wasm32 (compile-time gate). The Rust-side
//     wasmtime backend isn't compiled into the wasm32-unknown-unknown
//     target — there is no nested wasm-in-wasm execution from a browser
//     build of the engine.
//   - SANDBOX DSL BUILDER (packages/engine/src/sandbox.ts) STAYS PRESENT on
//     browser builds. Handlers may be authored in a browser for execution
//     on a Node-resident peer (Phase-3 P2P sync). The DSL must remain
//     callable so subgraph(...).sandbox(...) compiles and produces a valid
//     subgraph payload that ships over the wire to a Node peer.
//   - When a browser-build engine instance attempts to EXECUTE a SANDBOX
//     primitive locally, the call MUST fail with the typed error
//     E_SANDBOX_UNAVAILABLE_ON_WASM at sandbox-execution time, NOT at DSL
//     authoring time.
//
// Tests are RED at landing time; G10-A (wasm32-unknown-unknown target) +
// G7-C (TS surface) make them green together.
//
// Pin sources: brief explicit; r2-test-landscape.md §2.3 (exit-gate-3 wasm32
// targets) + §1.3 sandbox unit tests; sec-pre-r1-05 wasm32 SANDBOX gating.

import { describe, it, expect } from "vitest";
import { Engine, subgraph, sandbox } from "@benten/engine";
import type { ModuleManifest, SandboxArgs } from "@benten/engine";

describe("wasm32-unknown-unknown browser target — SANDBOX UX", () => {
  it("DSL builder subgraph(...).sandbox(...) STAYS PRESENT on browser builds", () => {
    // Authoring path MUST remain callable. The browser build of @benten/engine
    // ships the same DSL surface as Node — only the executor differs.
    const args: SandboxArgs = { module: "echo:identity", input: "$input" };

    const sg = subgraph("authored-in-browser")
      .action("run")
      .sandbox(args)
      .respond({ body: "$result" })
      .build();

    const sandboxNode = sg.nodes.find((n) => n.primitive === "sandbox");
    expect(sandboxNode).toBeDefined();
    expect(sandboxNode!.args.module).toBe("echo:identity");

    // Top-level sandbox() helper (re-exported from index.ts) MUST also remain
    // present — used by inline composition pattern in Phase-3 P2P authoring.
    const helper = sandbox({ module: "echo:identity" });
    expect(helper.primitive).toBe("sandbox");
  });

  it("SANDBOX-bearing handler registers cleanly on browser-target engine", async () => {
    // The handler can be REGISTERED on a browser-target Engine — registration
    // is pure-shape validation; it doesn't execute the SANDBOX. The handler
    // will run on a Node peer after Phase-3 sync; locally the engine's
    // intent is "yes, this is a valid handler shape, store it."
    const engine = await Engine.open(":memory:");
    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
    };
    await engine.installModule(manifest, "bafy...manifest");

    const sg = subgraph("h")
      .action("go")
      .sandbox({ module: "echo:identity" })
      .respond({ body: "$result" })
      .build();

    // Registration succeeds even when the executor is absent.
    await expect(engine.registerSubgraph(sg)).resolves.toBeDefined();

    await engine.close();
  });

  it("invoking a SANDBOX-bearing handler returns E_SANDBOX_UNAVAILABLE_ON_WASM at execution time", async () => {
    // The execution-time typed error fires WHEN the SANDBOX step is reached
    // during the evaluator walk — NOT at registration, NOT at handler-lookup,
    // NOT at module-install. Developers building browser-resident handlers
    // see a clear, actionable error pointing them at Phase-3 P2P routing.
    //
    // This test is conditionally meaningful only on wasm32-unknown-unknown
    // builds. On Node (the default test target), the executor IS present —
    // the test asserts the typed error code IF the build cfg toggles the
    // SANDBOX-disabled mode. R5 (G10-A) wires the cfg; the assertion shape
    // is locked here.
    const engine = await Engine.open(":memory:");

    if (!engine.targetSupportsSandbox()) {
      // Browser-target build path — SANDBOX execution is gated.
      const manifest: ModuleManifest = {
        name: "echo",
        version: "0.0.1",
        modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
      };
      await engine.installModule(manifest, "bafy...manifest");

      const sg = subgraph("h")
        .action("go")
        .sandbox({ module: "echo:identity" })
        .respond({ body: "$result" })
        .build();
      await engine.registerSubgraph(sg);

      await expect(engine.call("h", "go", {})).rejects.toMatchObject({
        code: "E_SANDBOX_UNAVAILABLE_ON_WASM",
      });
    } else {
      // Node-target build path — feature present; this branch is the
      // shape-of-the-test pin. R5 G10-A wires `targetSupportsSandbox()`
      // to read the same cfg the Rust executor reads.
      expect(engine.targetSupportsSandbox()).toBe(true);
    }

    await engine.close();
  });

  it("targetSupportsSandbox surface present on the Engine class", () => {
    // Compile-time pin: the introspection method exists, so callers writing
    // browser-aware code can `if (engine.targetSupportsSandbox()) { ... }`
    // without driving an actual SANDBOX call to learn the answer.
    type EngineMethods = keyof Engine;
    const methodIsKey: "targetSupportsSandbox" extends EngineMethods
      ? true
      : false = true;
    expect(methodIsKey).toBe(true);
  });
});

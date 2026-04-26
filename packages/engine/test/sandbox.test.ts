// R3-F red-phase — SANDBOX TS DSL composition (single-surface, no engine.sandbox).
//
// Tests are RED at landing time; G7-C (TS-side) makes them green.
//
// Surface contract (per dx-r1-2b SANDBOX):
//   - DSL composition ONLY: subgraph(...).sandbox(args)
//   - There is NO engine.sandbox(...) top-level method. Composition-only,
//     because SANDBOX is a subgraph primitive analogous to TRANSFORM, not a
//     top-level CALL surface. Exposing engine.sandbox would bypass the
//     evaluator (Inv-4 nest-depth, Inv-14 attribution, capability gates).
//   - Module lifecycle (engine.installModule / engine.uninstallModule) is
//     SEPARATE — covered in install_module.test.ts (G10-B exclusive).
//   - SandboxArgs: by-name vs by-caps mutually exclusive (TS union); defaults
//     fuel=1_000_000 / wallclockMs=30_000 / outputLimitBytes=1_048_576.
//
// Pin sources: r2-test-landscape.md §7 (rows 456-459); r1-dx-optimizer.json
// dsl_builder_test_writer_handoff.sandbox_test_fixture; D15 trap-loudly.

import { describe, it, expect } from "vitest";
import { Engine, subgraph } from "@benten/engine";
import type {
  ModuleManifest,
  SandboxArgs,
  SandboxArgsByName,
  SandboxArgsByCaps,
} from "@benten/engine";

describe("DSL .sandbox() composition", () => {
  it("compose SANDBOX inside a handler subgraph", async () => {
    const engine = await Engine.open(":memory:");
    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
    };
    await engine.installModule(manifest, "bafy...manifest");

    const sg = subgraph("identity-handler")
      .action("run")
      .sandbox({ module: "echo:identity", input: "$input", fuel: 100_000 })
      .respond({ body: "$result" })
      .build();
    await engine.registerSubgraph(sg);

    const result = await engine.call("identity-handler", "run", {
      hello: "world",
    });
    // SANDBOX runs inside the handler walk; the consumer sees a normal Outcome.
    expect(result.ok).toBe(true);

    await engine.close();
  });
});

describe("DSL .sandbox() — composition-only contract", () => {
  it("no top-level engine.sandbox surface exists", () => {
    // dx-r1-2b SANDBOX recommends ONE surface only — DSL composition.
    // Compile-time absence pin via @ts-expect-error: if a future R5 wave
    // adds `engine.sandbox(...)` this test FAILS COMPILE (the directive
    // becomes unused), which is exactly the regression signal we want.

    type EngineMethods = keyof Engine;

    // Negative compile-time pin via conditional type — true iff `sandbox`
    // is NOT a key on Engine.
    const sandboxNotKey: "sandbox" extends EngineMethods ? false : true = true;
    expect(sandboxNotKey).toBe(true);

    // Runtime pin — defense-in-depth in case the conditional type is
    // inadvertently widened by a future PR.
    const engineProto = Engine.prototype as unknown as Record<string, unknown>;
    expect(engineProto.sandbox).toBeUndefined();

    // Below: this expression must NOT compile. R5 implementers MUST keep
    // the `// @ts-expect-error` annotation valid by ensuring `engine.sandbox`
    // remains undefined on the public Engine surface.
    //
    // If the project ever adds a top-level engine.sandbox method, the
    // suppression becomes unused → TypeScript's
    // `noUnusedDirectives`-class behaviour fails the build (vitest type-check).
    const fakeEngine = {} as Engine;
    // @ts-expect-error — engine.sandbox does not exist (composition-only per dx-r1-2)
    void fakeEngine.sandbox;
  });

  it("SandboxArgs by name vs by caps mutually exclusive (TS union)", () => {
    // dx-r1-2b SANDBOX: SandboxArgsByName forbids `caps`; SandboxArgsByCaps
    // requires `caps`. The discriminated union prevents the half-and-half
    // shape that would otherwise let a developer mix manifest lookup and
    // explicit-caps escape hatch in the same call.

    const byName: SandboxArgsByName = {
      module: "echo:identity",
      input: "$input",
      fuel: 100_000,
    };
    expect(byName.module).toBe("echo:identity");

    const byCaps: SandboxArgsByCaps = {
      module: "bafy...wasm-cid",
      caps: ["host:compute:time"],
      fuel: 50_000,
    };
    expect(byCaps.caps).toEqual(["host:compute:time"]);

    // Compile-time negative: by-name with `caps` MUST NOT type-check.
    // @ts-expect-error — SandboxArgsByName forbids `caps` (must NEVER co-occur)
    const bad: SandboxArgsByName = {
      module: "echo:identity",
      caps: ["host:compute:time"],
    };
    void bad;

    // Both variants are assignable to the umbrella SandboxArgs union.
    const u1: SandboxArgs = byName;
    const u2: SandboxArgs = byCaps;
    expect(u1).toBeDefined();
    expect(u2).toBeDefined();
  });

  it("SandboxArgs defaults — omitting fuel / wallclockMs / outputLimitBytes uses 1M / 30s / 1MB", async () => {
    // dx-r1-2b-5 + D24: pin the canonical defaults. Omitting the three
    // tuning knobs MUST NOT cause registration friction; the engine fills in
    // fuel=1_000_000, wallclockMs=30_000, outputLimitBytes=1_048_576.
    const engine = await Engine.open(":memory:");
    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
    };
    await engine.installModule(manifest, "bafy...manifest");

    const sg = subgraph("default-knobs")
      .action("run")
      .sandbox({ module: "echo:identity", input: "$input" }) // no fuel/wallclock/output
      .respond({ body: "$result" })
      .build();
    const reg = await engine.registerSubgraph(sg);

    // The engine's introspection MUST report the defaults applied, not zero
    // / undefined. The exact accessor name is G7-C's call; this test will
    // become green once the introspection surface is final.
    const sandboxNode = reg.subgraph.nodes.find(
      (n) => n.primitive === "sandbox",
    );
    expect(sandboxNode).toBeDefined();

    const effective = await engine.describeSandboxNode(reg.id, sandboxNode!.id);
    expect(effective.fuel).toBe(1_000_000);
    expect(effective.wallclockMs).toBe(30_000);
    expect(effective.outputLimitBytes).toBe(1_048_576);

    await engine.close();
  });

  it("E_INV_SANDBOX_OUTPUT fires on output > limit (D15 trap-loudly)", async () => {
    // D15 trap-loudly default — exceeding outputLimitBytes is a typed error,
    // NOT a silent truncation. The escape hatch (`trust:output:truncate`) is
    // out of scope here.
    const engine = await Engine.open(":memory:");
    const manifest: ModuleManifest = {
      name: "oversize",
      version: "0.0.1",
      modules: [{ name: "emit", cid: "bafy...oversize-wasm", requires: [] }],
    };
    await engine.installModule(manifest, "bafy...oversize-manifest");

    const sg = subgraph("oversize")
      .action("run")
      .sandbox({ module: "oversize:emit", outputLimitBytes: 1_048_576 })
      .respond({ body: "$result" })
      .build();
    await engine.registerSubgraph(sg);

    await expect(engine.call("oversize", "run", {})).rejects.toMatchObject({
      code: "E_INV_SANDBOX_OUTPUT",
    });

    await engine.close();
  });
});

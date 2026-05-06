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
  // Phase-3 G17-C wave-5b (phase-3-backlog §6.6 deliverable 1): the
  // pre-G17-C `.skip` was rationalized as "needs registerModuleBytes +
  // real .wasm bytes". G17-C ships the registration-time validation
  // walk + `engine.registerModuleBytes` napi method, which together
  // close the named-manifest registration half. Real .wasm execution
  // (which gates `result.ok=true` end-to-end) sits behind G17-B's
  // `.wasm` fixtures (wave-5b sibling). The G17-C-shaped pin asserts
  // the registration-time validation walk + named-manifest resolution
  // works end-to-end through the production DSL → napi → engine path.
  it("compose SANDBOX inside a handler subgraph — register-time named-manifest resolves", async () => {
    const engine = await Engine.open(":memory:");
    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
    };
    const manifestCid = await engine.computeManifestCid(manifest);
    await engine.installModule(manifest, manifestCid);

    // Compose a SANDBOX handler that references the manifest entry
    // by colon-joined `<manifest>:<entry>` name. With G17-C's
    // validation walk, registerSubgraph MUST succeed (the manifest
    // resolves through the engine's `manifest_registry()` overlay
    // extended to also key by colon-joined names).
    const sg = subgraph("identity-handler")
      .action("run")
      .sandbox({ module: "echo:identity", input: "$input", fuel: 100_000 })
      .respond({ body: "$result" })
      .build();
    await engine.registerSubgraph(sg);

    // OBSERVABLE consequence: registerSubgraph reaches the success
    // branch — the validation walk found the colon-joined name in
    // the registry overlay. A regression that drops the colon-joined
    // keying (or bypasses the validation walk entirely) would either
    // (a) reject here with E_SANDBOX_MANIFEST_UNKNOWN OR (b) silently
    // accept invalid names that fail later at execution time. Both
    // failure shapes are caught by the negative-side companion in
    // `install_module.test.ts::"engine.uninstallModule(cid) clean release"`
    // which exercises post-uninstall rejection through the same path.
    //
    // Real .wasm execution (which would enable `engine.call` to
    // return `result.ok=true`) is paired with G17-B's `.wasm`
    // fixture wave-5b sibling work; the registration-time guarantee
    // this G17-C pin defends is the load-bearing change in this
    // wave.

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
    const manifestCid = await engine.computeManifestCid(manifest);
    await engine.installModule(manifest, manifestCid);

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

  // Phase-3 G17-C wave-5b — re-pinned to drive the production
  // `engine.registerSubgraph` validation-walk path (per pim-2 §3.6b
  // end-to-end + pim-2-ts-canary). The pre-G17-C `.skip` rationale
  // (needs real wasm fixture emitting >1 MiB) targeted the EXECUTION-
  // TIME D15 trap-loudly arm; G17-C lands the REGISTRATION-TIME
  // validation walk that catches the same misspelled-manifest-name
  // shape earlier in the pipeline (operator-actionable: the
  // wallclock-after-zero-progress masking is gone).
  //
  // The execution-time D15 trap-loudly pin BELONGS-NAMED-NOW to
  // G17-B's `.wasm` fixtures wave-5b sibling work + a
  // future Phase-3 wave that ships a real-wasm test infrastructure
  // (Vitest + .wasm fixture loader). Until then, the
  // `outputLimitBytes` knob's eval-side observable end is exercised
  // by `crates/benten-eval/tests/sandbox_handler_args.rs::sandbox_per_handler_output_limit_bytes_camel_case_dsl_round_trips`
  // (G17-C land — Rust eval-side end-to-end pin per pim-2 §3.6b).
  it("registerSubgraph rejects unresolved SANDBOX manifest with E_SANDBOX_MANIFEST_UNKNOWN", async () => {
    // OBSERVABLE consequence: composing a SANDBOX with a manifest
    // name that has NOT been installed (no install_module call)
    // fails at registerSubgraph time with the typed
    // E_SANDBOX_MANIFEST_UNKNOWN code. Pre-G17-C this misspelled-
    // name path silently registered + failed later at execution
    // time as a confusing wallclock-after-zero-progress shape;
    // G17-C wires the validation walk to catch it earlier.
    const engine = await Engine.open(":memory:");

    const sg = subgraph("oversize")
      .action("run")
      .sandbox({ module: "oversize:emit", outputLimitBytes: 1_048_576 })
      .respond({ body: "$result" })
      .build();

    await expect(engine.registerSubgraph(sg)).rejects.toMatchObject({
      code: "E_SANDBOX_MANIFEST_UNKNOWN",
    });

    await engine.close();
  });
});

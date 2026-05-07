// R3-F red-phase — engine.installModule / engine.uninstallModule (G10-B exclusive).
//
// Tests are RED at landing time; G10-B (TS-side) makes them green.
//
// Surface contract (per dx-r1-2b SANDBOX module-lifecycle + plan §3 G10-B):
//   - engine.installModule(manifest, manifestCid) -> Promise<Cid>
//     * REQUIRED expected_cid arg per D16-RESOLVED (no convenience overload
//       that omits it — drift would silently compute the CID and trust it).
//     * Throws E_MODULE_MANIFEST_CID_MISMATCH if the computed BLAKE3 of the
//       manifest's canonical DAG-CBOR encoding does not match the supplied
//       CID. Error carries dual-CID diff + manifest summary per dx-r1-2b D16.
//   - engine.uninstallModule(cid) -> Promise<void>
//     * Releases caps; subscriptions / IVM views referencing modules from
//       this manifest get cleaned up.
//
// Pin sources: r2-test-landscape.md §7 (rows 460-461); r2 §8 D16 row;
// r1-dx-optimizer.json sandbox_test_fixture install_module + dx-r1-2b SANDBOX.

import { describe, it, expect } from "vitest";
import { Engine } from "@benten/engine";
import type { ModuleManifest } from "@benten/engine";

describe("engine.installModule", () => {
  it("engine.installModule(manifest, manifestCid) round-trip", async () => {
    const engine = await Engine.open(":memory:");

    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [
        {
          name: "identity",
          cid: "bafy...echo-wasm",
          requires: ["host:compute:time"],
        },
      ],
    };

    // Compute the expected CID via the engine helper so the round-trip is
    // self-consistent (G10-B exposes computeManifestCid for callers that
    // don't pre-compute via testing_compute_manifest_cid).
    const expectedCid = await engine.computeManifestCid(manifest);
    const installedCid = await engine.installModule(manifest, expectedCid);
    expect(installedCid).toBe(expectedCid);

    // Re-installing the same manifest is idempotent at the CID level — same
    // CID means same content.
    const installedAgain = await engine.installModule(manifest, expectedCid);
    expect(installedAgain).toBe(expectedCid);

    await engine.close();
  });

  // Phase-3 G17-C wave-5b (phase-3-backlog §6.6 deliverable 1):
  // un-skipped per G17-C ratification. The pre-G17-C skip
  // rationale was that the registry projection only keyed by
  // `entry.name` (not `<manifestName>:<entryName>`); G17-C extends
  // `Engine::manifest_registry()` to ALSO key by colon-joined name
  // AND adds the `register_subgraph` validation walk that surfaces
  // the typed `E_SANDBOX_MANIFEST_UNKNOWN` rejection at registration
  // time (no longer at execution time as a wallclock-after-zero-
  // progress shape). The post-uninstall rejection path is the
  // load-bearing end-to-end pin per pim-2 §3.6b — drives the
  // production `engine.registerSubgraph` entry point + asserts the
  // typed-error observable consequence.
  it("engine.uninstallModule(cid) clean release", async () => {
    const engine = await Engine.open(":memory:");

    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
    };

    const cid = await engine.computeManifestCid(manifest);
    await engine.installModule(manifest, cid);

    await engine.uninstallModule(cid);

    // After uninstall, registering a handler that references the module by
    // name MUST fail registration (the manifest is no longer resolvable).
    // The exact rejection code lives behind G10-B; this is the typed-error
    // pin (E_SANDBOX_MANIFEST_UNKNOWN per dx-r1-2b-2 catalog list).
    const { subgraph } = await import("@benten/engine");
    const sg = subgraph("h")
      .action("go")
      .sandbox({ module: "echo:identity" })
      .respond({ body: "$result" })
      .build();

    await expect(engine.registerSubgraph(sg)).rejects.toMatchObject({
      code: "E_SANDBOX_MANIFEST_UNKNOWN",
    });

    // uninstallModule MUST be idempotent — second call is a no-op.
    await expect(engine.uninstallModule(cid)).resolves.toBeUndefined();

    await engine.close();
  });

  it("install rejects CID mismatch with dual-CID diff in error (D16)", async () => {
    const engine = await Engine.open(":memory:");

    const manifest: ModuleManifest = {
      name: "echo",
      version: "0.0.1",
      modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
    };

    // Use a structurally valid but content-wrong CID so the parse
    // succeeds + the mismatch arm fires (rather than the parse-time
    // E_INPUT_LIMIT pre-check rejecting the placeholder).
    const otherManifest: ModuleManifest = {
      name: "different",
      version: "0.0.1",
      modules: [],
    };
    const wrongCid = await engine.computeManifestCid(otherManifest);

    await expect(engine.installModule(manifest, wrongCid)).rejects.toMatchObject({
      code: "E_MODULE_MANIFEST_CID_MISMATCH",
    });

    await engine.close();
  });

  // R6 Round-2 r6-r2-napi-3 closure: Instance 8 round-trip pin.
  // Asserts `BentenError.context` populates from the structured JSON
  // envelope that `engine_err` emits per G19-B (supersedes the
  // pre-G19-B `$$benten-context$$` sentinel suffix carrier).
  // Pre-Instance-8 the context was structurally typed at the TS
  // surface but `mapNativeError` never populated it; this regression
  // pin guards against the JSON-envelope emit being collapsed back to
  // a Display-only formatting in a future regression.
  it("CID mismatch error round-trips structured context fields (Instance 8)", async () => {
    const engine = await Engine.open(":memory:");
    try {
      const manifest: ModuleManifest = {
        name: "echo",
        version: "0.0.1",
        modules: [{ name: "identity", cid: "bafy...echo-wasm", requires: [] }],
      };

      // Use a structurally valid but content-wrong CID so the parse
      // succeeds + the mismatch arm fires. Compute the real CID for a
      // different manifest and supply that as the "expected" CID for
      // this manifest.
      const otherManifest: ModuleManifest = {
        name: "different",
        version: "0.0.1",
        modules: [],
      };
      const wrongCid = await engine.computeManifestCid(otherManifest);

      let caught: unknown = null;
      try {
        await engine.installModule(manifest, wrongCid);
      } catch (err) {
        caught = err;
      }
      expect(caught).not.toBeNull();
      const e = caught as { code: string; context?: Record<string, unknown> };
      expect(e.code).toBe("E_MODULE_MANIFEST_CID_MISMATCH");
      // Instance 8 contract: structured context populates from the
      // sentinel suffix. The `expected` / `computed` keys are wired
      // by `EngineError::context_json` for the
      // `ModuleManifestCidMismatch` variant.
      expect(e.context).toBeDefined();
      expect(typeof e.context!.expected).toBe("string");
      expect(typeof e.context!.computed).toBe("string");
      expect(e.context!.expected).not.toBe(e.context!.computed);
    } finally {
      await engine.close();
    }
  });
});

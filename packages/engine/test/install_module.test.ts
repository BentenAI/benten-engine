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

    const wrongCid = "bafy...definitely-not-the-right-cid";

    await expect(engine.installModule(manifest, wrongCid)).rejects.toMatchObject({
      code: "E_MODULE_MANIFEST_CID_MISMATCH",
    });

    await engine.close();
  });
});

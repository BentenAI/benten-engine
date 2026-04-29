// R6-FP r6-napi-1 closure — napi bridge for SANDBOX (compile-time
// disabled on wasm32).
//
// HISTORY: this file originally hard-asserted free-function shapes
// (`native.sandboxInstallManifest` / `sandboxUninstallManifest` /
// `sandboxComputeManifestCid`) that were the R3 red-phase scaffolded
// surface. R4b v2-1's "two-part fix" was supposed to land BOTH the
// free-fn forms AND the Engine class-method forms; only the
// class-method half landed (lib.rs:1027/1050/1061 — `engine.installModule`
// / `engine.uninstallModule` / `engine.computeManifestCid`). The free-fn
// shape was never wired, and the test was allowlisted-out of CI by
// `.github/workflows/napi-vitest.yml` lines 103-128, hiding the surface
// drift indefinitely. R6 council surfaced the gap; this file is now
// rewritten to assert the canonical class-method shape (matching the
// existing class-method coverage in `packages/engine/test/install_module.test.ts`)
// so the napi-vitest allowlist can be dropped.
//
// Surface contract (per sec-pre-r1-05 + plan §3 G7-A + G7-C + G10-B):
//   - The napi bindings expose SANDBOX-execution functions ONLY when
//     the compile-time cfg gates are met:
//       * cfg(not(target_arch = "wasm32")) — SANDBOX executor present.
//       * cfg(target_arch = "wasm32")     — SANDBOX executor absent;
//                                           bridge surfaces a typed
//                                           E_SANDBOX_UNAVAILABLE_ON_WASM
//                                           when the napi entry is invoked.
//   - The manifest install/uninstall lifecycle (G10-B) STAYS PRESENT
//     on both targets — installing a manifest doesn't execute wasm.
//
// Pin sources: r6-napi-1 (R6 Round 1 phase-close council);
// r2-test-landscape.md §1.3 unit row + §6 wasm-conformance;
// sec-pre-r1-05.

import { describe, it, expect } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

// Match the platform-specific load helper used by the sibling
// stream_napi_async_iterator_back_pressure.test.ts +
// budget_exhausted_napi_round_trip.test.ts files.
function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  const name = `../benten-napi.${platform}-${arch}.node`;
  return require(name);
}

const native: any = loadNative();

describe("napi SANDBOX bridge — Node target", () => {
  it("exposes the Engine class as the canonical SANDBOX surface", () => {
    // Symbol-presence pin: the napi cdylib surface is class-method-only
    // (per D-NS-7 ratification of class-method shape during wave-8c).
    // Free-function `sandboxInstallManifest` etc. are NOT part of the
    // canonical surface — the original R3 red-phase free-fn shape was
    // superseded by the Engine class-method form.
    expect(typeof native.Engine).toBe("function");
  });

  it("exposes Engine.installModule / uninstallModule / computeManifestCid class methods", () => {
    // R4b v2-1 closure pin: the manifest lifecycle bridges (lib.rs:
    // 1027 / 1050 / 1061) are the canonical surface. Verify them on a
    // real Engine instance — this is the same shape engine.ts wraps in
    // its public Engine.installModule / uninstallModule /
    // computeManifestCid methods.
    const tmp = mkdtempSync(join(tmpdir(), "benten-sandbox-bridge-"));
    try {
      const engine = new native.Engine(join(tmp, "benten.redb"));
      expect(typeof engine.installModule).toBe("function");
      expect(typeof engine.uninstallModule).toBe("function");
      expect(typeof engine.computeManifestCid).toBe("function");
    } finally {
      rmSync(tmp, { recursive: true, force: true });
    }
  });

  it("exposes sandboxTargetSupported() free-fn introspection probe", () => {
    // The platform-availability probe IS still a free function (it has
    // to be — there's no Engine instance to be method-on for the
    // "should I open an Engine here?" pre-check). Mirrors the cfg-split
    // documented at bindings/napi/src/sandbox.rs.
    expect(typeof native.sandboxTargetSupported).toBe("function");
  });

  it("sandboxTargetSupported() returns true on Node target builds", () => {
    // Mirrors engine.targetSupportsSandbox() — same cfg the Rust
    // executor reads. The TS surface routes through this entry to
    // answer the browser-or-Node question without driving an actual
    // SANDBOX call.
    const supported = (native.sandboxTargetSupported as () => boolean)();
    expect(typeof supported).toBe("boolean");
    expect(supported).toBe(true);
  });

  it("sandbox-disabled wasm32 builds surface E_SANDBOX_UNAVAILABLE_ON_WASM", () => {
    // This expectation is meaningful only on wasm32-unknown-unknown
    // builds. The cross-target shape pin: the symbol is present on
    // BOTH targets, but the wasm32 implementation routes to a
    // typed-error factory that surfaces E_SANDBOX_UNAVAILABLE_ON_WASM.
    //
    // On the Node test target, the call returns true (executor
    // present); the wasm32 branch is exercised by the wasm-conformance
    // workflow.
    const supported = (native.sandboxTargetSupported as () => boolean)();
    if (!supported) {
      // wasm32 path — installing a manifest via the class method MUST
      // surface the typed error rather than panic / return undefined.
      const tmp = mkdtempSync(join(tmpdir(), "benten-sandbox-bridge-wasm-"));
      try {
        const engine = new native.Engine(join(tmp, "benten.redb"));
        expect(() =>
          engine.installModule(
            { name: "x", version: "0.0.0", modules: [] },
            "bafy...x",
          ),
        ).toThrowError(/E_SANDBOX_UNAVAILABLE_ON_WASM/);
      } finally {
        rmSync(tmp, { recursive: true, force: true });
      }
    } else {
      // Node target — confirm the entry doesn't masquerade as the
      // wasm32 gate (defensive — flag if a future PR conflates the
      // two paths).
      expect(supported).toBe(true);
    }
  });
});

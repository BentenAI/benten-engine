// R3-F red-phase — napi bridge for SANDBOX (compile-time disabled on wasm32).
//
// Surface contract (per sec-pre-r1-05 + plan §3 G7-A):
//   - The napi bindings expose SANDBOX-execution functions ONLY when the
//     compile-time cfg gates are met. Specifically:
//       * cfg(not(target_arch = "wasm32")) — SANDBOX executor present.
//       * cfg(target_arch = "wasm32")     — SANDBOX executor absent;
//                                           bridge surfaces a typed
//                                           E_SANDBOX_UNAVAILABLE_ON_WASM
//                                           when the napi entry is invoked.
//   - The napi-side install/uninstall lifecycle (G10-B) STAYS PRESENT on
//     both targets — installing a manifest doesn't execute wasm.
//
// Tests are RED at landing time; G7-A (Rust executor) + G7-C (TS surface)
// + G10-B (manifest install) make them green.
//
// Pin sources: brief §"Surface ownership"; r2-test-landscape.md §1.3 unit
// row + §6 wasm-conformance; sec-pre-r1-05.

import { describe, it, expect } from "vitest";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

// The native artifact lives at bindings/napi/index.js per package.json. Load
// via createRequire because it's a CJS .node binding and this test file is
// ESM (vitest under "type": "module"). The require will fail in red-phase
// only if the napi build hasn't run; the surface assertions below are the
// real R3 pin.
const native = require("../index.js") as Record<string, unknown>;

describe("napi SANDBOX bridge — Node target", () => {
  it("exposes sandbox-execution entry on non-wasm targets", () => {
    // Pin the symbol set the TS wrapper depends on. G7-A names these; the
    // exact entry-point names land in bindings/napi/src/sandbox.rs and get
    // re-exported via the napi-rs macro into index.d.ts.
    //
    // The TS wrapper engine.callStream / engine.call routes through these
    // when a SubgraphSpec contains a SANDBOX node. If the entry is missing
    // (e.g. a wasm32 build accidentally shipped to a Node consumer), the
    // wrapper raises E_SANDBOX_UNAVAILABLE_ON_WASM upfront rather than
    // failing mid-call.
    expect(typeof native.sandboxInstallManifest).toBe("function");
    expect(typeof native.sandboxUninstallManifest).toBe("function");
    expect(typeof native.sandboxComputeManifestCid).toBe("function");
    expect(typeof native.sandboxTargetSupported).toBe("function");
  });

  it("sandboxTargetSupported() returns true on Node target builds", () => {
    // Mirrors engine.targetSupportsSandbox() — same cfg the Rust executor
    // reads. The TS surface routes through this entry to answer the
    // browser-or-Node question without driving an actual SANDBOX call.
    const supported = (native.sandboxTargetSupported as () => boolean)();
    expect(typeof supported).toBe("boolean");
    expect(supported).toBe(true);
  });

  it("sandbox-disabled wasm32 builds surface E_SANDBOX_UNAVAILABLE_ON_WASM", () => {
    // This expectation is meaningful only on wasm32-unknown-unknown builds.
    // The cross-target shape pin is below (the symbol is present on BOTH
    // targets, but the wasm32 implementation routes to a typed-error
    // factory that surfaces E_SANDBOX_UNAVAILABLE_ON_WASM).
    //
    // On the Node test target, the call returns true (executor present);
    // the assertion shape is a placeholder for the wasm32 CI matrix entry
    // wired by G10-A-browser. R5 mini-review may collapse this into the
    // wasm-conformance.yml workflow runner.
    const supported = (native.sandboxTargetSupported as () => boolean)();
    if (!supported) {
      // wasm32 path — calling the executor entry MUST surface the typed
      // error, not panic, not return undefined.
      expect(() =>
        (native.sandboxInstallManifest as (m: unknown, c: string) => unknown)(
          { name: "x", version: "0.0.0", modules: [] },
          "bafy...x",
        ),
      ).toThrowError(/E_SANDBOX_UNAVAILABLE_ON_WASM/);
    } else {
      // Node target — confirm the entry doesn't masquerade as the wasm32
      // gate (defensive — flag if a future PR conflates the two paths).
      expect(supported).toBe(true);
    }
  });
});

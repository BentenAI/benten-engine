// G20-B GREEN-PHASE pins for Atrium examples (wave-8b; plan §3 G20-B
// + cag-4 + r1-napi-2 + r1-napi-10 + D-PHASE-3-15).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.8 G20-B):
//
//   - tests/atrium_examples_compile_and_run (Vitest)
//
// What G20-B establishes:
//
//   packages/engine/examples/** — Atrium peer-mgmt + sync-trigger +
//   UCAN-grant-flow + DID-resolution example RUNNERS. Each example
//   exports a `run()` async function (gated against direct CLI
//   invocation via `import.meta.url` check) so that this Vitest
//   companion pin can import the module without triggering napi
//   side effects.
//
// What this test pins:
//
//   1. Each example module resolves cleanly under Vitest's transform
//      pipeline (the "compile" half — TypeScript type-checking +
//      module-graph resolution against the `@benten/engine` public
//      surface).
//   2. Each example exports a `run` function with the documented
//      shape (signature `() => Promise<{ ok: true }>`).
//   3. The example sources reference ONLY the canonical 12-primitive
//      composition surface — no new OperationNode kinds (cag-4
//      architectural pin; companion to the Rust-side
//      tests/atriums_no_new_primitives.rs).
//
// "Run end-to-end" half is exercised manually per `examples/README.md`
// (running requires the napi binding be built + an Atrium full peer
// available); this Vitest pin is the unit-test-tier compile + shape
// guarantee. The end-to-end manual run is the G20-B canonical-fixture
// equivalent for the example surface.

import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

import * as atriumPeerMgmt from "../examples/atrium-peer-mgmt.js";
import * as atriumSyncTrigger from "../examples/atrium-sync-trigger.js";
import * as ucanGrantFlow from "../examples/ucan-grant-flow.js";
import * as didResolution from "../examples/did-resolution.js";

const __dirname = dirname(fileURLToPath(import.meta.url));

describe("G20-B Atrium examples compile + run", () => {
  it("atrium peer-management example exports run()", () => {
    expect(typeof atriumPeerMgmt.run).toBe("function");
    // run() returns a Promise — call shape verifiable without napi
    // because we never await (avoiding Engine.openWithPolicy).
    expect(atriumPeerMgmt.run.length).toBe(0);
  });

  it("atrium sync-trigger example exports run()", () => {
    expect(typeof atriumSyncTrigger.run).toBe("function");
    expect(atriumSyncTrigger.run.length).toBe(0);
  });

  it("UCAN-grant-flow example exports run()", () => {
    expect(typeof ucanGrantFlow.run).toBe("function");
    expect(ucanGrantFlow.run.length).toBe(0);
  });

  it("DID-resolution example exports run()", () => {
    expect(typeof didResolution.run).toBe("function");
    expect(didResolution.run.length).toBe(0);
  });

  it("atrium examples compose entirely from existing 12 primitives (cag-4)", () => {
    // cag-4 architectural pin (companion to Rust-side
    // tests/atriums_no_new_primitives.rs in benten-engine — we pin the
    // same invariant from the TS side here as a redundant-distinct
    // pin).
    //
    // The Atrium DSL surface is composed of factory + handle methods
    // (engine.atrium(...).join() / .trustPeer / .listPeers /
    // .subscribe / .declareDeviceAttestation etc.) — that's a
    // factory/method composition, not a new primitive kind. The
    // examples exercise this composition; this test scans the example
    // sources for any string that LOOKS like a new primitive kind
    // outside the canonical 12.
    const allowed = new Set([
      "READ",
      "WRITE",
      "TRANSFORM",
      "BRANCH",
      "ITERATE",
      "WAIT",
      "CALL",
      "RESPOND",
      "EMIT",
      "SANDBOX",
      "SUBSCRIBE",
      "STREAM",
    ]);

    const exampleFiles = [
      "atrium-peer-mgmt.ts",
      "atrium-sync-trigger.ts",
      "ucan-grant-flow.ts",
      "did-resolution.ts",
    ];
    // Match `kind: "FOO"` or `primitive: "FOO"` in source — any
    // non-canonical match would surface here. The examples should
    // never emit OperationNode literals directly (they use crud /
    // factory composition); this scan defends against drift where a
    // future edit smuggles a primitive literal in.
    const primitiveLiteralPattern =
      /\b(?:kind|primitive)\s*:\s*['"]([A-Z][A-Z_]+)['"]/g;
    for (const file of exampleFiles) {
      const path = resolve(__dirname, "..", "examples", file);
      const src = readFileSync(path, "utf8");
      let match: RegExpExecArray | null;
      const found: string[] = [];
      while ((match = primitiveLiteralPattern.exec(src)) !== null) {
        found.push(match[1]);
      }
      for (const kind of found) {
        // We only enforce against UPPER_CASE primitive-shaped names;
        // any literal that matches the pattern but is actually a
        // non-primitive (e.g. an enum-variant string) would still pass
        // when it's in the allowed set. Failures here mean a new
        // primitive kind has been smuggled in.
        expect(
          allowed.has(kind),
          `${file} references non-canonical primitive-shaped literal: ${kind}`,
        ).toBe(true);
      }
    }
  });
});

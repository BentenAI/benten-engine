// R3-C RED-PHASE TS Vitest pin for the namespaced engine.atrium
// architectural assertion (G16-D wave-6b; r1-napi-10).
//
// ## Pin source
//
// - r2-test-landscape §2.4 G16-D row
//   `atrium_namespaced_engine_atrium_only_no_top_level_engine_atrium_methods`.
// - `r1-napi-10` (namespacing surface; no top-level engine.atrium*).
// - `D-PHASE-3-15` (subsystem method namespacing).
//
// ## What this pins (distinct from atrium.test.ts)
//
// `atrium.test.ts` exercises the namespaced surface end-to-end.
// THIS file pins the architectural NEGATIVE: NO top-level
// `engine.atrium*` methods exist.

import { describe, it } from "vitest";

describe("atrium namespacing (R3-C RED-PHASE architectural pin)", () => {
  it.skip("RED-PHASE: G16-D wave-6b — r1-napi-10 — engine.atrium-only; no top-level engine.atrium* methods", async () => {
    // r1-napi-10 LOAD-BEARING architectural pin. G16-D implementer
    // wires this:
    //
    //   import { Engine } from "@benten/engine";
    //   const engine = await Engine.open(":memory:");
    //   // The atrium subsystem MUST be reachable as engine.atrium:
    //   expect(engine.atrium).toBeDefined();
    //   expect(typeof engine.atrium.join).toBe("function");
    //   // No flattened top-level atrium-prefixed methods:
    //   const proto = Object.getPrototypeOf(engine);
    //   const protoMethods = Object.getOwnPropertyNames(proto);
    //   const flattened = protoMethods.filter(m =>
    //     m.startsWith("atrium") && m !== "atrium"
    //   );
    //   expect(flattened).toHaveLength(0);
    //   // Same check on the .d.ts type surface (via runtime
    //   // Object.keys reflection of the engine instance):
    //   const instanceKeys = Object.keys(engine);
    //   const flatInstance = instanceKeys.filter(k =>
    //     k.startsWith("atrium") && k !== "atrium"
    //   );
    //   expect(flatInstance).toHaveLength(0);
    //
    // OBSERVABLE consequence: future refactors that flatten atrium
    // operations onto the top-level Engine fail this test.
    throw new Error("G16-D fills no-top-level-atrium-methods architectural assertion");
  });
});

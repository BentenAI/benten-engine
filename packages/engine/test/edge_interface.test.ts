// R3-E RED-PHASE pins for G19-D Edge interface fix
// (wave 7 parallel; §7.9 phantom cid + missing properties fix).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-D):
//
//   - tests/edge_interface_no_phantom_cid_field
//   - tests/edge_interface_exposes_properties_field
//
// What G19-D establishes (§7.9):
//
//   packages/engine/src/types.ts::Edge — drop `cid` field (napi never
//   emits it; phantom field per r1-doc-engineer findings); add
//   `properties` field (napi DOES emit it; consumers want it surfaced).
//
// RED-PHASE discipline:
//
//   The current Edge interface has `cid` (phantom) AND lacks
//   `properties`. R5 implementer drops .skip + verifies the corrected
//   shape.

import { describe, it, expect } from "vitest";

describe("G19-D Edge interface fix (§7.9)", () => {
  it.skip("RED-PHASE: G19-D wave-7 — Edge interface has no phantom `cid` field", async () => {
    // §7.9 pin. G19-D implementer wires this:
    //
    //   const { Engine } = await import("@benten/engine");
    //   const engine = await Engine.open(":memory:");
    //
    //   // Construct a real edge through a CRUD post:create + read flow:
    //   const post = await engine.registerSubgraph(crud("post"));
    //   const result = await engine.call(post.id, "post:create", { title: "x" });
    //   const fetched = await engine.read("post", result.cid);
    //
    //   // Inspect the edges:
    //   for (const edge of fetched.edges) {
    //     // POSITIVE pin: properties is present:
    //     expect(edge).toHaveProperty("properties");
    //     // NEGATIVE pin: cid is absent (phantom field dropped per §7.9):
    //     expect(edge).not.toHaveProperty("cid");
    //   }
    //
    // OBSERVABLE consequence: TS callers no longer access `edge.cid`
    // (which was always undefined). Defends against silent-undefined
    // failure shape where consumers assumed a field that never existed.
  });

  it.skip("RED-PHASE: G19-D wave-7 — Edge interface exposes `properties` field", async () => {
    // §7.9 pin. G19-D implementer wires this:
    //
    //   // Same setup as above:
    //   const fetched = await engine.read("post", result.cid);
    //
    //   for (const edge of fetched.edges) {
    //     // POSITIVE pin: properties is a record of edge metadata:
    //     expect(typeof edge.properties).toBe("object");
    //     expect(edge.properties).not.toBeNull();
    //   }
    //
    //   // The TS .d.ts must declare `properties: Record<string, unknown>`:
    //   //
    //   //   import type { Edge } from "@benten/engine";
    //   //   type Edge_HasProperties = Edge["properties"]; // compile-time pin
    //   //   const _: Record<string, unknown> = {} as Edge_HasProperties;
    //
    // OBSERVABLE consequence: callers can read edge metadata that was
    // already flowing through napi but unsurfaced in the TS shape. End-to-end
    // pin per pim-2 §3.6b — would FAIL if the field were declared in
    // .d.ts but not present in the runtime payload (sentinel-only fix).
  });
});

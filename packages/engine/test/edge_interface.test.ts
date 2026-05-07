// G19-D Edge interface fix — pure-TS structural pins
// (wave-7 parallel; §7.9 phantom cid + missing properties fix).
//
// What G19-D establishes (§7.9):
//
//   packages/engine/src/types.ts::Edge — drops `cid` field (napi never
//   emits it; phantom field per r1-doc-engineer findings); adds
//   `properties` field (napi DOES emit it; consumers want it surfaced).
//
// Why pure-TS not engine-end-to-end:
//
//   The Edge interface is a TS type-only declaration. The structural
//   guarantee (`cid` absent + `properties` present) is verifiable at
//   the type level + by constructing a fixture object that matches the
//   interface. The Rust-side LOAD-BEARING parity meta-test
//   (`crates/benten-engine/tests/ts_surface_parity_meta_test.rs`)
//   independently asserts the napi-side `edge_to_json` projector emits
//   `properties` and never `cid`.
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-D):
//
//   - tests/edge_interface_no_phantom_cid_field
//   - tests/edge_interface_exposes_properties_field

import { describe, it, expect } from "vitest";
import type { Edge } from "@benten/engine";

describe("G19-D Edge interface fix (§7.9)", () => {
  it("Edge interface declares the canonical post-fix fields (source/target/label/properties)", () => {
    // Compile-time pin: an Edge object literal with the canonical
    // post-fix shape MUST type-check. If `properties` were missing
    // from the interface declaration, the field assignment below
    // would error at TS compile time.
    const edge: Edge = {
      source: "bsource123",
      target: "btarget456",
      label: "GRANTED_TO",
      properties: { tag: "test" },
    };
    expect(edge.source).toBe("bsource123");
    expect(edge.target).toBe("btarget456");
    expect(edge.label).toBe("GRANTED_TO");
    expect(edge.properties).toEqual({ tag: "test" });
  });

  it("Edge interface accepts undefined properties (optional field semantics)", () => {
    // Edge.properties is OPTIONAL — undefined when the underlying napi
    // producer emits no property bag. The TS interface MUST tolerate
    // both shapes.
    const edge: Edge = {
      source: "bsource",
      target: "btarget",
      label: "NEXT",
    };
    expect(edge.properties).toBeUndefined();
  });

  it("Edge interface has no `cid` field (phantom dropped per §7.9)", () => {
    // Compile-time pin: assigning to `edge.cid` MUST fail TS check.
    // Vitest can't enforce a type-level negative directly; use an
    // `as never` indirect probe — the keyof Edge type set MUST NOT
    // contain "cid". This test will fail at compile time if `cid` is
    // re-added to the interface.
    type EdgeKeys = keyof Edge;
    // The following type-assertion would FAIL the typecheck if "cid"
    // were a member of EdgeKeys (the "cid" literal would extend
    // EdgeKeys, so the never branch wouldn't fire):
    type CidIsNotInEdge = "cid" extends EdgeKeys ? never : true;
    const _proof: CidIsNotInEdge = true;
    expect(_proof).toBe(true);
  });

  it("Edge interface preserves the by-design omission of anchor_id (Node.anchor_id precedent)", () => {
    // Per types.ts JSDoc on Edge: `anchor_id` is `#[serde(skip)]` on
    // the Rust side and consequently never emitted by the napi
    // projection. The TS interface intentionally omits it.
    type EdgeKeys = keyof Edge;
    type AnchorIdIsNotInEdge = "anchor_id" extends EdgeKeys ? never : true;
    const _proof: AnchorIdIsNotInEdge = true;
    expect(_proof).toBe(true);
  });
});

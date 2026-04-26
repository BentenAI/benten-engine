// Phase 2b R4-FP B-3 — TS-side parity check: `engine.installModule(manifest,
// manifestCid)` accepts the SAME ModuleManifest schema that the Rust-side
// `Engine::install_module` accepts. Without this pin, the TS surface and the
// Rust surface could drift independently and a manifest accepted on one side
// would crash on the other -- defeating the cross-language contract that
// makes the cross-process CID pin operator-actionable.
//
// TDD red-phase. Tests fail until G10-B (TS-side) lands the napi bridge.
//
// Pin sources:
//   - r2-test-landscape.md §2.4 `manifest_ts_validates_against_rust_schema`.
//   - .addl/phase-2b/00-implementation-plan.md §3.2 G10-B.
//   - r1-security-auditor.json D9 RESOLVED — canonical DAG-CBOR manifest
//     bytes MUST agree across language boundaries (the CID computed by
//     `engine.computeManifestCid(m)` MUST equal the CID computed by
//     `testing_compute_manifest_cid(&m)` in Rust over the same logical
//     manifest input).
//
// Owned by R4-FP B-3 (R3-followup); R5 owner G10-B (TS-side).

import { describe, it, expect } from "vitest";
import { Engine } from "@benten/engine";
import type { ModuleManifest } from "@benten/engine";

describe("ModuleManifest schema parity (TS <-> Rust)", () => {
  it("accepts the same field-set the Rust schema accepts", async () => {
    // Schema parity: every required + optional field on the Rust
    // ModuleManifest struct MUST also be representable in the TS type.
    // The R5 G10-B brief pins this set; R4-FP B-3 captures the names so
    // the implementer cannot quietly drop a field (the test would then
    // fail to compile -- TS type-check OR runtime).
    const manifest: ModuleManifest = {
      name: "acme.posts",
      version: "0.0.1",
      modules: [
        {
          name: "post-handler",
          cid: "bafy...post-handler-wasm",
          requires: ["host:compute:time"],
        },
      ],
      // signature: undefined — Phase-3 reserved; OMITTED from canonical
      // bytes when undefined (D9 forward-compat). Test pin lives in
      // module_manifest_signature_field_reserved.rs (Rust-side); this
      // TS pin asserts the `signature?: ManifestSignature` field is
      // typed as optional (the `?` is the parity check -- if the
      // implementer writes `signature: ManifestSignature | null` the
      // canonical-bytes serializer would emit `null` instead of
      // omitting the key, breaking forward-compat).
    };

    const engine = await Engine.open(":memory:");
    try {
      // The engine accepting the manifest with computeManifestCid is
      // the load-bearing parity check: if the Rust side serializes a
      // field the TS side doesn't, computeManifestCid would either
      // throw or produce a CID that doesn't match the Rust-side CID.
      const cid = await engine.computeManifestCid(manifest);
      expect(typeof cid).toBe("string");
      expect(cid).toMatch(/^bafy/); // CIDv1 base32 prefix sanity check
    } finally {
      await engine.close();
    }
  });

  it("computeManifestCid agrees with Rust testing_compute_manifest_cid", async () => {
    // The strongest parity assertion: a fixture manifest whose Rust-
    // computed CID is pinned in a fixture file MUST match the CID the
    // TS side computes. R5 G10-B + R3-fixtures land the pinned CID;
    // this test compares against it.
    //
    // R5 wires:
    //   1. const manifest: ModuleManifest = <fixture from
    //      crates/benten-engine/tests/fixtures/manifest_parity.json>;
    //   2. const expectedCidFromRust = <pinned in the same fixture file
    //      under key "expected_cid_v1_base32">;
    //   3. const tsCid = await engine.computeManifestCid(manifest);
    //   4. expect(tsCid).toBe(expectedCidFromRust);
    const engine = await Engine.open(":memory:");
    try {
      // RED-PHASE STUB: R5 G10-B fills in the fixture-driven assertion.
      // Failing the test loudly so red-phase status is unambiguous.
      throw new Error(
        "R5 G10-B pending — wire fixture-driven CID parity assertion " +
          "(Rust testing_compute_manifest_cid <-> TS engine.computeManifestCid)",
      );
    } finally {
      await engine.close();
    }
  });

  it("installModule REQUIRES the manifestCid arg at the type level (D16)", async () => {
    // D16 RESOLVED-FURTHER — the TS surface MUST mirror the Rust
    // 2-arg-required signature. A 1-arg overload would silently
    // compute-and-trust the CID, defeating the supply-chain pin.
    //
    // This is partly a compile-time check (the TS type signature for
    // installModule must declare `manifestCid: Cid` as required, not
    // optional) and partly a runtime sanity assertion.
    const engine = await Engine.open(":memory:");
    try {
      const manifest: ModuleManifest = {
        name: "acme.posts",
        version: "0.0.1",
        modules: [{ name: "h", cid: "bafy...wasm", requires: [] }],
      };
      const cid = await engine.computeManifestCid(manifest);
      // The 2-arg call must compile + run.
      const installed = await engine.installModule(manifest, cid);
      expect(installed).toBe(cid);
      // (Compile-time pin) the following SHOULD NOT compile:
      //   await engine.installModule(manifest);
      // If a future contributor adds a 1-arg overload, this test won't
      // catch it directly -- the Rust-side
      // install_module_requires_cid_arg_at_compile_time test does. The
      // TS-side guard is a tsd / @ts-expect-error harness that R5
      // G10-B can wire if drift becomes a real risk.
    } finally {
      await engine.close();
    }
  });
});

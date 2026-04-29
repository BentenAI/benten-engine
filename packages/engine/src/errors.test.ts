// Phase 1 R3 Vitest: typed error classes codegenned from ERROR-CATALOG.md.
// Covers E_DSL_* codes and the fixHint surfacing contract.
// Status: FAILING until B7 (error-catalog typed-class codegen) lands.

import { describe, expect, it } from "vitest";
import * as errors from "@benten/engine/errors";
import { mapNativeError } from "@benten/engine/errors";

describe("typed error classes", () => {
  it("typed_error_classes_generated_from_catalog", () => {
    // Spot-check a handful of codes from ERROR-CATALOG.md.
    expect(errors.ECapDenied).toBeTypeOf("function");
    expect(errors.EInvCycle).toBeTypeOf("function");
    expect(errors.EIvmViewStale).toBeTypeOf("function");
    expect(errors.EPrimitiveNotImplemented).toBeTypeOf("function");
    expect(errors.EDslInvalidShape).toBeTypeOf("function");
    expect(errors.EDslUnregisteredHandler).toBeTypeOf("function");

    // Each error has a stable .code property matching the catalog.
    const e = new errors.ECapDenied("denied");
    expect(e.code).toBe("E_CAP_DENIED");
    expect(e.fixHint).toBeTypeOf("string");
    expect(e.fixHint.length).toBeGreaterThan(0);
    expect(e.toString()).toContain("E_CAP_DENIED");
    expect(e.toString()).toContain(e.fixHint);
  });

  it("dsl_invalid_shape_thrown_on_malformed_input", async () => {
    const { Engine } = await import("@benten/engine");
    const { mkdtempSync } = await import("node:fs");
    const { tmpdir } = await import("node:os");
    const { join } = await import("node:path");
    const tmp = mkdtempSync(join(tmpdir(), "benten-err-"));
    const engine = await Engine.open(join(tmp, "benten.redb"));

    // Pass malformed subgraph (missing required shape).
    await expect(engine.registerSubgraph({} as never)).rejects.toMatchObject({ code: "E_DSL_INVALID_SHAPE" });
    await engine.close();
  });

  it("map_native_error_round_trips_context_via_benten_context_sentinel", () => {
    // R6FP-tail (Round-2 Instance 8) regression pin.
    //
    // Pre-fix: napi error message of the form `format!("{code}: {err}")`
    // dropped per-variant structured fields at the napi -> TS boundary;
    // `mapNativeError` constructed the typed subclass with `(message)`
    // only and `error.context` was always `undefined`.
    //
    // Post-fix: the Rust adapter (`engine_err`) appends a JSON-encoded
    // structured-field bag as a `$$benten-context$$` suffix. This test
    // simulates that exact napi message shape + asserts the parsed bag
    // surfaces on `error.context`. Contract: the sentinel string
    // `" :: $$benten-context$$"` is the cross-layer carrier.
    const sim = new Error(
      "E_INV_DEPTH_EXCEEDED: registration violated invariants " +
        ":: $$benten-context$$" +
        '{"depth_actual":42,"depth_max":32,"longest_path":["a","b","c"]}',
    );
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.EInvDepthExceeded);
    expect(typed.code).toBe("E_INV_DEPTH_EXCEEDED");
    expect(typed.context).toBeDefined();
    expect(typed.context).toMatchObject({
      depth_actual: 42,
      depth_max: 32,
    });
    // Message strips the sentinel suffix; consumers reading `.message`
    // see only the human-readable head.
    expect(typed.message).not.toContain("$$benten-context$$");
    expect(typed.message).toContain("registration violated invariants");
  });

  it("map_native_error_handles_missing_sentinel_unchanged", () => {
    // Backward-compatibility pin: a napi error with no
    // `$$benten-context$$` suffix is still mapped to the typed subclass
    // identically to pre-Instance-8 behavior — `error.context` is
    // `undefined` (NOT a partial / malformed bag).
    const sim = new Error("E_NOT_FOUND: node not found");
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.ENotFound);
    expect(typed.context).toBeUndefined();
    expect(typed.message).toContain("E_NOT_FOUND: node not found");
  });

  it("map_native_error_tolerates_malformed_context_json", () => {
    // Defensive pin: if the JSON tail is malformed (e.g. truncation
    // mid-stream), `mapNativeError` MUST fall back to `context = undefined`
    // rather than throwing — the typed-error path on the catalog code
    // remains the load-bearing shape.
    const sim = new Error(
      "E_NOT_FOUND: node missing :: $$benten-context$${not-json",
    );
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.ENotFound);
    expect(typed.context).toBeUndefined();
  });

  it("dsl_unregistered_handler_has_suggestions", async () => {
    const { Engine, crud } = await import("@benten/engine");
    const { mkdtempSync } = await import("node:fs");
    const { tmpdir } = await import("node:os");
    const { join } = await import("node:path");
    const tmp = mkdtempSync(join(tmpdir(), "benten-err-"));
    const engine = await Engine.open(join(tmp, "benten.redb"));
    await engine.registerSubgraph(crud("post"));

    try {
      await engine.call("nonexistent:handler", "post:create", {});
      expect.fail("should have thrown E_DSL_UNREGISTERED_HANDLER");
    } catch (err) {
      const e = err as errors.EDslUnregisteredHandler;
      expect(e.code).toBe("E_DSL_UNREGISTERED_HANDLER");
      // fixHint contains a "did you mean?" for near matches
      expect(e.fixHint).toMatch(/post/i);
    }
    await engine.close();
  });
});

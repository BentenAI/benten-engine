// Phase 1 R3 Vitest: typed error classes codegenned from ERROR-CATALOG.md.
// Covers E_DSL_* codes and the fixHint surfacing contract.
// Status: FAILING until B7 (error-catalog typed-class codegen) lands.

import { describe, expect, it } from "vitest";
import * as errors from "@benten/engine/errors";

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

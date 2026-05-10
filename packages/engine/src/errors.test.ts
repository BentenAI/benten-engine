// Phase 1 R3 Vitest: typed error classes codegenned from ERROR-CATALOG.md.
// Covers E_DSL_* codes and the fixHint surfacing contract.
// Status: FAILING until B7 (error-catalog typed-class codegen) lands.

import { describe, expect, it } from "vitest";
import * as errors from "@benten/engine/errors";
import { mapNativeError } from "@benten/engine/errors";

describe("typed error classes", () => {
  it("compound_stem_class_names_preserve_inner_pascal_case_per_rust_enum_parity", () => {
    // R6 R2 dx-r6-r2-2 closure: wire codes whose underscored part is a
    // 2-token compound (DEVSERVER, NONFINITE) must produce class names
    // matching the Rust enum's inner-PascalCase form (DevServerStopped,
    // ValueFloatNonFinite) — NOT the default first-cap+rest-lower form
    // (DevserverStopped, ValueFloatNonfinite). The codegen at
    // `scripts/codegen-errors.ts` carries a `COMPOUND_STEM_EXPANSIONS`
    // table that maps these stems to the inner-PascalCase form. This
    // pin defends against silent drift if a future codegen edit drops
    // the special-case map or the Rust enum spelling shifts without
    // a matching codegen update.
    expect(errors.EDevServerStopped).toBeTypeOf("function");
    expect(new errors.EDevServerStopped("test").code).toBe(
      "E_DEVSERVER_STOPPED",
    );
    expect(errors.EValueFloatNonFinite).toBeTypeOf("function");
    expect(new errors.EValueFloatNonFinite("test").code).toBe(
      "E_VALUE_FLOAT_NONFINITE",
    );
    // Negative parity: the pre-fix spellings MUST NOT exist as exports
    // (silent drift defense — if the special-case map ever gets dropped
    // these names would re-appear).
    expect(
      (errors as Record<string, unknown>).EDevserverStopped,
    ).toBeUndefined();
    expect(
      (errors as Record<string, unknown>).EValueFloatNonfinite,
    ).toBeUndefined();
  });

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

  it("map_native_error_round_trips_context_via_g19_b_json_envelope", () => {
    // R6FP-tail (Round-2 Instance 8) regression pin — UPDATED for G19-B.
    //
    // Pre-G19-B: the napi adapter (`engine_err`) appended a JSON-encoded
    // structured-field bag as a `$$benten-context$$` suffix on the
    // message string. The test originally pinned that exact carrier
    // shape ("`<message> :: $$benten-context$$<json>`").
    //
    // Post-G19-B (§7.2): the carrier is a JSON-shape envelope
    // `{ code, message, fields? }` that occupies the entire napi error
    // message body — NOT a sentinel-suffix on a prefix-carrier message.
    // `mapNativeError` JSON-parses the body via `tryParseJsonEnvelope`
    // (Path 1 in `errors.ts::mapNativeError`) and returns the typed
    // subclass with `.context` populated from `fields`.
    //
    // This test is the live continuation of the Instance-8 contract:
    // structured-field round-trip from napi to TS via the engine's
    // structured-field carrier. The carrier's wire shape changed; the
    // contract did not.
    const sim = new Error(
      JSON.stringify({
        code: "E_INV_DEPTH_EXCEEDED",
        message: "registration violated invariants",
        fields: {
          depth_actual: 42,
          depth_max: 32,
          longest_path: ["a", "b", "c"],
        },
      }),
    );
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.EInvDepthExceeded);
    expect(typed.code).toBe("E_INV_DEPTH_EXCEEDED");
    expect(typed.context).toBeDefined();
    expect(typed.context).toMatchObject({
      depth_actual: 42,
      depth_max: 32,
    });
    // Message is the `message` field from the envelope; the JSON
    // envelope itself is consumed by mapNativeError and never leaks
    // into the user-visible BentenError.message.
    expect(typed.message).toBe("registration violated invariants");
  });

  it("map_native_error_handles_legacy_prefix_carrier_unchanged", () => {
    // Backward-compatibility pin (G19-B Path 2 — legacy `code: prefix`
    // carrier): a napi error message that is NOT a JSON envelope is
    // still mapped to the typed subclass via the regex extractCode
    // path; `error.context` is `undefined` (the legacy carrier never
    // had structured fields).
    const sim = new Error("E_NOT_FOUND: node not found");
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.ENotFound);
    expect(typed.context).toBeUndefined();
    expect(typed.message).toContain("E_NOT_FOUND: node not found");
  });

  it("map_native_error_tolerates_malformed_envelope_json", () => {
    // Defensive pin (G19-B): if the JSON envelope is malformed (e.g.
    // truncation mid-stream), `mapNativeError` MUST fall back to the
    // legacy regex path rather than throwing — the typed-error path
    // on the catalog code remains the load-bearing shape.
    const sim = new Error("{not-valid-json E_NOT_FOUND: node missing");
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.ENotFound);
    expect(typed.context).toBeUndefined();
  });

  it("map_native_error_round_trips_devserver_stopped_to_typed_class", () => {
    // R6 Round-3 r6-r3-napi-1 regression pin (16th producer/consumer
    // drift instance closure).
    //
    // Pre-fix: PR #66 promoted `E_DEVSERVER_STOPPED` from a hand-typed
    // string to a typed catalog variant + generated `EDevServerStopped`
    // in `errors.generated.ts`, BUT the matching `CODE_TO_CTOR`
    // registration in this file was missed; the wire-form prefix
    // `E_DEVSERVER_STOPPED:` from `bindings/napi/src/devserver.rs::
    // devserver_stopped` round-tripped to the synthetic `E_UNKNOWN`
    // fallback rather than the typed subclass — defeating the original
    // promotion's purpose.
    //
    // Post-fix: `mapNativeError` produces the typed `EDevServerStopped`
    // instance + JS callers can dispatch on `instanceof`.
    const sim = new Error(
      "E_DEVSERVER_STOPPED: devserver method called after stop()",
    );
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.EDevServerStopped);
    expect(typed.code).toBe("E_DEVSERVER_STOPPED");
  });

  it("map_native_error_round_trips_reload_subscriber_unsubscribed_to_typed_class", () => {
    // R6 Round-3 r6-r3-napi-1 regression pin (continued).
    //
    // Same shape as `E_DEVSERVER_STOPPED` above — PR #66 promoted the
    // catalog variant; CODE_TO_CTOR registration was missed. Post-fix
    // the typed dispatch through `mapNativeError` works.
    const sim = new Error(
      "E_RELOAD_SUBSCRIBER_UNSUBSCRIBED: drain() after unsubscribe()",
    );
    const typed = mapNativeError(sim);
    expect(typed).toBeInstanceOf(errors.EReloadSubscriberUnsubscribed);
    expect(typed.code).toBe("E_RELOAD_SUBSCRIBER_UNSUBSCRIBED");
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

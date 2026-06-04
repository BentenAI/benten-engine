// Phase-3 G19-B ACTIVATED pins — mapNativeError + BentenError.context
// TS-side surface (wave-7 parallel; §7.2).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-B):
//
//   - errors.test.ts (TS Vitest)
//
// What G19-B establishes (§7.2):
//
//   - bindings/napi/src/error.rs::engine_err emits a JSON-shape
//     `{ code, message, fields? }` carrier on the napi error message
//     (replaces the pre-G19-B `<message> :: $$benten-context$$<json>`
//     sentinel-suffix shape).
//
//   - packages/engine/src/errors.ts::mapNativeError JSON-parses the
//     message body and returns a typed BentenError subclass with a
//     `.context` accessor surfacing the structured fields.
//
//   - packages/engine/src/errors.generated.ts emits CODE_TO_CTOR_GENERATED
//     keyed by ErrorCode → typed-subclass constructor.

import { describe, it, expect } from "vitest";
import {
  BentenError,
  ECapDenied,
  EInvRegistration,
  mapNativeError,
} from "../src/errors.js";
import { CODE_TO_CTOR_GENERATED } from "../src/errors.generated.js";

describe("G19-B BentenError.context structured-field surfacing (§7.2)", () => {
  it("G19-B wave-7 — mapNativeError carries context fields from the JSON envelope", () => {
    // Synthesize a napi-shaped Error (JSON-payload message body):
    const napiErr = new Error(
      JSON.stringify({
        code: "E_CAP_DENIED",
        message: "actor cannot write to target",
        fields: { actor: "alice", action: "kv:write", target: "post:1" },
      }),
    );

    const mapped = mapNativeError(napiErr);
    expect(mapped.code).toBe("E_CAP_DENIED");
    expect(mapped.context).toEqual({
      actor: "alice",
      action: "kv:write",
      target: "post:1",
    });
    // Typed-subclass surfacing — must NOT be the generic BentenError
    // class for a known catalog code:
    expect(mapped).toBeInstanceOf(ECapDenied);
    expect(mapped.constructor.name).toBe("ECapDenied");
    expect(mapped.message).toBe("actor cannot write to target");
  });

  it("G19-B wave-7 — CODE_TO_CTOR_GENERATED has no E_UNKNOWN fallback for known code", () => {
    // r1-napi-5 cross-pin. The codegen-emitted map covers every
    // catalog code; no key resolves to a generic BentenError.
    expect(Object.keys(CODE_TO_CTOR_GENERATED).length).toBeGreaterThan(0);

    for (const [code, ctor] of Object.entries(CODE_TO_CTOR_GENERATED)) {
      expect(typeof ctor).toBe("function");
      // Constructor name must NOT be the generic BentenError fallback;
      // each entry is a typed subclass per scripts/codegen-errors.ts.
      expect(ctor.name).not.toBe("BentenError");
      // The class itself is a subclass of BentenError.
      expect(ctor.prototype).toBeInstanceOf(BentenError);
      // Sanity: the static `code` property matches the map key.
      expect((ctor as unknown as { code: string }).code).toBe(code);
    }

    // Round-trip: every catalog code resolves to a typed subclass via
    // mapNativeError without E_UNKNOWN fallback.
    for (const code of Object.keys(CODE_TO_CTOR_GENERATED)) {
      const napiErr = new Error(
        JSON.stringify({ code, message: `synthesized ${code}` }),
      );
      const mapped = mapNativeError(napiErr);
      expect(mapped.code).toBe(code);
      // The synthetic fallback constructor name is the base
      // `BentenError`; a real typed subclass has a distinct ctor name.
      expect(mapped.constructor.name).not.toBe("BentenError");
    }
  });

  it("G19-B wave-7 — every EngineError variant surfaces structured fields", () => {
    // r1-napi-7 sizing pin (full structured-field coverage). Picks
    // representative variants spanning the EngineError surface; the
    // codegen-completeness pin above covers the long tail.
    const fixtures = [
      {
        code: "E_CAP_DENIED",
        message: "denied",
        fields: { actor: "x", action: "y", target: "z" },
      },
      {
        code: "E_MODULE_MANIFEST_CID_MISMATCH",
        message: "cid mismatch",
        fields: {
          expected: "bafyExpected",
          computed: "bafyComputed",
          summary: "checksum drift",
        },
      },
      {
        code: "E_IVM_VIEW_STALE",
        message: "view stale",
        fields: { viewId: "posts.byTag" },
      },
      {
        code: "E_INV_REGISTRATION",
        message: "registration error",
        fields: { invariantCode: "inv-2", summary: "duplicate node" },
      },
      {
        code: "E_DUPLICATE_HANDLER",
        message: "handler exists",
        fields: { handlerId: "post:create" },
      },
    ];
    for (const fx of fixtures) {
      const mapped = mapNativeError(
        new Error(JSON.stringify({ code: fx.code, message: fx.message, fields: fx.fields })),
      );
      expect(mapped.code).toBe(fx.code);
      expect(mapped.context).toEqual(fx.fields);
      expect(mapped.message).toBe(fx.message);
    }
  });

  it("G19-B wave-7 — mapNativeError preserves backwards-compat with legacy prefix carrier", () => {
    // Backwards-compat path: hand-rolled napi errors that throw a
    // plain `format!("E_*: ...")` message must still resolve to the
    // typed subclass. The JSON envelope is the new shape; the prefix
    // carrier is the legacy shape for non-engine-driven errors.
    const napiErr = new Error("E_CAP_DENIED: actor cannot write");
    const mapped = mapNativeError(napiErr);
    expect(mapped.code).toBe("E_CAP_DENIED");
    expect(mapped).toBeInstanceOf(ECapDenied);
    // No structured fields — legacy carrier doesn't have them.
    expect(mapped.context).toBeUndefined();
  });

  it("G19-B wave-7 — JSON envelope with unknown code synthesises BentenError but carries fields", () => {
    // Defensive: if the napi side emits a code that isn't yet in the
    // catalog (e.g. a future variant landed in Rust but TS regen
    // hadn't shipped), mapNativeError still surfaces fields — the
    // wrapper is a generic BentenError carrying the unknown-code
    // disposition.
    const napiErr = new Error(
      JSON.stringify({
        code: "E_FUTURE_VARIANT_NOT_IN_CATALOG_YET",
        message: "future error",
        fields: { detail: "from-the-future" },
      }),
    );
    const mapped = mapNativeError(napiErr);
    expect(mapped.context).toEqual({ detail: "from-the-future" });
    expect(mapped.message).toBe("future error");
    // Typed-subclass round-trip preserved across the envelope path.
    expect(mapped).toBeInstanceOf(BentenError);
  });

  it("G19-B wave-7 — Invariant variant surfaces invariantCode + summary", () => {
    // EInvRegistration is the classic Invariant carrier on the napi
    // side; the bag includes `invariantCode` + `summary` (per
    // EngineError::context_json's RegistrationError arm).
    const napiErr = new Error(
      JSON.stringify({
        code: "E_INV_REGISTRATION",
        message: "registration error: cycle detected",
        fields: {
          invariantCode: "inv-3",
          summary: "cycle: a -> b -> a",
        },
      }),
    );
    const mapped = mapNativeError(napiErr);
    expect(mapped).toBeInstanceOf(EInvRegistration);
    expect(mapped.context).toEqual({
      invariantCode: "inv-3",
      summary: "cycle: a -> b -> a",
    });
  });
});

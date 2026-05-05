// R3-E RED-PHASE pins for G19-B mapNativeError + BentenError.context
// TS-side surface (wave-7 parallel; §7.2).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-B):
//
//   - errors.test.ts (TS Vitest)
//
// What G19-B establishes (§7.2):
//
//   - packages/engine/src/errors.ts::mapNativeError consumes the JSON-shape
//     `{code, fields}` carrier emitted by bindings/napi/src/lib.rs::engine_err
//     and returns a typed BentenError subclass with a `.context` accessor
//     surfacing the structured fields.
//
//   - packages/engine/src/errors.generated.ts emits CODE_TO_CTOR_GENERATED
//     keyed by ErrorCode → typed-subclass constructor.
//
// RED-PHASE discipline:
//
//   These tests assert the post-G19-B shape. The pre-G19-B state has
//   message-prefix `E_*` carriers; mapNativeError parses message-string
//   prefixes. R5 implementer drops .skip after wiring the carrier.

import { describe, it, expect } from "vitest";

describe("G19-B BentenError.context structured-field surfacing (§7.2)", () => {
  it.skip("RED-PHASE: G19-B wave-7 — mapNativeError carries context fields", async () => {
    // §7.2 pin. G19-B implementer wires this:
    //
    //   const { mapNativeError } = await import("@benten/engine/dist/errors.js");
    //
    //   // Synthesize a napi-shaped Error (JSON-payload message body):
    //   const napiErr = new Error(
    //     JSON.stringify({
    //       code: "E_CAP_DENIED",
    //       fields: { actor: "alice", action: "kv:write", target: "post:1" },
    //     }),
    //   );
    //
    //   const mapped = mapNativeError(napiErr);
    //   expect(mapped.code).toBe("E_CAP_DENIED");
    //   expect(mapped.context).toEqual({
    //     actor: "alice",
    //     action: "kv:write",
    //     target: "post:1",
    //   });
    //
    //   // Typed-subclass surfacing — must NOT be the generic BentenError
    //   // class for a known catalog code:
    //   expect(mapped.constructor.name).not.toBe("BentenError");
    //
    // OBSERVABLE consequence: structured fields round-trip through the
    // napi boundary verbatim. Defends against the legacy message-prefix
    // shape where fields were dropped or coerced into the message string.
    throw new Error(
      "RED-PHASE: G19-B wave-7 wires mapNativeError + BentenError.context + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-B wave-7 — CODE_TO_CTOR_GENERATED has no E_UNKNOWN fallback for known code", async () => {
    // r1-napi-5 cross-pin. G19-B implementer wires this:
    //
    //   const { CODE_TO_CTOR_GENERATED } = await import(
    //     "@benten/engine/dist/errors.generated.js"
    //   );
    //
    //   // Synthesize napi errors for every known code; each must resolve
    //   // to a typed subclass via CODE_TO_CTOR_GENERATED.
    //   for (const code of Object.keys(CODE_TO_CTOR_GENERATED)) {
    //     const ctor = CODE_TO_CTOR_GENERATED[code];
    //     expect(typeof ctor).toBe("function");
    //     // Constructor name must NOT be the generic BentenError fallback:
    //     expect(ctor.name).not.toBe("BentenError");
    //   }
    //
    //   // Also: no orphan codes — every entry resolves.
    //   const napiErr = new Error(JSON.stringify({
    //     code: "E_INV_4_OVERFLOW",
    //     fields: { depth: 17 },
    //   }));
    //   const { mapNativeError } = await import("@benten/engine/dist/errors.js");
    //   const mapped = mapNativeError(napiErr);
    //   expect(mapped.code).toBe("E_INV_4_OVERFLOW");
    //   expect(mapped.constructor.name).not.toBe("BentenError");
    //
    // OBSERVABLE consequence: CODE_TO_CTOR_GENERATED has no E_UNKNOWN
    // fallback for any known catalog code; every code resolves to a
    // typed subclass.
    throw new Error(
      "RED-PHASE: G19-B wave-7 wires CODE_TO_CTOR_GENERATED no-fallback + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-B wave-7 — every EngineError variant surfaces structured fields", async () => {
    // r1-napi-7 sizing pin (full structured-field coverage across all
    // ~20 EngineError variants + EvalError tail).
    //
    // G19-B implementer wires this with a per-variant fixture table that
    // round-trips the napi carrier and asserts mapNativeError preserves
    // every named field. Pin shape:
    //
    //   const fixtures = [
    //     { code: "E_CAP_DENIED", fields: { actor: "x", action: "y", target: "z" } },
    //     { code: "E_INV_4_OVERFLOW", fields: { depth: 17, max: 16 } },
    //     // ... long tail per ERROR-CATALOG.md
    //   ];
    //   for (const fx of fixtures) {
    //     const mapped = mapNativeError(new Error(JSON.stringify(fx)));
    //     expect(mapped.code).toBe(fx.code);
    //     expect(mapped.context).toEqual(fx.fields);
    //   }
    //
    // OBSERVABLE consequence: every catalog entry's structured fields
    // surface through mapNativeError unchanged. End-to-end pin per pim-2
    // §3.6b — would FAIL if any variant's fields were silently dropped.
    throw new Error(
      "RED-PHASE: G19-B wave-7 wires every-EngineError-variant structured-field round-trip + drops .skip + un-comments assertions",
    );
  });
});

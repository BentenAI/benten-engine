// R6 Round-3 r6-r3-napi-2 regression pin (17th producer/consumer drift
// instance closure).
//
// The wrapper helper `resolveReplaceOutcome` (exported from
// `@benten/engine-devserver`) MUST honour the discriminated union
// return type:
//   - When the native binding exposes `replaceHandlerFromDslWithOutcome`
//     (Instance-10 surface), return the structured 6-key outcome.
//   - When the native binding lacks the structured surface (pre-Instance-10
//     legacy fallback), return `{ legacyOnly: true, handlerId }` so
//     consumer pivot logic `if (result.legacyOnly === true)` routes
//     correctly.
//
// Pre-fix: the legacy-fallback branch synthesized
// `{ chainDepth: 1, versionTag: "v1", replaced: false }` defaults +
// omitted the `legacyOnly` discriminator the docstring promised. Consumers
// pivoting on `result.legacyOnly === true` then misrouted to the
// engine-routed branch with fake audit-trail values (chainDepth and
// versionTag invented; `replaced: false` deceptively suggested a no-op
// when in fact the structured outcome was unavailable).

import { describe, expect, it } from "vitest";
import { resolveReplaceOutcome } from "@benten/engine-devserver";

describe("resolveReplaceOutcome — R6 Round-3 r6-r3-napi-2", () => {
  it("legacy_fallback_returns_discriminated_legacyOnly_shape", () => {
    // Synth a "pre-Instance-10" native binding stub: only the bare-CID
    // `replaceHandlerFromDsl` exists, no `replaceHandlerFromDslWithOutcome`.
    const calls: Array<{ handlerId: string; op: string; source: string }> = [];
    const stub = {
      replaceHandlerFromDsl(handlerId: string, op: string, source: string) {
        calls.push({ handlerId, op, source });
        return "bafyr4igreplacedlegacyfallbackcidplaceholder0aaaaaaaaaaaaaaa";
      },
    };

    const result = resolveReplaceOutcome(stub, "h1", "run", "handler {}");

    // The replace itself still happens (side-effect via the legacy
    // surface).
    expect(calls).toHaveLength(1);
    expect(calls[0]).toEqual({
      handlerId: "h1",
      op: "run",
      source: "handler {}",
    });

    // The discriminator is set + the rest of the shape is the legacy-only
    // surface — NOT the synthesized engine-routed shape with fake defaults.
    expect(result).toEqual({ legacyOnly: true, handlerId: "h1" });

    // Consumer pivot logic must route to the legacy branch.
    if ("legacyOnly" in result) {
      expect(result.legacyOnly).toBe(true);
      expect(result.handlerId).toBe("h1");
      // No misleading audit-trail keys present on the legacy shape.
      expect((result as Record<string, unknown>).chainDepth).toBeUndefined();
      expect((result as Record<string, unknown>).versionTag).toBeUndefined();
      expect((result as Record<string, unknown>).replaced).toBeUndefined();
    } else {
      throw new Error(
        "expected legacyOnly: true branch; pre-fix this assertion would have failed because the synthesized shape was returned without the discriminator",
      );
    }
  });

  it("instance10_surface_returns_structured_outcome", () => {
    // Instance-10 binding present: returns the 6-key shape; the wrapper
    // passes it through.
    const stub = {
      replaceHandlerFromDsl() {
        throw new Error("legacy surface should not be called when Instance-10 surface present");
      },
      replaceHandlerFromDslWithOutcome(
        handlerId: string,
        op: string,
        _source: string,
      ) {
        return {
          handlerId,
          cid: "bafyr4ignewcid000000000000000000000000000000000000000000aaaaaa",
          previousCid: "bafyr4igpreviousc000000000000000000000000000000000000000aaaaaa",
          chainDepth: 7,
          versionTag: "v7",
          replaced: true,
          op,
        };
      },
    };

    const result = resolveReplaceOutcome(stub, "h2", "run", "handler {}");

    if ("legacyOnly" in result) {
      throw new Error("expected structured outcome, got legacyOnly branch");
    }
    expect(result.handlerId).toBe("h2");
    expect(result.chainDepth).toBe(7);
    expect(result.versionTag).toBe("v7");
    expect(result.replaced).toBe(true);
  });

  it("instance10_surface_returning_legacyOnly_shape_passes_through", () => {
    // Instance-10 binding returns the discriminated legacyOnly shape
    // directly (the napi adapter at devserver.rs:182-187 does this when
    // engine routing is disabled). The wrapper passes it through
    // unchanged.
    const stub = {
      replaceHandlerFromDsl() {
        throw new Error("should not be called");
      },
      replaceHandlerFromDslWithOutcome(handlerId: string) {
        return { legacyOnly: true, handlerId };
      },
    };

    const result = resolveReplaceOutcome(stub, "h3", "run", "handler {}");
    expect(result).toEqual({ legacyOnly: true, handlerId: "h3" });
  });
});

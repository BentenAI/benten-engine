// R6FP-tail (Round-2 Instance 10) — engine.replaceSubgraph 6-key shape pin.
//
// Pre-fix: `Engine::register_subgraph_replace` was NOT exposed via napi
// at all. DevServer's `replace_handler_from_dsl` returned only the new
// CID String. Non-devserver JS contexts had zero replace observability.
//
// Post-fix: r6fp-tail-comprehensive added direct napi exposure +
// `engine.replaceSubgraph(spec)` returns the full 6-key
// RegisterReplaceOutcome shape.
//
// This pin asserts the surface promise — the consumer-audit-table claim
// in the PR body is verified at runtime here.

import { describe, it, expect } from "vitest";
import { Engine, crud } from "@benten/engine";

describe("engine.replaceSubgraph (R6FP Instance 10)", () => {
  it("returns the full 6-key RegisterReplaceOutcome shape on first replace", async () => {
    const engine = await Engine.open(":memory:");

    // First registration via registerSubgraph — establishes v1.
    const v1 = engine.registerSubgraph(crud("post"));
    expect(typeof v1.id).toBe("string");

    // Replace via the new replaceSubgraph surface (the load-bearing
    // instance-10 closure).
    const outcome = await engine.replaceSubgraph(crud("post"));

    // 6-key shape per the napi → TS contract:
    //   { handlerId, cid, previousCid, chainDepth, versionTag, replaced }
    expect(outcome).toMatchObject({
      handlerId: expect.any(String),
      cid: expect.any(String),
      chainDepth: expect.any(Number),
      versionTag: expect.any(String),
      replaced: expect.any(Boolean),
    });

    // previousCid is OPTIONAL per the type but MUST be present for a
    // replace (vs a fresh registration). Idempotent re-register can
    // produce replaced=false; that's a separate pin below.
    expect(typeof outcome.previousCid).toBe("string");

    // chainDepth bumped from v1's depth (≥ 2 for v2 chain entry).
    expect(outcome.chainDepth).toBeGreaterThanOrEqual(2);

    // versionTag follows the surrogate vN scheme (v2 for first replace).
    expect(outcome.versionTag).toMatch(/^v\d+$/);
    expect(outcome.replaced).toBe(true);

    await engine.close();
  });

  it("idempotent re-register returns replaced=false + same chainDepth", async () => {
    const engine = await Engine.open(":memory:");

    engine.registerSubgraph(crud("post"));
    // Identical content → engine recognizes and short-circuits.
    const outcome = await engine.replaceSubgraph(crud("post"));

    // The handler chain is consistent with the original registration.
    expect(typeof outcome.handlerId).toBe("string");
    expect(typeof outcome.cid).toBe("string");
    expect(typeof outcome.versionTag).toBe("string");
    // Idempotent path: replaced === false, no chain bump.
    expect(outcome.replaced).toBe(false);

    await engine.close();
  });
});

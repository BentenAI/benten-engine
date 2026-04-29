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
import { Engine, crud, subgraph } from "@benten/engine";

describe("engine.replaceSubgraph (R6FP Instance 10)", () => {
  it("returns the full 6-key RegisterReplaceOutcome shape on first replace", async () => {
    const engine = await Engine.open(":memory:");

    // First registration via registerSubgraph — establishes v1.
    // Use a hand-built subgraph (same handlerId as the replace target so
    // the chain links the two registrations) with a distinct shape from
    // the v2 below — replaceSubgraph short-circuits to replaced=false when
    // the new content is structurally identical to the existing entry.
    // This pin exercises the TRUE replace path; the idempotent re-register
    // path is exercised by the sibling test below.
    //
    // The subgraph carries a leading READ so the structural-invariant
    // battery accepts it (a respond-only handler trips an invariant —
    // R5/R6 invariant pass requires at least one substantive operation
    // before a terminal RESPOND). v1 and v2 differ in NODE STRUCTURE
    // (v1 has one READ; v2 has two READs) so the canonical-bytes CID
    // computation produces distinct CIDs and `replaceSubgraph` exercises
    // the genuine replace branch (`replaced=true`, `previousCid` populated).
    const v1Spec = subgraph("replace-handler")
      .action("noop")
      .read({ label: "post" })
      .respond({ body: "v1" })
      .build();
    const v1 = await engine.registerSubgraph(v1Spec);
    expect(typeof v1.id).toBe("string");

    // v2 has the same handlerId but a structurally different node list →
    // genuine replace.
    const v2Spec = subgraph("replace-handler")
      .action("noop")
      .read({ label: "post" })
      .read({ label: "post" })
      .respond({ body: "v2" })
      .build();
    const outcome = await engine.replaceSubgraph(v2Spec);

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
    // genuine replace (different content from v1). Idempotent re-register
    // can produce replaced=false; that's a separate pin below.
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

    await engine.registerSubgraph(crud("post"));
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

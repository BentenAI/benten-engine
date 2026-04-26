// R3-followup (R4-FP B-1) red-phase — WAIT TTL TS DSL surface (G12-E).
//
// Tests are RED at landing time; G12-E (TS-side) makes them green.
//
// Surface contract (per dx-r1-2b WAIT + plan §3 G12-E + D12-RESOLVED):
//   - WAIT primitive args carry optional `ttl_hours` (number, 1..720;
//     default 24h when omitted).
//   - Resume after deadline throws a typed error renderable to the
//     `E_WAIT_TTL_EXPIRED` code; bad ttl_hours at registration throws
//     `E_WAIT_TTL_INVALID`.
//
// Pin sources: r2-test-landscape.md §1.10 + §8.1; r4-qa-expert.json qa-r4-06.

import { describe, it, expect } from "vitest";
import { Engine, subgraph } from "@benten/engine";

describe("WAIT ttl_hours TS DSL", () => {
  it.skip("Phase 2b G12-E pending — wait({ ttl_hours: 24 }) compiles + suspends cleanly", async () => {
    const engine = await Engine.open(":memory:");

    const sg = subgraph("approval-flow")
      .action("await-decision")
      .wait({ ttl_hours: 24 })
      .respond({ body: "$resume_payload" })
      .build();

    const handler = await engine.registerSubgraph(sg);
    const envelope = await engine.callToSuspend(handler.id, "await-decision", {});
    expect(envelope).toBeDefined();
    expect(envelope.ttlHours).toBe(24);

    await engine.close();
  });

  it.skip("Phase 2b G12-E pending — omitted ttl_hours defaults to 24 (D12)", async () => {
    const engine = await Engine.open(":memory:");

    const sg = subgraph("approval-default")
      .action("await")
      .wait({}) // no ttl_hours
      .respond({ body: "$resume_payload" })
      .build();

    const handler = await engine.registerSubgraph(sg);
    const envelope = await engine.callToSuspend(handler.id, "await", {});
    expect(envelope.ttlHours).toBe(24);

    await engine.close();
  });

  it.skip("Phase 2b G12-E pending — ttl_hours: 0 rejected at registration with E_WAIT_TTL_INVALID", async () => {
    const engine = await Engine.open(":memory:");

    const badSg = subgraph("approval-bad")
      .action("await")
      .wait({ ttl_hours: 0 })
      .respond({ body: "$resume_payload" })
      .build();

    await expect(engine.registerSubgraph(badSg)).rejects.toMatchObject({
      message: expect.stringContaining("E_WAIT_TTL_INVALID"),
    });

    await engine.close();
  });

  it.skip("Phase 2b G12-E pending — ttl_hours: 721 rejected; 720 accepted (boundary)", async () => {
    const engine = await Engine.open(":memory:");

    const okSg = subgraph("approval-max")
      .action("await")
      .wait({ ttl_hours: 720 })
      .respond({ body: "$resume_payload" })
      .build();
    await engine.registerSubgraph(okSg);

    const badSg = subgraph("approval-over-max")
      .action("await")
      .wait({ ttl_hours: 721 })
      .respond({ body: "$resume_payload" })
      .build();
    await expect(engine.registerSubgraph(badSg)).rejects.toMatchObject({
      message: expect.stringContaining("E_WAIT_TTL_INVALID"),
    });

    await engine.close();
  });

  it.skip("Phase 2b G12-E pending — resume after expiry throws E_WAIT_TTL_EXPIRED", async () => {
    const engine = await Engine.open(":memory:");

    const sg = subgraph("approval-expiry")
      .action("await")
      .wait({ ttl_hours: 1 })
      .respond({ body: "$resume_payload" })
      .build();

    const handler = await engine.registerSubgraph(sg);
    const envelope = await engine.callToSuspend(handler.id, "await", {});

    // Test backdoor — TS bridges to testing_advance_wait_clock under the hood.
    await engine.testingAdvanceWaitClock(2 * 3600 * 1000); // 2 hours in ms

    await expect(
      engine.resumeWithMeta(envelope, "approved"),
    ).rejects.toMatchObject({
      message: expect.stringContaining("E_WAIT_TTL_EXPIRED"),
    });

    await engine.close();
  });
});

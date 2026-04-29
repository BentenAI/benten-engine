// R6-FP r6-napi-2 closure — WAIT TTL TS DSL surface (G12-E carry).
//
// HISTORY: these 5 tests were originally rationaled as "Phase 2b G12-E
// pending" — but G12-E shipped (PR #43 + #57) without lifting the
// `wait({ ttl_hours })` DSL surface OR the `engine.callToSuspend` /
// `engine.testingAdvanceWaitClock` / `engine.resumeWithMeta` test
// surfaces. The actual TS surface that landed is `engine.callWithSuspension`
// + `engine.resumeFromBytesUnauthenticated` (no callToSuspend);
// `WaitArgs` accepts only `signal | duration` (no `ttl_hours` field);
// no `engine.testingAdvanceWaitClock` exists on the TS Engine class.
//
// Per R6 council finding r6-napi-2: the original "Phase 2b G12-E
// pending" rationale violated HARD RULE because G12-E shipped + the
// destination expired. The CURRENT real blockers per audited TS
// surface:
//
//   - `wait({ ttl_hours })` DSL surface: Phase 3 — TTL is engine-side
//     wired (D12-RESOLVED + WaitMetadata.timeout_ms in Rust), but the
//     TS DSL `WaitArgs` union does NOT yet accept `ttl_hours`. Adding
//     it requires extending the `wait` builder.
//   - `engine.callToSuspend(handlerId, op, input) -> envelope` shape:
//     Phase 3 — actual is `callWithSuspension` returning a typed
//     `{ kind: 'suspended' | 'complete' }` discriminated union, NOT a
//     bare envelope.
//   - `envelope.ttlHours` introspection accessor: Phase 3 — the
//     suspended-branch payload doesn't expose ttl_hours yet.
//   - `engine.testingAdvanceWaitClock(deltaMs)`: Phase 3 — engine has
//     `testing_set_iteration_budget` pattern but no wallclock-advance
//     hook on the TS surface.
//   - `engine.resumeWithMeta(envelope, signal)`: Phase 3 — actual TS
//     name is `resumeFromBytesUnauthenticated(bytes, signal)` taking
//     raw bytes rather than the typed envelope object.
//
// Named destination: `docs/future/phase-3-backlog.md` (created /
// enriched by R6-FP Group 4) — entry titled "WAIT TTL TS DSL +
// suspend/resume DX surface widening (post-G12-E)" tracks the lift.
// The .skip cases stay parked on this rationale per HARD RULE
// disposition (b) BELONGS-ELSEWHERE-SPECIFICALLY.
//
// Pin sources: r6-napi-2 finding (R6 Round 1 phase-close council);
// r2-test-landscape.md §1.10 + §8.1; r4-qa-expert.json qa-r4-06.

import { describe, it, expect } from "vitest";
import { Engine, subgraph } from "@benten/engine";

describe("WAIT ttl_hours TS DSL", () => {
  it.skip("Phase 3 (post-G12-E TS DSL ttl_hours) — wait({ ttl_hours: 24 }) compiles + suspends cleanly", async () => {
    // BLOCKER: `WaitArgs` does not accept `ttl_hours`; `engine.callToSuspend`
    // does not exist (actual: `engine.callWithSuspension`). See file
    // header for the named-destination Phase 3 entry.
    const engine = await Engine.open(":memory:");

    const sg = subgraph("approval-flow")
      .action("await-decision")
      // @ts-expect-error — ttl_hours not yet in WaitArgs
      .wait({ ttl_hours: 24 })
      .respond({ body: "$resume_payload" })
      .build();

    const handler = await engine.registerSubgraph(sg);
    // @ts-expect-error — engine.callToSuspend doesn't exist (Phase 3 carry)
    const envelope = await engine.callToSuspend(handler.id, "await-decision", {});
    expect(envelope).toBeDefined();
    expect(envelope.ttlHours).toBe(24);

    await engine.close();
  });

  it.skip("Phase 3 (post-G12-E TS DSL ttl_hours) — omitted ttl_hours defaults to 24 (D12)", async () => {
    // BLOCKER: same as above — WaitArgs doesn't carry ttl_hours; the
    // engine-side D12 default applies but TS introspection accessor
    // (`envelope.ttlHours`) doesn't surface it.
    const engine = await Engine.open(":memory:");

    const sg = subgraph("approval-default")
      .action("await")
      .wait({ duration: 100 })
      .respond({ body: "$resume_payload" })
      .build();

    const handler = await engine.registerSubgraph(sg);
    // @ts-expect-error — engine.callToSuspend doesn't exist (Phase 3 carry)
    const envelope = await engine.callToSuspend(handler.id, "await", {});
    expect(envelope.ttlHours).toBe(24);

    await engine.close();
  });

  it.skip("Phase 3 (post-G12-E TS DSL ttl_hours) — ttl_hours: 0 rejected at registration with E_WAIT_TTL_INVALID", async () => {
    // BLOCKER: WaitArgs doesn't carry ttl_hours; the registration-time
    // E_WAIT_TTL_INVALID guard is engine-side wired but TS DSL surface
    // is the gap.
    const engine = await Engine.open(":memory:");

    const badSg = subgraph("approval-bad")
      .action("await")
      // @ts-expect-error — ttl_hours not yet in WaitArgs
      .wait({ ttl_hours: 0 })
      .respond({ body: "$resume_payload" })
      .build();

    await expect(engine.registerSubgraph(badSg)).rejects.toMatchObject({
      message: expect.stringContaining("E_WAIT_TTL_INVALID"),
    });

    await engine.close();
  });

  it.skip("Phase 3 (post-G12-E TS DSL ttl_hours) — ttl_hours: 721 rejected; 720 accepted (boundary)", async () => {
    // BLOCKER: same DSL surface gap as above.
    const engine = await Engine.open(":memory:");

    const okSg = subgraph("approval-max")
      .action("await")
      // @ts-expect-error — ttl_hours not yet in WaitArgs
      .wait({ ttl_hours: 720 })
      .respond({ body: "$resume_payload" })
      .build();
    await engine.registerSubgraph(okSg);

    const badSg = subgraph("approval-over-max")
      .action("await")
      // @ts-expect-error — ttl_hours not yet in WaitArgs
      .wait({ ttl_hours: 721 })
      .respond({ body: "$resume_payload" })
      .build();
    await expect(engine.registerSubgraph(badSg)).rejects.toMatchObject({
      message: expect.stringContaining("E_WAIT_TTL_INVALID"),
    });

    await engine.close();
  });

  it.skip("Phase 3 (post-G12-E TS DSL ttl_hours) — resume after expiry throws E_WAIT_TTL_EXPIRED", async () => {
    // BLOCKER: requires `engine.testingAdvanceWaitClock` (does not
    // exist on TS surface) + `engine.resumeWithMeta` (actual is
    // `engine.resumeFromBytesUnauthenticated` taking raw bytes).
    const engine = await Engine.open(":memory:");

    const sg = subgraph("approval-expiry")
      .action("await")
      // @ts-expect-error — ttl_hours not yet in WaitArgs
      .wait({ ttl_hours: 1 })
      .respond({ body: "$resume_payload" })
      .build();

    const handler = await engine.registerSubgraph(sg);
    // @ts-expect-error — engine.callToSuspend doesn't exist (Phase 3 carry)
    const envelope = await engine.callToSuspend(handler.id, "await", {});

    // @ts-expect-error — engine.testingAdvanceWaitClock doesn't exist on TS Engine
    await engine.testingAdvanceWaitClock(2 * 3600 * 1000); // 2 hours in ms

    await expect(
      // @ts-expect-error — engine.resumeWithMeta doesn't exist (actual: resumeFromBytesUnauthenticated)
      engine.resumeWithMeta(envelope, "approved"),
    ).rejects.toMatchObject({
      message: expect.stringContaining("E_WAIT_TTL_EXPIRED"),
    });

    await engine.close();
  });
});

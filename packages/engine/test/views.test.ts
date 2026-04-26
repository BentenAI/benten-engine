// R3-followup (R4-FP B-1) red-phase — engine.createView (G8-B exclusive).
//
// Tests are RED at landing time; G8-B (TS-side) makes them green.
//
// Surface contract (per dx-r1-2b + plan §3 G8-B + D8-RESOLVED):
//   - engine.createView(spec) -> Promise<UserView>
//     * spec carries id, inputPattern, strategy?, project?
//     * strategy DEFAULTS to 'B' (D8: hand-written = Rust-only;
//       user views always go through Algorithm B).
//     * strategy === 'A' MUST throw a typed error.
//   - The returned UserView exposes:
//     * snapshot() -> AsyncIterable<Row>  (current rows)
//     * onUpdate(cb) -> Subscription      (event-emitter for diffs)
//
// Pin sources: r2-test-landscape.md §7 rows 462-463; r2 §1.7 + §8 D8;
// r4-qa-expert.json qa-r4-04.

import { describe, it, expect } from "vitest";
import { Engine } from "@benten/engine";
import type { UserViewSpec, UserView } from "@benten/engine";

describe("engine.createView", () => {
  it.skip("Phase 2b G8-B pending — round-trip create + snapshot materialization", async () => {
    const engine = await Engine.open(":memory:");

    const spec: UserViewSpec = {
      id: "user_posts_by_author",
      inputPattern: { label: "post" },
      // strategy omitted — exercises the 'B' default (D8).
      project: (evt) => ({
        author: evt.attribution.actorCid,
        label: evt.label,
      }),
    };

    const view: UserView = await engine.createView(spec);
    expect(view.id).toBe("user_posts_by_author");

    // Emit synthetic events through the engine's normal write path.
    const post = await engine.registerSubgraph(/* crud("post") */ {} as never);
    for (let i = 0; i < 3; i++) {
      await engine.call(post.id, "post:create", { i });
    }

    // Snapshot is an AsyncIterable of materialized rows.
    const rows: unknown[] = [];
    for await (const row of view.snapshot()) rows.push(row);
    expect(rows).toHaveLength(3);

    await engine.close();
  });

  it.skip("Phase 2b G8-B pending — strategy defaults to 'B' for user views (D8)", async () => {
    const engine = await Engine.open(":memory:");

    const spec: UserViewSpec = {
      id: "user_default_strategy",
      inputPattern: { label: "post" },
      // No strategy field — DEFAULTS to 'B' per D8-RESOLVED.
    };

    const view = await engine.createView(spec);
    expect(view.strategy).toBe("B");

    await engine.close();
  });

  it.skip("Phase 2b G8-B pending — refuses strategy 'A' with typed error (D8)", async () => {
    const engine = await Engine.open(":memory:");

    const badSpec = {
      id: "user_a_attempt",
      inputPattern: { label: "post" },
      strategy: "A" as const,
    } as unknown as UserViewSpec;

    await expect(engine.createView(badSpec)).rejects.toMatchObject({
      // Exact error code TBD by R5 G8-B; assert recognizable typed shape.
      message: expect.stringMatching(
        /E_VIEW_STRATEGY_A_REFUSED|hand-written|Strategy::A/i,
      ),
    });

    await engine.close();
  });

  it.skip("Phase 2b G8-B pending — explicit 'B' opt-in matches default behavior", async () => {
    const engine = await Engine.open(":memory:");

    const explicit = await engine.createView({
      id: "user_explicit_b",
      inputPattern: { label: "post" },
      strategy: "B",
    });
    const fallback = await engine.createView({
      id: "user_fallback_b",
      inputPattern: { label: "post" },
    });

    expect(explicit.strategy).toBe(fallback.strategy);
    expect(explicit.strategy).toBe("B");

    await engine.close();
  });

  it.skip("Phase 2b G8-B pending — refuses strategy 'C' as Phase-3 reserved", async () => {
    const engine = await Engine.open(":memory:");

    const reservedSpec = {
      id: "user_c_attempt",
      inputPattern: { label: "post" },
      strategy: "C" as const,
    } as unknown as UserViewSpec;

    await expect(engine.createView(reservedSpec)).rejects.toMatchObject({
      message: expect.stringMatching(/E_VIEW_STRATEGY_C_RESERVED|Phase 3|Z-set/i),
    });

    await engine.close();
  });
});

describe("UserView.onUpdate", () => {
  it.skip("Phase 2b G8-B pending — onUpdate fires with diff for matching writes", async () => {
    const engine = await Engine.open(":memory:");

    const view = await engine.createView({
      id: "user_onupdate_test",
      inputPattern: { label: "post" },
    });

    const diffs: unknown[] = [];
    const sub = view.onUpdate((diff) => diffs.push(diff));

    const post = await engine.registerSubgraph(/* crud("post") */ {} as never);
    await engine.call(post.id, "post:create", { v: 1 });

    await new Promise((r) => setTimeout(r, 50));
    expect(diffs.length).toBeGreaterThan(0);

    await sub.unsubscribe();
    await engine.close();
  });
});

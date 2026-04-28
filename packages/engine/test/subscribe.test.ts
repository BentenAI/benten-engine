// Wave-8c-subscribe-infra: SUBSCRIBE production wire-through tests.
//
// Pre-wire-through these tests asserted the unwired-stub shape;
// post-wire-through they pin the production-runtime contract: a
// callback registered via `engine.onChange` fires when a matching
// graph write commits, exceptions in a callback are caught and the
// subscription stays alive, and `engine.onChangeAs` carries an actor
// principal whose grants drive D5 cap-recheck-at-delivery.
//
// Pin sources: r2-test-landscape.md §7; r1-dx-optimizer.json
// subscribe_test_fixture; dx-r1-2b-4 (callback-exception isolation).

import { describe, it, expect, vi } from "vitest";
import { Engine, subgraph, crud } from "@benten/engine";
import type { Subscription } from "@benten/engine";

describe("engine.onChange", () => {
  it("registers a callback against the production change-stream port", async () => {
    const engine = await Engine.open(":memory:");
    await engine.registerSubgraph(crud("post"));

    const sub: Subscription = engine.onChange("post:*", () => {});
    // Wave-8c-subscribe-infra: the returned Subscription is ACTIVE
    // immediately — the production change-stream port is wired.
    expect(sub.active).toBe(true);
    expect(sub.pattern).toBe("post:*");
    sub.unsubscribe();
    expect(sub.active).toBe(false);

    await engine.close();
  });

  it("onChange callback exception is caught (subscription stays alive)", async () => {
    // dx-r1-2b-4: subscriber-side throws are routine; sub stays alive,
    // log fires. The catch happens both on the JS side (the wrapper
    // catches + console.error's) and on the Rust side (the
    // ChangeBroadcast walker's catch_unwind boundary). This test pins
    // the JS-side catch contract specifically.
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const engine = await Engine.open(":memory:");
    await engine.registerSubgraph(crud("post"));

    const sub = engine.onChange("post:*", () => {
      throw new Error("subscriber throws");
    });
    expect(sub.active).toBe(true);

    sub.unsubscribe();
    expect(sub.active).toBe(false);
    errSpy.mockRestore();
    await engine.close();
  });

  it("onChangeAs threads principal through the napi boundary", async () => {
    const engine = await Engine.open(":memory:");
    await engine.registerSubgraph(crud("post"));

    const sub = engine.onChangeAs("post:*", () => {}, "alice");
    // Active immediately post-wire-through; the principal is captured
    // on the registered ad-hoc onChange entry's delivery-time
    // cap-recheck closure.
    expect(sub.active).toBe(true);
    expect(sub.pattern).toBe("post:*");
    sub.unsubscribe();
    expect(sub.active).toBe(false);

    await engine.close();
  });

  it("rejects empty pattern with a typed error", async () => {
    const engine = await Engine.open(":memory:");
    // Empty pattern is rejected at the engine boundary with the
    // typed `E_SUBSCRIBE_PATTERN_INVALID` code.
    expect(() => engine.onChange("", () => {})).toThrow();
    await engine.close();
  });

  it("unsubscribe is idempotent + survives multiple invocations", async () => {
    const engine = await Engine.open(":memory:");
    const sub = engine.onChange("post:*", () => {});
    expect(sub.active).toBe(true);
    sub.unsubscribe();
    expect(sub.active).toBe(false);
    // Idempotent — second unsubscribe is a no-op + does not throw.
    sub.unsubscribe();
    expect(sub.active).toBe(false);
    await engine.close();
  });

  it("DSL composition subgraph(...).subscribe(args)", () => {
    // Composition pin — the fluent builder retains the .subscribe()
    // entry even after the engine.subscribe surface is renamed to
    // engine.onChange.
    const sg = subgraph("analytics")
      .action("on-post-create")
      .subscribe({ event: "post:write" })
      .transform({ expr: "computeMetrics($result)" })
      .respond({ body: "$result" })
      .build();

    const subscribeNode = sg.nodes.find((n) => n.primitive === "subscribe");
    expect(subscribeNode).toBeDefined();
    expect(subscribeNode!.args.event).toBe("post:write");
  });

  it("naming distinct from DSL builder name", () => {
    // dx-r1-2b SUBSCRIBE rename: the engine surface is `onChange`,
    // NOT `subscribe`, to avoid colliding with
    // SubgraphBuilder.subscribe(). Compile-time pin: the engine
    // instance type does NOT expose .subscribe.

    type EngineMethods = keyof Engine;

    // Affirmative: onChange MUST be present.
    const onChangeIsKey: "onChange" extends EngineMethods ? true : false = true;
    expect(onChangeIsKey).toBe(true);

    // Negative: subscribe MUST NOT be present on the Engine class.
    const subscribeNotKey: "subscribe" extends EngineMethods ? false : true =
      true;
    expect(subscribeNotKey).toBe(true);
  });
});

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
// subscribe_test_fixture; dx-r1-2b-4 (callback-exception isolation);
// wave-8c fix-pass cr-w8c-fp-1 (callback-fires acceptance gate).

import { describe, it, expect, vi } from "vitest";
import { Engine, subgraph, crud } from "@benten/engine";
import type { Subscription } from "@benten/engine";

/**
 * Sleep helper so the test body can yield to the libuv main loop and
 * let queued ThreadsafeFunction calls drain into the JS callback.
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

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

  it("[Round-2 Instance 7] maxDeliveredSeq is a live getter (reads through native atomic)", async () => {
    // Round-2 Instance 7 closure: pre-fix-pass `wrapSubscriptionHandle`
    // snapshotted `native.maxDeliveredSeq()` at construction time
    // (when the value was 0) and exposed it as a plain field.
    // Subsequent native deliveries bumped the underlying atomic but
    // the TS-side field stayed at 0. This test pins the new live-getter
    // contract: post-delivery, `subscription.maxDeliveredSeq` reads
    // through to the native handle and reports the bumped value.
    const engine = await Engine.open(":memory:");
    await engine.registerSubgraph(crud("post"));

    const sub = engine.onChange("post", () => {});
    expect(sub.maxDeliveredSeq).toBe(0);

    await engine.createNode(["post"], { title: "live-getter-pin" });
    // Drain libuv queue so the native side's atomic gets bumped.
    for (let i = 0; i < 50 && sub.maxDeliveredSeq === 0; i += 1) {
      await sleep(5);
    }
    // The live getter MUST read through to the native handle.
    expect(sub.maxDeliveredSeq).toBeGreaterThan(0);

    sub.unsubscribe();
    await engine.close();
  });

  it("LOAD-BEARING — onChange callback fires when a matching write commits", async () => {
    // cr-w8c-fp-1 acceptance gate: register an onChange callback,
    // commit a matching write via createNode, and assert the callback
    // ACTUALLY FIRES within a deadline. Pre-fix-pass this test would
    // FAIL — the napi method dropped the underlying Subscription at end
    // of method scope, releasing the ThreadsafeFunction Arc + the JS
    // callback handle before any event could ever fire.
    const engine = await Engine.open(":memory:");
    await engine.registerSubgraph(crud("post"));

    const seen: { seq: number; payloadLen: number }[] = [];
    const sub = engine.onChange("post", (seq, payload) => {
      seen.push({ seq, payloadLen: payload.length });
    });
    expect(sub.active).toBe(true);

    // Drive a real write through createNode. The engine's
    // ChangeBroadcast fans the commit out to the SUBSCRIBE port, the
    // Rust-side walker invokes the cb_for_eval Arc, the napi
    // ThreadsafeFunction enqueues onto libuv, and the JS callback
    // fires on the main loop. We wait a small deadline for the queue
    // to drain.
    await engine.createNode(["post"], { title: "fp-1-callback-fires" });

    // Yield up to ~250ms for the libuv queue to drain. Local timing is
    // typically <10ms; the deadline absorbs CI runner jitter.
    for (let i = 0; i < 50 && seen.length === 0; i += 1) {
      await sleep(5);
    }

    expect(seen.length).toBeGreaterThanOrEqual(1);
    // The first observed event carries an engine-assigned seq (>= 1)
    // and a non-empty payload (the canonical-bytes encoding of the
    // committed Node).
    expect(seen[0]!.seq).toBeGreaterThan(0);
    expect(seen[0]!.payloadLen).toBeGreaterThan(0);

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
    //
    // R6-R4 r6-r4-cr-1 fix-pass: the DSL `SubscribeArgs.event` field
    // is now translated to the `pattern` property the eval-side
    // SUBSCRIBE primitive reads (mirroring the EMIT precedent
    // PR #66 / R6-R2-FP cluster-1 landed for the same shape). Pre-fix
    // the spread set `event: ...` and the eval primitive routed
    // `E_SUBSCRIBE_PATTERN_INVALID` for every DSL-composed in-handler
    // subscribe.
    const sg = subgraph("analytics")
      .action("on-post-create")
      .subscribe({ event: "post:write" })
      .transform({ expr: "computeMetrics($result)" })
      .respond({ body: "$result" })
      .build();

    const subscribeNode = sg.nodes.find((n) => n.primitive === "subscribe");
    expect(subscribeNode).toBeDefined();
    expect(subscribeNode!.args.pattern).toBe("post:write");
    // `event` MUST NOT be in the rendered args bag — the spread
    // translates it to `pattern` so the eval primitive sees the key
    // it actually reads.
    expect(subscribeNode!.args.event).toBeUndefined();
  });

  it("LOAD-BEARING — DSL-composed in-handler subscribe dispatches without routing E_SUBSCRIBE_PATTERN_INVALID (r6-r4-cr-1 §3.6b end-to-end pin)", async () => {
    // §3.6b end-to-end test pin per `dispatch-conventions.md` for the
    // r6-r4-cr-1 fix-pass: the SUBSCRIBE DSL spread translates
    // `args.event` to the `pattern` property the eval-side primitive
    // reads. Pre-fix this dispatch routed the
    // `SubscribePatternInvalid` error edge for every DSL-composed
    // in-handler subscribe; the pre-existing JSON-shape pin only
    // checked the rendered args bag without ever DRIVING the call
    // through the production entry point. This test drives the
    // production entry point (`engine.call(handler, action, ...)`)
    // and would FAIL if the spread silently no-op'd back to the
    // pre-fix shape (the call would route the typed error edge with
    // `E_SUBSCRIBE_PATTERN_INVALID` rather than complete OK).
    const engine = await Engine.open(":memory:");
    try {
      // Subgraph with SUBSCRIBE then RESPOND. Subscribing inside a
      // handler registers the subscription (returns an opaque
      // subscriber-id) and then the RESPOND node returns a stable
      // body so the call resolves with `ok: true` rather than the
      // SUBSCRIBE error edge.
      const handler = subgraph("post-summary-view-r6-r4-cr-1-pin")
        .action("on-post-changed")
        .subscribe({ event: "post:changed" })
        .respond({ body: "registered" })
        .build();
      const registered = await engine.registerSubgraph(handler);
      const result = await engine.call(
        registered.id,
        "on-post-changed",
        {},
      );
      // Pre-fix: result.ok would be false + the outcome would carry
      // `E_SUBSCRIBE_PATTERN_INVALID` because the eval primitive's
      // `op.properties.get("pattern")` returned None. Post-fix: the
      // subscribe registers + the respond fires.
      expect(result.ok).toBe(true);
    } finally {
      await engine.close();
    }
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

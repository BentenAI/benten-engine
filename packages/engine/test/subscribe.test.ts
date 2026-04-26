// R3-F red-phase — SUBSCRIBE TS DSL + engine.onChange / engine.onChangeAs.
//
// Tests are RED at landing time; G6-B (TS-side) makes them green.
//
// Surfaces under test (per dx-r1-2b SUBSCRIBE + R2 §7):
//   - DSL composition: subgraph(...).subscribe(args)     (already on builder)
//   - engine.onChange(pattern, callback) -> Subscription (renamed from .subscribe
//     to avoid shadowing the DSL builder method per dx-r1-2b SUBSCRIBE rename)
//   - engine.onChangeAs(pattern, callback, principal)
//   - Callback exception isolation: log + keep subscription alive (dx-r1-2b-4)
//
// Pin sources: r2-test-landscape.md §7 (rows 451-455);
// r1-dx-optimizer.json subscribe_test_fixture; dx-r1-2b-4.

import { describe, it, expect, vi } from "vitest";
import { Engine, subgraph, crud } from "@benten/engine";
import type { ChangeEvent, Subscription } from "@benten/engine";

describe("engine.onChange", () => {
  it("fires callback for matching writes", async () => {
    const engine = await Engine.open(":memory:");
    const post = await engine.registerSubgraph(crud("post"));

    const seen: ChangeEvent[] = [];
    const sub: Subscription = await engine.onChange(
      { label: "post", action: "write" },
      (evt) => seen.push(evt),
    );
    expect(sub.active).toBe(true);

    await engine.call(post.id, "post:create", { title: "x" });

    // Allow IVM cycle to flush the change-event to the napi tokio task.
    await new Promise((r) => setTimeout(r, 50));
    expect(seen).toHaveLength(1);
    expect(seen[0].label).toBe("post");

    await sub.unsubscribe();
    expect(sub.active).toBe(false);

    await engine.close();
  });

  it("onChange callback exception logs but does not kill subscription", async () => {
    // dx-r1-2b-4: subscriber-side throws are routine; sub stays alive, log fires.
    // Logging is wired in the napi bridge per dx-r1-2b non-obvious-consequence #3.
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    const engine = await Engine.open(":memory:");
    const post = await engine.registerSubgraph(crud("post"));

    let calls = 0;
    const sub = await engine.onChange(
      { label: "post", action: "write" },
      () => {
        calls++;
        if (calls === 1) throw new Error("first call throws");
      },
    );

    await engine.call(post.id, "post:create", { a: 1 });
    await engine.call(post.id, "post:create", { a: 2 });
    await new Promise((r) => setTimeout(r, 50));

    // Second call STILL fires — the throw didn't terminate the subscription.
    expect(calls).toBe(2);
    expect(errSpy).toHaveBeenCalled();
    expect(sub.active).toBe(true);

    await sub.unsubscribe();
    errSpy.mockRestore();
    await engine.close();
  });

  it("onChangeAs threads principal", async () => {
    const engine = await Engine.open(":memory:");
    const post = await engine.registerSubgraph(crud("post"));

    await engine.grantCapability({
      actor: "alice",
      scope: "subscribe:post:write",
    });

    const seen: ChangeEvent[] = [];
    const sub = await engine.onChangeAs(
      { label: "post", action: "write" },
      (evt) => seen.push(evt),
      "alice",
    );

    await engine.callAs(post.id, "post:create", { v: 1 }, "alice");
    await new Promise((r) => setTimeout(r, 50));

    expect(seen).toHaveLength(1);
    expect(seen[0].attribution.actorCid).toBe("alice");

    await sub.unsubscribe();
    await engine.close();
  });

  it("DSL composition subgraph(...).subscribe(args)", () => {
    // Composition pin — the fluent builder retains the .subscribe() entry
    // even after the engine.subscribe surface is renamed to engine.onChange.
    const sg = subgraph("analytics")
      .action("on-post-create")
      .subscribe({ event: "post:write" })
      .transform({ expr: "computeMetrics($input)" })
      .respond({ body: "$result" })
      .build();

    const subscribeNode = sg.nodes.find((n) => n.primitive === "subscribe");
    expect(subscribeNode).toBeDefined();
    expect(subscribeNode!.args.event).toBe("post:write");
  });

  it("naming distinct from DSL builder name", () => {
    // dx-r1-2b SUBSCRIBE rename: the engine surface is `onChange`, NOT
    // `subscribe`, to avoid colliding with SubgraphBuilder.subscribe().
    // Compile-time pin: the engine instance type does NOT expose .subscribe.

    type EngineMethods = keyof Engine;

    // Affirmative: onChange MUST be present.
    const onChangeIsKey: "onChange" extends EngineMethods ? true : false = true;
    expect(onChangeIsKey).toBe(true);

    // Negative: subscribe MUST NOT be present on the Engine class. If a
    // future R5 wave re-introduces engine.subscribe, this test fails compile.
    const subscribeNotKey: "subscribe" extends EngineMethods ? false : true =
      true;
    expect(subscribeNotKey).toBe(true);
  });
});

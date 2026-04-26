// R3-F red-phase — STREAM TS DSL + engine.callStream / engine.openStream surfaces.
//
// Tests are RED at landing time; G6-B (TS-side) makes them green.
//
// Surfaces under test (per dx-r1-2b STREAM + R2 §7):
//   - DSL composition: subgraph(...).stream(args)        (already on builder)
//   - engine.callStream(handlerId, action, input) -> AsyncIterable<Chunk>
//   - engine.openStream(handlerId, action, input) -> StreamHandle (close + iterable)
//   - engine.callStreamAs(handlerId, action, input, principal) — auth variant
//
// Pin sources: r2-test-landscape.md §7 (rows 446-450); r1-dx-optimizer.json
// dsl_builder_test_writer_handoff.stream_test_fixture; dx-r1-2b-3 (for-await
// break propagation); dx-r1-2b STREAM (handle / authenticated variant).

import { describe, it, expect } from "vitest";
import { Engine, subgraph } from "@benten/engine";
import type { Chunk, StreamHandle } from "@benten/engine";

describe("engine.callStream", () => {
  it("yields chunks in seq order with for-await", async () => {
    const engine = await Engine.open(":memory:");
    const sg = subgraph("counter")
      .action("count")
      .stream({ source: "$input.upTo", chunkSize: 1 })
      .respond({ body: "$result" });
    await engine.registerSubgraph(sg.build());

    const seen: number[] = [];
    for await (const chunk of engine.callStream("counter", "count", {
      upTo: 5,
    })) {
      seen.push(chunk.seq);
    }
    expect(seen).toEqual([0, 1, 2, 3, 4]);

    await engine.close();
  });

  it("for-await break releases producer (no orphan)", async () => {
    // dx-r1-2b-3: AsyncIterator return() must propagate to producer-close so
    // breaking out of for-await cleans up Rust-side resources before GC.
    const engine = await Engine.open(":memory:");
    const sg = subgraph("infinite")
      .action("go")
      .stream({ source: "$input" })
      .respond({ body: "$result" });
    await engine.registerSubgraph(sg.build());

    let count = 0;
    for await (const _c of engine.callStream("infinite", "go", {})) {
      count++;
      if (count >= 3) break;
    }

    // Pin: producer-side count drains to 0 within 100ms (allows the napi
    // bridge's tokio task to observe channel-close + tear down).
    await new Promise((r) => setTimeout(r, 100));
    expect(await engine.activeStreamCount()).toBe(0);

    await engine.close();
  });

  it("openStream explicit close idempotent", async () => {
    const engine = await Engine.open(":memory:");
    const sg = subgraph("infinite-2")
      .action("go")
      .stream({ source: "$input" })
      .respond({ body: "$result" });
    await engine.registerSubgraph(sg.build());

    const handle: StreamHandle = engine.openStream("infinite-2", "go", {});
    expect(handle.closed).toBe(false);

    await handle.close();
    expect(handle.closed).toBe(true);

    // Second close MUST NOT throw — idempotent contract per dx-r1-2b STREAM.
    await expect(handle.close()).resolves.toBeUndefined();
    expect(handle.closed).toBe(true);

    await engine.close();
  });

  it("callStreamAs threads principal", async () => {
    // Mirrors callAs / callWithSuspensionAs naming pattern; cap-grant on the
    // STREAM source resolves under the named principal not the engine default.
    const engine = await Engine.open(":memory:");
    const sg = subgraph("authed-stream")
      .action("go")
      .stream({ source: "$input.upTo", chunkSize: 1 })
      .respond({ body: "$result" });
    await engine.registerSubgraph(sg.build());

    await engine.grantCapability({
      actor: "alice",
      scope: "stream:authed-stream:read",
    });

    const seen: Chunk[] = [];
    for await (const chunk of engine.callStreamAs(
      "authed-stream",
      "go",
      { upTo: 2 },
      "alice",
    )) {
      seen.push(chunk);
    }
    expect(seen.map((c) => c.seq)).toEqual([0, 1]);

    await engine.close();
  });

  it("DSL composition subgraph(...).stream(args)", () => {
    // Compile-time + structural pin: the fluent builder accepts .stream and
    // produces a StreamArgs-bearing OperationNode. R5 widening (G12-D) may
    // expand StreamArgs; this test just asserts the composition entry survives.
    const sg = subgraph("compose")
      .action("go")
      .read({ label: "doc" })
      .stream({ source: "$result", chunkSize: 64 })
      .respond({ body: "$result" })
      .build();

    const streamNode = sg.nodes.find((n) => n.primitive === "stream");
    expect(streamNode).toBeDefined();
    expect(streamNode!.args.source).toBe("$result");
    expect(streamNode!.args.chunkSize).toBe(64);
  });
});

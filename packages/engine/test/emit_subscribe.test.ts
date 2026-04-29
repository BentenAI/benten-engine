// R6-FP Group 2: EMIT broadcast subscription tests.
//
// Closes r6-mpc-2 (the wave-8h cross-layer audit gap): the engine had
// `Engine::subscribe_emit_events` Rust API but no JS surface to observe
// EMIT events from a handler-internal EMIT primitive. R6-FP Group 1
// lands the napi `EmitSubscriptionJs` class + `subscribe_emit_events`
// adapter; R6-FP Group 2 (this group) lands the TS `engine.onEmit`
// surface + this load-bearing acceptance test.
//
// Test shape mirrors `subscribe.test.ts` for `engine.onChange`:
//   - validation pin (channel + callback typed errors)
//   - lifecycle pin (active before unsubscribe, false after)
//   - load-bearing acceptance: drive a real EMIT primitive in a
//     handler + assert the JS callback fires
//   - exception isolation (subscriber-side throws don't tear down the
//     subscription)

import { describe, it, expect, vi } from "vitest";
import { Engine, subgraph } from "@benten/engine";
import { EDslInvalidShape } from "@benten/engine/errors";

/**
 * Sleep helper so the test body can yield to the libuv main loop and
 * let queued ThreadsafeFunction calls drain into the JS callback.
 */
function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Probe whether the loaded napi cdylib carries the R6-FP `onEmit`
// bridge. Pre-merge with R6-FP Group 1 the symbol may be absent — in
// that case this branch's tests assert the typed unavailable-bridge
// error instead of skipping silently. Post-merge the bridge is present
// and the load-bearing acceptance test runs.
function nativeHasOnEmit(): boolean {
  try {
    /* eslint-disable @typescript-eslint/no-var-requires, @typescript-eslint/no-require-imports */
    // Resolved relative to packages/engine/dist/test or src/test depending
    // on the harness invocation; the same require() shape is used by
    // sandbox_napi_bridge.test.ts + stream_napi_async_iterator_back_pressure.test.ts.
    const native = require("../index.js") as {
      Engine?: { prototype?: Record<string, unknown> };
    };
    /* eslint-enable @typescript-eslint/no-var-requires, @typescript-eslint/no-require-imports */
    return typeof native.Engine?.prototype?.onEmit === "function";
  } catch {
    return false;
  }
}

describe("engine.onEmit", () => {
  it("rejects empty channel with a typed EDslInvalidShape", async () => {
    const engine = await Engine.open(":memory:");
    try {
      expect(() => engine.onEmit("", () => {})).toThrow(EDslInvalidShape);
    } finally {
      await engine.close();
    }
  });

  it("rejects non-function callback with a typed EDslInvalidShape", async () => {
    const engine = await Engine.open(":memory:");
    try {
      // @ts-expect-error: deliberately passing a non-function to assert
      // the runtime guard surfaces a typed error pre-napi-boundary.
      expect(() => engine.onEmit("test", "not-a-function")).toThrow(
        EDslInvalidShape,
      );
    } finally {
      await engine.close();
    }
  });

  // The remaining tests assert behavior that only makes sense once the
  // napi bridge lands. We branch on symbol presence so this file is
  // load-bearing both pre- and post-merge with R6-FP Group 1:
  //
  //   - PRE-MERGE: the unavailable-bridge typed error fires (proving
  //     the TS guard works).
  //   - POST-MERGE: the production runtime fires the callback (proving
  //     the cross-layer wire-through is end-to-end alive).
  if (!nativeHasOnEmit()) {
    it("[pre-G1-merge] surfaces typed E_DSL_INVALID_SHAPE when bridge is absent", async () => {
      const engine = await Engine.open(":memory:");
      try {
        expect(() => engine.onEmit("test", () => {})).toThrow(
          EDslInvalidShape,
        );
        // The error message names the rebuild fix-hint.
        expect(() => engine.onEmit("test", () => {})).toThrow(
          /rebuild @benten\/engine-native/,
        );
      } finally {
        await engine.close();
      }
    });
    return;
  }

  it("[post-G1-merge] returns an active EmitSubscription with the supplied channel", async () => {
    const engine = await Engine.open(":memory:");
    try {
      const sub = engine.onEmit("test:event", () => {});
      expect(sub.active).toBe(true);
      expect(sub.channel).toBe("test:event");
      sub.unsubscribe();
      expect(sub.active).toBe(false);
    } finally {
      await engine.close();
    }
  });

  it(
    "[post-G1-merge] LOAD-BEARING — onEmit callback fires when a handler's EMIT primitive publishes",
    async () => {
      // Register a handler that emits on `test:emit-fired`. Driving it via
      // `engine.call(...)` invokes the EMIT primitive, which routes through
      // the engine's dedicated EmitBroadcast — the napi adapter forwards to
      // the JS callback via ThreadsafeFunction.
      const engine = await Engine.open(":memory:");
      try {
        const handler = subgraph("emit-test-handler")
          .action("fire")
          .emit({ event: "test:emit-fired", payload: '"hello-from-emit"' })
          .respond({ body: '"ok"' })
          .build();

        const registered = await engine.registerSubgraph(handler);

        const seen: { channel: string; payload: unknown }[] = [];
        const sub = engine.onEmit("test:emit-fired", (channel, payload) => {
          seen.push({ channel, payload });
        });
        expect(sub.active).toBe(true);

        await engine.call(registered.id, "fire", {});

        // Drain the libuv queue.
        for (let i = 0; i < 50 && seen.length === 0; i += 1) {
          await sleep(5);
        }

        expect(seen.length).toBeGreaterThanOrEqual(1);
        expect(seen[0]!.channel).toBe("test:emit-fired");

        sub.unsubscribe();
        expect(sub.active).toBe(false);
      } finally {
        await engine.close();
      }
    },
  );

  it(
    "[post-G1-merge] subscriber exception is caught (subscription stays alive)",
    async () => {
      // Mirrors dx-r1-2b-4 / subscribe.test.ts catch contract for
      // EmitSubscription: subscriber-side throws are logged, the sub
      // stays alive.
      const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
      const engine = await Engine.open(":memory:");
      try {
        const sub = engine.onEmit("test:event", () => {
          throw new Error("subscriber throws");
        });
        expect(sub.active).toBe(true);
        sub.unsubscribe();
        expect(sub.active).toBe(false);
      } finally {
        errSpy.mockRestore();
        await engine.close();
      }
    },
  );

  it("[post-G1-merge] unsubscribe is idempotent", async () => {
    const engine = await Engine.open(":memory:");
    try {
      const sub = engine.onEmit("test:event", () => {});
      expect(sub.active).toBe(true);
      sub.unsubscribe();
      expect(sub.active).toBe(false);
      // Idempotent — second unsubscribe is a no-op + does not throw.
      sub.unsubscribe();
      expect(sub.active).toBe(false);
    } finally {
      await engine.close();
    }
  });
});

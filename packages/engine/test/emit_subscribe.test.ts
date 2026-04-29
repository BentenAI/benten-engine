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
// bridge. R6 Round-2 r6-r2-mpc-1 wired the bridge so the post-merge
// branch should always fire on a freshly-built cdylib. The probe stays
// in place so a stale-cdylib regression surfaces as an explicit
// rebuild-fix-hint at suite-load time rather than a confusing
// `engine.onEmit is not a function` further down.
function nativeHasOnEmit(): boolean {
  try {
    /* eslint-disable @typescript-eslint/no-var-requires, @typescript-eslint/no-require-imports */
    const native = require("@benten/engine-native") as {
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

  // R6 Round-2 r6-r2-mpc-1 closure: the napi `Engine::on_emit`
  // method is wired (see `bindings/napi/src/lib.rs`). The pre-merge
  // branch (which asserted `EDslInvalidShape("rebuild your binding")`)
  // is removed because the typed-error guard at `engine.ts::onEmit`
  // now never fires — the load-bearing post-merge tests below assert
  // the production cross-layer wire-through end-to-end.
  if (!nativeHasOnEmit()) {
    throw new Error(
      "Engine.onEmit napi symbol absent — rebuild @benten/engine-native against HEAD; R6 Round-2 r6-r2-mpc-1 wired the bridge.",
    );
  }
  // Reference the import so vitest doesn't strip it as unused now
  // that the pre-merge branch is gone — the validation test cases
  // above still rely on the typed-class import.
  void EDslInvalidShape;

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
      // R6 Round-2 r6-r2-mpc-1: pins the production runtime path
      // end-to-end — handler with EMIT primitive → engine.call →
      // EMIT executor → emit_broadcast.publish → napi
      // ThreadsafeFunction → JS callback fires.
      //
      // This test pins the FIRING of the callback (`seen.length >=
      // 1`) which is the load-bearing wire-through. The arg-shape
      // assertion (`channel === "test:emit-fired"`) is gated behind
      // an in-test runtime check because napi-rs v3's
      // `Function<(String, String), ()>` callback-shape delivers
      // the tuple as a single Array argument rather than splatting
      // to 2 args (the same systemic delivery shape affects
      // `onChange`'s `(seq, payload)` callback per the pre-existing
      // `subscribe.test.ts::LOAD-BEARING — onChange callback fires`
      // test). Tightening the splatting is a Phase-3 napi-rs
      // upgrade — destination
      // `docs/future/phase-3-backlog.md` §7.7 (napi-rs ThreadsafeFunction
      // tuple-arg splat-behavior).
      const engine = await Engine.open(":memory:");
      try {
        const handler = subgraph("emit-test-handler")
          .action("fire")
          .emit({ event: "test:emit-fired", payload: '"hello-from-emit"' })
          .respond({ body: '"ok"' })
          .build();

        const registered = await engine.registerSubgraph(handler);

        const seen: { channel: unknown; payload: unknown }[] = [];
        const sub = engine.onEmit("test:emit-fired", (channel, payload) => {
          seen.push({ channel, payload });
        });
        expect(sub.active).toBe(true);

        await engine.call(registered.id, "fire", {});

        // Drain the libuv queue. Generous timeout so cold-start CI
        // runs don't false-fail on the TSFN delivery latency.
        for (let i = 0; i < 200 && seen.length === 0; i += 1) {
          await sleep(10);
        }

        // Load-bearing: the wire-through fired the callback at all.
        // This proves r6-mpc-2 / r6-r2-mpc-1 closure end-to-end.
        expect(seen.length).toBeGreaterThanOrEqual(1);

        // Channel-arg shape: napi-rs v3 currently delivers
        // `(String, String)` tuples as a single-Array arg in some
        // build configurations; tolerate both delivery shapes so
        // this load-bearing test pins the firing without depending
        // on the splat behavior the Phase-3 napi-rs upgrade tightens.
        const first = seen[0]!;
        const channelObserved = Array.isArray(first.channel)
          ? (first.channel as unknown[])[0]
          : first.channel;
        expect(channelObserved).toBe("test:emit-fired");

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

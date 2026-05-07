// Phase-3 G19-B ACTIVATED pins — napi-rs ThreadsafeFunction tuple-arg
// splat (wave-7 parallel; §7.7 + r1-napi-4 keep-wrapper path).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-B +
// .addl/phase-3/00-implementation-plan.md §3 G19-B must-pass column):
//
//   - tests/onchange_onemit_callback_splats_args_correctly — §7.7
//
// What G19-B establishes (§7.7 + r1-napi-4):
//
//   napi-rs ThreadsafeFunction passes args as a single tuple-shaped JS
//   array. Per r1-napi-4 keep-wrapper path: G19-B updates engine.ts so
//   that `onChange` and `onEmit` receive the single tuple-arg +
//   destructure inside; the user-facing callback ALWAYS sees discrete
//   `(channel, payload)` / `(seq, payload)` args. The in-test
//   `Array.isArray(...)` workaround is retired at every call site.

import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { resolve, dirname } from "node:path";
import { Engine, subgraph, crud } from "@benten/engine";

const HERE = dirname(fileURLToPath(import.meta.url));

async function sleep(ms: number): Promise<void> {
  await new Promise((res) => setTimeout(res, ms));
}

describe("G19-B napi-rs ThreadsafeFunction tuple-arg splat (§7.7)", () => {
  it("G19-B wave-7 — onEmit callback splats args correctly", async () => {
    // r1-napi-4 keep-wrapper path test pin.
    const engine = await Engine.open(":memory:");
    try {
      const events: Array<{ channel: unknown; payload: unknown }> = [];

      // Subscribe — callback signature MUST receive (channel, payload)
      // discrete args, NOT a single tuple-array:
      const sub = engine.onEmit("alerts", (channel, payload) => {
        events.push({ channel, payload });
      });
      expect(sub.active).toBe(true);

      await engine.emitEvent("alerts", { msg: "hello" });

      // Drain libuv queue.
      for (let i = 0; i < 200 && events.length === 0; i += 1) {
        await sleep(10);
      }

      // OBSERVABLE consequence: callback fires with discrete args.
      expect(events.length).toBeGreaterThanOrEqual(1);
      const first = events[0]!;
      // Defensive guard: channel must be a string, NOT an Array of
      // [string, unknown] (which would indicate the tuple-arg
      // unwrapping never happened — the workaround state).
      expect(typeof first.channel).toBe("string");
      expect(Array.isArray(first.channel)).toBe(false);
      expect(first.channel).toBe("alerts");
      expect(first.payload).toEqual({ msg: "hello" });

      sub.unsubscribe();
    } finally {
      await engine.close();
    }
  });

  it("G19-B wave-7 — onChange callback splats args correctly", async () => {
    // Companion pin for onChange (parallel surface to onEmit).
    const engine = await Engine.open(":memory:");
    try {
      const post = await engine.registerSubgraph(crud("post"));

      const events: Array<{ seq: unknown; chunk: unknown }> = [];
      const sub = engine.onChange("*", (seq, chunk) => {
        events.push({ seq, chunk });
      });

      // Trigger a change — call a CRUD post:create handler so the
      // wildcard subscription fires:
      await engine.call(post.id, "post:create", { title: "x" });

      // Drain libuv queue.
      for (let i = 0; i < 200 && events.length === 0; i += 1) {
        await sleep(10);
      }

      // OBSERVABLE consequence: onChange callback receives discrete
      // (seq, payload) args — not a tuple-array.
      expect(events.length).toBeGreaterThanOrEqual(1);
      const first = events[0]!;
      expect(typeof first.seq).toBe("number");
      expect(Array.isArray(first.seq)).toBe(false);
      // Buffer is the payload type per OnChangeCallback.
      expect(first.chunk).toBeInstanceOf(Buffer);

      sub.unsubscribe();
    } finally {
      await engine.close();
    }
  });

  it("G19-B wave-7 — in-test Array.isArray(channel) workaround retired", () => {
    // r1-napi-4 keep-wrapper-path closure pin: post-G19-B, the
    // `packages/engine/test/emit_subscribe.test.ts` callback site
    // does NOT contain `Array.isArray(channel)`-based unwrapping.
    // The workaround is retired (per dispatch-conventions §3.5b
    // HARDENED post-fix doc-coupling sweep).
    const src = readFileSync(
      resolve(HERE, "emit_subscribe.test.ts"),
      "utf-8",
    );
    expect(src).not.toContain("Array.isArray(first.channel)");
  });
});

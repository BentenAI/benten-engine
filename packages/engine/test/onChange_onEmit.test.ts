// R3-E RED-PHASE pins for G19-B napi-rs ThreadsafeFunction tuple-arg splat
// (wave-7 parallel; §7.7 + r1-napi-4 keep-wrapper path).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-B +
// .addl/phase-3/00-implementation-plan.md §3 G19-B must-pass column):
//
//   - tests/onchange_onemit_callback_splats_args_correctly — §7.7
//
// What G19-B establishes (§7.7 + r1-napi-4):
//
//   The Phase-2b state: napi-rs ThreadsafeFunction passes args as a single
//   tuple-shaped JS array; engine.ts callbacks rely on an in-test
//   `Array.isArray(channel)` workaround at the receiving site. Per
//   r1-napi-4 RECOMMEND keep-wrapper: G19-B updates engine.ts so that
//   `onChange` and `onEmit` receive a single tuple-arg + destructure
//   inside; retires the in-test workaround.
//
// RED-PHASE discipline:
//
//   These tests assert the post-G19-B shape — callback receives discrete
//   args (channel, payload) NOT a single Array-wrapped tuple. They are
//   .skip'd until G19-B ships. R5 implementer drops .skip + un-comments
//   the assertion bodies.

import { describe, it, expect } from "vitest";
import { Engine } from "@benten/engine";

describe("G19-B napi-rs ThreadsafeFunction tuple-arg splat (§7.7)", () => {
  it.skip("RED-PHASE: G19-B wave-7 — onEmit callback splats args correctly", async () => {
    // r1-napi-4 keep-wrapper path test pin. G19-B implementer wires this:
    //
    //   const engine = await Engine.open(":memory:");
    //   const events: Array<{ channel: string; payload: unknown }> = [];
    //
    //   // Subscribe — callback signature MUST receive (channel, payload)
    //   // discrete args, NOT a single tuple-array:
    //   engine.onEmit("alerts", (channel, payload) => {
    //     // Defensive guard: channel must be a string, NOT an Array of
    //     // [string, unknown] (which would indicate the tuple-arg
    //     // unwrapping never happened — the workaround state).
    //     expect(typeof channel).toBe("string");
    //     expect(Array.isArray(channel)).toBe(false);
    //     events.push({ channel, payload });
    //   });
    //
    //   await engine.emitEvent("alerts", { msg: "hello" });
    //
    //   // OBSERVABLE consequence: callback fires with discrete args.
    //   expect(events).toHaveLength(1);
    //   expect(events[0].channel).toBe("alerts");
    //   expect(events[0].payload).toEqual({ msg: "hello" });
    //
    // Defends against the workaround-fossil shape where engine.ts kept
    // the Array.isArray(channel) check post-G19-B (which would be a
    // pim-1 doc-coupling failure: code shipped, workaround stayed).
    throw new Error(
      "RED-PHASE: G19-B wave-7 wires onEmit ThreadsafeFunction tuple-arg splat + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-B wave-7 — onChange callback splats args correctly", async () => {
    // Companion pin for onChange (parallel surface to onEmit).
    //
    //   const engine = await Engine.open(":memory:");
    //   const changes: Array<{ pattern: string; event: unknown }> = [];
    //
    //   engine.onChange({ kind: "label", value: "post" }, (pattern, event) => {
    //     // Same discrete-args contract:
    //     expect(typeof pattern).not.toBe("undefined");
    //     expect(Array.isArray(pattern)).toBe(false);
    //     changes.push({ pattern, event });
    //   });
    //
    //   // Trigger a change — call a CRUD post:create handler so the
    //   // subscribed pattern fires:
    //   // const post = await engine.registerSubgraph(crud("post"));
    //   // await engine.call(post.id, "post:create", { title: "x" });
    //
    //   expect(changes.length).toBeGreaterThanOrEqual(1);
    //   // Discrete-args assertion: pattern + event are separate values.
    //
    // OBSERVABLE consequence: onChange callback receives discrete
    // (pattern, event) args — not a tuple-array.
    throw new Error(
      "RED-PHASE: G19-B wave-7 wires onChange ThreadsafeFunction tuple-arg splat + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-B wave-7 — in-test Array.isArray(channel) workaround retired", async () => {
    // r1-napi-4 keep-wrapper-path closure pin: post-G19-B, the
    // `packages/engine/test/emit_subscribe.test.ts` callback site
    // should NOT contain `Array.isArray(channel)`-based unwrapping.
    // The workaround is retired.
    //
    //   import { readFileSync } from "node:fs";
    //   import { fileURLToPath } from "node:url";
    //   import { resolve, dirname } from "node:path";
    //
    //   const here = dirname(fileURLToPath(import.meta.url));
    //   const src = readFileSync(
    //     resolve(here, "..", "test", "emit_subscribe.test.ts"),
    //     "utf-8",
    //   );
    //   expect(src).not.toContain("Array.isArray(channel)");
    //
    // OBSERVABLE consequence: doc-coupling discipline holds — post-fix
    // sweep retires the workaround at its call site (per
    // dispatch-conventions §3.5b HARDENED).
    throw new Error(
      "RED-PHASE: G19-B wave-7 retires Array.isArray(channel) workaround in emit_subscribe.test.ts + drops .skip + un-comments assertions",
    );
  });
});

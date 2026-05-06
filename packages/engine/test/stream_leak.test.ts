// R3-E RED-PHASE pins for G19-C2 openStream FinalizationRegistry leak
// detector + requiresExplicitClose accessor (wave-7 parallel; §7.1.2 +
// stream-r1-4 4-scenario enumeration + stream-r1-10 cross-browser scope).
//
// Pin sources (per .addl/phase-3/r2-test-landscape.md §2.7 G19-C2 +
// .addl/phase-3/00-implementation-plan.md §3 G19-C2 must-pass column):
//
//   - tests/stream_handle_leak_detected_via_finalization_registry_fires_e_stream_handle_leaked
//   - tests/stream_handle_leak_finalization_registry_callback_fires_scenario_a (stream-r1-4)
//   - tests/stream_handle_leak_close_not_called_assertion_scenario_b (stream-r1-4)
//   - tests/stream_handle_leak_gc_pressure_timeout_scenario_c (stream-r1-4)
//   - tests/stream_natural_completion_does_not_fire_e_stream_handle_leaked (stream-r1-4 explicit-close-semantics)
//
// What G19-C2 establishes (§7.1.2 + stream-r1-4):
//
//   packages/engine/src/stream.ts wraps openStream returns with a
//   FinalizationRegistry-backed leak detector. When a handle is GC'd
//   without explicit close()/cancel(), the detector fires E_STREAM_HANDLE_LEAKED.
//
//   Per stream-r1-4 enumerated 4 scenarios:
//     (a) handler returns + handle GC'd without close()/cancel() — MUST fire
//     (b) handler throws + handle GC'd — MUST fire
//     (c) handler completes the stream naturally (final_chunk emitted) +
//         handle GC'd without close() — must NOT fire (the producer
//         ended naturally; explicit-close-semantics)
//     (d) Engine.shutdown() called while handle still open — MUST fire
//         on shutdown drain
//
// Per stream-r1-10 + stream-r4r1-8: tests are Node-only (FinalizationRegistry
// GC scheduling differs across Chromium V8 / Gecko SpiderMonkey / WebKit
// JavaScriptCore); promotion to cross-browser via deterministic
// GC-pressure helper landing in a Phase-3 narrow-iter cycle.
//
// `it.skipIf` guard below enforces the Node-only restriction at the
// vitest level rather than relying on rationale-comment convention —
// G18-A's Playwright matrix (br-r1-10 cross-browser-determinism.yml
// per-PR cadence) MUST NOT consume cross-browser flake budget on these
// tests when run under non-Node runtimes (Gecko / WebKit). The
// `IS_NODE_RUNTIME` predicate inspects `process.versions.node` which is
// truthy under Node + falsy under browser-like runtimes.
//
// RED-PHASE discipline:
//
//   None of these surfaces exists yet. R5 implementer wires them.

import { describe, it, expect } from "vitest";

const IS_NODE_RUNTIME =
  typeof process !== "undefined" &&
  typeof process.versions !== "undefined" &&
  typeof process.versions.node === "string";

describe.skipIf(!IS_NODE_RUNTIME)(
  "G19-C2 openStream FinalizationRegistry leak detector (§7.1.2 + stream-r1-4)",
  () => {
  // r4-r2-napi-1 scenario renumbering (2026-05-05): test functions
  // align with the master stream-r1-4 brief enumeration:
  //   (a) handler-returns + handle GC'd without close() — MUST fire
  //   (b) handler-throws + handle GC'd — MUST fire
  //   (c) natural-completion (final_chunk emitted) + handle GC'd
  //       without close() — must NOT fire (negative pin)
  //   (d) Engine.shutdown() called while handle still open — MUST fire
  // The GC-pressure-timeout polling fallback is a SUB-MECHANISM
  // orthogonal to the 4-scenario enumeration; pinned separately as
  // `gc_pressure_polling_fallback`.
  //
  // Outer describe.skipIf(!IS_NODE_RUNTIME) wraps these per stream-r4r1-8
  // (R2-FP-E PR #99) — FinalizationRegistry leak detector is Node-only;
  // browser-target tests skip cleanly per cross-browser-determinism scope.

  it.skip("RED-PHASE: G19-C2 wave-7 — scenario (a) handler-returns-no-close: leak fires E_STREAM_HANDLE_LEAKED via FinalizationRegistry callback", async () => {
    // stream-r1-4 scenario (a): handler returns + handle GC'd without
    // close()/cancel(). G19-C2 implementer wires this:
    //
    //   const { Engine } = await import("@benten/engine");
    //   const engine = await Engine.open(":memory:");
    //   const sg = await engine.registerStreamHandler(/* ... */);
    //
    //   const errors: Array<{ code: string }> = [];
    //   engine.onStreamLeaked((err) => errors.push(err));
    //
    //   {
    //     const stream = await engine.openStream(sg, "main", {});
    //     // Handler RETURNS without consuming the stream OR closing it.
    //     // Skip read/close intentionally to simulate the leak shape.
    //   }
    //
    //   // Force GC pressure (Node-only via --expose-gc):
    //   if (typeof global.gc === "function") {
    //     for (let i = 0; i < 5; i++) {
    //       global.gc();
    //       await new Promise((r) => setTimeout(r, 50));
    //     }
    //   }
    //
    //   // OBSERVABLE consequence: FinalizationRegistry callback fired:
    //   expect(errors.length).toBeGreaterThanOrEqual(1);
    //   expect(errors[0].code).toBe("E_STREAM_HANDLE_LEAKED");
    //
    // Defends against the sentinel-presence failure mode where the
    // leak detector exists but never fires.
    throw new Error(
      "RED-PHASE: G19-C2 wave-7 wires FinalizationRegistry leak scenario-a + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-C2 wave-7 — scenario (b) handler-throws-no-close: leak fires when close() not called assertion", async () => {
    // stream-r1-4 scenario (b): handler throws + handle GC'd. G19-C2
    // implementer wires this:
    //
    //   const errors: Array<{ code: string; cause?: string }> = [];
    //   engine.onStreamLeaked((err) => errors.push(err));
    //
    //   try {
    //     const stream = await engine.openStream(sg, "main", {});
    //     throw new Error("simulated handler error");
    //   } catch (_e) {
    //     // discarded
    //   }
    //
    //   // GC pressure:
    //   if (typeof global.gc === "function") {
    //     for (let i = 0; i < 5; i++) global.gc();
    //   }
    //
    //   // E_STREAM_HANDLE_LEAKED fires AND attribution distinguishes
    //   // throw-vs-return at the callback (different cause field):
    //   expect(errors.length).toBeGreaterThanOrEqual(1);
    //   expect(errors[0].code).toBe("E_STREAM_HANDLE_LEAKED");
    //
    // OBSERVABLE consequence: leak detector fires on handler-throw + GC.
    throw new Error(
      "RED-PHASE: G19-C2 wave-7 wires FinalizationRegistry leak scenario-b + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-C2 wave-7 — gc_pressure_polling_fallback: GC pressure timeout fires leak (sub-mechanism, orthogonal to 4-scenario enumeration)", async () => {
    // r4-r2-napi-1 renumbering: this is a SUB-MECHANISM orthogonal to
    // the master 4-scenario stream-r1-4 enumeration (NOT scenario-c).
    // The natural-completion negative pin is the master scenario (c);
    // see the function below.
    //
    // GC-pressure-timeout polling fallback for environments where
    // FinalizationRegistry callbacks are unreliable (e.g. Node without
    // --expose-gc).
    //
    //   const stream = await engine.openStream(sg, "main", {});
    //   // Drop the reference; rely on a polling-based detector with
    //   // bounded retry budget that asserts "if the handle has not
    //   // been close()d within the timeout, fire E_STREAM_HANDLE_LEAKED".
    //   // The timeout fires the leak event independently of GC scheduling.
    //
    //   await new Promise((r) => setTimeout(r, /* gc-pressure-timeout */ 5000));
    //
    //   // Defends against the GC-non-determinism flake mode that
    //   // stream-r1-10 named (Phase-2b coverage.yml flake precedent on
    //   // wait_signal_arrives_after_timeout_fires_e_wait_timeout).
    //
    // OBSERVABLE consequence: GC-pressure timeout polling fallback fires
    // independently of FinalizationRegistry callback scheduling.
    throw new Error(
      "RED-PHASE: G19-C2 wave-7 wires GC-pressure timeout fallback + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-C2 wave-7 — scenario (c) natural-completion-no-fire (negative pin): natural completion does NOT fire leak (stream-r1-4 explicit-close-semantics)", async () => {
    // r4-r2-napi-1 renumbering: per master stream-r1-4 enumeration
    // this is scenario (c) — the negative pin asserting natural
    // completion does NOT fire the leak. (Pre-r4-r2-napi-1 this test
    // was misnumbered scenario (d); now aligned with master brief.)
    // stream-r1-4 scenario (c): handler completes naturally (final_chunk
    // emitted) + handle GC'd. MUST NOT fire E_STREAM_HANDLE_LEAKED.
    //
    //   const errors: Array<{ code: string }> = [];
    //   engine.onStreamLeaked((err) => errors.push(err));
    //
    //   {
    //     const stream = await engine.openStream(sg, "main-with-natural-end", {});
    //     // Consume the stream to natural end:
    //     for await (const _chunk of stream) { /* drain */ }
    //     // Don't call close() — the producer ended naturally.
    //   }
    //
    //   // GC pressure:
    //   if (typeof global.gc === "function") {
    //     for (let i = 0; i < 5; i++) global.gc();
    //   }
    //
    //   // Critical negative pin: NO E_STREAM_HANDLE_LEAKED for natural
    //   // completion (the producer ended cleanly; close() is redundant):
    //   expect(errors.filter((e) => e.code === "E_STREAM_HANDLE_LEAKED")).toHaveLength(0);
    //
    // Defends against the false-positive failure mode where natural
    // completion erroneously fires the leak event (stream-r1-4 named
    // this as "the easy false-positive").
    throw new Error(
      "RED-PHASE: G19-C2 wave-7 wires natural-completion negative pin (no leak fires) + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-C2 wave-7 — scenario (d) engine-shutdown-while-open: Engine.shutdown() drains open handles + fires leak", async () => {
    // stream-r1-4 scenario (d): Engine.shutdown() called while a handle
    // is still open. Must fire E_STREAM_HANDLE_LEAKED on shutdown drain
    // rather than wait for GC.
    //
    //   const errors: Array<{ code: string }> = [];
    //   engine.onStreamLeaked((err) => errors.push(err));
    //
    //   const stream = await engine.openStream(sg, "main", {});
    //   // Don't close; trigger shutdown:
    //   await engine.shutdown();
    //
    //   // OBSERVABLE consequence: shutdown drain fires the leak event
    //   // for the still-open handle (no GC required):
    //   expect(errors.some((e) => e.code === "E_STREAM_HANDLE_LEAKED")).toBe(true);
    throw new Error(
      "RED-PHASE: G19-C2 wave-7 wires Engine.shutdown() drain leak + drops .skip + un-comments assertions",
    );
  });

  it.skip("RED-PHASE: G19-C2 wave-7 — engine.openStream returned handle exposes requiresExplicitClose accessor", async () => {
    // §7.1.2 sentinel-presence pin. G19-C2 implementer wires this:
    //
    //   const stream = await engine.openStream(sg, "main", {});
    //   expect(typeof (stream as any).requiresExplicitClose).toBe("function");
    //   expect((stream as any).requiresExplicitClose()).toBe(true);
    //
    // Sentinel-presence test (the accessor exists). Composes with the
    // 4-scenario end-to-end pins above per pim-2 §3.6b — sentinel pins
    // are useful scaffolding but the load-bearing assertion is the
    // observable consequence of the leak detector firing/not-firing
    // per scenario.
    throw new Error(
      "RED-PHASE: G19-C2 wave-7 wires requiresExplicitClose accessor + drops .skip + un-comments assertions",
    );
  });
  },
);

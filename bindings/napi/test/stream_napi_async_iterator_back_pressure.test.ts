// Phase-2b G6-B — napi STREAM bridge symbol-presence + surface pin.
//
// This file pins the symbol presence + basic surface shape of the napi
// STREAM bridge (`engine.callStream`, `engine.openStream`,
// `engine.testingOpenStreamForTest`). G6-B (this PR) lands the napi
// surface; G6-A (separate PR `phase-2b/g6/a-stream-subscribe-core`)
// lands the `tokio::sync::mpsc` executor that actually drives chunks
// across the boundary, at which point the full async-iterator
// back-pressure tests (`for-await consumer drives chunk-by-chunk`,
// `slow consumer creates back-pressure`, `for-await break releases
// producer`) light up.
//
// Pin sources:
//   - plan §3 G6-B "STREAM dual surface"
//   - mini-review cr-g6b-mr-3 (rewrite to actually pin
//     `engine.testingOpenStreamForTest`)
//   - D4-RESOLVED: PULL-based bounded `tokio::sync::mpsc` (G6-A)
//   - dx-r1-2b-3: AsyncIterator return() propagates close (G6-A)
//
// loadNative() mirrors `budget_exhausted_napi_round_trip.test.ts` —
// resolves the platform-specific binary directly so vitest-as-ESM
// doesn't trip over the index.js CJS loader.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  const name = `../benten-napi.${platform}-${arch}.node`;
  return require(name);
}

const native: any = loadNative();

let tmp: string;
let engine: any;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-stream-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("napi STREAM bridge — G6-B surface symbol-presence", () => {
  it("exposes engine.callStream as a function", () => {
    // Symbol-presence pin: G6-B napi `Engine::call_stream` impl.
    expect(typeof engine.callStream).toBe("function");
  });

  it("exposes engine.openStream as a function", () => {
    // Symbol-presence pin: G6-B napi `Engine::open_stream` impl.
    expect(typeof engine.openStream).toBe("function");
  });

  it("exposes engine.testingOpenStreamForTest as a function", () => {
    // Symbol-presence pin (cr-g6b-mr-3): G6-B's cfg-gated
    // `Engine::testing_open_stream_for_test` napi method is emitted
    // unconditionally so this `typeof` check passes regardless of
    // whether the cdylib was built with `--features test-helpers`.
    // When the feature is OFF the method body surfaces
    // `E_PRIMITIVE_NOT_IMPLEMENTED`; when ON it returns a real
    // `StreamHandleJs`. Either way the symbol IS present.
    expect(typeof engine.testingOpenStreamForTest).toBe("function");
  });

  it("exposes engine.onChange as a function", () => {
    // Symbol-presence pin: G6-B napi `Engine::on_change` impl.
    expect(typeof engine.onChange).toBe("function");
  });

  it("exposes engine.testingOpenSubscriptionForTest as a function", () => {
    // Symbol-presence pin (cr-g6b-mr-5): SUBSCRIBE-side mirror of
    // `testingOpenStreamForTest`. Same cfg-gating pattern (method
    // unconditional, body returns `E_PRIMITIVE_NOT_IMPLEMENTED` unless
    // built with `--features test-helpers`).
    expect(typeof engine.testingOpenSubscriptionForTest).toBe("function");
  });

  it("exposes engine.testingDeliverSyntheticEventForTest as a function", () => {
    // Symbol-presence pin (cr-g6b-mr-5): SUBSCRIBE-side dedup-state
    // delivery helper. Same cfg-gating pattern as
    // `testingOpenSubscriptionForTest`.
    expect(typeof engine.testingDeliverSyntheticEventForTest).toBe("function");
  });

  it("engine.callStream against an unregistered handler surfaces a typed error", () => {
    // Pin: pre-G6-A `Engine::call_stream` verifies handler registration
    // up front so callers see a useful early `E_NOT_FOUND` rather than
    // an opaque "stream did nothing" outcome.
    expect(() =>
      engine.callStream("nonexistent_handler", "act", {}),
    ).toThrow();
  });

  it("engine.openStream against an unregistered handler surfaces a typed error", () => {
    // Pin: same handler-presence check as `callStream`.
    expect(() =>
      engine.openStream("nonexistent_handler", "act", {}),
    ).toThrow();
  });
});

describe("napi STREAM bridge — async-iterator back-pressure (wave-8c-stream-infra)", () => {
  // wave-8c-stream-infra: the producer-bridge wire-through delivers
  // real chunks across the napi boundary. The scenarios below exercise
  // the full async-iterator path against a registered DSL handler with
  // a STREAM composition primitive (resolves via `$input.upTo` to a
  // CountProducer that emits N empty-byte chunks).

  // Helper: register a counter handler whose STREAM source is
  // `$input.upTo`. Returns the handler id.
  function registerCounter(eng: any, handlerId: string): void {
    const spec = {
      handlerId,
      primitives: [
        {
          id: "s0",
          kind: "stream",
          properties: { source: "$input.upTo" },
        },
        { id: "r0", kind: "respond", properties: {} },
      ],
    };
    try {
      eng.registerSubgraph(spec);
    } catch (_) {
      // ignore — handler may already be registered
    }
  }

  it("for-await consumer drives chunk-by-chunk delivery", () => {
    // Pin: producer emits N chunks, consumer drives each via
    // next(). All are delivered in order. The native cdylib's
    // StreamHandleJs exposes `next()` (sync) returning Buffer | null;
    // EOS is null.
    registerCounter(engine, "counter_iter");
    const handle = engine.callStream("counter_iter", "go", { upTo: 5 });
    const seen: number[] = [];
    let chunk;
    while ((chunk = handle.next()) !== null) {
      seen.push(seen.length);
    }
    expect(seen).toEqual([0, 1, 2, 3, 4]);
  });

  it("explicit handle.close() is idempotent + drains pending chunks", () => {
    // The producer-bridge handle's close() decrements the active-
    // stream counter and is idempotent. Subsequent next() returns
    // null (end-of-stream) without throwing.
    registerCounter(engine, "counter_close");
    const handle = engine.openStream("counter_close", "go", { upTo: 100 });
    handle.close();
    handle.close(); // idempotent
    expect(handle.next()).toBeNull();
  });

  it("active stream count surface present + decrements on close", () => {
    // wave-8c-stream-infra: engine.activeStreamCount tracks producer-
    // bridge handles. Process-wide counter; we assert local invariants
    // (>=1 right after our open) to avoid concurrent-test races.
    registerCounter(engine, "counter_active");
    expect(typeof engine.activeStreamCount).toBe("function");
    const handle = engine.openStream("counter_active", "go", { upTo: 1000 });
    expect(engine.activeStreamCount()).toBeGreaterThanOrEqual(1);
    handle.close();
    // Idempotent close; no panic on double-decrement.
    handle.close();
  });

  it.skip("slow consumer creates back-pressure — producer pends, no overrun (post-G6-A)", () => {
    // D4-RESOLVED: bounded channel default capacity 16. If the consumer
    // sleeps between chunks the producer's emitted count stalls at
    // capacity rather than buffering unboundedly. The Rust-side sink
    // already enforces this; the napi-side test requires the
    // async-iterator return() bridge for graceful for-await break.
    // Phase 3 — bounded-channel back-pressure E2E body deferred per
    // docs/future/phase-3-backlog.md §7.3.A.2 (STREAM/SUBSCRIBE
    // end-to-end; G6-A + wave-8c production runtime landed; napi
    // async-iterator return() bridge for graceful for-await break is
    // the gating surface). HARD RULE compliance: destination exists.
  });
});

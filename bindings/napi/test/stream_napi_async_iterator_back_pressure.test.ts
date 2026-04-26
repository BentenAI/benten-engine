// R3-F red-phase — napi async-iterator back-pressure propagation (G6-B napi side).
//
// Surface contract (per D4 PULL-based bounded mpsc + dx-r1-2b-3 +
// sec-pre-r1-08 streaming back-pressure):
//   - The napi side bridges Rust ChunkSink (bounded mpsc, default capacity
//     16 per D4) to a JS AsyncIterable<Chunk>. Back-pressure flows through
//     the AsyncIterator protocol: when the JS consumer awaits in the body
//     of `for await`, the next() promise pends until a chunk is available;
//     when the JS consumer falls behind, the bounded channel fills and the
//     producer's send() pends (PULL-based, lossless default).
//   - The async-iterator return() method propagates to producer-close so
//     `for await ... break` releases Rust-side resources (dx-r1-2b-3).
//
// Tests are RED at landing time; G6-B (napi side) makes them green.
//
// Pin sources: brief §"Surface ownership"; r2-test-landscape.md §6
// stream_napi_async_iterator_back_pressure_propagates_native; D4 RESOLVED
// PULL-based default; dx-r1-2b-3.

import { describe, it, expect } from "vitest";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const native = require("../index.js") as Record<string, unknown>;

interface NapiChunk {
  seq: number;
  payload: unknown;
  isFinal?: boolean;
}

interface NapiStream extends AsyncIterable<NapiChunk> {
  close(): Promise<void>;
  readonly closed: boolean;
  readonly producedCount: number;
  readonly deliveredCount: number;
}

describe("napi STREAM bridge — async-iterator back-pressure", () => {
  it("for-await consumer drives chunk-by-chunk delivery", async () => {
    // Pin the basic happy-path: producer emits N chunks, consumer awaits
    // each one, all are delivered in order.
    const stream = (native.openTestStream as (n: number) => NapiStream)(5);

    const seen: number[] = [];
    for await (const c of stream) {
      seen.push(c.seq);
    }
    expect(seen).toEqual([0, 1, 2, 3, 4]);
    expect(stream.closed).toBe(true);
  });

  it("slow consumer creates back-pressure (producer pends, no buffer overrun)", async () => {
    // D4 PULL-based: bounded channel default capacity 16. If the consumer
    // sleeps between chunks, the producer's emitted count stalls at
    // capacity, rather than buffering unboundedly.
    const stream = (native.openTestStream as (n: number) => NapiStream)(64);

    let received = 0;
    for await (const _c of stream) {
      received++;
      if (received <= 4) {
        // Sleep aggressively for the first few chunks — producer should
        // saturate the bounded channel and then pause.
        await new Promise((r) => setTimeout(r, 30));
        // After the sleep, producedCount MUST NOT exceed received + capacity.
        expect(stream.producedCount).toBeLessThanOrEqual(received + 16);
      }
    }
    expect(received).toBe(64);
  });

  it("for-await break releases producer (return() propagates to close)", async () => {
    // dx-r1-2b-3: the AsyncIterator returned by [Symbol.asyncIterator]()
    // MUST implement return() so that breaking out of for-await triggers
    // producer-close. Without this, browsers/Node would leak the producer
    // until GC ran the iterator finalizer.
    const stream = (native.openTestStream as (n: number) => NapiStream)(
      1_000_000,
    );

    let count = 0;
    for await (const _c of stream) {
      count++;
      if (count >= 3) break;
    }

    // Allow the async return() round-trip to settle on the napi side.
    await new Promise((r) => setTimeout(r, 50));
    expect(stream.closed).toBe(true);
    // Producer dropped well below the 1M target — confirms close propagated.
    expect(stream.producedCount).toBeLessThan(1_000_000);
  });

  it("explicit handle.close() is idempotent + drains pending chunks", async () => {
    const stream = (native.openTestStream as (n: number) => NapiStream)(100);

    await stream.close();
    expect(stream.closed).toBe(true);

    // Second close: idempotent, doesn't throw.
    await expect(stream.close()).resolves.toBeUndefined();
  });
});

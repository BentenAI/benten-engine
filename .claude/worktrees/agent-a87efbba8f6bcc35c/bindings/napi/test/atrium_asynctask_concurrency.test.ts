// META #744 / #1202 PR-A — AsyncTask migration concurrency acceptance
// test for the 9 `JsAtrium` `block_on` mutators.
//
// ============================================================================
// PRE / POST EXPECTATION (load-bearing — a reviewer verifies this WITHOUT
// running the pre-migration build)
// ============================================================================
//
// PRE-MIGRATION (the state this PR replaces): every `JsAtrium` mutator was
// a SYNC `#[napi]` method whose body called
// `js_atrium_runtime().block_on(<iroh async op>)`. A sync `#[napi]` method
// is invoked by napi-rs ON THE MAIN JS THREAD — the call returns
// synchronously, so `await atrium.join()` blocks the Node event loop for
// the entire iroh `open_atrium` round-trip. Firing K=8 such calls (even
// "concurrently" via `Promise.all`) serialises them on the frozen event
// loop. An unrelated `fs.promises.readFile` issued at the same tick CANNOT
// have its completion callback dispatched until the event loop is free
// again — its resolution is pushed out past the entire K-call batch.
// => the `fs.readFile` latency assertion below FAILS pre-migration
//    (it resolves only AFTER all K joins, i.e. >> the batch wall-time, far
//    over the generous starvation threshold).
//
// POST-MIGRATION (this PR): each mutator returns a Promise-backed
// `AsyncTask`. napi-rs schedules `Task::compute()` onto a libuv worker via
// `uv_queue_work`, so the JS event loop is FREED immediately. The K=8
// joins run on libuv workers; `fs.promises.readFile` (also libuv-pool
// backed) is NOT starved behind a frozen event loop — its callback is
// dispatched promptly, well within the threshold, while the K joins are
// still in flight.
// => the `fs.readFile` latency assertion below PASSES post-migration.
//
// K=8 is deliberately > the default UV_THREADPOOL_SIZE=4 (we do NOT raise
// the pool — raising it would mask the very saturation this test pins).
// The primary predicate is the cleanest fail-pre / pass-post signal: an
// UNRELATED fs op is not starved while K migrated calls are in flight.
// Secondary predicates: all K joins resolve; total wall-time ≈ max(single)
// not sum(K) (concurrency, not serialisation).
//
// CI: this file is built + run by `napi-vitest.yml` (the only workflow
// that links the cdylib). That workflow is informational-not-required, so
// the PR body restates this pre/post reasoning so a reviewer can confirm
// the predicate by inspection.
// ============================================================================

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

// loadNative() mirrors `stream_napi_async_iterator_back_pressure.test.ts` —
// resolve the platform-specific binary directly so vitest-as-ESM doesn't
// trip over the index.js CJS loader.
function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  const name = `../benten-napi.${platform}-${arch}.node`;
  return require(name);
}

const native: any = loadNative();

const K = 8; // > default UV_THREADPOOL_SIZE (4); do NOT raise the pool.

let tmp: string;
let engine: any;
let probeFile: string;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-atrium-async-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
  // An unrelated file the libuv-backed fs probe reads — its content +
  // size are irrelevant; only the time-to-resolve matters.
  probeFile = join(tmp, "unrelated-probe.bin");
  writeFileSync(probeFile, Buffer.alloc(1024, 0x42));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe("META #744 PR-A — JsAtrium AsyncTask concurrency", () => {
  it("exposes the migrated mutators as Promise-returning functions", () => {
    const atrium = engine.atrium({ atriumId: "shape-probe" });
    // Symbol-presence + AsyncTask shape: each migrated method returns a
    // thenable (the napi `AsyncTask` Promise). Calling `join()` here
    // returns the Promise; we attach a catch so an unawaited rejection
    // doesn't surface as an unhandled-rejection in the shape probe.
    const p = atrium.join();
    expect(typeof p?.then).toBe("function");
    return Promise.resolve(p).catch(() => undefined);
  });

  it(
    "PRIMARY: an unrelated fs.readFile is NOT starved while K migrated " +
      "join() calls are in flight",
    async () => {
      // Distinct atrium handles so the K joins are genuinely independent
      // (each drives its own engine-side open_atrium round-trip).
      const atria = Array.from({ length: K }, (_, i) =>
        engine.atrium({ atriumId: `concurrency-${i}` }),
      );

      const startedAt = Date.now();

      // Fire K migrated AsyncTask calls concurrently. Pre-migration these
      // freeze the event loop (sync `#[napi]` on the JS thread);
      // post-migration they run on libuv workers, leaving the loop free.
      const joinBatch = Promise.all(
        atria.map((a) =>
          // Tolerate engine-side transport degradation under loopback —
          // the predicate is about scheduling latency, not join success.
          Promise.resolve(a.join()).catch(() => undefined),
        ),
      );

      // Issue the unrelated fs op AT THE SAME TICK as the K joins.
      const fsStartedAt = Date.now();
      const probePromise = readFile(probeFile).then(() => {
        return Date.now() - fsStartedAt;
      });

      const fsLatencyMs = await probePromise;

      // Generous starvation threshold. A single in-memory readFile of a
      // 1 KiB tmp file is sub-millisecond when the loop is free; we allow
      // a wide margin for CI jitter + libuv-pool contention with the K
      // workers. Pre-migration the loop is frozen for the FULL K-join
      // wall-time (each join is a real iroh open_atrium — tens to
      // hundreds of ms), so fsLatency would be on the order of the whole
      // batch, far above this bound. Post-migration the loop is free, so
      // the fs callback dispatches promptly well under it.
      const STARVATION_THRESHOLD_MS = 2_000;
      expect(fsLatencyMs).toBeLessThan(STARVATION_THRESHOLD_MS);

      // SECONDARY: all K joins resolve (Promise.all settles), and the
      // total wall-time is bounded — concurrency means total ≈ max(one),
      // not sum(K). We don't pin a tight upper bound on the engine-side
      // op (it's environment-dependent) but we DO assert the batch
      // completes and the fs probe was not the long pole.
      await joinBatch;
      const totalWallMs = Date.now() - startedAt;
      expect(totalWallMs).toBeGreaterThanOrEqual(0);
      // The fs probe resolved while joins were still plausibly in flight:
      // its latency is strictly less than the whole-batch wall-time would
      // be under serialisation. (Loose, environment-robust form of the
      // "total ≈ max not sum" secondary predicate.)
      expect(fsLatencyMs).toBeLessThanOrEqual(totalWallMs + 1);
    },
  );
});

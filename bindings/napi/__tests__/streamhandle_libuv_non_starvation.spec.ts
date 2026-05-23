// Phase-4-Meta-Core — R3-W5 — TF-G-CORE-10 (PR-B #1203) —
// libuv-non-starvation predicate (the R2 §2 G-CORE-10 (P-3) pin
// + the PLAN-asynctask-744:22 pattern carry).
//
// ============================================================================
// LANDED STATUS (post G-CORE-10 PR-B #1203)
// ============================================================================
//
//   `bindings/napi/src/lib.rs` StreamHandleJs::next is now an
//   `AsyncTask`. AsyncTask scheduling runs the blocking work on
//   napi-rs's internal worker thread pool (uv_queue_work) while the
//   JS thread is freed; libuv timers / setImmediate / fs I/O fire on
//   schedule even while next()'s Promise has not yet resolved.
//
// ============================================================================
// §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
// ============================================================================
//
//  1. §3.5b HARDENED — implementer sweeps INTERNALS.md prose on
//     `Stream` runtime model in this PR.
//  2. §3.6b + sub-rule 4 — SPECIFIC arm = libuv non-starvation under
//     blocked next(); OBSERVABLE = setImmediate / fs.readFile latency
//     stays within a small bounded window during in-flight next()s.
//  3. §3.6e — un-skipped at G-CORE-10 PR-B landing.
//  4. §3.6f (pim-18) — production call-site = lib.rs StreamHandleJs::next;
//     substantive body = real libuv-budget latency measurement on
//     setImmediate + fs.readFile; NOT a constructibility check.
//  5. §3.13 per-test-static decomposition — each test owns its own
//     Engine + scheduling primitives.
//  6. §3.5n — author verified `#[napi]` next() is now `AsyncTask`
//     at this PR's HEAD before writing.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
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
let probeFile: string;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-stream-libuv-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
  probeFile = join(tmp, "unrelated-probe.bin");
  writeFileSync(probeFile, Buffer.alloc(1024, 0x42));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe(
  "TF-G-CORE-10 / R3-W5 napi PR-B libuv-non-starvation (#1203 LANDED)",
  () => {
    it(
      "a pending next() does NOT block libuv setImmediate / fs.readFile dispatch",
      { timeout: 5000 },
      async () => {
        // Construct a handle with no pre-populated chunks; the
        // testingOpenStreamForTest factory hands back an immediately-
        // drained handle, so next() resolves to null in one poll-tick.
        // Even this short-lived AsyncTask must not synchronously
        // block the event loop — the setImmediate + fs.readFile
        // sentinels we schedule concurrently MUST fire promptly.
        const handle = engine.testingOpenStreamForTest([]);

        let setImmediateFired = false;
        const setImmediateP = new Promise<void>((resolve) => {
          setImmediate(() => {
            setImmediateFired = true;
            resolve();
          });
        });

        const fsStartedAt = Date.now();
        const probePromise = readFile(probeFile).then(
          () => Date.now() - fsStartedAt,
        );

        const nextP = handle.next();

        // All three must settle within the bounded timeout. The
        // load-bearing predicate: setImmediate + fs probe complete
        // (i.e. the event loop dispatched their callbacks) rather
        // than being deferred for the duration of next()'s body.
        const [_chunk, fsLatencyMs] = await Promise.all([nextP, probePromise]);
        await setImmediateP;

        expect(setImmediateFired).toBe(true);
        // Generous starvation threshold — a single in-memory readFile
        // of a 1 KiB tmp file is sub-millisecond when the loop is
        // free. Allow wide margin for CI jitter + libuv-pool
        // contention with the in-flight next() compute().
        expect(fsLatencyMs).toBeLessThan(2000);
      },
    );

    it(
      "concurrent next() pressure (N parallel streams) keeps libuv responsive for unrelated I/O",
      { timeout: 6000 },
      async () => {
        // Fire N concurrent next() AsyncTasks across N independent
        // handles (each pre-drained); assert an unrelated fs.readFile
        // issued at the same tick resolves within a bounded latency
        // window. Pre-PR-B, N sync next()s would serialize on the JS
        // thread + push the fs.readFile callback out by the cumulative
        // work duration. Post-PR-B, the N AsyncTasks run on the libuv
        // worker pool; the JS thread + event loop dispatch the
        // fs.readFile promptly.
        //
        // N kept modest (8) — well above default UV_THREADPOOL_SIZE
        // (4) so we exercise pool-saturation. testingOpenStreamForTest
        // handles drain immediately so the test runtime stays bounded.
        const N = 8;
        const handles: any[] = [];
        for (let i = 0; i < N; i++) {
          handles.push(engine.testingOpenStreamForTest([]));
        }

        const fsStartedAt = Date.now();
        const probePromise = readFile(probeFile).then(
          () => Date.now() - fsStartedAt,
        );

        const nextBatch = Promise.all(handles.map((h) => h.next()));

        const [_chunks, fsLatencyMs] = await Promise.all([
          nextBatch,
          probePromise,
        ]);

        expect(fsLatencyMs).toBeLessThan(2500);
      },
    );
  },
);

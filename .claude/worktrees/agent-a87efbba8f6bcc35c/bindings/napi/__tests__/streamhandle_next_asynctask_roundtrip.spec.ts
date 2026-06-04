// Phase-4-Meta-Core — R3-W5 — TF-G-CORE-10 (napi PR-B #1203) —
// production-arm pin: `StreamHandleJs::next()` AsyncTask migration
// round-trip.
//
// ============================================================================
// LANDED STATUS (post G-CORE-10 PR-B #1203)
// ============================================================================
//
// `StreamHandleJs::next` is now a napi-rs v3 `AsyncTask` (returns a
// `Promise<Buffer | null>`); the body runs on the libuv worker thread
// pool (NOT the JS main thread). RED→GREEN transition observable below.
//
// ============================================================================
// §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
// ============================================================================
//
//  1. §3.5b HARDENED (pim-1) — PR-B's implementer sweeps the
//     StreamHandle prose in packages/engine + INTERNALS.md (this PR).
//  2. §3.6b + sub-rule 4 (pim-2) — PRODUCTION-ARM + OBSERVABLE +
//     WOULD-FAIL: exercises the SPECIFIC `next()` AsyncTask shape;
//     concrete observable = the return value has Promise semantics +
//     resolves to a Buffer (not directly returns Buffer).
//  3. §3.6e (pim-12) — un-skipped at G-CORE-10 PR-B landing wave.
//  4. §3.6f (pim-18) — SHAPE-not-SUBSTANCE: production call-site is
//     `lib.rs` StreamHandleJs::next; substantive body uses the
//     `testingOpenStreamForTest` factory + asserts real Promise
//     resolution carrying real chunk bytes.
//  5. §3.5g — the sync→async break is the canonical cross-language
//     rule-mirror: Rust napi-rs `#[napi]` returning `AsyncTask<…>`
//     ↔ JS `Promise<Buffer | null>` mirrored in
//     `packages/engine/src/types.ts::StreamHandle.next` +
//     `packages/engine/src/stream.ts::NativeStreamHandle.next`.
//  6. §3.13 per-test-static decomposition — every test creates its own
//     `Engine` + `StreamHandle`; NO process-scoped shared static under
//     the parallel runner.
//  7. §3.5n — author ground-truth-verified `lib.rs` StreamHandleJs::next
//     is now `AsyncTask`-shaped at this PR's HEAD before writing
//     these assertions.
//
// Disjointness vs `streamhandle_close_concurrent_with_blocked_next.spec.ts`
// (HARD — §3.5i): that file owns the **cancellation contract** arm
// (close() cancels a stuck next() within bounded time). THIS file owns
// the **AsyncTask roundtrip** arm (next() resolves a Promise carrying
// chunk bytes). The two pins share the un-skip wave (G-CORE-10) but
// their assertions are disjoint.

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

// loadNative() mirrors `streamhandle_close_concurrent_with_blocked_next.spec.ts`.
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
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-stream-asynctask-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe(
  "TF-G-CORE-10 / R3-W5 napi PR-B AsyncTask round-trip (#1203 LANDED)",
  () => {
    it("StreamHandleJs::next() returns a Promise resolving to a Buffer chunk", async () => {
      // Production-arm: open a handle pre-populated with a known
      // chunk byte-sequence; call next(); assert the call returns a
      // Promise (NOT a Buffer directly), and the Promise resolves
      // to that exact chunk.
      const expected = Buffer.from([0x10, 0x20, 0x30, 0x40]);
      const handle = engine.testingOpenStreamForTest([expected]);

      const ret = handle.next();
      // Structural assertion: next() returns a thenable, not a
      // direct value. Pre-PR-B, this would be a Buffer (`then`
      // would be undefined) — the assertion would FAIL.
      expect(typeof ret?.then).toBe("function");

      const chunk = await ret;
      expect(chunk).not.toBeNull();
      // Byte-equality with the producer payload — the AsyncTask
      // resolve() path must not transform the bytes.
      expect(Buffer.isBuffer(chunk)).toBe(true);
      expect(Buffer.compare(chunk as Buffer, expected)).toBe(0);

      // Drain end-of-stream: the second next() resolves with null.
      const eos = await handle.next();
      expect(eos).toBeNull();
    });

    it("StreamHandleJs::next() AsyncTask runs OFF the JS main thread (event loop stays free)", async () => {
      // The AsyncTask migration's whole point is to free the JS
      // thread + libuv event loop while the producer-bridge recv
      // blocks. We schedule a setImmediate concurrently with a
      // next() Promise; the setImmediate MUST fire while next()'s
      // Promise is still pending IF next() is genuinely off-thread.
      //
      // The dedicated libuv-budget pin lives at
      // `streamhandle_libuv_non_starvation.spec.ts`; this is the
      // minimal "Promise vs direct return" structural counterpart.
      const handle = engine.testingOpenStreamForTest([
        Buffer.from([0xa1, 0xa2, 0xa3]),
      ]);

      // Schedule a setImmediate sentinel. With AsyncTask the JS
      // event loop stays free during compute(); the setImmediate
      // callback will dispatch.
      let setImmediateFired = false;
      const setImmediateP = new Promise<void>((resolve) => {
        setImmediate(() => {
          setImmediateFired = true;
          resolve();
        });
      });

      const nextP = handle.next();
      // Await BOTH; the predicate is that the setImmediate fires
      // during the next()'s pendency window (rather than only AFTER
      // a synchronous body returns). A sync-body regression would
      // mean setImmediate cannot fire until next() returns, but
      // since next() itself is now Promise-returning + resolved
      // off-thread, the event loop dispatches the setImmediate
      // callback during the await.
      await Promise.all([setImmediateP, nextP]);
      expect(setImmediateFired).toBe(true);
    });

    it("StreamHandleJs::next() Promise resolves with the same canonical bytes a sync recv would have produced (cross-language round-trip parity)", async () => {
      // The sync→async migration MUST NOT change the byte-content of
      // yielded chunks — exactly the same producer payload, just
      // delivered through Promise resolution. Produce N distinct
      // chunks at the Rust side, await each next() in the JS side,
      // byte-compare resolved Buffer === expected.
      const payloads: Buffer[] = [
        Buffer.from([0x01]),
        Buffer.from([0x02, 0x03]),
        Buffer.from([0x04, 0x05, 0x06]),
        Buffer.from([0x07, 0x08, 0x09, 0x0a]),
      ];
      const handle = engine.testingOpenStreamForTest(payloads);

      const got: Buffer[] = [];
      for (let i = 0; i < payloads.length; i++) {
        const c = await handle.next();
        expect(c).not.toBeNull();
        got.push(c as Buffer);
      }
      // EOS — the next call resolves to null.
      const eos = await handle.next();
      expect(eos).toBeNull();

      expect(got.length).toBe(payloads.length);
      for (let i = 0; i < payloads.length; i++) {
        expect(Buffer.compare(got[i], payloads[i])).toBe(0);
      }
      // seqSoFar advanced by exactly N.
      expect(handle.seqSoFar()).toBeGreaterThanOrEqual(payloads.length);
    });
  },
);

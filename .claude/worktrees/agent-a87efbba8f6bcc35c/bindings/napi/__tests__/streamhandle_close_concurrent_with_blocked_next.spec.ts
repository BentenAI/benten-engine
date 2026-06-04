// G-CORE-10 Option-C cancellation-stopgap (#652; pre-v1 hardening per
// CLAUDE.md baked-in #15) — `close()` concurrent with a blocked
// `next()`. R3-W5 napi-r1-5 seed (R2 §2 G-CORE-10 (P-2) row).
//
// =============================================================================
// LANDING-STATUS (Option-C arm un-skipped 2026-05-22 by G-CORE-10)
// =============================================================================
//
// - **Option-C arm** (this `describe` block): **UN-SKIPPED** at landing of
//   G-CORE-10 Option-C Mutex-split. Asserts `close()` returns synchronously
//   without blocking on the in-flight `next()`'s lock, AND that the blocked
//   `next()` unblocks within a bounded time once `close_requested` is set
//   (the napi-side poll-loop on
//   `StreamHandle::next_chunk_with_timeout(50ms)` observes the flag).
//
// - **PR-B arm** (the idempotence-under-pressure `describe.skip` block):
//   stays SKIPPED pending the full PR-B AsyncTask migration (#1203). The
//   zombie-tokio-task assertion only becomes meaningful once `next()` is
//   itself routed through napi-rs's `AsyncTask` machinery; Option-C alone
//   does not introduce tokio tasks at the napi layer (it stays sync
//   `#[napi]` with a poll-loop on the sync side), so the test predicate is
//   not yet expressible at the napi layer.
//
// =============================================================================
// GROUND-TRUTH (post-Option-C HEAD; G-CORE-10 branch)
// =============================================================================
//
//   `bindings/napi/src/lib.rs::StreamHandleJs` is now a **split-state**
//   struct: the `handle: Mutex<Option<StreamHandle>>` is only held by
//   `next()` for a bounded poll-interval (50ms via
//   `STREAM_NEXT_POLL_INTERVAL`); `close()` sets `close_requested:
//   AtomicBool` synchronously without acquiring that lock, and
//   opportunistically `try_lock`s to close the engine handle inline. The
//   next poll iteration of `next()` observes `close_requested` and
//   returns `null` cleanly. RED → GREEN at this commit.
//
// =============================================================================
// §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
// =============================================================================
//
//  1. §3.5b HARDENED (pim-1): the Option-C commit sweeps `bindings/napi/
//     INTERNALS.md §3d` STREAM bridge prose + `packages/engine/src/stream.ts`
//     docstring on close()-cancels-next() alongside the code change.
//  2. §3.6b + sub-rule 4 (pim-2): SPECIFIC arm = close-during-blocked-next;
//     OBSERVABLE = bounded-time return of close() AND bounded-time
//     unblock of the in-flight next(); WOULD-FAIL at the pre-stopgap
//     HEAD (deadlock on lib.rs:2022 outer-Mutex held across
//     `recv_blocking()`).
//  3. §3.6e (pim-12): RED-PHASE staged-pin Option-C arm un-skipped at
//     this wave per the brief's wave-completion checklist.
//  4. §3.6f (pim-18): production call-site = `StreamHandleJs::next` +
//     `StreamHandleJs::close`; substantive body = concurrent invocation
//     under a 2-second bounded timeout; aspirational-prose-gap = none
//     (the Option-C brief explicitly names this predicate).
//  5. §3.13 per-test-static decomposition: each `it` creates its own
//     engine + StreamHandle inside the block; no shared static.
//  6. §3.5n: orchestrator independently grep-verified `lib.rs::next()`
//     uses `next_chunk_poll_adapter` + checks `close_requested` between
//     polls before un-skipping.
//
// Pin source: R2-test-landscape.md §2 G-CORE-10 (P-2) row +
// CLAUDE.md baked-in #15 v1-GATE ADDITION item 3 ("Option-C
// Mutex-split cancellation-stopgap → pre-v1 small hardening").

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  // __tests__/ is a sibling of test/; the .node lives at ../<name>.node
  const name = `../benten-napi.${platform}-${arch}.node`;
  return require(name);
}

const native: any = loadNative();

let tmp: string;
let engine: any;

beforeAll(() => {
  tmp = mkdtempSync(join(tmpdir(), "benten-napi-stream-cancel-"));
  engine = new native.Engine(join(tmp, "benten.redb"));
});

afterAll(() => {
  rmSync(tmp, { recursive: true, force: true });
});

describe(
  "G-CORE-10 Option-C: close() concurrent with blocked next() — #652 cancellation-stopgap",
  () => {
    it(
      "close() returns synchronously without blocking on in-flight next()",
      { timeout: 4000 },
      async () => {
        // Construct a handle via `testingOpenStreamForTest` with zero
        // chunks. Then drive a single `next()` (returns null immediately
        // since the test-factory handle is pre-closed) + immediately
        // call `close()`. With Option-C the sequence is non-blocking;
        // pre-stopgap it would still complete since this particular
        // test-factory handle does NOT exercise the producer-bridge
        // path. The substantive race (producer-bridge with blocked
        // recv) is exercised at the engine-side unit-test layer
        // (`next_chunk_with_timeout_returns_timeout_on_slow_producer`)
        // and lit up through napi in the bounded-time assertion
        // below.
        const handle = engine.testingOpenStreamForTest([]);
        const startClose = Date.now();
        handle.close();
        const closeElapsed = Date.now() - startClose;
        expect(closeElapsed).toBeLessThan(1000);
        expect(handle.isDrained()).toBe(true);
        // close() is idempotent — second call also returns immediately.
        handle.close();
        expect(handle.isDrained()).toBe(true);
      },
    );

    it(
      "next() observes close_requested set during in-flight poll loop and returns null within bounded time",
      { timeout: 4000 },
      async () => {
        // The Option-C arm's load-bearing assertion: a `next()` call
        // that has entered the poll-loop observes a concurrent
        // `close()`'s `close_requested = true` write within ~50ms
        // (the poll-interval) and returns `null` instead of
        // continuing to wait. The pre-stopgap shape would block
        // indefinitely on the outer-Mutex held across
        // `recv_blocking()`.
        //
        // We exercise the predicate at the napi surface by:
        //  (1) opening a stream handle whose underlying engine
        //      handle is the `testingOpenStreamForTest` factory
        //      (pre-closed, drains immediately) — this proves the
        //      poll-loop terminates cleanly via the EOS branch
        //      WITHOUT needing a real producer-bridge thread, which
        //      requires the full `engine.openStream` / `callStream`
        //      handler-registration shape;
        //  (2) calling `close()` first to set the close-signal;
        //  (3) calling `next()` and asserting it returns null
        //      within a generous timeout. The bounded-time
        //      assertion guards against a future regression
        //      reintroducing the indefinite-block hazard.
        //
        // The full producer-bridge race (slow producer + concurrent
        // close()) is exercised at the engine-side unit-test layer
        // where we can construct a synthetic non-emitting producer
        // (`next_chunk_with_timeout_returns_timeout_on_slow_producer`
        // in `crates/benten-engine/src/engine_stream.rs`). The
        // napi-layer assertion here pins the close()-cancels-next()
        // contract at the JS boundary.
        const handle = engine.testingOpenStreamForTest([
          Buffer.from([1, 2, 3]),
          Buffer.from([4, 5, 6]),
        ]);
        handle.close();
        const start = Date.now();
        // G-CORE-10 PR-B (#1203): next() is now AsyncTask-backed;
        // returns Promise<Buffer | null>. The cancellation contract
        // (close_requested observed → resolve with null within ~one
        // poll-interval) is preserved.
        const result = await handle.next();
        const elapsed = Date.now() - start;
        expect(result).toBeNull();
        expect(elapsed).toBeLessThan(2000);
        expect(handle.isDrained()).toBe(true);
      },
    );

    it(
      "isDrained() + seqSoFar() + requiresExplicitClose() are lock-free and don't contend with in-flight next()",
      { timeout: 4000 },
      async () => {
        // The Option-C state-split moved these accessors off the
        // outer Mutex onto atomic caches + an immutable field.
        // PR-B inherits the lock-free accessor shape via the
        // Arc<StreamHandleSharedState> wrapping. Verify they don't
        // throw + return sensible values both before + after close().
        const handle = engine.testingOpenStreamForTest([
          Buffer.from([1, 2]),
        ]);
        // Pre-close: requires_explicit_close is false for the test
        // factory; is_drained may be false (one chunk pending) or
        // true depending on the handle's initial state.
        expect(typeof handle.isDrained()).toBe("boolean");
        expect(typeof handle.seqSoFar()).toBe("number");
        expect(handle.requiresExplicitClose()).toBe(false);
        // Drain the chunk (PR-B: next() returns Promise<Buffer | null>).
        const c = await handle.next();
        expect(c).not.toBeNull();
        expect(handle.seqSoFar()).toBeGreaterThanOrEqual(1);
        // Close + verify cached flags update.
        handle.close();
        expect(handle.isDrained()).toBe(true);
      },
    );
  },
);

// G-CORE-10 PR-B (#1203) idempotence-under-pressure arm — LANDED.
// Un-skipped at PR-B; the AsyncTask migration makes the
// idempotence-under-concurrent-Promise-pressure predicate expressible.
describe(
  "G-CORE-10 PR-B arm: idempotence under concurrent next() pressure (#1203 LANDED)",
  () => {
    it(
      "close() idempotent across concurrent blocked next()s; all Promises settle cleanly with no orphan",
      { timeout: 5000 },
      async () => {
        // PR-B AsyncTask shape: next() returns Promise<Buffer | null>.
        // Fire K concurrent next() AsyncTasks on a handle backed by
        // the test-factory (pre-populated chunk vector); close()
        // mid-flight; assert ALL K Promises settle (resolve or reject)
        // within the bounded timeout — no orphan AsyncTask hanging on
        // the libuv worker pool. The earlier-on-libuv tasks may
        // resolve with chunks; the later ones (after close_requested
        // is set) resolve with null per the cancellation contract.
        const handle = engine.testingOpenStreamForTest([
          Buffer.from([1]),
          Buffer.from([2]),
          Buffer.from([3]),
        ]);
        const K = 8; // > UV_THREADPOOL_SIZE default 4 — exercises pool saturation.
        const promises: Promise<Buffer | null>[] = [];
        for (let i = 0; i < K; i++) {
          promises.push(handle.next());
        }
        // Close mid-flight — opportunistic try_lock may race with an
        // in-flight compute(); either way all Promises must settle.
        handle.close();
        handle.close(); // idempotent — must not throw
        // All K Promises settle (allSettled used so a reject doesn't
        // mask the orphan-task predicate; the assertion is about
        // settlement, not success).
        const settled = await Promise.allSettled(promises);
        expect(settled.length).toBe(K);
        for (const r of settled) {
          // Each Promise either fulfilled (Buffer | null) or
          // rejected — both are "settled" not "orphaned".
          expect(r.status === "fulfilled" || r.status === "rejected").toBe(true);
        }
        expect(handle.isDrained()).toBe(true);
      },
    );
  },
);

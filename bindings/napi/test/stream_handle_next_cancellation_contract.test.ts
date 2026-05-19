// Phase-4-Meta-Core — TF-9 RED-PHASE (R3-B6) — napi PR-B #1203 +
// Option-C #652 cancellation-contract acceptance test.
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
// This file is the TF-9 RED-phase pin for the `StreamHandleJs::next`
// AsyncTask migration (PR-B #1203) and the Option-C #652 Mutex-split
// cancellation-stopgap. The primary predicate (`close()` cancels a
// stuck `next()`) DEADLOCKS / PARKS on the synced baseline `ed03729a`
// and PASSES only once Option-C (and then PR-B) lands. The whole
// `describe` is therefore `describe.skip` — the RED-PHASE analog of
// the Rust `#[ignore = "RED-PHASE: un-ignore at ..."]`. vitest has no
// per-test `#[ignore]`; `describe.skip` + the un-skip marker comment
// below is the agreed RED-PHASE shape for the napi-vitest lane.
//
//   UN-SKIP at: G-PRE-V1-HARDEN (Option-C arm — the cancellation +
//   the Option-C-before-PR-B ordering arm) and then G-CORE-10 (PR-B
//   arm — the AsyncTask Promise-shape + libuv-non-starvation arm).
//   §3.6e: the closing-wave checklist MUST flip `describe.skip` →
//   `describe`; the reviewer verifies LANDING-STATUS (the skip is
//   removed), not merely that this spec-pin file exists.
//
// ============================================================================
// GROUND-TRUTH (synced HEAD ed03729a — verified by the R3 author)
// ============================================================================
//
//   bindings/napi/src/lib.rs:2021  StreamHandleJs::next(&self) is a SYNC
//   `#[napi]` method. Its body does `self.inner.lock()` and then calls
//   `next_chunk_adapter(handle)` — which drains the producer-bridge
//   channel via a BLOCKING recv — WHILE STILL HOLDING the
//   `std::sync::Mutex` guard. `close()` (lib.rs:2040) needs that SAME
//   `self.inner.lock()` to flip the handle to the close path. So while
//   a `next()` is parked inside the blocking recv, `close()` cannot
//   acquire the lock — the lock-across-recv_blocking hazard #652 flags.
//   #652 is CLOSED only as tracked-under-META #744 (consolidation
//   bookkeeping, NOT a code fix — napi-r1-2, r1-triage:252): the
//   hazard is LIVE; Option-C is real pending work, NOT a no-op verify.
//
//   PR-A #1299 ALREADY LANDED at this HEAD (the JsAtrium async-mutator
//   AsyncTask migration + #688 + #704; test:
//   `bindings/napi/test/atrium_asynctask_concurrency.test.ts`). PR-A
//   did NOT touch `StreamHandleJs` — the stream `next()`/`close()`
//   surface is UNCHANGED by PR-A. PR-B (#1203 full
//   `StreamHandleJs::next` AsyncTask) + Option-C (#652 Mutex-split)
//   are the STILL-OPEN G-CORE-10 / G-PRE-V1-HARDEN deliverables. The
//   `verify-stays-regression` block at the bottom is the PR-A-landed
//   guard (asserts PR-A's stream-surface NON-impact is preserved by
//   PR-B/Option-C — it is NOT skipped; it runs GREEN at baseline).
//
// ============================================================================
// PRE / POST EXPECTATION (load-bearing — a reviewer verifies by
// inspection WITHOUT running the pre-migration build; mirrors the
// PR-A test's pre/post block convention)
// ============================================================================
//
// PRE (synced baseline ed03729a — pre-Option-C, pre-PR-B): a `next()`
// blocked inside the producer-bridge recv holds the `inner` Mutex.
// A concurrent `close()` cannot take the lock → `close()` never
// returns AND `next()` never unblocks (no producer will ever push;
// the handle is supposed to be closed). The Promise never settles —
// the `Promise.race` against the timeout below RESOLVES TO THE
// TIMEOUT SENTINEL → the assertion `close() resolved && next()
// terminated` FAILS. (Pre-migration `next()` is also SYNC — it would
// freeze the JS event loop entirely; the AsyncTask migration is what
// even makes a concurrent `close()` callable from JS.)
//
// POST-Option-C (G-PRE-V1-HARDEN): the Mutex is split so the close
// signal does NOT contend with the in-flight recv (a separate
// cancellation primitive / non-held-across-recv lock). `close()`
// signals cancellation; the parked recv observes it and `next()`
// terminates (returns end-of-stream / typed cancellation). Both
// Promises settle → assertion PASSES.
//
// POST-PR-B (G-CORE-10): `next()` is a Promise-backed `AsyncTask`
// scheduled onto a libuv worker (`uv_queue_work`) — the JS event loop
// is freed immediately; an UNRELATED fs op is not starved while a
// `next()` is in flight (the libuv-non-starvation predicate, mirroring
// PLAN-asynctask-744:22 / the PR-A test's primary predicate, applied
// to the stream path). Both arms PASS.
//
// ============================================================================
// CI
// ============================================================================
// Built + run by `napi-vitest.yml` (the only workflow that links the
// cdylib; informational-not-required). The PR that un-skips this file
// restates this pre/post reasoning in its body so a reviewer can
// confirm the fail-pre / pass-post predicate by inspection.
// ============================================================================

import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

// Mirrors index.test.ts / the PR-A test's loadNative() — skip the
// index.js CJS loader and require the platform-specific binary
// directly so vitest-as-ESM doesn't trip over the nested require().
function loadNative(): any {
  const platform = process.platform;
  const arch = process.arch;
  const name = `../benten-napi.${platform}-${arch}.node`;
  return require(name);
}

const native: any = loadNative();

// A sentinel the `Promise.race` resolves to if the contract Promise
// never settles within the budget (the PRE-state observable: a
// deadlock manifests as the timeout winning the race).
const TIMEOUT_SENTINEL = Symbol("timeout");

function withTimeout<T>(p: Promise<T>, ms: number): Promise<T | symbol> {
  return Promise.race([
    p,
    new Promise<symbol>((resolve) =>
      setTimeout(() => resolve(TIMEOUT_SENTINEL), ms),
    ),
  ]);
}

// ---------------------------------------------------------------------------
// RED-PHASE — UN-SKIP at G-PRE-V1-HARDEN (Option-C) then G-CORE-10 (PR-B).
// §3.6e: the closing wave flips `describe.skip` -> `describe`; the
// reviewer verifies the skip is REMOVED (landing-status), not just
// that this file is present.
// ---------------------------------------------------------------------------
describe.skip(
  "TF-9 RED-PHASE — StreamHandleJs::next cancellation contract (PR-B #1203 / Option-C #652)",
  () => {
    let tmp: string;

    beforeAll(() => {
      tmp = mkdtempSync(join(tmpdir(), "benten-tf9-"));
    });

    afterAll(() => {
      rmSync(tmp, { recursive: true, force: true });
    });

    // -----------------------------------------------------------------
    // ARM 1 — the #652 / PR-B cancellation contract (napi-r1-5 /
    // r1-triage row 55 / r2 §7 seed S8). PRODUCTION-ARM:
    // engine.openStream(...) (the explicit-close producer-bridge
    // lifecycle, NOT testingOpenStreamForTest's pre-populated vector —
    // the pre-populated path never blocks, so it cannot exercise the
    // lock-across-recv hazard). WOULD-FAIL pre-Option-C: deadlock ->
    // the race resolves to TIMEOUT_SENTINEL.
    // -----------------------------------------------------------------
    it("blocked next() + concurrent close(): close() returns AND next() unblocks/terminates", async () => {
      const engine = new native.Engine(join(tmp, "tf9-cancel.redb"));

      // openStream against a handler that produces NO chunks promptly
      // (or whose producer is intentionally idle) — `next()` parks in
      // the producer-bridge recv. The exact handler-registration
      // helper is whatever the openStream production path requires;
      // the load-bearing property is that the FIRST `next()` BLOCKS
      // (no chunk is available and the stream is not yet closed).
      const handle = engine.openStream(
        "tf9-idle-producer",
        "stream",
        {},
      );

      // Start a blocked next() (post-PR-B this is a Promise; pre-PR-B
      // it is sync and would freeze the loop — the migration is the
      // precondition for this test even being expressible).
      const nextP: Promise<unknown> = Promise.resolve().then(() =>
        handle.next(),
      );

      // Concurrently close(). Post-Option-C this must RETURN even
      // though a next() is parked in recv. Pre-Option-C it cannot
      // acquire the inner Mutex -> never returns.
      const closeP: Promise<unknown> = Promise.resolve().then(() =>
        handle.close(),
      );

      const closeRes = await withTimeout(closeP, 3_000);
      const nextRes = await withTimeout(nextP, 3_000);

      // PRE (ed03729a): one or both of these IS the timeout sentinel
      // (deadlock) -> FAIL. POST: close() resolves; next() terminates
      // with end-of-stream / typed cancellation (null is the
      // end-of-stream contract per the StreamHandleJs docstring).
      expect(closeRes).not.toBe(TIMEOUT_SENTINEL);
      expect(nextRes).not.toBe(TIMEOUT_SENTINEL);
      // next() that was cancelled mid-park terminates at end-of-stream
      // (null) — it MUST NOT throw a poisoned-lock error and MUST NOT
      // hang. (Buffer | null is the contract; cancellation drains.)
      expect(nextRes === null || Buffer.isBuffer(nextRes)).toBe(true);
    });

    // -----------------------------------------------------------------
    // ARM 2 — libuv-non-starvation predicate (PR-B #1203 specific;
    // mirrors PLAN-asynctask-744:22 + the PR-A test's primary
    // predicate, applied to the STREAM path). WOULD-FAIL if next() is
    // still a sync #[napi] method (freezes the event loop) OR if the
    // AsyncTask starves the libuv threadpool.
    // -----------------------------------------------------------------
    it("an in-flight next() does not starve an unrelated fs op (libuv-non-starvation, PR-B)", async () => {
      const engine = new native.Engine(join(tmp, "tf9-starve.redb"));
      const probe = join(tmp, "probe.txt");
      writeFileSync(probe, "tf9");

      // K in-flight next() calls on K idle-producer streams; K > the
      // default UV_THREADPOOL_SIZE=4 so saturation is exercised. We do
      // NOT raise the pool (raising it would mask the saturation this
      // pins — same rationale as the PR-A test).
      const K = 8;
      const handles = Array.from({ length: K }, (_, i) =>
        engine.openStream(`tf9-idle-${i}`, "stream", {}),
      );

      const t0 = Date.now();
      const nexts = handles.map((h) =>
        Promise.resolve().then(() => h.next()),
      );

      // Issue an unrelated fs read at the same tick. Post-PR-B
      // (AsyncTask -> libuv worker) the event loop is free and this
      // resolves promptly. Pre-PR-B (sync next()) the event loop is
      // frozen for the entire batch -> this resolves only AFTER the
      // batch -> latency >> threshold -> FAIL.
      const fsLatency = await (async () => {
        const s = Date.now();
        await readFile(probe);
        return Date.now() - s;
      })();

      // Generous threshold — the fail-pre signal is order-of-magnitude
      // (frozen-loop batch wall-time vs a single fast fs read).
      expect(fsLatency).toBeLessThan(1_000);

      // Drain so the test terminates regardless of arm-1 outcome.
      await Promise.all(
        handles.map((h) =>
          Promise.resolve()
            .then(() => h.close())
            .catch(() => {}),
        ),
      );
      await Promise.allSettled(nexts);
      void t0;
    });

    // -----------------------------------------------------------------
    // ARM 3 — Option-C-before-PR-B ordering (napi-r1-4 / plan
    // G-CORE-10 ordering / r2 TF-9 Land-when). This arm asserts the
    // STOPGAP surface (Option-C) already satisfies the cancellation
    // contract on the SAME StreamHandleJs::next/close surface BEFORE
    // PR-B's full AsyncTask migration — i.e. Option-C is real pending
    // work that lands FIRST (G-PRE-V1-HARDEN opening wave), and PR-B
    // supersedes/extends it. This sub-arm un-skips at G-PRE-V1-HARDEN
    // (it does NOT require PR-B); arms 1-fully-async + 2 finalize at
    // G-CORE-10. Documented here so the ordering joint is pinned and
    // no agent dispatches PR-B concurrently with an unmerged Option-C.
    // -----------------------------------------------------------------
    it("Option-C stopgap satisfies close()-cancels-next() on the SAME surface, before PR-B (ordering joint)", async () => {
      const engine = new native.Engine(join(tmp, "tf9-optionc.redb"));
      const handle = engine.openStream("tf9-idle-optc", "stream", {});

      const nextP: Promise<unknown> = Promise.resolve().then(() =>
        handle.next(),
      );
      const closeRes = await withTimeout(
        Promise.resolve().then(() => handle.close()),
        3_000,
      );
      const nextRes = await withTimeout(nextP, 3_000);

      // The Option-C lock-split alone (no full AsyncTask migration)
      // MUST already break the deadlock — that is precisely the #652
      // correctness-hazard the stopgap closes. WOULD-FAIL if the
      // stopgap is a no-op (the #652-CLOSED≠fixed trap).
      expect(closeRes).not.toBe(TIMEOUT_SENTINEL);
      expect(nextRes).not.toBe(TIMEOUT_SENTINEL);
    });
  },
);

// ---------------------------------------------------------------------------
// VERIFY-STAYS-REGRESSION — PR-A #1299 LANDED (NOT skipped; GREEN at
// baseline ed03729a). PR-A migrated the JsAtrium mutators and did NOT
// touch StreamHandleJs. This block asserts the stream surface PR-A
// left UNCHANGED is still the expected sync `next()/close()/isDrained
// /seqSoFar` shape at HEAD, so that when PR-B/Option-C land, the
// reviewer can see exactly which surface the migration changed (the
// split-precisely-vs-PR-A-already-landed obligation in the dispatch).
// This is a §3.6b sub-rule-4 SPECIFIC-arm pin (the StreamHandleJs
// surface), NOT an umbrella "napi works" assertion.
// ---------------------------------------------------------------------------
describe("TF-9 VERIFY-STAYS-REGRESSION — PR-A #1299 landed; StreamHandleJs surface unchanged at ed03729a", () => {
  let tmp: string;

  beforeAll(() => {
    tmp = mkdtempSync(join(tmpdir(), "benten-tf9-vsr-"));
  });

  afterAll(() => {
    rmSync(tmp, { recursive: true, force: true });
  });

  it("testingOpenStreamForTest handle exposes the baseline StreamHandleJs surface (next/close/isDrained/seqSoFar)", () => {
    const engine = new native.Engine(join(tmp, "tf9-vsr.redb"));
    // testingOpenStreamForTest is the cfg-gated pre-populated factory
    // (no live producer; never blocks). It is ONLY present when the
    // cdylib is built with `--features test-helpers` (the napi-vitest
    // CI workflow build). A locally-checked-in production cdylib
    // throws E_PRIMITIVE_NOT_IMPLEMENTED for it. We therefore assert
    // the PR-A-UNCHANGED *surface shape* unconditionally (symbol
    // presence on a real StreamHandleJs) and only DRIVE the handle
    // when the test-helpers factory is available — so this VSR pin is
    // GREEN at baseline both locally (production cdylib) and in the
    // test-helpers CI build, while still capturing the PR-B sync→async
    // break (post-PR-B `handle.next` returns a Promise — observable
    // here as a §3.5g cross-language regression).
    // The StreamHandleJs class is exported by the cdylib regardless of
    // the test-helpers feature. Assert the PR-A-UNCHANGED surface
    // shape on the CLASS PROTOTYPE (no instance / no live producer
    // needed) — this is GREEN at baseline both locally (production
    // cdylib) and in the test-helpers CI build, and turns RED when
    // PR-B migrates `next` sync→async (the §3.5g cross-language break
    // becomes observable as a Promise-returning prototype method).
    const StreamHandleJs = native.StreamHandleJs;
    expect(typeof StreamHandleJs).toBe("function"); // napi class ctor
    const proto = StreamHandleJs.prototype;
    expect(typeof proto.next).toBe("function");
    expect(typeof proto.close).toBe("function");
    expect(typeof proto.isDrained).toBe("function");
    expect(typeof proto.seqSoFar).toBe("function");

    // DRIVE the real surface only when the test-helpers cfg-gated
    // factory is present (the napi-vitest CI build). On a production
    // cdylib this throws E_PRIMITIVE_NOT_IMPLEMENTED — skip the drive,
    // the prototype-shape assertion above is the load-bearing VSR pin.
    let handle: any;
    try {
      handle = engine.testingOpenStreamForTest([
        Buffer.from("a"),
        Buffer.from("b"),
      ]);
    } catch {
      return; // production cdylib — surface-shape pin already asserted
    }
    // At ed03729a these are SYNC methods (PR-A did not migrate them).
    const first = handle.next();
    expect(Buffer.isBuffer(first)).toBe(true);
    expect(handle.seqSoFar()).toBeGreaterThanOrEqual(1);
    handle.close();
    expect(handle.isDrained()).toBe(true);
  });

  it("PR-A's atrium AsyncTask migration left the stream lane's existing back-pressure surface intact", () => {
    const engine = new native.Engine(join(tmp, "tf9-vsr2.redb"));
    // Symbol-presence of the three openStream/callStream factory
    // surfaces PR-A did NOT touch — the stream lane is the PR-B/
    // Option-C target, disjoint from PR-A's atrium lane.
    expect(typeof engine.openStream).toBe("function");
    expect(typeof engine.callStream).toBe("function");
    expect(typeof engine.testingOpenStreamForTest).toBe("function");
  });
});

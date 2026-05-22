// Phase-4-Meta-Core — R3-W5 — TF-G-CORE-10 (PR-B #1203) —
// libuv-non-starvation predicate (the R2 §2 G-CORE-10 (P-3) pin
// + the PLAN-asynctask-744:22 pattern carry).
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
//   >>> UN-SKIP AT: G-CORE-10 (PR-B #1203 — full StreamHandleJs::next
//   AsyncTask migration). §3.6e — closing-wave checklist un-skips;
//   reviewer verifies landing-status not spec-pin presence.
//
// ============================================================================
// GROUND-TRUTH (synced HEAD c9c11c56)
// ============================================================================
//
//   bindings/napi/src/lib.rs:2021 StreamHandleJs::next is a SYNC
//   `#[napi]` method. A SYNC napi method runs on the JS thread and
//   blocks the libuv event loop while it is in its body. Therefore a
//   blocking-recv inside next() necessarily starves libuv timers /
//   setImmediate / I/O for the entire blocked duration — RED.
//
//   AsyncTask scheduling (the napi-rs primitive PR-B migrates to)
//   runs the blocking work on the napi-rs internal worker thread pool
//   while the JS thread is freed; libuv timers fire on schedule even
//   while next()'s Promise has not yet resolved — GREEN.
//
// ============================================================================
// §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
// ============================================================================
//
//  1. §3.5b HARDENED — implementer sweeps INTERNALS.md prose on
//     `Stream` runtime model before push.
//  2. §3.6b + sub-rule 4 — SPECIFIC arm = libuv non-starvation under
//     blocked next(); OBSERVABLE = a setImmediate scheduled
//     concurrently FIRES within bounded time; WOULD-FAIL at HEAD
//     (sync next() blocks the JS thread; the setImmediate fires only
//     after next() returns, well past the bounded budget).
//  3. §3.6e — RED-PHASE staged-pin; un-skip = G-CORE-10.
//  4. §3.6f (pim-18) — production call-site = lib.rs:2021;
//     substantive body = real libuv timer/setImmediate budget
//     measurement; NOT a constructibility check.
//  5. §3.13 per-test-static decomposition — each test owns its own
//     Engine + scheduling primitives.
//  6. §3.5n — R3-W5 verified `#[napi]` on next() is SYNC at HEAD
//     before writing.
//
// Pin source: R2-test-landscape.md §2 G-CORE-10 (P-3) row +
// PLAN-asynctask-744:22 pattern reference.

import { describe, it } from "vitest";

describe.skip(
    "TF-G-CORE-10 / R3-W5 napi PR-B libuv-non-starvation (RED until PR-B #1203 lands)",
    () => {
        it(
            "a long-running next() does NOT block libuv setImmediate (would-FAIL at HEAD: sync next() starves the event loop)",
            async () => {
                throw new Error(
                    "RED-PHASE: un-skip at G-CORE-10 (PR-B #1203). At \
synced HEAD lib.rs:2021 StreamHandleJs::next is `#[napi]` SYNC and \
holds the JS thread for its entire blocking-recv duration — a \
concurrently-scheduled setImmediate cannot fire until next() returns, \
starving the libuv event loop. Body at un-skip: start a long-running \
next() (~500 ms of producer work), schedule N setImmediates \
concurrently, assert each fires within a bounded ms window measured \
on a high-resolution clock. AsyncTask migration delegates next()'s \
blocking work to the napi-rs worker pool; the JS thread + libuv stay \
free; setImmediates fire on schedule. This is the PLAN-asynctask-744:22 \
pattern PR-B is supposed to produce.",
                );
            },
            { timeout: 5000 },
        );

        it(
            "concurrent next() pressure (N=8 streams) keeps libuv responsive for unrelated I/O (would-FAIL at HEAD: 8 sync next()s × on a single JS thread)",
            async () => {
                throw new Error(
                    "RED-PHASE: un-skip at G-CORE-10 (PR-B #1203). Pre- \
PR-B the JS thread can only have ONE sync napi call in flight at a \
time; 8 sync next() calls serialize on the JS thread; an unrelated \
fs.readFile during that window blocks for the entire serialized \
duration. Post-PR-B AsyncTask: 8 next()s run in parallel on the \
napi-rs worker pool; the JS thread + libuv stay free; an unrelated \
fs.readFile resolves within its normal latency window. The pin \
asserts the cross-stream parallelism property the migration enables \
(N concurrent streams do not block unrelated libuv I/O).",
                );
            },
            { timeout: 6000 },
        );
    },
);

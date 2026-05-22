// Phase-4-Meta-Core — R3-W5 — TF-G-CORE-10 / G-PRE-V1-HARDEN (Option-C
// #652 cancellation-stopgap + PR-B #1203) — `close()` concurrent with a
// blocked `next()` (the R2 §2 G-CORE-10 (P-2) napi-r1-5 seed).
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
//   >>> UN-SKIP AT: G-PRE-V1-HARDEN (Option-C arm — the cancellation +
//   the Option-C-before-PR-B ordering arm) AND THEN G-CORE-10 (PR-B
//   arm). §3.6e — closing wave un-skips; reviewer verifies landing.
//
// The brief at R2 §7 W5 specifies this file at __tests__/; a separate
// pre-existing file in bindings/napi/test/ (the W4-territory existing
// `stream_handle_next_cancellation_contract.test.ts`) owns the broader
// Option-C / PR-B contract narrative. THIS file is the dedicated
// R3-W5 napi-r1-5 pin per the R2 §2 G-CORE-10 (P-2) row.
//
// Disjointness vs `test/stream_handle_next_cancellation_contract.test.ts`
// (HARD — §3.5i): that file is the broader narrative (Option-C arm +
// PR-B arm together, with the verify-stays-regression block).  THIS
// file is the dedicated, focused, R2 (P-2) napi-r1-5 seed: a SINGLE
// bounded-time assertion that close() returns AND next() unblocks
// within a timeout. Distinct filename; both un-skip at the same wave.
//
// ============================================================================
// GROUND-TRUTH (synced HEAD c9c11c56)
// ============================================================================
//
//   bindings/napi/src/lib.rs:2021 StreamHandleJs::next(&self) is SYNC;
//   its body acquires a `std::sync::Mutex` on `self.inner` then drains
//   the producer-bridge channel via blocking recv WHILE STILL HOLDING
//   the mutex guard. close() (lib.rs:2040) needs the SAME lock to flip
//   the handle to the close path. So while next() is parked inside
//   blocking recv, close() cannot acquire the lock — the lock-across-
//   recv_blocking hazard #652 names. At HEAD this DEADLOCKS / PARKS
//   under the test's concurrent shape — RED.
//
//   Post-Option-C (Mutex-split) AND post-PR-B (full AsyncTask): close()
//   acquires its half of the split mutex (or the close-signal) while
//   next() blocks on the producer recv side; close() returns + next()
//   returns within a bounded time. GREEN.
//
// ============================================================================
// §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
// ============================================================================
//
//  1. §3.5b HARDENED (pim-1): implementer sweeps INTERNALS.md +
//     packages/engine/src docs on close()-cancels-next() before push.
//  2. §3.6b + sub-rule 4 (pim-2): SPECIFIC arm = close-during-blocked-
//     next; OBSERVABLE = bounded-time termination of both promises;
//     WOULD-FAIL at HEAD (deadlocks; test would time out).
//  3. §3.6e (pim-12): RED-PHASE staged-pin; un-skip destination named
//     = G-PRE-V1-HARDEN (Option-C) → G-CORE-10 (PR-B).
//  4. §3.6f (pim-18): production call-site = lib.rs:2021 next() +
//     lib.rs:2040 close(); substantive body = concurrent invocation
//     under a timeout; aspirational-prose-gap = NONE (the Option-C +
//     PR-B briefs name this exact predicate).
//  5. §3.13 per-test-static decomposition: each test creates its own
//     Engine + StreamHandle + completes within its `it` block; NO
//     shared static under the parallel vitest runner.
//  6. §3.5n: R3-W5 author independently grep-verified `lib.rs:2021`
//     locks-across-recv before writing.
//
// Pin source: R2-test-landscape.md §2 G-CORE-10 (P-2) row (the
// napi-r1-5 seed) + r2 §5 Replay/concurrency lane.

import { describe, it } from "vitest";

describe.skip(
    "TF-G-CORE-10 / R3-W5 napi-r1-5 close() concurrent with blocked next() (RED until Option-C + PR-B land)",
    () => {
        it(
            "close() returns AND a blocked next() unblocks within bounded time (would-FAIL at HEAD: deadlocks on lib.rs:2021 lock-across-recv_blocking)",
            async () => {
                throw new Error(
                    "RED-PHASE: un-skip at G-PRE-V1-HARDEN (Option-C \
arm — #652 Mutex-split cancellation-stopgap) FIRST, then G-CORE-10 \
(PR-B arm — full AsyncTask migration). At synced HEAD `bindings/napi/ \
src/lib.rs:2021` StreamHandleJs::next holds the inner Mutex across \
the blocking recv, so a concurrent close() (lib.rs:2040) DEADLOCKS \
waiting for that lock — this assertion would time out. Body at \
un-skip: spawn a long-blocking next() (no chunk produced), then call \
close() concurrently, assert both Promises resolve within a 2-second \
bounded timeout (no parking, no deadlock). #652-CLOSED ≠ fixed \
(napi-r1-2 / r1-triage:252): #652 is closed only as tracked-under- \
META #744 (bookkeeping consolidation), the code hazard is LIVE — \
Option-C is real pending work. The complementary pin in `test/ \
stream_handle_next_cancellation_contract.test.ts` carries the broader \
narrative; this pin is the focused R2 §2 (P-2) napi-r1-5 seed.",
                );
            },
            { timeout: 4000 },
        );

        it(
            "close() is idempotent across concurrent next() pressure (multiple close() calls during concurrent blocked next()s all resolve cleanly; no zombie tokio task)",
            async () => {
                throw new Error(
                    "RED-PHASE: un-skip at G-CORE-10 (PR-B #1203). The \
cancellation contract is not just 'close() returns once' — it is also \
'close() called twice + next() called concurrently TWICE all settle \
without leaving a zombie tokio task on the engine's async runtime'. \
Pre-PR-B + pre-Option-C: undefined (the deadlock masks the \
idempotence question). Post-Option-C + post-PR-B: each close()/next() \
Promise resolves; no orphaned producer-bridge tasks remain. This pin \
exercises the (F-1) row in R2 §2 G-CORE-10 (cancellation after partial \
chunk delivered: clean state; no orphaned chunks; no zombie task).",
                );
            },
            { timeout: 4000 },
        );
    },
);

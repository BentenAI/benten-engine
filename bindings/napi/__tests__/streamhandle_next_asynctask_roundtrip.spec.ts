// Phase-4-Meta-Core — R3-W5 — TF-G-CORE-10 (napi PR-B #1203) — production-arm
// pin: `StreamHandleJs::next()` AsyncTask migration round-trip.
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
// vitest has no per-test `#[ignore]` ; the agreed analog (see
// `bindings/napi/test/stream_handle_next_cancellation_contract.test.ts`
// header) is `describe.skip` + a loud RED-PHASE throw inside each `it`
// body, with the un-skip destination NAMED.
//
//   >>> UN-SKIP AT: G-CORE-10 (PR-B #1203 — full `StreamHandleJs::next`
//   AsyncTask migration). §3.6e — the closing-wave checklist MUST flip
//   `describe.skip` → `describe`. Reviewer verifies LANDING-STATUS
//   (the skip is removed), not merely that this spec file exists.
//
// ============================================================================
// GROUND-TRUTH (synced HEAD c9c11c56 — R3-W5 author verified via §3.5n)
// ============================================================================
//
//   `bindings/napi/src/lib.rs:2021`  StreamHandleJs::next(&self) is a
//   SYNC `#[napi]` method returning `Buffer | null` (verified). PR-A
//   (#1299) migrated the JsAtrium async-mutator surface; PR-A did NOT
//   touch `StreamHandleJs`. PR-B (#1203) is the still-open deliverable
//   that will (a) return a Promise (AsyncTask) from `next()` AND (b)
//   reflect the sync→async break in `packages/engine/src/types.ts` +
//   `index.d.ts` atomically (§3.5g cross-language rule-mirror).
//
//   Therefore at HEAD, calling `next()` returns a Buffer / null
//   directly — NOT a Promise. The structural assertion "`next()`
//   returns a Promise" FAILS at HEAD (RED) and PASSES post-PR-B (GREEN).
//
// ============================================================================
// §3.6g LITERAL discipline checklist (reproduced, NOT §-referenced)
// ============================================================================
//
//  1. §3.5b HARDENED (pim-1) — tests-only here; PR-B's implementer sweeps
//     the StreamHandle prose in packages/engine + INTERNALS.md before push.
//  2. §3.6b + sub-rule 4 (pim-2) — PRODUCTION-ARM + OBSERVABLE +
//     WOULD-FAIL: exercises the SPECIFIC `next()` AsyncTask shape (not
//     an umbrella "streams work"); concrete observable = the value
//     returned has Promise semantics; would-FAIL at HEAD (sync return).
//  3. §3.6e (pim-12) — RED-PHASE staged-pin; un-skip destination named
//     = G-CORE-10. Reviewer verifies landing-status, not spec-pin presence.
//  4. §3.6f (pim-18) — SHAPE-not-SUBSTANCE: production call-site is
//     `lib.rs:2021` StreamHandleJs::next; substantive body exercises a
//     real handler streaming production data via Promise resolution.
//  5. §3.5g — the sync→async break is the canonical cross-language
//     rule-mirror case (Rust napi-rs sync `Buffer | null` → JS async
//     `Promise<Buffer | null>`); this pin's PASS state implies
//     `packages/engine/src/types.ts` + `index.d.ts` already mirror.
//  6. §3.13 per-test-static decomposition — every test creates its own
//     `Engine` + `StreamHandle`; NO process-scoped shared static under
//     the parallel runner.
//  7. §3.6h — no rule/codification originates here.
//  8. §3.5n — R3-W5 ground-truth-verified `lib.rs:2021` independently
//     before writing this pin (see header). Plan-doc claims confirmed
//     against actual HEAD source.
//
// Disjointness vs the existing
// `test/stream_handle_next_cancellation_contract.test.ts` (HARD —
// §3.5i): that file owns the **cancellation contract** arm (close()
// cancels a stuck next()). THIS file owns the **AsyncTask roundtrip**
// arm (next() resolves a Promise carrying chunk bytes). The two pins
// share the un-skip wave (G-CORE-10) but their assertions are disjoint.
// __tests__/ vs test/ separation per the W5 partition brief.
//
// Pin source: R2-test-landscape.md §2 G-CORE-10 row, pin (P-1) +
// plan §1.A.FROZEN item 10 (StreamHandle.next final sync-vs-Promise
// shape) + r2 §6 case 17/18 (TS-side parity gate empty-diff
// post-FREEZE).

import { describe, it, expect } from "vitest";

// We use a deferred require pattern matching the existing
// `test/stream_handle_next_cancellation_contract.test.ts` shape so
// the import does not eagerly bind to a not-yet-built native binding.
// The `describe.skip` keeps vitest from even attempting to load the
// engine at the synced HEAD where PR-B has not landed.

describe.skip(
    "TF-G-CORE-10 / R3-W5 napi PR-B AsyncTask round-trip (RED until PR-B #1203 lands)",
    () => {
        it("StreamHandleJs::next() returns a Promise resolving to a Buffer chunk (would-FAIL on sync `Buffer | null` baseline)", async () => {
            // Loud RED defensive throw — the §3.6b ts-canary extension
            // pattern from `tf12_js_public_api_parity_gate_freeze.test.ts`.
            // An early un-skip-before-wire fails loudly rather than
            // silently passing.
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-10 (PR-B #1203). At synced \
HEAD `bindings/napi/src/lib.rs:2021` StreamHandleJs::next(&self) is \
SYNC (returns `Buffer | null` directly, NOT a Promise) — a Promise \
assertion would-FAIL. PR-B migrates to AsyncTask: next() returns \
Promise<Buffer | null>. When PR-B lands, remove `.skip`, replace this \
throw with the real handler + stream + `await next()` round-trip \
exercising the Promise resolution shape.",
            );
        });

        it("StreamHandleJs::next() AsyncTask runs OFF the libuv event-loop thread (would-FAIL if PR-B keeps the work on the JS thread)", async () => {
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-10 (PR-B #1203). The \
AsyncTask migration's whole point is to free the libuv event loop \
while the producer-bridge recv blocks (napi-r1-5 + the libuv-non- \
starvation predicate). The post-PR-B body of this test schedules an \
unrelated libuv timer concurrently with a long-running next() and \
asserts the timer FIRES at its scheduled time (would-FAIL pre-PR-B: \
the sync next() blocks the loop and the timer fires late). See the \
sibling `streamhandle_libuv_non_starvation.spec.ts` for the dedicated \
libuv-budget pin.",
            );
        });

        it("StreamHandleJs::next() Promise resolves with the same canonical bytes a sync recv would have produced (cross-language round-trip parity)", async () => {
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-10 (PR-B #1203). The \
sync→async migration MUST NOT change the byte-content of yielded \
chunks — exactly the same producer payload, just delivered through \
Promise resolution. Body at un-skip: produce N-byte chunks at the \
Rust side, await next() in the JS side, byte-compare resolved \
Buffer === expected. Pin verifies the AsyncTask body does not \
inadvertently transcode/transform the chunk. Cross-references the \
§3.5g rule-mirror (Rust producer ↔ JS consumer same canonical bytes).",
            );
        });
    },
);

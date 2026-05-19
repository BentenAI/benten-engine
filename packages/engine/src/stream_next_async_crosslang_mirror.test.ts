// Phase-4-Meta-Core — TF-9 RED-PHASE (R3-B6) — §3.5g cross-language
// Rust↔TS atomic-mirror pin for the PR-B #1203 `StreamHandle.next`
// sync→async break + §1.A.FROZEN item 10 (TS/JS public API frozen).
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
// At synced baseline ed03729a `StreamHandle.next()` is declared
// SYNCHRONOUS in BOTH the Rust napi surface
// (`bindings/napi/src/lib.rs:2021` — sync `#[napi] pub fn next`) and
// the TS surface (`packages/engine/src/types.ts:1138` —
// `next(): Chunk | null`) AND in the generated d.ts
// (`bindings/napi/index.d.ts` — `next(): Buffer | null`). G-CORE-10
// PR-B migrates `StreamHandleJs::next` to a Promise-backed AsyncTask;
// the §3.5g cross-language rule-mirror discipline + plan G-CORE-10 +
// §1.A.FROZEN item 10 REQUIRE that break to land ATOMICALLY across
// `bindings/napi/index.d.ts` + `packages/engine/src/types.ts` +
// `packages/engine/src/index.ts` IN THE SAME PR-B PR.
//
// The async-shape assertions below FAIL at baseline (the surface is
// still sync) and PASS once PR-B lands the atomic Rust↔TS mirror.
// vitest has no per-test `#[ignore]`; the RED-PHASE `describe` is
// `describe.skip` (the agreed RED-PHASE shape — un-skip at the
// closing wave; §3.6e: reviewer verifies the skip is REMOVED, i.e.
// landing-status, not merely spec-pin presence).
//
//   UN-SKIP at: G-CORE-10 (PR-B #1203). Option-C #652
//   (G-PRE-V1-HARDEN) is a Mutex-split internal stopgap and does NOT
//   change the public `next()` SHAPE — so this cross-language SHAPE
//   mirror is gated on PR-B only, NOT on Option-C. (The cancellation
//   BEHAVIOR — close()-cancels-next() — is pinned separately in
//   `bindings/napi/test/stream_handle_next_cancellation_contract.test.ts`.)
//
// ============================================================================
// GROUND-TRUTH (synced HEAD ed03729a — verified by the R3 author)
// ============================================================================
//
//   packages/engine/src/types.ts:1138   next(): Chunk | null;   (SYNC)
//   bindings/napi/index.d.ts             next(): Buffer | null   (SYNC)
//   bindings/napi/src/lib.rs:2021        sync `#[napi] pub fn next`
//
//   PR-A #1299 (LANDED at ed03729a) migrated the JsAtrium mutators
//   only — it did NOT touch StreamHandleJs / the TS StreamHandle
//   surface. PR-B (#1203) is the still-open G-CORE-10 deliverable
//   that breaks `next()` sync→async. The §1.A.FROZEN item 10
//   inventory (StreamHandle incl. `next()` final sync-vs-Promise
//   shape) freezes at G-CORE-9 AGAINST the PR-B-landed async shape.
//
// ============================================================================
// VERIFY-STAYS-REGRESSION (NOT skipped — GREEN at baseline): the
// bottom block asserts the PR-A-landed-vs-PR-B-RED split — at HEAD the
// TS surface is the UNCHANGED sync shape PR-A did not touch.
// ============================================================================

import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const here = dirname(fileURLToPath(import.meta.url));
const typesDts = join(here, "types.ts");
// index.d.ts is the napi-rs-generated declaration the TS surface
// mirrors; resolved relative to the repo bindings dir.
const napiIndexDts = join(
  here,
  "..",
  "..",
  "..",
  "bindings",
  "napi",
  "index.d.ts",
);

function read(p: string): string {
  return readFileSync(p, "utf8");
}

// ---------------------------------------------------------------------------
// RED-PHASE — UN-SKIP at G-CORE-10 (PR-B #1203). §3.6e: the closing
// wave flips `describe.skip` -> `describe`; reviewer verifies the
// skip is REMOVED (landing-status), not just file presence.
// ---------------------------------------------------------------------------
describe.skip(
  "TF-9 RED-PHASE — StreamHandle.next sync→async §3.5g Rust↔TS atomic mirror (PR-B #1203 / §1.A.FROZEN item 10)",
  () => {
    it("TS types.ts StreamHandle.next is migrated to Promise (async) — would-FAIL while it stays `Chunk | null`", () => {
      const src = read(typesDts);
      // PRE (ed03729a): `next(): Chunk | null;` — this assertion FAILS.
      // POST PR-B: `next(): Promise<Chunk | null>;` (the migrated
      // async signature). The §3.5g mirror obligation: this change is
      // in the SAME PR as the Rust-side AsyncTask migration.
      expect(src).toMatch(/next\(\)\s*:\s*Promise<\s*Chunk\s*\|\s*null\s*>/);
      // And the prose contract no longer says "synchronously".
      expect(src).not.toMatch(/Pull the next chunk synchronously/);
    });

    it("napi index.d.ts StreamHandleJs.next is regenerated to a Promise — atomic with the TS side (#1204 parity surface)", () => {
      const src = read(napiIndexDts);
      // PRE: `next(): Buffer | null` — FAILS. POST PR-B: the
      // regenerated d.ts carries `next(): Promise<Buffer | null>`.
      // #1204 (index.d.ts regen + JS-side parity gate) is homed to
      // G-CORE-10 — this pin asserts the regen actually happened
      // ATOMICALLY with the TS-side change (§3.5g: both sides update
      // in ONE PR; a one-sided update is a §3.5g violation).
      expect(src).toMatch(
        /next\(\)\s*:\s*Promise<\s*Buffer\s*\|\s*null\s*>/,
      );
    });

    it("§3.5g atomicity: BOTH the d.ts AND types.ts carry the async shape (neither side lags)", () => {
      const dts = read(napiIndexDts);
      const ts = read(typesDts);
      const dtsAsync = /next\(\)\s*:\s*Promise<\s*Buffer\s*\|\s*null\s*>/.test(
        dts,
      );
      const tsAsync = /next\(\)\s*:\s*Promise<\s*Chunk\s*\|\s*null\s*>/.test(
        ts,
      );
      // The §3.5g rule-mirror invariant: it is NEVER the case that one
      // side is migrated and the other is not. Post-PR-B both are
      // true; pre-PR-B both are false. A half-migrated state (XOR) is
      // the precise §3.5g violation this pin exists to catch.
      expect(dtsAsync).toBe(tsAsync);
      expect(dtsAsync && tsAsync).toBe(true);
    });
  },
);

// ---------------------------------------------------------------------------
// VERIFY-STAYS-REGRESSION — PR-A #1299 LANDED; the TS StreamHandle
// surface is the UNCHANGED sync shape at ed03729a (NOT skipped; GREEN
// at baseline). Documents the exact PR-A-landed-vs-PR-B-RED split:
// PR-A migrated atrium, NOT stream — so the stream TS surface is still
// `next(): Chunk | null`. When PR-B lands, this block flips RED,
// making the §3.5g break observable in this same file.
// ---------------------------------------------------------------------------
describe("TF-9 VERIFY-STAYS-REGRESSION — TS StreamHandle.next is still SYNC at ed03729a (PR-A did not touch it)", () => {
  it("types.ts declares the baseline sync StreamHandle.next (Chunk | null)", () => {
    const src = read(typesDts);
    expect(src).toMatch(/next\(\)\s*:\s*Chunk\s*\|\s*null/);
    expect(src).toMatch(/Pull the next chunk synchronously/);
  });

  it("index.d.ts declares the baseline sync StreamHandleJs.next (Buffer | null)", () => {
    const src = read(napiIndexDts);
    expect(src).toMatch(/next\(\)\s*:\s*Buffer\s*\|\s*null/);
  });
});

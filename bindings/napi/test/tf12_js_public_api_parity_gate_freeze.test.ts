// Phase-4-Meta-Core — TF-12 obligation (2) — RED-PHASE (R3-C1, the
// LAST R3 wave; freeze-time). #1204 TS-side public-API parity gate
// empty-diff post-FREEZE, asserted against the PQ-HYBRID-CAPABLE JS
// shape (NOT an Ed25519-shaped baseline).
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
// vitest has no per-test `#[ignore]`; the analog of the Rust
// `#[ignore = "RED-PHASE: un-ignore at G-CORE-9"]` is `describe.skip`
// + the un-skip marker comment below. Each `it` body additionally
// throws a loud `RED-PHASE` defensive error (the §3.6b ts-canary
// extension) so an early un-skip-before-wire fails loudly rather than
// silently passing.
//
//   >>> UN-SKIP AT: G-CORE-9 (the single atomic FREEZE wave). <<<
//
// ============================================================================
// §3.6g LITERAL discipline checklist (reproduced, not §-referenced)
// ============================================================================
//
//  1. Land-when = FREEZE. `describe.skip` + per-`it` RED-PHASE throw;
//     un-skip destination named = G-CORE-9.
//  2. Campaign-tail landed-vs-RED split (§3.5n): R3-C1 ground-truthed
//     the surface at ed03729a — there is NO TS-side parity baseline
//     file (no `ts-public-api` / `.d.ts.baseline`; `drift-detect.yml`
//     is the SEPARATE ErrorCatalog-T7 detector, not the #1204
//     JS-public-API gate). `StreamHandleJs.next()` is still SYNC
//     (`next(): Buffer | null`) — pre-PR-B. `ManifestSignature` has
//     ONLY `ed25519?: string` — NOT PQ-hybrid-widened. So the #1204
//     gate + the PQ-hybrid JS-shape widening are freeze-DELIVERABLES
//     NOT yet built -> RED.
//  3. SHAPE-not-SUBSTANCE (pim-18 / §3.6f): this is necessarily the
//     structural backstop pin per r2 §4-A TF-12 row ("the pin asserts
//     the baseline is committed at the FREEZE and any post-freeze
//     delta is a CI FAIL — the test is the structural backstop, not a
//     snapshot of whatever-shipped"). The would-FAIL signal is
//     concrete: the real committed TS-parity baseline file + the real
//     `index.d.ts` / `errors.generated.ts` / `types.ts` surfaces + the
//     PQ-hybrid widening markers. NOT a constructibility assertion.
//     `cargo-public-api` does NOT cover the TS surface — this pin
//     closes exactly that gap (§1.A.FROZEN item 10).
//  4. pim-2 sub-rule-4 (§3.6b): exercises the SPECIFIC §1.A.FROZEN
//     item 10 obligation (StreamHandle.next final shape + #1204 parity
//     gate empty-diff + the PQ-hybrid JS-shape widening of
//     TypedCallInputShapes / TypedCallOutputShapes / ManifestSignature
//     with NO hardcoded 32B-key/64B-sig Ed25519 assumption), not an
//     umbrella "the TS API is stable".
//  5. §3.13: no shared process-scoped state — each `it` uses locals.
//  6. §3.5j: N/A to TS (clippy is Rust); the napi crate's Rust pins
//     carry the scoped-clippy obligation, this TS file does not.
//  7. §3.6e: introduces no stranded skipped pin; the named un-skip
//     destination IS G-CORE-9.
//
// Disjointness vs R3-B6 (HARD — §3.5i): R3-B6 owns the TF-9 files
// `bindings/napi/test/stream_handle_next_cancellation_contract.test.ts`
// + `packages/engine/src/stream_next_async_crosslang_mirror.test.ts`
// (the StreamHandle.next BEHAVIOR / cancellation contract). THIS file
// asserts the G-CORE-9 FREEZE PROPERTY of the JS public-API parity
// gate (the JS analog of cargo-public-api). It does NOT re-test
// cancellation / async iteration. Distinct `tf12_` filename; no R3-B6
// file is touched.
//
// Pin source: r2-test-landscape.md TF-12 obligation (2) + §2.A S9 +
// §2.B "#1204 index.d.ts regen + JS-side parity CI gate" + plan
// §1.A.FROZEN item 10 (incl. the PQ-hybrid JS-shape widening
// napi-r1-1) + §0 Freeze-completeness cluster (c).

import { describe, it, expect } from "vitest";
import { readFileSync, existsSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const WORKSPACE_ROOT = resolve(HERE, "..", "..", "..");

function read(rel: string): string {
  const p = resolve(WORKSPACE_ROOT, rel);
  if (!existsSync(p)) {
    throw new Error(
      `TF-12 (2): expected surface file absent at G-CORE-9: ${rel}`,
    );
  }
  return readFileSync(p, "utf8");
}

describe.skip("TF-12 (2) — #1204 JS public-API parity gate frozen at G-CORE-9", () => {
  it("a committed TS-side public-API parity baseline exists (the JS analog of cargo-public-api)", () => {
    throw new Error(
      "RED-PHASE: un-skip at G-CORE-9 (#1204 TS parity gate). " +
        "Pre-freeze there is NO TS-side parity baseline.",
    );
    // Post-G-CORE-9 body (the structural backstop): a committed
    // baseline of the `@benten/engine` public surface must exist so a
    // post-freeze TS-API delta is a CI FAIL. The G-CORE-9 brief decides
    // the exact path; the pin asserts the PROPERTY (a committed,
    // non-vacuous baseline pinned to the freeze commit).
    // eslint-disable-next-line no-unreachable
    const candidates = [
      "bindings/napi/index.d.ts.baseline",
      "docs/public-api/ts-benten-engine.d.ts",
      "docs/public-api/benten-engine-ts.txt",
    ];
    const found = candidates.find((c) =>
      existsSync(resolve(WORKSPACE_ROOT, c)),
    );
    expect(
      found,
      `#1204: a committed TS public-API parity baseline must exist at ` +
        `G-CORE-9 (cargo-public-api does NOT cover the TS surface). ` +
        `Candidates: ${candidates.join(", ")}`,
    ).toBeTruthy();
    const body = read(found as string);
    expect(
      body.includes("StreamHandleJs") || body.includes("StreamHandle"),
      "#1204 baseline must enumerate the StreamHandle public surface",
    ).toBe(true);
  });

  it("StreamHandleJs.next() final shape is frozen (post-PR-B async Promise shape, not the pre-PR-B sync shape)", () => {
    throw new Error(
      "RED-PHASE: un-skip at G-CORE-9. Pre-PR-B `next()` is SYNC " +
        "(`next(): Buffer | null`); the freeze locks the FINAL shape.",
    );
    // eslint-disable-next-line no-unreachable
    const dts = read("bindings/napi/index.d.ts");
    // §1.A.FROZEN item 10: G-CORE-10's PR-B makes `next()` async; the
    // sync→async break is reflected ATOMICALLY in index.d.ts (§3.5g
    // cross-language rule-mirror). Post-freeze the frozen shape is the
    // async one — would-FAIL if the sync shape survives the freeze.
    const syncShape = /next\(\)\s*:\s*Buffer\s*\|\s*null/;
    expect(
      syncShape.test(dts),
      "§1.A.FROZEN item 10: `StreamHandleJs.next()` still has the " +
        "pre-PR-B SYNC shape at the freeze — the final (post-PR-B " +
        "Promise) shape must be frozen, mirrored Rust↔TS atomically.",
    ).toBe(false);
    expect(
      /next\(\)\s*:\s*Promise</.test(dts),
      "§1.A.FROZEN item 10: the frozen `next()` must be the PR-B async " +
        "Promise shape.",
    ).toBe(true);
  });

  it("the PQ-hybrid JS-shape widening is applied — NO hardcoded Ed25519 32B-key/64B-sig assumption (napi-r1-1)", () => {
    throw new Error(
      "RED-PHASE: un-skip at G-CORE-9. Pre-freeze `ManifestSignature` " +
        "has ONLY `ed25519?: string` — NOT PQ-hybrid-widened.",
    );
    // eslint-disable-next-line no-unreachable
    const types = read("packages/engine/src/types.ts");
    // §1.A.FROZEN item 10 PQ-hybrid JS-shape widening: the frozen JS
    // surface inventory MUST include the crypto-size-touching shapes
    // widened to carry the hybrid ML-DSA-65 (~1952B key / ~3309B sig) +
    // ML-KEM dimensions — mirrored ATOMICALLY Rust↔TS (§3.5g); the
    // #1204 empty-diff is asserted against THIS PQ-hybrid-capable shape,
    // never an Ed25519-shaped baseline. Would-FAIL if `ManifestSignature`
    // is still Ed25519-only at the freeze.
    const manifestSigBlock =
      types.slice(
        types.indexOf("interface ManifestSignature"),
        types.indexOf("interface ManifestSignature") + 400,
      ) || "";
    const onlyEd25519 =
      manifestSigBlock.includes("ed25519?: string") &&
      !/ml[_-]?dsa|hybrid|mldsa|pq[_-]?sig/i.test(manifestSigBlock);
    expect(
      onlyEd25519,
      "napi-r1-1: `ManifestSignature` is still Ed25519-only at the " +
        "freeze — the PQ-hybrid JS-shape widening (hybrid Ed25519⊕" +
        "ML-DSA-65 dimensions, no hardcoded 32B/64B assumption) MUST be " +
        "applied + mirrored atomically Rust↔TS before G-CORE-9.",
    ).toBe(false);
    // The typed-CALL crypto shapes likewise must not bake a 32B/64B
    // Ed25519-only assumption into the frozen JS surface.
    expect(
      /ml[_-]?dsa|ml[_-]?kem|hybrid/i.test(types),
      "napi-r1-1: the frozen TS surface must include the PQ-hybrid " +
        "(ML-DSA-65 / ML-KEM-768) size-touching shapes — the #1204 " +
        "parity gate is asserted against the PQ-hybrid-capable shape.",
    ).toBe(true);
  });

  it("the #1204 JS-parity CI gate is wired and enforcing (not informational)", () => {
    throw new Error(
      "RED-PHASE: un-skip at G-CORE-9. The #1204 JS-parity CI gate is " +
        "not yet wired.",
    );
    // eslint-disable-next-line no-unreachable
    // The brief decides the exact workflow; the pin asserts the
    // PROPERTY: a CI gate diffs the TS public surface against the
    // committed baseline and FAILS on drift (the JS analog of the
    // promoted cargo-public-api gate). Candidate workflow names.
    const wfCandidates = [
      ".github/workflows/ts-public-api.yml",
      ".github/workflows/js-api-parity.yml",
      ".github/workflows/napi-public-api.yml",
    ];
    const wf = wfCandidates.find((w) =>
      existsSync(resolve(WORKSPACE_ROOT, w)),
    );
    expect(
      wf,
      `#1204: a TS public-API parity CI gate workflow must exist at ` +
        `G-CORE-9. Candidates: ${wfCandidates.join(", ")}`,
    ).toBeTruthy();
    const wfBody = read(wf as string);
    expect(
      !/INFORMATIONAL ONLY|always exits 0|\|\| true/.test(wfBody),
      "#1204: the JS-parity gate must be ENFORCING post-freeze (a TS " +
        "public-API delta is a CI FAIL), not informational — the " +
        "structural backstop for the frozen TS surface.",
    ).toBe(true);
  });
});

// Phase-4-Meta-Core — R4-FP-1 / R3-W5 cross-language pin —
// G-CORE-3w SubgraphSpec walker BFS-ordering through napi (§3.5g
// cross-language rule-mirror obligation). Closes R4.1 L1 m-2
// (orchestrator triage 2026-05-22).
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
//   >>> UN-SKIP AT: G-CORE-3w (the wave that ships the SubgraphSpec
//   walker as the Subgraph primitive in benten-core per Spike F finding
//   "walker-as-Subgraph-shipped-once-in-benten-core"; per RATIFIED-
//   sharing-and-confidentiality-2026-05-21.md spike-derived refinement
//   #3). The walker's BFS-ordering is the R4 BFS ratification (R4 path
//   canonicalization = BFS-order, carry canonical path in grant).
//   §3.6e — closing-wave checklist un-skips; reviewer verifies landing.
//
// ============================================================================
// GROUND-TRUTH (§3.5n at R4-FP-1 author-time, 2026-05-22)
// ============================================================================
//
//   `git grep "SubgraphSpec"` across bindings/napi/ + packages/engine/
//   returns ZERO hits at R3-W5 branch base. The walker type +
//   BFS-ordering napi surface have NOT been minted; this RED-PHASE pin
//   is the forward-looking §3.5g contract that the walker's BFS
//   ordering invariant survives the napi boundary byte-for-byte.
//
// ============================================================================
// PIN SOURCE + DISPATCH PROVENANCE
// ============================================================================
//
//   - R2 §2 G-CORE-3w row (X-1 cross-language obligation): "§3.5g
//     atomic-mirror; R3 RED-PHASE pin against the not-yet-existent
//     napi surface".
//   - R4.1 triage finding m-2 (L1 coverage-completeness, FIX-NOW): R2
//     listed the X-1 obligation; R3 did not stake the pin. This file
//     is the R4-FP-1 closure.
//   - Spike F findings: walker shipped ONCE in benten-core (not
//     per-crate-duplicated); reference impl per `.addl/spikes/`
//     spike-F sandbox notes.
//   - RATIFIED-sharing-and-confidentiality-2026-05-21.md R4: Path
//     canonicalization = BFS-order, carry canonical path in grant
//     (ratification-of-record; this pin enforces the cross-language
//     half of that contract).
//
// ============================================================================
// §3.6g LITERAL discipline checklist
// ============================================================================
//
//  1. §3.5b HARDENED — at G-CORE-3w un-skip, README/INTERNALS/types.ts
//     swept in the SAME PR.
//  2. §3.6b sub-rule 4 — SPECIFIC arm: walker BFS-ordering through napi
//     (not "a walker exists" sentinel).
//  3. §3.6e — RED-PHASE; un-skip = G-CORE-3w.
//  4. §3.6f (pim-18) — production-arm: invokes the real napi-bound
//     walker surface; substantive body-of-test.
//  5. §3.5g (CRITICAL on this pin) — Rust-side BFS impl + TS-side
//     consumer MUST yield byte-identical traversal order for a fixed
//     subgraph fixture. The R5 G-CORE-3w implementer brief MUST
//     enumerate "napi binding regen + atomic types.ts update + un-skip
//     this pin" as part of the wave's checklist.
//  6. §3.5n — verified at R4-FP-1 author-time that no SubgraphSpec /
//     walker references exist in `bindings/napi/index.d.ts` or
//     `packages/engine/src/types.ts`; the surface is genuinely unminted.

import { describe, it, expect } from "vitest";

describe.skip(
    "G-CORE-3w SubgraphSpec walker BFS-ordering through napi (R4-FP-1 m-2 closure; un-skip at G-CORE-3w)",
    () => {
        it("walker returns BFS-ordered Node CIDs for a fixed fan-out subgraph fixture (matches canonical Rust-side order)", () => {
            // R5 G-CORE-3w implementer wires this body to the real napi
            // surface at un-skip time. The TYPE signature is:
            //   walkSubgraph(spec: SubgraphSpec, roots: Cid[]): Cid[]
            // (or equivalent name; final decision = G-CORE-3w implementer).
            //
            // Fixture shape: a subgraph with 3 roots, each having 2
            // children + 1 grandchild (depth-2 fan-out). BFS order is:
            //   [root_a, root_b, root_c,
            //    child_a1, child_a2, child_b1, child_b2, child_c1, child_c2,
            //    grandchild_a1, grandchild_a2, grandchild_b1, ..., grandchild_c2]
            //
            // The CANONICAL Rust-side order (per RATIFIED R4 ratification)
            // is the byte-canonical BFS via stable CID ordering at each
            // level. The body of this test asserts the napi-returned
            // Cid[] EQUALS the canonical-bytes-expected array (encoded
            // as `Buffer[]` or `string[]` per napi-rs convention).
            //
            // The WOULD-FAIL arm: at G-CORE-3w wave-completion if the
            // napi binding inadvertently re-sorts (e.g. by insertion
            // order rather than BFS) the assertion fails immediately
            // for the asymmetric fan-out test fixture.
            //
            // At R4-FP-1 author-time the import target does not exist
            // yet; the pin stays describe.skip and body is placeholder.
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-3w (the wave that mints \
the SubgraphSpec walker as the Subgraph primitive in benten-core). At \
un-skip, wire the body against the real napi-bound walker surface name \
(per the §3.5g atomic-mirror contract). Body shape: fixed 3-root × 2-child \
× 1-grandchild fixture; invoke walker via napi; assert returned Cid[] \
equals canonical BFS order byte-identically. The Rust-side test in \
benten-core asserts the SAME order from the SAME fixture (golden \
fixture cross-language). § R4.1 L1 m-2 + R2 §2 G-CORE-3w X-1 + Spike F \
walker-as-Subgraph + RATIFIED R4 BFS-order.",
            );
        });

        it("the canonical-path-in-grant is preserved through napi serialization (couples G-CORE-3b AuthorizationGrant)", () => {
            // RATIFIED R4 second half: "carry canonical path in grant".
            // The walker's BFS-derived canonical path is encoded into the
            // AuthorizationGrant.key_material (or a parallel field); this
            // pin asserts that the canonical-path bytes survive the napi
            // serialization byte-identically.
            //
            // Cross-pin coupling: `authorization_grant_ts_round_trip.spec.ts`
            // exercises the grant's round-trip generally; THIS pin exercises
            // the walker → canonical-path → grant.key_material composition
            // specifically, and verifies the canonical-bytes contract holds.
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-3w (couples G-CORE-3b). At \
un-skip: invoke walker on fixture → extract canonical-path bytes → \
construct AuthorizationGrant with those bytes as key_material → serialize \
via napi → reparse → walker again → assert canonical path is byte- \
identical. The WOULD-FAIL arm: any re-canonicalization step on the napi \
boundary that does not preserve BFS-order byte-identically. § R4.1 L1 m-2 \
+ RATIFIED R4 BFS + path-carry-in-grant.",
            );
        });
    },
);

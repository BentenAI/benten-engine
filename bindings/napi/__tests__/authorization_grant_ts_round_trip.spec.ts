// Phase-4-Meta-Core — R4-FP-1 / R3-W5 cross-language pin —
// G-CORE-3b AuthorizationGrant TS round-trip (§3.5g cross-language
// rule-mirror obligation). Closes R4.1 L1 m-1 (orchestrator triage
// 2026-05-22).
//
// ============================================================================
// RED-PHASE STATUS
// ============================================================================
//
//   >>> UN-SKIP AT: G-CORE-3b (the wave that mints the
//   `AuthorizationGrant` Rust type per RATIFIED-sharing-and-
//   confidentiality-2026-05-21.md R3 — ONE signed artifact
//   `{ucan, key_material, binding_sig}` — and the corresponding
//   napi binding + TS surface).
//   §3.6e — closing-wave checklist un-skips; reviewer verifies
//   landing-status not just spec-pin presence.
//
// ============================================================================
// GROUND-TRUTH (§3.5n at R4-FP-1 author-time, 2026-05-22)
// ============================================================================
//
//   `git grep "AuthorizationGrant"` across bindings/napi/ + packages/engine/
//   returns ZERO hits at the R3-W5 branch base. The type has NOT yet been
//   minted Rust-side; this RED-PHASE pin is a forward-looking §3.5g
//   contract: when G-CORE-3b lands the type on Rust side + the napi
//   binding regenerates `index.d.ts`, this pin's positive arm verifies
//   the TS-side round-trip works (constructs a grant, serializes via
//   the napi-bound surface, asserts byte-identity or field-equality
//   on round-trip).
//
// ============================================================================
// PIN SOURCE + DISPATCH PROVENANCE
// ============================================================================
//
//   - R2 §2 G-CORE-3b row (X-1 cross-language obligation): "§3.5g
//     atomic-mirror obligation; R3 RED-PHASE pin against the not-yet-
//     existent napi surface".
//   - R4.1 triage finding m-1 (L1 coverage-completeness, FIX-NOW): R2
//     listed the X-1 obligation; R3 did not stake the RED-PHASE pin.
//     This file is the R4-FP-1 closure.
//   - CLAUDE.md baked-in #5 (crypto-agility): AuthorizationGrant's
//     `binding_sig` field carries a Varsig-multiformats signature; the
//     PQ-hybrid v1-beta default (codepoint dispatch) lives on the Rust
//     side; the TS surface treats `binding_sig` opaquely as bytes.
//   - RATIFIED-sharing-and-confidentiality-2026-05-21.md R3: the grant
//     is ONE signed artifact `{ucan, key_material, binding_sig}` (post-
//     spike-sequence ratification; supersedes earlier multi-artifact
//     proposals).
//
// ============================================================================
// §3.6g LITERAL discipline checklist
// ============================================================================
//
//  1. §3.5b HARDENED — at G-CORE-3b un-skip, README/INTERNALS/types.ts
//     swept in the SAME PR.
//  2. §3.6b sub-rule 4 — SPECIFIC arm: AuthorizationGrant TS round-trip
//     through the napi binding (not "a type exists" sentinel).
//  3. §3.6e — RED-PHASE; un-skip = G-CORE-3b.
//  4. §3.6f (pim-18) — production-arm: round-trips through the real
//     napi-bound surface; substantive body-of-test, not a constructibility
//     assertion.
//  5. §3.5g (CRITICAL on this pin) — the WHOLE point is the Rust ↔ TS
//     atomic update at G-CORE-3b wave-completion. The R5 G-CORE-3b
//     implementer brief MUST enumerate "regenerate index.d.ts +
//     atomically update types.ts (or equivalent) + un-skip this pin"
//     as part of the wave's 4-step checklist (per R4.1 L5-OBS-1).
//  6. §3.5n — verified at R4-FP-1 author-time that
//     `bindings/napi/index.d.ts` + `packages/engine/src/types.ts` do
//     NOT contain `AuthorizationGrant` references; the type is genuinely
//     unminted at this branch.

import { describe, it, expect } from "vitest";

describe.skip(
    "G-CORE-3b AuthorizationGrant TS round-trip (R4-FP-1 m-1 closure; un-skip at G-CORE-3b)",
    () => {
        it("constructs a grant via the napi surface, serializes, round-trips byte-identical (or field-equal on JSON path)", () => {
            // R5 G-CORE-3b implementer wires this body to the real napi
            // surface name at un-skip time. The TYPE signature is:
            //   AuthorizationGrant {
            //     ucan: Ucan | Bytes,            // serialized UCAN token
            //     key_material: Bytes,           // codepoint-prefixed; PQ-hybrid
            //     binding_sig: Bytes,            // Varsig multiformats
            //   }
            // Round-trip target: construct on TS side → serialize via
            // napi-bound `toBytes()` (or equivalent name; final decision
            // = G-CORE-3b implementer) → reparse via `fromBytes()` →
            // assert field-equality (or byte-identity if the on-wire
            // form is canonical-bytes).
            //
            // The WOULD-FAIL arm: at G-CORE-3b wave-completion if the
            // implementer mints the Rust type but forgets the napi
            // binding regen (the §3.5g atomic-mirror reverse-direction
            // failure), `import { AuthorizationGrant } from "@benten/..."`
            // throws ImportNotFound, fairing this test RED.
            //
            // At R4-FP-1 author-time the import target does not exist
            // yet, so the pin stays `describe.skip` and the body is a
            // placeholder; un-skip lands when the binding lands.
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-3b (the wave that mints \
AuthorizationGrant Rust type + napi binding + TS surface). At un-skip, \
wire the body against the real napi-bound surface name (per the §3.5g \
atomic-mirror contract). Body shape: construct grant with mock UCAN + \
KeyMaterial(codepoint-prefixed) + BindingSig; serialize via napi-bound \
toBytes; reparse via fromBytes; assert byte-identity (or field-equality \
on the JSON path). § R4.1 L1 m-1 + R2 §2 G-CORE-3b X-1.",
            );
        });

        it("the binding_sig field round-trips a Varsig-multiformats header (PQ-hybrid codepoint per CLAUDE.md baked-in #5)", () => {
            // §3.5g coupling with Gate 20: the AuthorizationGrant.binding_sig
            // is a Varsig-multiformats byte string. The TS surface treats
            // it as opaque bytes; this pin asserts the round-trip preserves
            // the multiformats prefix byte-identically (no truncation, no
            // re-encoding). At un-skip, the body extracts the first ~5
            // bytes (multihash + sig-algo codepoint) and asserts they
            // survive serialize→reparse byte-identically.
            //
            // The PQ-hybrid v1-beta default is codepoint `0x647a` per
            // RATIFIED-sharing-and-confidentiality-2026-05-21 §"crypto-
            // agility CONFIRMED". The body does NOT lock that exact
            // codepoint (implementer's R5 final decision) but DOES
            // assert SOME Varsig prefix is preserved.
            throw new Error(
                "RED-PHASE: un-skip at G-CORE-3b (couples with G-CORE-3a \
crypto-suite + the Gate 20 UCAN Varsig pin in benten-crypto-suite). At \
un-skip: extract the multihash + sig-algo codepoint prefix bytes from \
binding_sig pre-serialize; round-trip; assert prefix is byte-identical \
post-reparse (no truncation / no codepoint loss). § R4.1 L1 m-1 + \
CLAUDE.md baked-in #5 multiformats permanence.",
            );
        });
    },
);

# Pre-R1 Triage — Phase 1 Implementation Plan

**Stage:** Pre-R1 (planning review, before R1 spec-agent debate).
**Plan under review:** `.addl/phase-1/00-implementation-plan.md`
**Critics:** `code-reviewer` + `benten-core-guardian` (dispatched in parallel, non-overlapping lenses).
**Raw findings:** `.addl/phase-1/pre-r1-code-reviewer.json`, `.addl/phase-1/pre-r1-benten-core-guardian.json`.

## Headline

| Critic | Verdict | Critical | Major | Minor |
|---|---|---|---|---|
| `benten-core-guardian` (scope fidelity + content-hash invariant protection) | pass | 0 | 0 | 7 |
| `code-reviewer` (coherence + ADDL dispatch + testability) | **revise** | 0 | 5 | 5 |
| **Net** | **revise** | **0** | **5** | **12** |

Zero overlap between the two critics' findings — they reviewed complementary aspects. All 17 findings are valid plan-improvements (not architectural disagreements), all have clear fixes, all are cheap edits. **Every finding dispositioned as "fix now in plan."** Zero deferrals, zero disagreements.

## Triage table

### Major findings (5 — all from `code-reviewer`)

| # | Area | Finding (summary) | Disposition | Applied where |
|---|---|---|---|---|
| M1 | Coherence | G3's "Gates next group" lists G4; Section 7 says G2+G4 parallel. Contradiction. | **Fix in plan.** Drop G4 from G3's gates; add note that G3's transaction module imports the `CapabilityPolicy` trait shape (defined in G4 itself), not G3's code. G4 does not depend on G3. | §3 G3 "Gates next group" |
| M2 | Error-catalog ordering | C7 (G1) hand-authors Rust codes, T7 (G8) codegens — ambiguous whether Rust enum is duplicated, replaced, or never generated. | **Fix in plan.** Decided: Rust enum is source of truth (hand-authored in G1-A); T7 generates TypeScript from the catalog + runs a bidirectional drift detector. Avoids "duplicate enum" and "late-Phase-1 replacement" traps. | §2.1 C7, §2.8 T7, §4.6 drift-detector |
| M3 | Exit criterion | Headline references `npm run dev` but Rank 10 flags dev-server-with-hot-reload as Phase 2. Contradictory. | **Fix in plan.** Replace with `npm test` — single Vitest file asserting 6 specific behaviors (registration, 3 creates + list, cap-denied → `ON_DENIED`, `trace()` non-zero timings, `toMermaid()` parses, CID round-trip). Mechanically verifiable. | §1 Executive Summary |
| M4 | Missing risk | napi-rs v3 dual-target build integration not in risk ranking — spike proved one codebase compiles to WASM, but plan never commits to Phase 1 CI compile-check for `bindings/napi`. | **Fix in plan.** New Rank 4.5 added; new T8 deliverable (`cargo check --target wasm32-unknown-unknown -p benten-napi` in CI). Runtime WASM remains Phase 2. | §5 Rank 4.5, §2.8 T8, Appendix A |
| M5 | Scope escalation | Rank 10 incomplete. Missing: capability enforcement across all 12 primitives, paper-prototype re-validation, network-fetch KVBackend impl; wasmtime "instance pool" and "host-function manifest" not named. | **Fix in plan.** Rank 10 extended with all four missing items. | §5 Rank 10 |

### Minor findings (12 — 5 code-reviewer + 7 benten-core-guardian)

| # | Area | Finding (summary) | Disposition | Applied where |
|---|---|---|---|---|
| m1 | ADDL dispatch | R5 agent roster omits `cargo-runner` + `rust-engineer` from DEV-METHODOLOGY R5 mapping. | **Fix in plan.** R5 row rewritten with scoped roles for both. | §6 R5 row |
| m2 | ADDL dispatch | R6 council sizing ambiguous (14 vs 16 with domain swaps). | **Fix in plan.** 14 seats fixed. Slot 11 is domain-swap slot — Phase 1 uses `ivm-algorithm-b-reviewer` there; `benten-core-guardian`/`ucan-capability-auditor` run at R5 mini-review (no R6 duplication). | §6 R6 row |
| m3 | Test landscape | Two bench targets not literal from §14.6 — should be annotated. | **Fix in plan.** Every bench annotated `(§14.6 direct)` / `(§14.6 derived)` / `(non-§14.6 — informational)`. | §4.4 |
| m4 | Coherence | G5 I-row vs View-number numbering is visually confusing (I3/I4/I6 = Views 1/2/4). | **Fix in plan.** Legend added to G5 block. | §3 G5 |
| m5 | Traceability | Appendix A missing "I9 Phase 2" row. | **Fix in plan.** Row added; also added T8 + T9 rows. | Appendix A |
| m6 | Content-hash invariant | No regression test asserting edge creation / version linking doesn't shift Node CID. | **Fix in plan.** Two explicit tests named for G1-B: `edge_creation_does_not_change_endpoint_node_cids`, `version_chain_linking_does_not_change_version_node_cids`. Also added to §4.1. | §3 G1-B, §4.1 benten-core |
| m7 | Content-hash invariant | C4 `cid`-crate migration asserted byte-compat; should be CI-enforced. | **Fix in plan.** T4/T6 phrasing standardized to `assert Node::cid()?.to_string() == <fixture>` — survives C4 migration. | §2.1 C4, §2.8 T6 |
| m8 | Scope fidelity | `diag` feature default-enabled in `benten-eval` inflates the thin-engine crate. | **Fix in plan.** Feature flipped: **default OFF in `benten-eval`**, default ON via `benten-engine` default-features. Preserves thinness test. | §2.5 E7, E8, §5 Rank 5 (by reference) |
| m9 | Primitive coverage | Phase-2 primitive types must pass structural validation (types are defined in E1), only fail at call time. | **Fix in plan.** E1 + E5 rows explicit about this. New test `phase_two_primitives_pass_structural_validation` in §4.1. | §2.5 E1, E5, §4.1 benten-eval |
| m10 | Determinism axis | T4/T5/T6 each assert against the fixture but not against each other; transitive agreement is a failure mode. | **Fix in plan.** New T9 deliverable: cross-leg determinism gate CI job asserts byte-for-byte equality across T4/T5/T6 legs. | §2.8 T9, §4.6 |
| m11 | Validated decisions | Zero-config `crud('post')` path breaks silently if user doesn't supply `createdAt` (View 3 sorts by it). | **Fix in plan.** B6 requires deterministic HLC `createdAt` injection; test `crud_post_zero_config_injects_createdat_deterministically`. | §2.7 B6 |
| m12 | Spike-triage preservation | P1.graph.stress-tests (multi-MB + concurrent reader+writer) routed to "G2 R3 tests" but §4.1 only named atomicity. | **Fix in plan.** §4.1 benten-graph bullet extended with multi-MB + concurrent-reader+writer explicitly. | §4.1 benten-graph |

## Deferrals

**None.** All 17 findings fixed in the plan directly.

## Disagreements

**None.** Every finding accepted as stated.

## Plan revision summary

17 targeted edits to `.addl/phase-1/00-implementation-plan.md`. No Section restructure. New additions:

- Section 1: Exit criterion rewritten as 6 mechanical assertions.
- Section 2.1: C4 CI-phrasing clarified.
- Section 2.5: E1/E5 structural-validation-covers-all-12 made explicit; E7/E8 diag flipped to default-off.
- Section 2.7: B6 createdAt injection requirement added.
- Section 2.8: T6 phrasing tightened; T7 ambiguity resolved (Rust = hand-authored source of truth, TS codegenned); T8 (napi wasm32 compile-check) and T9 (cross-leg determinism gate) added.
- Section 3: G1-B must-pass tests extended; G3 "Gates next group" corrected; G5 view-legend added.
- Section 4.1: Multi-MB + concurrent reader+writer + phase-2-primitive-validation tests named.
- Section 4.4: Every bench annotated with §14.6 source.
- Section 4.6: T9 determinism-gate description added.
- Section 5: New Rank 4.5 (napi dual-target); Rank 10 extended with 4 missing items.
- Section 6: R5 row names cargo-runner + rust-engineer; R6 row clarifies 14-seat domain-swap model.
- Appendix A: I9 + T8 + T9 rows added.

## Ready for R1

With these edits applied, the plan is R1-ready. The 5-agent R1 team (architect-reviewer, code-reviewer, security-auditor, dx-optimizer, benten-engine-philosophy [to-create]) can now debate architecture quality rather than plan coherence.

## Process next steps

1. Commit plan revisions + this triage doc + pre-R1 JSON findings (already in `.addl/phase-1/`).
2. Create the `benten-engine-philosophy` agent just-in-time before R1 begins.
3. Launch R1 — 5 parallel subagents (or team mode) with distinct lenses on the revised plan.

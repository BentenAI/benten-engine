# Phase 2a Pre-R1 Triage

**Date:** 2026-04-21
**Plan reviewed:** `.addl/phase-2a/00-implementation-plan.md` (uncommitted)
**Critics:** `benten-engine-philosophy`, `architect-reviewer`, `code-reviewer`
**Findings total:** 1 critical + 9 major + 17 minor = 27

**Verdicts:** all three **PASS_WITH_FINDINGS**. Confidence 0.78–0.82. Both baked decisions (DAG-CBOR+CIDv1 ExecutionState envelope; capability-derived SANDBOX manifest) confirmed **sound** by all three critics. Scope split (2a/2b) confirmed **natural joint**.

**Triage result:** 1 critical + 7 major fixed-now; 3 architectural + 2 philosophical majors escalated to R1 debate with concrete agenda; 17 minors fixed-now.

---

## Fix-now findings (plan update before R1 dispatch)

### Critical

**cr-1 — file ownership overlap on `crates/benten-eval/src/invariants.rs` (G4-A, G5-A, G5-B)**

**Fix:** split `invariants.rs` into per-invariant sub-modules mirroring Phase 1's `engine.rs` split pattern. New layout:

```
crates/benten-eval/src/invariants/
  mod.rs             — public re-exports + validate_subgraph dispatch
  structural.rs      — invariants 1/2/3/5/6/9/10/12 (Phase-1 set; no-op-move)
  budget.rs          — invariant 8 multiplicative (G4-A owns this file)
  system_zone.rs     — invariant 11 full enforcement (G5-B owns this file — see arch-3 split)
  immutability.rs    — invariant 13 (G5-A owns this file — no evaluator dep)
  attribution.rs     — invariant 14 structural (G5-B owns this file)
```

Each sub-file is single-owner. `mod.rs` only re-exports; `validate_subgraph` dispatches to each sub-module. Adds to G1 scope as a structural refactor (zero semantic change for Phase-1 invariants — they migrate file locations).

### Major (architectural / procedural)

**arch-3 — group dep ordering: G4+G5 don't both need G3.** Split G5 into G5-A (Inv-13 immutability — storage-layer; parallel with G3) and G5-B (Inv-11 full + Inv-14 structural — evaluator-coupled; serial after G3). Saves ~1 day on the 2a critical path.

**cr-2 / cr-3 — file overlaps on `evaluator.rs` and `engine.rs`.** Use the existing sibling-module pattern Phase 1 already established:
- `evaluator.rs` changes partition: G3-A owns the suspend/resume machinery (top-level `run` + new `suspend_to_bytes` / `resume_from_bytes`); G5-B attribution threading lives in a new `evaluator/attribution.rs` sub-module (same pattern as invariants split above).
- `engine.rs` is already split into `engine.rs + engine_crud.rs + engine_caps.rs + engine_views.rs + engine_diagnostics.rs` (Phase 1 5d-K). G2-B durability work touches `engine.rs` (the open/build path); G3-B WAIT-resume surface lands in new `engine_wait.rs` sibling; G8-B's `view_stale_count` lands in `engine_diagnostics.rs` (already owned). Update plan §3 group file lists.

**cr-4 / cr-5 / cr-6 / cr-7 / cr-8 — plan residue from the 2a/2b split:**
- §4.3 stale gate references — rewrite against Phase 2a's 5 exit gates
- §6 path references `.addl/phase-2/` — all change to `.addl/phase-2a/`
- §6 R6 council lists 14 seats including `wasmtime-sandbox-auditor` / `ivm-algorithm-b-reviewer` (2b-only) — trim to Phase 2a's lens surface (proposed composition below)
- §6 R1 agent list similarly trim
- §7 calendar totals ~17 days covering both 2a + 2b groups — trim to ~5 days HE for 2a scope

**cr-9 — `D3` reference:** should be `D2`. (After D2 removal earlier, renumbering left a D3 reference.)

**cr-10 — `wait()` DSL owner:** G3-B adds `packages/engine/src/dsl.ts` (extend existing `wait()` stub — executor-side shape maps to the new Rust surface) + `bindings/napi/src/wait.rs` (new) to G3-B's file ownership.

### Minor (plan-hygiene)

**cr-11 / cr-12** — §1 opener "two gates" + "four-of-six" phrasing confusion. Rewrite: "two headline gates + three bundled gates"; gate 2 becomes "Four new invariants firing (8 multiplicative, 11 full, 13 immutability, 14 structural causal attribution)."

**cr-13** — G1-A file ownership includes `crates/benten-eval/src/error.rs` which does not exist at HEAD. Correct to `crates/benten-eval/src/lib.rs` (where `EvalError` lives).

**cr-14** — `view_stale_count_tallies` test claimed by both G8-B (2b) and G11-A (2a). Since `view_stale_count` wire-up is 2a scope (per plan G11 explicit carve-out), the test lives in G11-A only. G8-B's claim is a 2b residue — removed.

**cr-15** — §11 "if file < 1200 lines" threshold phrasing. Rewrite: "transaction.rs at 711 lines — split trigger is when Phase 2 adds enough transaction machinery that it crosses ~1200 lines."

**cr-16** — E13 string mismatch on `allow(missing_docs, reason=...)`. Trivial — update to the actual reason string from HEAD.

**cr-17** — `engine_diagnostics.rs:170-184` → correct line for `0.0` hardcode is `185`. Update citation.

**cr-18** — workflow filename `.github/workflows/phase-2-exit-criteria.yml` → `phase-2a-exit-criteria.yml`.

**cr-19 / cr-20 / cr-21** — §6 stage parallelism + critic references — trim to 2a scope.

**phil-4** — `TimeSource` in `benten-core` violates "core is data-shape-only." **Fix:** move `TimeSource` to `benten-eval` (where WAIT lives). Plan update: G3-B file ownership moves `crates/benten-core/src/time_source.rs` → `crates/benten-eval/src/time_source.rs`. No impact on Phase-1 HLC stamping (keeps its existing location).

**phil-7** — arch-1 thinning durability. **Fix:** G11-A adds a workspace-wide assert that `benten-eval/Cargo.toml` does not gain `benten-graph` as a dep (`#[test] fn arch_1_dep_break_preserved` reads the Cargo.toml at test time and asserts). Also run on every Phase-2 PR via a new CI job.

**arch-4** — Inv-13 immutability firing matrix for dedup (content matches) vs corruption (content differs) on privileged path. **Fix:** G5-A adds an explicit section to `docs/SECURITY-POSTURE.md` describing the 4-quadrant matrix (privileged-yes/no × content-matches-yes/no) and which path fires `E_INV_IMMUTABILITY` vs `E_WRITE_CONFLICT` vs `Ok(cid_dedup)`.

**arch-5** — Inv-11 TRANSFORM-computed-CID cost ceiling. **Fix:** G5-B commits to a static-table probe for the system-zone-prefix check (amortised O(1) via `phf` const-compiled hashmap) and includes a criterion bench gated at <1µs/check. No Node-fetch path; the system-zone prefix list is static.

**arch-7** — SANDBOX namespace owner. **Fix:** add a line to `.addl/phase-2b/00-scope-outline.md` §5 that the `host:<domain>:<action>` namespace is owned by a new `docs/HOST-FUNCTIONS.md` catalog (author-able via a Phase 2b process; parallel surface to `docs/ERROR-CATALOG.md`). Phase 2a locks the *format* (cap-string shape) in `benten-errors` so the codegen works; Phase 2b fills in the actual functions.

**arch-8** — G7 (Phase 2b) stale dep list. **Fix:** in the 2b scope outline, G7's dep list references G3+G4+G5+G6; G6 is now 2b-scope and G7's references to "G3/G4/G5" become "Phase 2a completion." Trivial doc update.

**phil-6** — Module manifest option C under-specified. **Fix:** plan §9.6 defaults to Option B (JSON), names Option C (manifest-as-subgraph) as a Phase-2b R1 agenda item rather than open-ended — R1 decides at 2b kickoff. Phase 2a does not touch module manifest; this is a 2b concern but the default goes in now.

---

## Escalate to R1 debate (major issues requiring cross-phase lens)

### arch-1 — ExecutionState payload shape is cross-phase load-bearing

Deferred to R1 with explicit agenda: the payload shape (attribution triple carry-through, pinned-subgraph-CID set, context-binding Value resolution on resume) affects Phase 3 sync, Phase 6 AI forking, Phase 7 Garden approvals. **R1 addition:** include one agent with cross-phase consumer perspective — `ucan-capability-auditor` (for Phase 3 sync / cap-chain carry-through) or open a targeted agenda item for `benten-engine-philosophy` to re-examine the payload shape specifically.

### arch-2 — HostError contract freeze at G1 without Phase-3 sync representation

Deferred to R1 with explicit agenda: HostError is the contract every downstream Phase-2 primitive + Phase-3 sync error-mapping consumes. R1 must decide whether it's opaque-Box (phil-3) or enum-based, and whether variants anticipate Phase-3 sync error shapes (hash-mismatch, HLC drift, cap-chain invalid). **R1 addition:** include `ucan-capability-auditor` for the Phase-3 consumer lens.

### phil-1 — Inv-14 causal attribution: structural property or engine policy?

Deferred to R1 with explicit agenda: is Inv-14 a *subgraph-data* property (like Inv-1/10) or an *evaluator-policy* property (like runtime iteration budget)? If policy, move out of invariants.rs entirely; if structural, lock an extensible 3-tuple-or-delegation-chain schema now. Phase 6 AI delegation chains would foreclose under a rigid 3-tuple.

### phil-2 — Inv-11 runtime enforcement placement

Deferred to R1 with explicit agenda: does the system-zone-label check live in `benten-eval/invariants.rs` (current plan) or `benten-engine/primitive_host.rs`? The philosophy critic argues engine placement preserves the arch-1 thinning — knowledge of "what's system-zone" is engine-policy, not evaluator-structural. R1 decides.

### phil-3 — `HostError` shape

Deferred to R1 with explicit agenda: Option A (opaque `Box<dyn StdError>`), Option B (enum of known variants), Option C (ErrorCode-only). Philosophy critic prefers A; architect critic flags B re-creates `GraphError`-style coupling under a new name. R1 decides and locks the shape before G1 proceeds.

### arch-6 — G5-B attribution vs G3 serialization order on evaluator.rs

Deferred to R1 agenda as "evaluator.rs sub-module partitioning." Proposed fix in the plan update above (sub-modules); R1 confirms or adjusts.

### phil-5 — Inv-4/7/8 shared `Budget` abstraction across 2a/2b

Deferred to R1 with explicit agenda: should Phase 2a's G4 Inv-8 multiplicative-through-CALL use a `Budget` abstraction that Phase 2b's SANDBOX fuel composes on, or are they independent? If shared, 2a locks the abstract shape now; if independent, 2b retrofits.

---

## Acknowledged / no-action

None in this pass. Every finding has either a fix-now action or an R1-debate escalation.

---

## Summary for R1 dispatch

Phase 2a plan is **dispatch-ready with these revisions landed:**

1. `invariants.rs` split into per-invariant sub-modules (cr-1 critical)
2. G5 split into G5-A (Inv-13) + G5-B (Inv-11 full + Inv-14) (arch-3 major)
3. `evaluator.rs` partitioned — G3-A owns suspend/resume, G5-B attribution in sub-module (cr-2)
4. `engine.rs` ownership uses the existing Phase-1-established sibling modules (cr-3)
5. Plan text 2a/2b-split residue scrubbed (cr-4 through cr-8)
6. D3/D2 reference fix + 14 other minors
7. `TimeSource` moves to `benten-eval` (phil-4)
8. Inv-13 4-quadrant firing matrix spec added to SECURITY-POSTURE (arch-4)
9. Inv-11 static-prefix table commitment (arch-5)
10. `docs/HOST-FUNCTIONS.md` placeholder added to 2b scope outline (arch-7)
11. Module manifest default = JSON (Option B) with Option C reserved for 2b R1 (phil-6)
12. Workspace-wide `arch_1_dep_break_preserved` test added to G11-A + CI (phil-7)

**R1 agenda items (escalated from triage):**

- arch-1: ExecutionState payload shape (add cross-phase consumer lens to R1 agents)
- arch-2: HostError freeze at G1 (add Phase-3 sync lens)
- phil-1: Inv-14 attribution — structural vs policy (open-ended)
- phil-2: Inv-11 placement — evaluator vs engine (open-ended)
- phil-3: HostError shape — A/B/C (R1 chooses before G1)
- phil-5: Shared Budget abstraction across 2a/2b (open-ended)
- arch-6: evaluator.rs sub-module partitioning confirmation

**R1 proposed composition (Phase 2a lens surface):**

- `architect-reviewer` — cross-phase coherence (carried from pre-R1)
- `benten-engine-philosophy` — thin-engine thesis + cross-phase foreclosure (carried from pre-R1)
- `code-reviewer` — tactical + generic correctness baseline (carried)
- `security-auditor` — persisted ExecutionState is a new attack surface; TOCTOU delegation is security-class
- `ucan-capability-auditor` — cross-phase Phase-3 sync / cap-chain lens for arch-1 HostError + ExecutionState attribution carry-through
- `code-as-graph-reviewer` — Inv-11/13/14 enforcement (the 14-invariant surface is its direct domain)
- `dx-optimizer` — WAIT developer ergonomics + devserver hot-reload surface

7 agents. No `wasmtime-sandbox-auditor` or `ivm-algorithm-b-reviewer` — those are 2b R1 composition.

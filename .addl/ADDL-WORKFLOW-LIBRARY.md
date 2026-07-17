# ADDL-as-Workflows Library — ongoing sub-project tracker

**Status:** ACTIVE (kicked off 2026-07-17). **Owner:** orchestrator. **Ben ratifications:** build-core-now + dogfood-on-GAP-KDB; lean-closer-to-full-automation on the convergence loop (reversible → audit-not-gate). Keep THIS doc current + a memory pointer.

## Goal

Re-express the fully-spec'd ADDL pipeline (PRE-WORK → R1 → R2 → R3 → R4 → R5 → R4b → R6) as a **coherent, parameterized workflow library** that makes the ~64 catalogued disciplines *unskippable control-flow* rather than prose — replacing both the brittle hardcoded `f-full-*` snapshots and the ad-hoc one-off scripts. Dogfood each stage on the remaining GAP-KDB mini-ADDL (R3→R5→R6) as we build it.

## The design (ratified shape)

1. **Config pattern (the linchpin — fixes the root cause).** `Workflow({name})` registry isn't wired here + large `args` bind unreliably → so: the workflow SCRIPT holds only **minimal config as top-of-file consts** (refs, tier, mode, canary, wave-slicing, thresholds); the **agents read the big content — plan/spec/canon/prior-round-state — from GIT-TRACKED docs via `git show <ref>:<path>`**. Nothing round-specific inlined. The plan doc IS the config, versioned + external. Invoke by **scriptPath** only.
2. **Guards as control-flow, not prose.** Universal empty/partial-panel guard BEFORE any consolidator (`if (live.length===0) return {aborted:'PANEL-EMPTY'}`; `if (live.length < expected) return {status:'PANEL-INCOMPLETE', missing}`). Panel-gate the fix-loop convergence shortcut (kill the false-CONVERGE hole at converging-fix-loop.js:222).
3. **Two parameter axes.** `tier` (R1/R2/R3/R4/R4b/R5/R6 → selects standardCatalog, iterates?, lens-reduction-allowed?) + `mode` (checkpoint | autonomous integration). The stage-workflows become pure functions of `{tier, mode, cfg{refs, crates+features, canon-pointer, MAX_ROUNDS, BUDGET_FLOOR, recurrenceThreshold, requiredConsecutiveConverged, batchSize}, lenses|dimensions|waves|canary}`.
4. **workflow-common tripartition (preserve as the contract).** STRUCTURAL disciplines → stages/guards; PREAMBLE disciplines → inlined into every agent (ground-truth §3.5n, HARD-RULE-12 disposition, pre-flight tree-state, refuse-on-partial); NOT-CODIFIED/human-gated → stay orchestrator/Ben-side (the tag/merge decision, orchestrator-side ground-truth re-check, divergent-fork surfacing). Never auto-encode the human-gated ones.
5. **Convergence driver (LEAN-AUTOMATION per Ben).** A driver that runs council → fix-loop → re-review → repeat until two-consecutive-CONVERGED, running autonomously WITH: strong in-workflow adversarial-verify (default-REFUTED), rich decision-log (plain-English-with-prediction), a FLAG-FOR-BEN valve (surfaces divergent forks, does NOT halt), MAX_ROUNDS + budget-floor backstops, `seen`-set anti-oscillation. The **tag + #1382 merge stay HARD-HELD for Ben** (never in the automation). Orchestrator audits the decision-log + flags + spot-checks BLK/MAJ rather than gating every round.
6. **Known-broken pieces to keep fixed:** lens-architect + completeness-critic MUST be schema-FREE (a schema architect HUNG 39 min); softSchema (retry-once→surfacing fallback) for consolidators; string prompts never arrays; `node --check` apostrophe-free framing before invoke; batch fan-out into sub-waves of ≤4 (rate-limit).

## The canonical library (target: ~6 stage-workflows + 1 driver)

Base = the 6 existing generics (already well-built; harden them). RETIRE the 14 `f-full-*` snapshots (archive as as-run history).

| Workflow | Tiers | Status | Notes |
|---|---|---|---|
| `addl-review-council` | R1 / R4 / R4b / R6 | ⬜ harden | add control-flow panel guard; round-state via git-show not default CFG; softSchema in concrete runs; delete dead LENSSET_SCHEMA |
| `addl-r2-test-landscape` | R2 | ⬜ harden | add empty-discovery ABORT; add adversarial-verify on coverage matrix; iterate-on-completeness |
| `addl-r3-test-writers` | R3 | ⬜ harden (DOGFOOD #1 — GAP-KDB R3) | replace regex canary-gate with schema'd gate; abort if all waves drop; dynamic wave count from R2 slicing |
| `addl-r5-impl-to-green` | R5 | ⬜ harden (DOGFOOD #2 — GAP-KDB R5) | distinguish dropped-vs-approved waves before success check; config from consts not frozen body |
| `converging-fix-loop` | R4-fix / R6-fix | ⬜ harden | panel-gate the `converged=true` shortcut; parameterize mainBase/corpusBranch/integWorktree (drop baked absolute path); retire 4 older hardcoded-lens variants |
| `addl-r1-critic-council` (or reuse review-council@tier=R1) | R1 / pre-work | ⬜ | Pattern-6 architect + adversarial-verify (the f-full-r1* instances lacked verify) |
| **`addl-converge-driver`** (NEW) | R6 phase-close | ⬜ build | the lean-automation two-consecutive-CONVERGED driver (item 5) |

## Discipline coverage checklist (the ~64; ✓ = encoded as guard/stage/param, ~ = preamble, H = human-gated)

Full catalog in the D-46 research (Agent B). Groups: (A) convergence/termination A1-A11 · (B) panel composition B1-B12 · (C) implementation C1-C17 · (D) decision D1-D10 · (E) operational E1-E14. Track per-discipline encoding here as each stage is hardened. **Load-bearing / most-violated (encode FIRST):** D2 HARD-RULE-12, A3 full-council-every-round, A1 iterate-to-convergence, A4 full-phase-review-not-delta, D5 extra-reflection-pass, D4 surface-arch-decisions, B4 refuse-on-partial (→ control-flow), A2 two-consecutive-CONVERGED (→ driver state).

## Done / not-done log

- 2026-07-17: sub-project kicked off; research complete (Agent A per-script audit + Agent B ~64-discipline catalog); design ratified by Ben; tracking doc + memory + CLAUDE.md pointer created. NEXT: harden `addl-r3-test-writers` as GAP-KDB R3 dogfood #1 (once GAP-KDB R2 `wzf65yc6d` returns its R3 slicing).

## Retire list (archive, do not reuse)

The 14 `f-full-*` snapshots + `issue-audit.js` — hardcoded round-state; keep as `.addl/phase-4-meta/workflows/_archive-as-run/` for forensic history once the canonical library covers their stages.
